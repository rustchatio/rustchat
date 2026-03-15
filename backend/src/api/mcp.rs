//! MCP API Endpoints
//!
//! Provides HTTP endpoints for the Model Context Protocol.

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::mcp::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::mcp::security::{McpApprovalStore, McpAuditLog, McpSecurityContext};
use crate::mcp::server::McpServer;

/// Build MCP routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mcp", post(mcp_http_handler))
        .route("/mcp/ws", get(mcp_websocket_handler))
}

/// MCP HTTP handler - accepts JSON-RPC 2.0 requests
async fn mcp_http_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<Value>,
) -> Result<Json<Value>, AppError> {
    // Check if MCP is enabled
    if !state.config.mcp.enabled {
        return Err(AppError::BadRequest("MCP is disabled".to_string()));
    }

    // Parse JSON-RPC request
    let json_rpc_request: JsonRpcRequest = serde_json::from_value(request)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON-RPC: {}", e)))?;

    // Create security context
    let approval_store = McpApprovalStore::new(state.redis.clone());
    let audit_log = McpAuditLog::new(state.redis.clone(), state.config.mcp.enable_audit_logging);
    let security_context = McpSecurityContext::new(auth.user_id, approval_store, audit_log)
        .with_db(state.db.clone())
        .with_redis(state.redis.clone());

    // Create MCP server
    let server = McpServer::new(state.config.mcp.clone());

    // Handle the request
    let response = server
        .handle_request(json_rpc_request, &security_context)
        .await;

    Ok(Json(serde_json::to_value(response).map_err(|e| {
        AppError::Internal(format!("Serialization error: {}", e))
    })?))
}

/// MCP WebSocket handler for streaming
async fn mcp_websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    auth: AuthUser,
) -> Response {
    // Check if MCP is enabled
    if !state.config.mcp.enabled {
        return (StatusCode::SERVICE_UNAVAILABLE, "MCP is disabled").into_response();
    }

    ws.on_upgrade(move |socket| handle_mcp_websocket(socket, state, auth))
}

/// Handle MCP WebSocket connection
async fn handle_mcp_websocket(
    mut socket: axum::extract::ws::WebSocket,
    state: AppState,
    auth: AuthUser,
) {
    use axum::extract::ws::Message;

    // Create MCP server
    let server = McpServer::new(state.config.mcp.clone());

    // Create security context
    let approval_store = McpApprovalStore::new(state.redis.clone());
    let audit_log = McpAuditLog::new(state.redis.clone(), state.config.mcp.enable_audit_logging);
    let security_context = McpSecurityContext::new(auth.user_id, approval_store, audit_log)
        .with_db(state.db.clone())
        .with_redis(state.redis.clone());

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse JSON-RPC request
                match serde_json::from_str::<JsonRpcRequest>(&text) {
                    Ok(request) => {
                        // Handle request
                        let response = server.handle_request(request, &security_context).await;
                        let response_json = match serde_json::to_string(&response) {
                            Ok(json) => json,
                            Err(e) => {
                                let error_response = JsonRpcResponse::error(
                                    None,
                                    JsonRpcError::internal_error().with_data(serde_json::json!({
                                        "error": e.to_string()
                                    })),
                                );
                                serde_json::to_string(&error_response).unwrap_or_default()
                            }
                        };

                        if socket
                            .send(Message::Text(response_json.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        let error_response = JsonRpcResponse::error(
                            None,
                            JsonRpcError::parse_error().with_data(serde_json::json!({
                                "error": e.to_string()
                            })),
                        );
                        let response_json =
                            serde_json::to_string(&error_response).unwrap_or_default();
                        if socket
                            .send(Message::Text(response_json.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
}
