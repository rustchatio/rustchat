//! MCP Approval API Endpoints
//!
//! Provides endpoints for users to approve/deny MCP tool invocations.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::error::{ApiResult, AppError};
use crate::mcp::security::{McpApprovalStore, PendingApproval};

/// Build MCP approval routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mcp/approvals", get(list_pending_approvals))
        .route("/mcp/approvals/{id}/approve", post(approve_request))
        .route("/mcp/approvals/{id}/deny", post(deny_request))
}

/// List pending approval requests for the authenticated user
async fn list_pending_approvals(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<PendingApprovalsResponse>> {
    // Check if MCP is enabled
    if !state.config.mcp.enabled {
        return Err(AppError::BadRequest("MCP is disabled".to_string()));
    }

    let approval_store = McpApprovalStore::new(state.redis.clone());
    let pending = approval_store
        .list_pending_for_user(auth.user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch approvals: {}", e)))?;

    Ok(Json(PendingApprovalsResponse {
        approvals: pending,
        total: 0,
    }))
}

/// Approve a pending MCP request
async fn approve_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(approval_id): Path<String>,
) -> ApiResult<Json<ApprovalResponse>> {
    // Check if MCP is enabled
    if !state.config.mcp.enabled {
        return Err(AppError::BadRequest("MCP is disabled".to_string()));
    }

    let approval_store = McpApprovalStore::new(state.redis.clone());

    // Get the approval to verify ownership
    let approval = approval_store
        .get_approval(&approval_id)
        .await
        .map_err(|e| AppError::NotFound(format!("Approval not found: {}", e)))?;

    match approval {
        Some(approval) if approval.user_id == auth.user_id => {
            let approved = approval_store
                .approve(&approval_id)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to approve: {}", e)))?;

            Ok(Json(ApprovalResponse {
                id: approved.id,
                status: "approved".to_string(),
                tool_name: approved.tool_name,
            }))
        }
        Some(_) => Err(AppError::Forbidden(
            "You can only approve your own requests".to_string(),
        )),
        None => Err(AppError::NotFound("Approval request not found".to_string())),
    }
}

/// Deny a pending MCP request
async fn deny_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(approval_id): Path<String>,
) -> ApiResult<Json<ApprovalResponse>> {
    // Check if MCP is enabled
    if !state.config.mcp.enabled {
        return Err(AppError::BadRequest("MCP is disabled".to_string()));
    }

    let approval_store = McpApprovalStore::new(state.redis.clone());

    // Get the approval to verify ownership
    let approval = approval_store
        .get_approval(&approval_id)
        .await
        .map_err(|e| AppError::NotFound(format!("Approval not found: {}", e)))?;

    match approval {
        Some(approval) if approval.user_id == auth.user_id => {
            let denied = approval_store
                .deny(&approval_id)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to deny: {}", e)))?;

            Ok(Json(ApprovalResponse {
                id: denied.id,
                status: "denied".to_string(),
                tool_name: denied.tool_name,
            }))
        }
        Some(_) => Err(AppError::Forbidden(
            "You can only deny your own requests".to_string(),
        )),
        None => Err(AppError::NotFound("Approval request not found".to_string())),
    }
}

/// Response for listing pending approvals
#[derive(Debug, Serialize)]
pub struct PendingApprovalsResponse {
    pub approvals: Vec<PendingApproval>,
    pub total: usize,
}

/// Response for approve/deny actions
#[derive(Debug, Serialize)]
pub struct ApprovalResponse {
    pub id: String,
    pub status: String,
    pub tool_name: String,
}

#[cfg(test)]
mod tests {
    // Note: These tests would require a full integration test setup
    // with a running server and database. The actual handler logic
    // is tested indirectly through the MCP module tests.
}
