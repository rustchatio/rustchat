//! WebSocket API endpoint

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::HeaderMap,
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;

use super::AppState;
use crate::api::websocket_core::{self, EnvelopeCommandOptions};
use crate::realtime::WsEnvelope;

/// Build WebSocket routes
pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
) -> Response {
    let requested_protocol = websocket_core::requested_protocol(&headers);
    let token = websocket_core::resolve_auth_token(
        query.token.as_deref(),
        &headers,
        requested_protocol.as_deref(),
        true,
    );

    tracing::info!(
        "WS Handshake - Token present: {}, Protocol: {:?}",
        token.is_some(),
        requested_protocol
    );

    let user_id = match websocket_core::validate_user_id(token.as_deref(), &state.jwt_secret) {
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
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
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

    // Cleanup
    state.ws_hub.remove_connection(user_id, connection_id).await;
    websocket_core::handle_disconnect(&state, user_id, &connection_id_str).await;
}
