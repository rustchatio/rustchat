//! MCP Message Tools
//!
//! Tools for posting messages to channels.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::{optional_uuid_arg, require_string_arg, require_uuid_arg};
use crate::mcp::capabilities::schemas;
use crate::mcp::capabilities::{ToolDefinition, ToolSchema};
use crate::mcp::protocol::{ToolCallResult, McpError};
use crate::mcp::security::McpSecurityContext;

/// Create the post_message tool definition
pub fn create_post_message_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("channel_id".to_string(), schemas::channel_id());
    properties.insert("message".to_string(), schemas::message_content());
    properties.insert("thread_id".to_string(), schemas::thread_id());

    let schema = ToolSchema::object(
        properties,
        vec!["channel_id".to_string(), "message".to_string()],
    )
    .with_description("Post a message to a channel");

    // This tool requires user approval
    ToolDefinition::new(
        "post_message",
        "Post a message to a channel. Requires user approval before execution.",
        schema,
    )
    .with_approval()
}

/// Output format for posted message
#[derive(Debug, Serialize)]
struct PostedMessage {
    post_id: Uuid,
    channel_id: Uuid,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_id: Option<Uuid>,
}

/// Execute post_message tool
pub async fn execute_post_message(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments.ok_or_else(|| McpError::InvalidToolParameters("Missing arguments".to_string()))?;
    let channel_id = require_uuid_arg(&args, "channel_id")?;
    let message = require_string_arg(&args, "message")?;
    let thread_id = optional_uuid_arg(&args, "thread_id");

    // Validate message length
    if message.len() > 4000 {
        return Err(McpError::InvalidToolParameters(
            "Message exceeds maximum length of 4000 characters".to_string(),
        ));
    }

    // Verify user has access to post in the channel
    if let Some(ref db) = context.db {
        let can_post: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM channel_members cm
                JOIN channels c ON cm.channel_id = c.id
                WHERE cm.channel_id = $1 
                AND cm.user_id = $2
                AND c.is_archived = false
            )
            "#
        )
        .bind(channel_id)
        .bind(context.user_id)
        .fetch_one(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?;

        if !can_post {
            return Err(McpError::ScopeViolation(
                "User cannot post in this channel".to_string(),
            ));
        }

        // Insert the post
        let post_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO posts (
                id, channel_id, user_id, message, thread_id, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#
        )
        .bind(post_id)
        .bind(channel_id)
        .bind(context.user_id)
        .bind(&message)
        .bind(thread_id)
        .bind(now)
        .execute(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Failed to post message: {}", e)))?;

        // Log to audit log
        context
            .audit_log
            .log_invocation(
                context.user_id,
                "post_message",
                &serde_json::json!({"channel_id": channel_id, "message_length": message.len()}),
                crate::mcp::security::AuditResult::Success,
                0,
                None,
            )
            .await
            .map_err(|e| McpError::Internal(format!("Audit log error: {}", e)))?;

        let result = PostedMessage {
            post_id,
            channel_id,
            created_at: now.to_rfc3339(),
            thread_id,
        };

        ToolCallResult::success_json(result)
            .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
    } else {
        // Mock response for testing
        let result = PostedMessage {
            post_id: Uuid::new_v4(),
            channel_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            thread_id,
        };

        ToolCallResult::success_json(result)
            .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::security::{McpApprovalStore, McpAuditLog};

    fn create_test_context() -> McpSecurityContext {
        McpSecurityContext::new(
            Uuid::new_v4(),
            McpApprovalStore::new_mock(),
            McpAuditLog::new_mock(),
        )
    }

    #[tokio::test]
    async fn test_post_message_missing_args() {
        let context = create_test_context();
        let result = execute_post_message(Some(serde_json::json!({})), &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_post_message_too_long() {
        let context = create_test_context();
        let long_message = "a".repeat(4001);

        let result = execute_post_message(
            Some(serde_json::json!({
                "channel_id": Uuid::new_v4(),
                "message": long_message
            })),
            &context,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_post_message_success() {
        let context = create_test_context();

        let result = execute_post_message(
            Some(serde_json::json!({
                "channel_id": Uuid::new_v4(),
                "message": "Hello, world!"
            })),
            &context,
        )
        .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_post_message_with_thread() {
        let context = create_test_context();
        let thread_id = Uuid::new_v4();

        let result = execute_post_message(
            Some(serde_json::json!({
                "channel_id": Uuid::new_v4(),
                "message": "Reply in thread",
                "thread_id": thread_id
            })),
            &context,
        )
        .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_error);
    }
}
