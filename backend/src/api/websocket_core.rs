//! Shared websocket internals used by both `/api/v1/ws` and `/api/v4/websocket`.
//!
//! This module centralizes behavior that must stay consistent across websocket
//! entry points: auth token normalization, connection limits, subscription
//! bootstrap, presence lifecycle, and shared command handling.

use axum::http::HeaderMap;
use chrono::Utc;
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::validate_token;
use crate::realtime::{
    ClientEnvelope, EventType, TypingCommandData, TypingEvent, WsBroadcast,
    WsEnvelope,
};

#[derive(Debug, Clone, Copy)]
pub struct ConnectionLimitExceeded {
    pub current: usize,
    pub max: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvelopeCommandOptions {
    pub allow_send_message: bool,
    pub emit_unknown_error: bool,
    pub acknowledge_unsubscribe: bool,
}

impl EnvelopeCommandOptions {
    pub const V1: Self = Self {
        allow_send_message: true,
        emit_unknown_error: true,
        acknowledge_unsubscribe: true,
    };

    pub const V4: Self = Self {
        allow_send_message: false,
        emit_unknown_error: false,
        acknowledge_unsubscribe: false,
    };
}

pub async fn get_max_simultaneous_connections(state: &AppState) -> usize {
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

pub async fn enforce_connection_limit(
    state: &AppState,
    user_id: Uuid,
) -> Result<(), ConnectionLimitExceeded> {
    let max = get_max_simultaneous_connections(state).await;
    let current = state.ws_hub.user_connection_count(user_id).await;
    if current >= max {
        return Err(ConnectionLimitExceeded { current, max });
    }
    Ok(())
}

pub async fn fetch_username(state: &AppState, user_id: Uuid) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
}

pub fn requested_protocol(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Sec-WebSocket-Protocol")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
}

pub fn normalize_auth_token(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("undefined") {
        return None;
    }

    let stripped = trimmed
        .strip_prefix("Bearer ")
        .or_else(|| trimmed.strip_prefix("Token "))
        .unwrap_or(trimmed)
        .trim();

    if stripped.is_empty() || stripped.eq_ignore_ascii_case("undefined") {
        return None;
    }

    Some(stripped.to_string())
}

pub fn resolve_auth_token(
    query_token: Option<&str>,
    headers: &HeaderMap,
    protocol_token: Option<&str>,
    allow_protocol_fallback: bool,
) -> Option<String> {
    if let Some(token) = query_token.and_then(normalize_auth_token) {
        return Some(token);
    }

    if let Some(auth_header) = headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = normalize_auth_token(auth_header) {
            return Some(token);
        }
    }

    if allow_protocol_fallback {
        if let Some(protocol) = protocol_token.and_then(normalize_auth_token) {
            // Keep v1 behavior: only treat the protocol field as token when it
            // looks token-like and not a normal short protocol name.
            if protocol.len() > 20 || protocol.contains('.') {
                return Some(protocol);
            }
        }
    }

    None
}

pub fn validate_user_id(token: Option<&str>, jwt_secret: &str) -> Option<Uuid> {
    token.and_then(|t| validate_token(t, jwt_secret).ok().map(|c| c.claims.sub))
}

pub async fn initialize_connection_state(
    state: &AppState,
    user_id: Uuid,
    subscribe_channels: bool,
) {
    subscribe_default_scopes(state, user_id, subscribe_channels).await;
    persist_presence_and_broadcast(state, user_id, "online", false).await;
}

pub async fn set_offline_if_last_connection(state: &AppState, user_id: Uuid) {
    if state.ws_hub.user_connection_count(user_id).await > 0 {
        return;
    }
    persist_presence_and_broadcast(state, user_id, "offline", false).await;
}

pub async fn persist_presence_and_broadcast(
    state: &AppState,
    user_id: Uuid,
    status: &str,
    manual: bool,
) {
    let now = Utc::now();

    // Update user presence and last activity in database
    let _ = sqlx::query("UPDATE users SET presence = $1, last_login_at = $2 WHERE id = $3")
        .bind(status)
        .bind(now)
        .bind(user_id)
        .execute(&state.db)
        .await;

    state.ws_hub.set_presence(user_id, status.to_string()).await;

    // Create status change event with full broadcast info
    let evt = WsEnvelope::event(
        EventType::UserPresence,
        serde_json::json!({
            "user_id": user_id,
            "status": status,
            "manual": manual,
            "last_activity_at": now.timestamp_millis()
        }),
        None,
    );
    // No .with_broadcast(...) filter means it broadcasts to ALL connected users
    // which is necessary for everyone else to see this user's status change.
    
    tracing::debug!(
        user_id = %user_id,
        status = %status,
        "Broadcasting status change event"
    );
    
    state.ws_hub.broadcast(evt).await;
}

pub async fn handle_client_envelope_message(
    state: &AppState,
    user_id: Uuid,
    username: &str,
    text: &str,
    options: EnvelopeCommandOptions,
) -> bool {
    let envelope = match serde_json::from_str::<ClientEnvelope>(text) {
        Ok(envelope) => envelope,
        Err(_) => return false,
    };

    handle_client_envelope(state, user_id, username, envelope, options).await;
    true
}

pub async fn handle_client_envelope(
    state: &AppState,
    user_id: Uuid,
    username: &str,
    envelope: ClientEnvelope,
    options: EnvelopeCommandOptions,
) {
    match envelope.event.as_str() {
        "send_message" if options.allow_send_message => {
            if let Some(channel_id) = envelope.channel_id {
                if let Ok(input) =
                    serde_json::from_value::<crate::models::CreatePost>(envelope.data)
                {
                    if let Err(e) = crate::services::posts::create_post(
                        state,
                        user_id,
                        channel_id,
                        input,
                        envelope.client_msg_id,
                    )
                    .await
                    {
                        send_direct(
                            state,
                            user_id,
                            WsEnvelope::error(&format!("Failed to send message: {}", e)),
                        )
                        .await;
                    }
                }
            }
        }
        "subscribe_channel" => {
            if let Some(channel_id) = envelope.channel_id {
                state.ws_hub.subscribe_channel(user_id, channel_id).await;
                let evt = WsEnvelope::event(
                    EventType::ChannelSubscribed,
                    serde_json::json!({ "channel_id": channel_id }),
                    None,
                );
                send_direct(state, user_id, evt).await;
            }
        }
        "unsubscribe_channel" => {
            if let Some(channel_id) = envelope.channel_id {
                state.ws_hub.unsubscribe_channel(user_id, channel_id).await;
                if options.acknowledge_unsubscribe {
                    let evt = WsEnvelope::event(
                        EventType::ChannelUnsubscribed,
                        serde_json::json!({ "channel_id": channel_id }),
                        None,
                    );
                    send_direct(state, user_id, evt).await;
                }
            }
        }
        "typing" | "typing_start" => {
            if let Some(channel_id) = envelope.channel_id {
                let thread_root_id = serde_json::from_value::<TypingCommandData>(envelope.data)
                    .ok()
                    .and_then(|v| v.thread_root_id);

                let event = WsEnvelope::event(
                    EventType::UserTyping,
                    TypingEvent {
                        user_id,
                        display_name: username.to_string(),
                        thread_root_id,
                    },
                    Some(channel_id),
                )
                .with_broadcast(WsBroadcast {
                    channel_id: Some(channel_id),
                    user_id: None,
                    team_id: None,
                    exclude_user_id: Some(user_id),
                });
                state.ws_hub.broadcast(event).await;
            }
        }
        "typing_stop" => {
            if let Some(channel_id) = envelope.channel_id {
                let thread_root_id = serde_json::from_value::<TypingCommandData>(envelope.data)
                    .ok()
                    .and_then(|v| v.thread_root_id);

                let event = WsEnvelope::event(
                    EventType::UserTypingStop,
                    TypingEvent {
                        user_id,
                        display_name: username.to_string(),
                        thread_root_id,
                    },
                    Some(channel_id),
                )
                .with_broadcast(WsBroadcast {
                    channel_id: Some(channel_id),
                    user_id: None,
                    team_id: None,
                    exclude_user_id: Some(user_id),
                });
                state.ws_hub.broadcast(event).await;
            }
        }
        "presence" => {
            if let Some(status) = envelope.data.get("status").and_then(|v| v.as_str()) {
                persist_presence_and_broadcast(state, user_id, status, true).await;
            }
        }
        "ping" => {
            // Extract seq from the envelope for the response
            let seq = envelope.seq;
            send_direct(state, user_id, WsEnvelope::pong(seq)).await;
        }
        _ => {
            if options.emit_unknown_error {
                send_direct(state, user_id, WsEnvelope::error("Unknown command")).await;
            }
        }
    }
}

async fn send_direct(state: &AppState, user_id: Uuid, envelope: WsEnvelope) {
    state
        .ws_hub
        .broadcast(envelope.with_broadcast(WsBroadcast {
            user_id: Some(user_id),
            channel_id: None,
            team_id: None,
            exclude_user_id: None,
        }))
        .await;
}

async fn subscribe_default_scopes(state: &AppState, user_id: Uuid, subscribe_channels: bool) {
    let teams =
        sqlx::query_scalar::<_, Uuid>("SELECT team_id FROM team_members WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

    for team_id in teams {
        state.ws_hub.subscribe_team(user_id, team_id).await;
    }

    if !subscribe_channels {
        return;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn normalize_token_handles_prefixes() {
        assert_eq!(
            normalize_auth_token("Bearer abc.def.ghi"),
            Some("abc.def.ghi".to_string())
        );
        assert_eq!(
            normalize_auth_token("Token abc.def.ghi"),
            Some("abc.def.ghi".to_string())
        );
        assert_eq!(normalize_auth_token("   "), None);
        assert_eq!(normalize_auth_token("undefined"), None);
    }

    #[test]
    fn resolve_token_prefers_query_then_authorization() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str("Bearer auth-token").unwrap(),
        );

        let token = resolve_auth_token(Some("query-token"), &headers, None, false);
        assert_eq!(token.as_deref(), Some("query-token"));
    }

    #[test]
    fn resolve_token_can_use_protocol_fallback() {
        let headers = HeaderMap::new();
        let token = resolve_auth_token(None, &headers, Some("abc.def.ghi"), true);
        assert_eq!(token.as_deref(), Some("abc.def.ghi"));
    }
}
