//! Mattermost-compatible WebSocket endpoint with session resumption
//!
//! Implements:
//! - Protocol-level Ping/Pong (WebSocket control frames)
//! - Connection ID & sequence number based session resumption
//! - 60s ping interval, 100s pong timeout, 30s write deadline
//! - Message buffering for replay on reconnect

use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Query, State,
    },
    http::HeaderMap,
    response::Response,
};
use serde::Deserialize;
use serde_json::json;
use tokio::time::timeout;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::validate_token;
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::realtime::{
    websocket_actor::{close_codes, WebSocketActor, WsEvent},
    WsEnvelope, WsBroadcast,
};

/// WebSocket query parameters
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Authentication token
    pub token: Option<String>,
    /// Connection ID for session resumption
    pub connection_id: Option<String>,
    /// Last sequence number received by client
    pub sequence_number: Option<i64>,
}

/// Get max simultaneous connections from config
async fn get_max_simultaneous_connections(state: &AppState) -> usize {
    let value: Option<String> = sqlx::query_scalar(
        "SELECT site->>'max_simultaneous_connections' FROM server_config WHERE id = 'default'",
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match value.and_then(|val| val.parse::<i64>().ok()) {
        Some(max) if max > 0 => max as usize,
        _ => 5,
    }
}

/// Main WebSocket handler
pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    let mut token = query.token.clone();
    let sequence_number = query.sequence_number;
    let connection_id = query.connection_id.clone();

    // Check Authorization header if token not in query
    if token.is_none() {
        if let Some(auth_header) = headers.get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    token = Some(auth_str[7..].to_string());
                } else if auth_str.starts_with("Token ") {
                    token = Some(auth_str[6..].to_string());
                } else {
                    token = Some(auth_str.to_string());
                }
            }
        }
    }

    // Validate token
    let user_id = if let Some(ref t) = token {
        validate_token(t, &state.jwt_secret).ok().map(|c| c.claims.sub)
    } else {
        None
    };

    trace!(
        addr = %addr,
        has_token = token.is_some(),
        has_user = user_id.is_some(),
        connection_id = ?connection_id,
        sequence_number = ?sequence_number,
        "WebSocket connection request"
    );

    ws.on_upgrade(move |socket| {
        handle_socket(socket, state, user_id, connection_id, sequence_number, addr)
    })
}

/// Handle the WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: Option<Uuid>,
    connection_id: Option<String>,
    sequence_number: Option<i64>,
    addr: SocketAddr,
) {
    // Handle authentication if not already done
    let user_id = match user_id {
        Some(id) => id,
        None => {
            // Try to authenticate via WebSocket message
            match authenticate_via_websocket(socket, &state).await {
                Some((id, sock)) => {
                    // Continue with authenticated socket
                    return run_connection(sock, state, id, connection_id, sequence_number, addr).await;
                }
                None => {
                    warn!(addr = %addr, "WebSocket authentication failed");
                    return;
                }
            }
        }
    };

    run_connection(socket, state, user_id, connection_id, sequence_number, addr).await;
}

/// Authenticate via WebSocket message exchange
async fn authenticate_via_websocket(
    mut socket: WebSocket,
    state: &AppState,
) -> Option<(Uuid, WebSocket)> {
    // Wait for authentication challenge
    let timeout_duration = Duration::from_secs(30);
    
    loop {
        match timeout(timeout_duration, socket.recv()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    if value["action"] == "authentication_challenge" {
                        if let Some(token) = value["data"]["token"].as_str() {
                            if let Ok(claims) = validate_token(token, &state.jwt_secret) {
                                let user_id = claims.claims.sub;
                                
                                // Send OK response
                                let resp = json!({
                                    "status": "OK",
                                    "seq_reply": value["seq"]
                                });
                                
                                let _ = socket
                                    .send(Message::Text(resp.to_string().into()))
                                    .await;
                                
                                return Some((user_id, socket));
                            } else {
                                // Send error response
                                let resp = json!({
                                    "status": "FAIL",
                                    "seq_reply": value["seq"],
                                    "error": "Invalid token"
                                });
                                let _ = socket
                                    .send(Message::Text(resp.to_string().into()))
                                    .await;
                            }
                        }
                    }
                }
            }
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => {
                return None;
            }
            Ok(Some(Err(_))) => {
                return None;
            }
            Err(_) => {
                // Timeout
                return None;
            }
            _ => {}
        }
    }
}

/// Run the main connection loop with session resumption
async fn run_connection(
    socket: WebSocket,
    state: AppState,
    user_id: Uuid,
    connection_id: Option<String>,
    sequence_number: Option<i64>,
    addr: SocketAddr,
) {
    // Check connection limits
    let max_connections = get_max_simultaneous_connections(&state).await;
    let current_connections = state.ws_hub.user_connection_count(user_id).await;
    
    if current_connections >= max_connections {
        warn!(
            user_id = %user_id,
            current = current_connections,
            max = max_connections,
            "Too many connections for user"
        );
        
        // Send close frame and return
        // Note: In axum 0.8, we can't easily split the socket, so we just drop it
        // The client will see the connection close
        return;
    }

    // Get or create connection store
    let store = state.connection_store.clone();

    // Check if this is a resumption attempt before moving connection_id
    let is_resumption_attempt = connection_id.is_some();
    
    // Create WebSocket actor with session resumption
    let (actor, missed_messages) = WebSocketActor::new(
        socket,
        store,
        user_id,
        connection_id,
        sequence_number,
        Some(addr),
    )
    .await;

    let actor_connection_id = actor.connection_id.clone();
    let is_resumed = !missed_messages.is_empty() || is_resumption_attempt;

    info!(
        connection_id = %actor_connection_id,
        user_id = %user_id,
        resumed = is_resumed,
        missed_count = missed_messages.len(),
        addr = %addr,
        "WebSocket connection established"
    );

    // Get username
    let username = match sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(name) => name,
        Err(_) => {
            error!(user_id = %user_id, "Failed to get username");
            actor.close(close_codes::INTERNAL_ERROR, "User not found");
            return;
        }
    };

    // Add connection to hub
    let (hub_conn_id, mut hub_rx) = state.ws_hub.add_connection(user_id, username.clone()).await;

    // Subscribe to teams and channels
    let teams = sqlx::query_scalar::<_, Uuid>("SELECT team_id FROM team_members WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    
    for team_id in teams {
        state.ws_hub.subscribe_team(user_id, team_id).await;
    }

    let channels =
        sqlx::query_scalar::<_, Uuid>("SELECT channel_id FROM channel_members WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    
    for channel_id in channels {
        state.ws_hub.subscribe_channel(user_id, channel_id).await;
    }

    // Update presence
    let _ = sqlx::query("UPDATE users SET presence = 'online' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;

    let presence_evt = WsEnvelope::event(
        crate::realtime::EventType::UserPresence,
        crate::realtime::PresenceEvent {
            user_id,
            status: "online".to_string(),
        },
        None,
    );
    state.ws_hub.broadcast(presence_evt).await;

    // Send hello event
    let hello = mm::WebSocketMessage {
        seq: Some(0),
        event: "hello".to_string(),
        data: json!({
            "connection_id": actor_connection_id.clone(),
            "server_version": "rustchat-0.1.0",
            "protocol_version": "1.0"
        }),
        broadcast: mm::Broadcast {
            omit_users: None,
            user_id: String::new(),
            channel_id: String::new(),
            team_id: String::new(),
        },
    };

    if let Err(e) = actor.send(hello) {
        warn!(
            connection_id = %actor_connection_id,
            error = %e,
            "Failed to send hello message"
        );
        return;
    }

    // Replay missed messages if resuming
    for msg in missed_messages {
        if let Err(e) = actor.send(msg) {
            warn!(
                connection_id = %actor_connection_id,
                error = %e,
                "Failed to send missed message"
            );
            break;
        }
    }

    // Main event loop
    let state_clone = state.clone();
    let actor_clone = actor.clone();

    // Spawn task to forward hub messages to client
    let mut hub_forward_task = tokio::spawn(async move {
        while let Ok(msg_str) = hub_rx.recv().await {
            if let Ok(envelope) = serde_json::from_str::<WsEnvelope>(&msg_str) {
                if let Some(mm_msg) = map_envelope_to_mm(&envelope) {
                    if let Err(_) = actor_clone.send(mm_msg) {
                        break;
                    }
                }
            }
        }
    });

    // Handle events from WebSocket actor
    loop {
        tokio::select! {
            // Handle events from the WebSocket actor
            event = actor.recv() => {
                match event {
                    Some(WsEvent::MessageReceived(text)) => {
                        handle_client_message(
                            &state,
                            user_id,
                            &username,
                            &text,
                        )
                        .await;
                    }
                    Some(WsEvent::PongReceived) => {
                        trace!(connection_id = %actor_connection_id, "Pong received");
                    }
                    Some(WsEvent::Closed(reason)) => {
                        info!(
                            connection_id = %actor_connection_id,
                            code = reason.code,
                            reason = %reason.reason,
                            "Connection closed"
                        );
                        break;
                    }
                    Some(WsEvent::Error(e)) => {
                        error!(
                            connection_id = %actor_connection_id,
                            error = %e,
                            "Connection error"
                        );
                        break;
                    }
                    None => {
                        debug!(connection_id = %actor_connection_id, "Event channel closed");
                        break;
                    }
                }
            }
            
            // If hub forward task ends, we should also close
            _ = &mut hub_forward_task => {
                debug!(connection_id = %actor_connection_id, "Hub forward task ended");
                break;
            }
        }
    }

    // Cleanup
    hub_forward_task.abort();
    
    // Mark connection as disconnected (for potential resumption)
    actor.disconnect();
    
    // Remove from hub
    state.ws_hub.remove_connection(user_id, hub_conn_id).await;

    // Update presence if no other connections
    if state.ws_hub.user_connection_count(user_id).await == 0 {
        let _ = sqlx::query("UPDATE users SET presence = 'offline' WHERE id = $1")
            .bind(user_id)
            .execute(&state.db)
            .await;

        let presence_evt = WsEnvelope::event(
            crate::realtime::EventType::UserPresence,
            crate::realtime::PresenceEvent {
                user_id,
                status: "offline".to_string(),
            },
            None,
        );
        state.ws_hub.broadcast(presence_evt).await;
    }

    info!(
        connection_id = %actor_connection_id,
        user_id = %user_id,
        "WebSocket connection ended"
    );
}

/// Handle a message from the client
async fn handle_client_message(
    state: &AppState,
    user_id: Uuid,
    username: &str,
    text: &str,
) {
    trace!(
        user_id = %user_id,
        text = %text,
        "Received client message"
    );

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        // Handle action-based messages (Mattermost style)
        if let Some(action) = value.get("action").and_then(|v| v.as_str()) {
            match action {
                "user_typing" => {
                    if let Some(data) = value.get("data") {
                        if let Some(channel_id_str) = data.get("channel_id").and_then(|v| v.as_str()) {
                            if let Some(channel_id) = parse_mm_or_uuid(channel_id_str) {
                                let broadcast = WsEnvelope::event(
                                    crate::realtime::EventType::UserTyping,
                                    crate::realtime::TypingEvent {
                                        user_id,
                                        display_name: username.to_string(),
                                        thread_root_id: data
                                            .get("parent_id")
                                            .and_then(|v| v.as_str())
                                            .and_then(parse_mm_or_uuid),
                                    },
                                    Some(channel_id),
                                )
                                .with_broadcast(WsBroadcast {
                                    channel_id: Some(channel_id),
                                    team_id: None,
                                    user_id: None,
                                    exclude_user_id: Some(user_id),
                                });
                                state.ws_hub.broadcast(broadcast).await;
                            }
                        }
                    }
                }
                _ => {
                    trace!(action = %action, "Unknown action received");
                }
            }
        }
        
        // Handle envelope-style messages (our internal format)
        if let Ok(envelope) = serde_json::from_str::<crate::realtime::ClientEnvelope>(text) {
            match envelope.event.as_str() {
                "subscribe_channel" => {
                    if let Some(cid) = envelope.channel_id {
                        state.ws_hub.subscribe_channel(user_id, cid).await;
                        let evt = WsEnvelope::event(
                            crate::realtime::EventType::ChannelSubscribed,
                            json!({ "channel_id": cid }),
                            None,
                        )
                        .with_broadcast(WsBroadcast {
                            user_id: Some(user_id),
                            channel_id: None,
                            team_id: None,
                            exclude_user_id: None,
                        });
                        state.ws_hub.broadcast(evt).await;
                    }
                }
                "unsubscribe_channel" => {
                    if let Some(cid) = envelope.channel_id {
                        state.ws_hub.unsubscribe_channel(user_id, cid).await;
                    }
                }
                "typing" | "typing_start" => {
                    if let Some(cid) = envelope.channel_id {
                        let event = WsEnvelope::event(
                            crate::realtime::EventType::UserTyping,
                            crate::realtime::TypingEvent {
                                user_id,
                                display_name: username.to_string(),
                                thread_root_id: None,
                            },
                            Some(cid),
                        )
                        .with_broadcast(WsBroadcast {
                            channel_id: Some(cid),
                            user_id: None,
                            team_id: None,
                            exclude_user_id: Some(user_id),
                        });
                        state.ws_hub.broadcast(event).await;
                    }
                }
                "presence" => {
                    if let Some(status) = envelope.data.get("status").and_then(|v| v.as_str()) {
                        state.ws_hub.set_presence(user_id, status.to_string()).await;
                        let event = WsEnvelope::event(
                            crate::realtime::EventType::UserPresence,
                            crate::realtime::PresenceEvent {
                                user_id,
                                status: status.to_string(),
                            },
                            None,
                        );
                        state.ws_hub.broadcast(event).await;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Map internal envelope to Mattermost-compatible message
fn map_envelope_to_mm(env: &WsEnvelope) -> Option<mm::WebSocketMessage> {
    let seq = None; // Will be assigned by actor

    match env.event.as_str() {
        "message_created" | "thread_reply_created" => {
            if let Ok(post_resp) =
                serde_json::from_value::<crate::models::post::PostResponse>(env.data.clone())
            {
                let mm_post: mm::Post = post_resp.into();
                let post_json = serde_json::to_string(&mm_post).unwrap_or_default();

                let data = json!({
                    "post": post_json,
                    "channel_display_name": "",
                    "channel_name": "",
                    "channel_type": "O",
                    "sender_name": mm_post.user_id,
                    "team_id": ""
                });

                Some(mm::WebSocketMessage {
                    seq,
                    event: "posted".to_string(),
                    data,
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "user_typing" => {
            if let Ok(typing) = serde_json::from_value::<crate::realtime::TypingEvent>(env.data.clone()) {
                let parent_id = typing
                    .thread_root_id
                    .map(encode_mm_id)
                    .unwrap_or_default();
                Some(mm::WebSocketMessage {
                    seq,
                    event: "typing".to_string(),
                    data: json!({
                        "parent_id": parent_id,
                        "user_id": encode_mm_id(typing.user_id),
                    }),
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "message_updated" | "thread_reply_updated" => {
            if let Ok(post_resp) =
                serde_json::from_value::<crate::models::post::PostResponse>(env.data.clone())
            {
                let mm_post: mm::Post = post_resp.into();
                let post_json = serde_json::to_string(&mm_post).unwrap_or_default();
                Some(mm::WebSocketMessage {
                    seq,
                    event: "post_edited".to_string(),
                    data: json!({ "post": post_json }),
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "message_deleted" | "thread_reply_deleted" => {
            if let Ok(post_resp) =
                serde_json::from_value::<crate::models::post::PostResponse>(env.data.clone())
            {
                let mm_post: mm::Post = post_resp.into();
                let post_json = serde_json::to_string(&mm_post).unwrap_or_default();
                Some(mm::WebSocketMessage {
                    seq,
                    event: "post_deleted".to_string(),
                    data: json!({ "post": post_json }),
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "reaction_added" => {
            if let Ok(reaction) =
                serde_json::from_value::<crate::models::post::Reaction>(env.data.clone())
            {
                let mm_reaction = mm::Reaction {
                    user_id: encode_mm_id(reaction.user_id),
                    post_id: encode_mm_id(reaction.post_id),
                    emoji_name: crate::mattermost_compat::emoji_data::get_short_name_for_emoji(&reaction.emoji_name),
                    create_at: reaction.created_at.timestamp_millis(),
                    update_at: reaction.created_at.timestamp_millis(),
                    delete_at: 0,
                    channel_id: env.channel_id.map(encode_mm_id).unwrap_or_default(),
                    remote_id: "".to_string(),
                };
                let reaction_json = serde_json::to_string(&mm_reaction).unwrap_or_default();
                Some(mm::WebSocketMessage {
                    seq,
                    event: "reaction_added".to_string(),
                    data: json!({ "reaction": reaction_json }),
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "reaction_removed" => {
            if let Ok(reaction) =
                serde_json::from_value::<crate::models::post::Reaction>(env.data.clone())
            {
                let mm_reaction = mm::Reaction {
                    user_id: encode_mm_id(reaction.user_id),
                    post_id: encode_mm_id(reaction.post_id),
                    emoji_name: crate::mattermost_compat::emoji_data::get_short_name_for_emoji(&reaction.emoji_name),
                    create_at: reaction.created_at.timestamp_millis(),
                    update_at: reaction.created_at.timestamp_millis(),
                    delete_at: 0,
                    channel_id: env.channel_id.map(encode_mm_id).unwrap_or_default(),
                    remote_id: "".to_string(),
                };
                let reaction_json = serde_json::to_string(&mm_reaction).unwrap_or_default();
                Some(mm::WebSocketMessage {
                    seq,
                    event: "reaction_removed".to_string(),
                    data: json!({ "reaction": reaction_json }),
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "user_updated" => {
            if let Some(status_str) = env.data.get("status").and_then(|v| v.as_str()) {
                let user_id = env
                    .data
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .and_then(parse_mm_or_uuid)
                    .map(encode_mm_id)
                    .unwrap_or_default();
                Some(mm::WebSocketMessage {
                    seq,
                    event: "status_change".to_string(),
                    data: json!({ "user_id": user_id, "status": status_str }),
                    broadcast: map_broadcast(env.broadcast.as_ref()),
                })
            } else {
                None
            }
        }
        "channel_viewed" => {
            let channel_id = extract_mm_id(env.data.get("channel_id"));
            Some(mm::WebSocketMessage {
                seq,
                event: "channel_viewed".to_string(),
                data: json!({ "channel_id": channel_id }),
                broadcast: map_broadcast(env.broadcast.as_ref()),
            })
        }
        "member_added" => {
            let user_id = extract_mm_id(env.data.get("user_id"));
            let channel_id = extract_mm_id(env.data.get("channel_id"));
            let team_id = extract_mm_id(env.data.get("team_id"));
            Some(mm::WebSocketMessage {
                seq,
                event: "user_added".to_string(),
                data: json!({
                    "user_id": user_id,
                    "channel_id": channel_id,
                    "team_id": team_id,
                }),
                broadcast: map_broadcast(env.broadcast.as_ref()),
            })
        }
        "member_removed" => {
            let user_id = extract_mm_id(env.data.get("user_id"));
            let remover_id = extract_mm_id(env.data.get("remover_id"));
            Some(mm::WebSocketMessage {
                seq,
                event: "user_removed".to_string(),
                data: json!({
                    "user_id": user_id,
                    "remover_id": remover_id,
                }),
                broadcast: map_broadcast(env.broadcast.as_ref()),
            })
        }
        _ => None,
    }
}

fn extract_mm_id(value: Option<&serde_json::Value>) -> String {
    value
        .and_then(|v| v.as_str())
        .and_then(parse_mm_or_uuid)
        .map(encode_mm_id)
        .unwrap_or_default()
}

fn map_broadcast(b_opt: Option<&crate::realtime::WsBroadcast>) -> mm::Broadcast {
    if let Some(b) = b_opt {
        mm::Broadcast {
            omit_users: None,
            user_id: b.user_id.map(encode_mm_id).unwrap_or_default(),
            channel_id: b.channel_id.map(encode_mm_id).unwrap_or_default(),
            team_id: b.team_id.map(encode_mm_id).unwrap_or_default(),
        }
    } else {
        mm::Broadcast {
            omit_users: None,
            user_id: String::new(),
            channel_id: String::new(),
            team_id: String::new(),
        }
    }
}
