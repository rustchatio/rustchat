//! WebSocket API endpoint

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Query, State,
    },
    http::HeaderMap,
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::time::Duration;

use super::AppState;
use crate::api::websocket_core::{self, EnvelopeCommandOptions};
use crate::middleware::rate_limit::{self, RateLimitConfig};
use crate::realtime::WsEnvelope;

/// Build WebSocket routes
pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    // Note: token in query is no longer supported for security
    // Use Authorization header or Sec-WebSocket-Protocol instead
    _legacy_token: Option<String>,
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(state): State<AppState>,
    Query(_query): Query<WsQuery>,
    headers: HeaderMap,
) -> Response {
    // Check rate limiting if enabled
    if state.config.security.rate_limit_enabled {
        let config = RateLimitConfig::websocket_per_minute(
            state.config.security.rate_limit_ws_per_minute,
        );
        let ip = rate_limit::extract_client_ip(&addr, &headers);
        
        match rate_limit::check_rate_limit(&state.redis, &config, &ip).await {
            Ok(rate_result) if !rate_result.allowed => {
                tracing::warn!(
                    ip = %ip,
                    "Rate limit exceeded for WebSocket connection"
                );
                return Response::builder()
                    .status(429)
                    .body("Too many connection attempts. Please try again later.".into())
                    .unwrap();
            }
            Err(e) => {
                tracing::error!("Rate limit check failed: {}", e);
                // Continue anyway - don't block connections due to Redis issues
            }
            _ => {}
        }
    }

    let requested_protocol = websocket_core::requested_protocol(&headers);
    
    // ALWAYS use secure mode - never accept token from query params
    let token = websocket_core::resolve_auth_token_secure(
        &headers,
        requested_protocol.as_deref(),
    );

    tracing::info!(
        "WS Handshake - Token present: {}, Protocol: {:?}",
        token.is_some(),
        requested_protocol
    );

    let user_id = match websocket_core::validate_user_id(token.as_deref(), &state) {
        Some(user_id) => user_id,
        None => {
            tracing::warn!("WS Handshake failed: Invalid token");
            return Response::builder()
                .status(401)
                .body("Unauthorized".into())
                .unwrap();
        }
    };

    if websocket_core::enforce_connection_limit(&state, user_id)
        .await
        .is_err()
    {
        return Response::builder()
            .status(429)
            .body("Too many connections".into())
            .unwrap();
    }

    let username = websocket_core::fetch_username(&state, user_id)
        .await
        .unwrap_or_else(|_| "Unknown".to_string());

    let mut response = ws.on_upgrade(move |socket| handle_socket(socket, user_id, username, state));

    // Spec compliance: if client requested a protocol, we MUST return it
    if let Some(p) = requested_protocol {
        if let Ok(header_val) = p.parse() {
            response
                .headers_mut()
                .insert("Sec-WebSocket-Protocol", header_val);
        }
    }

    response
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, user_id: uuid::Uuid, username: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Add connection to hub
    let (connection_id, mut rx) = state.ws_hub.add_connection(user_id, username.clone()).await;
    let connection_id_str = connection_id.to_string();
    websocket_core::register_presence_connection(&state, user_id, &connection_id_str).await;

    websocket_core::initialize_connection_state(&state, user_id, true).await;

    let heartbeat_state = state.clone();
    let heartbeat_connection_id = connection_id_str.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(20));
        loop {
            interval.tick().await;
            websocket_core::heartbeat_presence_connection(
                &heartbeat_state,
                user_id,
                &heartbeat_connection_id,
            )
            .await;
        }
    });

    // Send hello message with connection_id for reliable WebSocket support
    let connection_uuid = uuid::Uuid::new_v4();
    let hello = WsEnvelope::hello(
        connection_uuid,
        &format!("rustchat-{}", env!("CARGO_PKG_VERSION")),
    );
    if let Ok(msg) = serde_json::to_string(&hello) {
        let _ = sender.send(Message::Text(msg.into())).await;
    }

    // Spawn task to forward hub messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from client
    let state_for_receive = state.clone();
    let username_for_receive = username.clone();
    let connection_id_for_receive = connection_id_str.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(action) = value.get("action").and_then(|v| v.as_str()) {
                            if crate::api::v4::calls_plugin::handle_ws_action(
                                &state_for_receive,
                                user_id,
                                &connection_id_for_receive,
                                action,
                                value.get("data"),
                            )
                            .await
                            {
                                continue;
                            }
                        }
                    }

                    if !websocket_core::handle_client_envelope_message(
                        &state_for_receive,
                        user_id,
                        &username_for_receive,
                        &text,
                        EnvelopeCommandOptions::V1,
                    )
                    .await
                    {
                        tracing::debug!("Failed to parse ClientEnvelope: {}", text);
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    heartbeat_task.abort();

    // Cleanup
    state.ws_hub.remove_connection(user_id, connection_id).await;
    websocket_core::handle_disconnect(&state, user_id, &connection_id_str).await;
}
