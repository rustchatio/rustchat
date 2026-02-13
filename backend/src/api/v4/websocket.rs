//! Mattermost-compatible WebSocket endpoint with session resumption
//!
//! Implements:
//! - Protocol-level Ping/Pong (WebSocket control frames)
//! - Connection ID & sequence number based session resumption
//! - 60s ping interval, 100s pong timeout, 30s write deadline
//! - Message buffering for replay on reconnect

use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::time::Duration;

use axum::{
    extract::{
        ws::{rejection::WebSocketUpgradeRejection, Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::api::v4::calls_plugin;
use crate::api::websocket_core::{self, EnvelopeCommandOptions};
use crate::api::AppState;
use crate::auth::validate_token;
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::realtime::{
    websocket_actor::{close_codes, WebSocketActor, WsEvent},
    WsBroadcast, WsEnvelope,
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

/// Main WebSocket handler
pub async fn handle_websocket(
    ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Response {
    let requested_protocol = websocket_core::requested_protocol(&headers);
    let ws = match ws {
        Ok(upgrade) => upgrade,
        Err(err) => {
            warn!(
                error = %err,
                connection_header = ?headers.get("connection").and_then(|v| v.to_str().ok()),
                upgrade_header = ?headers.get("upgrade").and_then(|v| v.to_str().ok()),
                has_sec_websocket_key = headers.contains_key("sec-websocket-key"),
                sec_websocket_version = ?headers.get("sec-websocket-version").and_then(|v| v.to_str().ok()),
                user_agent = ?headers.get("user-agent").and_then(|v| v.to_str().ok()),
                "WebSocket upgrade rejected"
            );
            return err.into_response();
        }
    };

    let token = websocket_core::resolve_auth_token(
        query.token.as_deref(),
        &headers,
        requested_protocol.as_deref(),
        true,
    );
    let sequence_number = query.sequence_number;
    let connection_id = query.connection_id.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    let user_id = websocket_core::validate_user_id(token.as_deref(), &state.jwt_secret);

    trace!(
        has_token = token.is_some(),
        has_protocol = requested_protocol.is_some(),
        has_user = user_id.is_some(),
        connection_id = ?connection_id,
        sequence_number = ?sequence_number,
        "WebSocket connection request"
    );

    let mut response = ws.on_upgrade(move |socket| {
        handle_socket(socket, state, user_id, connection_id, sequence_number, None)
    });

    // Match Mattermost behavior by echoing the requested protocol when present.
    if let Some(protocol) = requested_protocol {
        if let Ok(value) = protocol.parse() {
            response
                .headers_mut()
                .insert("Sec-WebSocket-Protocol", value);
        }
    }

    response
}

/// Handle the WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: Option<Uuid>,
    connection_id: Option<String>,
    sequence_number: Option<i64>,
    addr: Option<SocketAddr>,
) {
    // Handle authentication if not already done
    let user_id = match user_id {
        Some(id) => id,
        None => {
            // Try to authenticate via WebSocket message
            match authenticate_via_websocket(socket, &state).await {
                Some((id, sock)) => {
                    // Continue with authenticated socket
                    return run_connection(sock, state, id, connection_id, sequence_number, addr)
                        .await;
                }
                None => {
                    warn!(addr = ?addr, "WebSocket authentication failed");
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
                if let Some(challenge) = parse_authentication_challenge(&text) {
                    let valid_user = websocket_core::normalize_auth_token(&challenge.token)
                        .and_then(|t| validate_token(&t, &state.jwt_secret).ok())
                        .map(|c| c.claims.sub);

                    if let Some(user_id) = valid_user {
                        let resp = json!({
                            "status": "OK",
                            "seq_reply": challenge.seq_reply
                        });
                        let _ = socket.send(Message::Text(resp.to_string().into())).await;
                        return Some((user_id, socket));
                    }

                    let resp = json!({
                        "status": "FAIL",
                        "seq_reply": challenge.seq_reply,
                        "error": "Invalid token"
                    });
                    let _ = socket.send(Message::Text(resp.to_string().into())).await;
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
    addr: Option<SocketAddr>,
) {
    // Check connection limits
    if let Err(limit) = websocket_core::enforce_connection_limit(&state, user_id).await {
        warn!(
            user_id = %user_id,
            current = limit.current,
            max = limit.max,
            "Too many connections for user"
        );

        // Send close frame and return
        // Note: In axum 0.8, we can't easily split the socket, so we just drop it
        // The client will see the connection close
        return;
    }

    // Get or create connection store
    let store = state.connection_store.clone();
    let replay_store = store.clone();

    // Treat empty connection IDs as "not provided" (fresh connection),
    // matching Mattermost reliable websocket semantics.
    let requested_connection_id = connection_id.filter(|id| !id.is_empty());
    let is_resumption_attempt = requested_connection_id.is_some();

    // Create WebSocket actor with session resumption
    let (actor, missed_messages) = WebSocketActor::new(
        socket,
        store,
        user_id,
        requested_connection_id.clone(),
        sequence_number,
        addr,
    )
    .await;

    let actor_connection_id = actor.connection_id.clone();
    let is_resumed = !missed_messages.is_empty() || is_resumption_attempt;

    info!(
        connection_id = %actor_connection_id,
        user_id = %user_id,
        resumed = is_resumed,
        missed_count = missed_messages.len(),
        addr = ?addr,
        "WebSocket connection established"
    );

    // Get username
    let username = match websocket_core::fetch_username(&state, user_id).await {
        Ok(name) => name,
        Err(_) => {
            error!(user_id = %user_id, "Failed to get username");
            actor.close(close_codes::INTERNAL_ERROR, "User not found");
            return;
        }
    };

    // Add connection to hub
    let (hub_conn_id, mut hub_rx) = state.ws_hub.add_connection(user_id, username.clone()).await;

    websocket_core::initialize_connection_state(&state, user_id, true).await;

    // Send hello event. Mattermost reliable websocket clients reset their local sequence
    // to 0 whenever connection_id changes, so hello.seq must also be 0 in that case.
    // Only preserve requested sequence when we truly resumed the same connection_id.
    let requested_seq = sequence_number.unwrap_or(0).max(0);
    let resumed_same_connection = requested_connection_id
        .as_deref()
        .map(|id| id == actor_connection_id.as_str())
        .unwrap_or(false);
    let hello_seq = if resumed_same_connection {
        requested_seq
    } else {
        0
    };
    info!(
        connection_id = %actor_connection_id,
        requested_connection_id = ?requested_connection_id,
        requested_seq = requested_seq,
        hello_seq = hello_seq,
        resumed_same_connection = resumed_same_connection,
        "Prepared hello message"
    );
    let hello = mm::WebSocketMessage {
        seq: Some(hello_seq),
        event: "hello".to_string(),
        data: json!({
            "connection_id": actor_connection_id.clone(),
            "server_version": format!("rustchat-{}", env!("CARGO_PKG_VERSION")),
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
    let actor_clone = actor.clone();
    let replay_connection_id = actor_connection_id.clone();

    // Spawn task to forward hub messages to client
    let mut hub_forward_task = tokio::spawn(async move {
        loop {
            let msg_str = match hub_rx.recv().await {
                Ok(msg) => msg,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(
                        connection_id = %replay_connection_id,
                        skipped = skipped,
                        "Hub receiver lagged; dropping stale websocket messages"
                    );
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    debug!(
                        connection_id = %replay_connection_id,
                        "Hub receiver closed"
                    );
                    break;
                }
            };

            let envelope = match serde_json::from_str::<WsEnvelope>(&msg_str) {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        connection_id = %replay_connection_id,
                        error = %err,
                        "Failed to deserialize hub websocket envelope"
                    );
                    continue;
                }
            };

            let Some(mut mm_msg) = map_envelope_to_mm(&envelope) else {
                continue;
            };

            let replay_payload = json!({
                "event": mm_msg.event.clone(),
                "data": mm_msg.data.clone(),
                "broadcast": mm_msg.broadcast.clone(),
            });

            if let Some(seq) = replay_store.queue_message(&replay_connection_id, replay_payload) {
                mm_msg.seq = Some(seq);
            }

            if actor_clone.send(mm_msg).is_err() {
                break;
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
                        handle_client_text_message(
                            &state,
                            &actor,
                            user_id,
                            &username,
                            &actor_connection_id,
                            &text,
                        )
                        .await;
                    }
                    Some(WsEvent::BinaryReceived(bytes)) => {
                        handle_client_binary_message(
                            &state,
                            &actor,
                            user_id,
                            &username,
                            &actor_connection_id,
                            &bytes,
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

    // Calls websocket clients may disconnect without sending an explicit
    // `leave` action. Apply a short grace window for reconnect and then
    // best-effort participant cleanup if this exact session is still inactive.
    let disconnect_state = state.clone();
    let disconnect_connection_id = actor_connection_id.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(3)).await;
        let still_inactive = disconnect_state
            .connection_store
            .get_connection(&disconnect_connection_id)
            .map(|conn| !conn.is_active.load(Ordering::SeqCst))
            .unwrap_or(true);
        if still_inactive {
            calls_plugin::handle_ws_connection_closed(
                &disconnect_state,
                user_id,
                &disconnect_connection_id,
            )
            .await;
        }
    });

    // Remove from hub
    state.ws_hub.remove_connection(user_id, hub_conn_id).await;

    websocket_core::set_offline_if_last_connection(&state, user_id).await;

    info!(
        connection_id = %actor_connection_id,
        user_id = %user_id,
        "WebSocket connection ended"
    );
}

/// Handle a message from the client
async fn handle_client_text_message(
    state: &AppState,
    actor: &std::sync::Arc<WebSocketActor>,
    user_id: Uuid,
    username: &str,
    connection_id: &str,
    text: &str,
) {
    trace!(
        user_id = %user_id,
        connection_id = connection_id,
        text = %text,
        "Received client message"
    );

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        handle_client_value_message(state, actor, user_id, username, connection_id, &value).await;
    }

    let _ = websocket_core::handle_client_envelope_message(
        state,
        user_id,
        username,
        text,
        EnvelopeCommandOptions::V4,
    )
    .await;
}

async fn handle_client_binary_message(
    state: &AppState,
    actor: &std::sync::Arc<WebSocketActor>,
    user_id: Uuid,
    username: &str,
    connection_id: &str,
    bytes: &[u8],
) {
    if let Some(value) = decode_msgpack_value(bytes) {
        trace!(
            user_id = %user_id,
            connection_id = connection_id,
            "Received binary client message"
        );
        handle_client_value_message(state, actor, user_id, username, connection_id, &value).await;
    } else {
        warn!(
            user_id = %user_id,
            connection_id = connection_id,
            "Failed to decode binary websocket message as msgpack"
        );
    }
}

async fn handle_client_value_message(
    state: &AppState,
    actor: &std::sync::Arc<WebSocketActor>,
    user_id: Uuid,
    username: &str,
    connection_id: &str,
    value: &serde_json::Value,
) {
    let Some(action) = value.get("action").and_then(|v| v.as_str()) else {
        return;
    };

    if calls_plugin::handle_ws_action(state, user_id, connection_id, action, value.get("data"))
        .await
    {
        return;
    }

    if action == "ping" {
        let seq_reply = value.get("seq").cloned().unwrap_or(serde_json::Value::Null);
        let server_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or_default();
        let response = json!({
            "status": "OK",
            "seq_reply": seq_reply,
            "data": {
                "text": "pong",
                "version": format!("rustchat-{}", env!("CARGO_PKG_VERSION")),
                "server_time": server_time_ms,
                "node_id": ""
            }
        });

        if let Err(err) = actor.send_raw(response) {
            if err.contains("closing") {
                debug!(
                    user_id = %user_id,
                    connection_id = connection_id,
                    error = %err,
                    "Skipping ping response on closing websocket"
                );
            } else {
                warn!(
                    user_id = %user_id,
                    connection_id = connection_id,
                    error = %err,
                    "Failed to send ping response"
                );
            }
        }
    } else if action == "user_typing" {
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
    } else {
        trace!(action = %action, "Unknown action received");
    }
}

fn decode_msgpack_value(bytes: &[u8]) -> Option<serde_json::Value> {
    let mut idx = 0usize;
    decode_msgpack_at(bytes, &mut idx)
}

fn decode_msgpack_at(bytes: &[u8], idx: &mut usize) -> Option<serde_json::Value> {
    let marker = *bytes.get(*idx)?;
    *idx += 1;

    match marker {
        0x00..=0x7f => Some(serde_json::Value::from(marker as u64)),
        0xe0..=0xff => Some(serde_json::Value::from((marker as i8) as i64)),
        0xc0 => Some(serde_json::Value::Null),
        0xc2 => Some(serde_json::Value::Bool(false)),
        0xc3 => Some(serde_json::Value::Bool(true)),
        0xcc => Some(serde_json::Value::from(read_u8(bytes, idx)? as u64)),
        0xcd => Some(serde_json::Value::from(read_u16(bytes, idx)? as u64)),
        0xce => Some(serde_json::Value::from(read_u32(bytes, idx)? as u64)),
        0xd0 => Some(serde_json::Value::from(read_i8(bytes, idx)? as i64)),
        0xd1 => Some(serde_json::Value::from(read_i16(bytes, idx)? as i64)),
        0xd2 => Some(serde_json::Value::from(read_i32(bytes, idx)? as i64)),
        0xa0..=0xbf => {
            let len = (marker & 0x1f) as usize;
            decode_str(bytes, idx, len)
        }
        0xd9 => {
            let len = read_u8(bytes, idx)? as usize;
            decode_str(bytes, idx, len)
        }
        0xda => {
            let len = read_u16(bytes, idx)? as usize;
            decode_str(bytes, idx, len)
        }
        0xdb => {
            let len = read_u32(bytes, idx)? as usize;
            decode_str(bytes, idx, len)
        }
        0xc4 => {
            let len = read_u8(bytes, idx)? as usize;
            decode_bin_as_json_array(bytes, idx, len)
        }
        0xc5 => {
            let len = read_u16(bytes, idx)? as usize;
            decode_bin_as_json_array(bytes, idx, len)
        }
        0xc6 => {
            let len = read_u32(bytes, idx)? as usize;
            decode_bin_as_json_array(bytes, idx, len)
        }
        0x90..=0x9f => decode_array(bytes, idx, (marker & 0x0f) as usize),
        0xdc => {
            let len = read_u16(bytes, idx)? as usize;
            decode_array(bytes, idx, len)
        }
        0xdd => {
            let len = read_u32(bytes, idx)? as usize;
            decode_array(bytes, idx, len)
        }
        0x80..=0x8f => decode_map(bytes, idx, (marker & 0x0f) as usize),
        0xde => {
            let len = read_u16(bytes, idx)? as usize;
            decode_map(bytes, idx, len)
        }
        0xdf => {
            let len = read_u32(bytes, idx)? as usize;
            decode_map(bytes, idx, len)
        }
        _ => None,
    }
}

fn decode_array(bytes: &[u8], idx: &mut usize, len: usize) -> Option<serde_json::Value> {
    let mut items = Vec::with_capacity(len);
    for _ in 0..len {
        items.push(decode_msgpack_at(bytes, idx)?);
    }
    Some(serde_json::Value::Array(items))
}

fn decode_map(bytes: &[u8], idx: &mut usize, len: usize) -> Option<serde_json::Value> {
    let mut map = serde_json::Map::with_capacity(len);
    for _ in 0..len {
        let key = decode_msgpack_at(bytes, idx)?.as_str()?.to_string();
        let value = decode_msgpack_at(bytes, idx)?;
        map.insert(key, value);
    }
    Some(serde_json::Value::Object(map))
}

fn decode_str(bytes: &[u8], idx: &mut usize, len: usize) -> Option<serde_json::Value> {
    let slice = read_exact(bytes, idx, len)?;
    let text = std::str::from_utf8(slice).ok()?.to_string();
    Some(serde_json::Value::String(text))
}

fn decode_bin_as_json_array(
    bytes: &[u8],
    idx: &mut usize,
    len: usize,
) -> Option<serde_json::Value> {
    let slice = read_exact(bytes, idx, len)?;
    Some(serde_json::Value::Array(
        slice
            .iter()
            .map(|b| serde_json::Value::from(*b as u64))
            .collect(),
    ))
}

fn read_exact<'a>(bytes: &'a [u8], idx: &mut usize, len: usize) -> Option<&'a [u8]> {
    let end = idx.checked_add(len)?;
    let slice = bytes.get(*idx..end)?;
    *idx = end;
    Some(slice)
}

fn read_u8(bytes: &[u8], idx: &mut usize) -> Option<u8> {
    let value = *bytes.get(*idx)?;
    *idx += 1;
    Some(value)
}

fn read_i8(bytes: &[u8], idx: &mut usize) -> Option<i8> {
    read_u8(bytes, idx).map(|v| v as i8)
}

fn read_u16(bytes: &[u8], idx: &mut usize) -> Option<u16> {
    let arr: [u8; 2] = read_exact(bytes, idx, 2)?.try_into().ok()?;
    Some(u16::from_be_bytes(arr))
}

fn read_i16(bytes: &[u8], idx: &mut usize) -> Option<i16> {
    let arr: [u8; 2] = read_exact(bytes, idx, 2)?.try_into().ok()?;
    Some(i16::from_be_bytes(arr))
}

fn read_u32(bytes: &[u8], idx: &mut usize) -> Option<u32> {
    let arr: [u8; 4] = read_exact(bytes, idx, 4)?.try_into().ok()?;
    Some(u32::from_be_bytes(arr))
}

fn read_i32(bytes: &[u8], idx: &mut usize) -> Option<i32> {
    let arr: [u8; 4] = read_exact(bytes, idx, 4)?.try_into().ok()?;
    Some(i32::from_be_bytes(arr))
}

#[derive(Debug, Clone)]
struct AuthenticationChallenge {
    token: String,
    seq_reply: serde_json::Value,
}

fn parse_authentication_challenge(text: &str) -> Option<AuthenticationChallenge> {
    let value = serde_json::from_str::<serde_json::Value>(text).ok()?;
    if value.get("action").and_then(|v| v.as_str()) != Some("authentication_challenge") {
        return None;
    }
    let token = value
        .get("data")
        .and_then(|v| v.get("token"))
        .and_then(|v| v.as_str())?
        .to_string();
    let seq_reply = value.get("seq").cloned().unwrap_or(serde_json::Value::Null);
    Some(AuthenticationChallenge { token, seq_reply })
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
            if let Ok(typing) =
                serde_json::from_value::<crate::realtime::TypingEvent>(env.data.clone())
            {
                let parent_id = typing.thread_root_id.map(encode_mm_id).unwrap_or_default();
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
                    emoji_name: crate::mattermost_compat::emoji_data::get_short_name_for_emoji(
                        &reaction.emoji_name,
                    ),
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
                    emoji_name: crate::mattermost_compat::emoji_data::get_short_name_for_emoji(
                        &reaction.emoji_name,
                    ),
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
        "user_updated" | "user_presence" => {
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
        event_name if event_name.starts_with("custom_") => Some(mm::WebSocketMessage {
            seq,
            event: event_name.to_string(),
            data: env.data.clone(),
            broadcast: map_broadcast(env.broadcast.as_ref()),
        }),
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

#[cfg(test)]
mod tests {
    use super::map_envelope_to_mm;
    use super::parse_authentication_challenge;
    use crate::realtime::{WsBroadcast, WsEnvelope};
    use uuid::Uuid;

    #[test]
    fn parse_authentication_challenge_accepts_valid_payload() {
        let msg = r#"{
            "action":"authentication_challenge",
            "seq":7,
            "data":{"token":"abc.def.ghi"}
        }"#;

        let challenge = parse_authentication_challenge(msg).expect("challenge should parse");
        assert_eq!(challenge.token, "abc.def.ghi");
        assert_eq!(challenge.seq_reply, serde_json::json!(7));
    }

    #[test]
    fn parse_authentication_challenge_rejects_non_challenge() {
        let msg = r#"{"action":"ping","data":{"token":"abc.def.ghi"}}"#;
        assert!(parse_authentication_challenge(msg).is_none());
    }

    #[test]
    fn parse_authentication_challenge_requires_token() {
        let msg = r#"{"action":"authentication_challenge","seq":3,"data":{}}"#;
        assert!(parse_authentication_challenge(msg).is_none());
    }

    #[test]
    fn map_envelope_to_mm_passes_custom_events() {
        let channel_id = Uuid::new_v4();
        let env = WsEnvelope {
            msg_type: "event".to_string(),
            event: "custom_com.mattermost.calls_signal".to_string(),
            seq: None,
            channel_id: Some(channel_id),
            data: serde_json::json!({
                "signal": { "type": "answer", "sdp": "v=0" }
            }),
            broadcast: Some(WsBroadcast {
                channel_id: Some(channel_id),
                team_id: None,
                user_id: None,
                exclude_user_id: None,
            }),
        };

        let mapped = map_envelope_to_mm(&env).expect("custom event should map");
        assert_eq!(mapped.event, "custom_com.mattermost.calls_signal");
        assert_eq!(mapped.data["signal"]["type"], "answer");
    }
}
