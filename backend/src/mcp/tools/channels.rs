//! MCP Channel Tools
//!
//! Tools for listing channels, getting channel history, and searching messages.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    optional_int_arg, optional_string_arg, optional_uuid_arg, require_uuid_arg,
};
use crate::mcp::capabilities::schemas;
use crate::mcp::capabilities::{ToolDefinition, ToolSchema};
use crate::mcp::protocol::{ToolCallResult, McpError};
use crate::mcp::security::McpSecurityContext;

/// Create the list_channels tool definition
pub fn create_list_channels_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert(
        "team_id".to_string(),
        schemas::team_id(),
    );

    let schema = ToolSchema::object(
        properties,
        vec![], // team_id is optional
    )
    .with_description("List channels accessible to the user");

    ToolDefinition::new(
        "list_channels",
        "List channels that the user has access to. Returns channel names, types, and member counts.",
        schema,
    )
}

/// Create the get_channel_history tool definition
pub fn create_get_channel_history_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("channel_id".to_string(), schemas::channel_id());
    properties.insert(
        "limit".to_string(),
        schemas::limit(50),
    );

    let schema = ToolSchema::object(
        properties,
        vec!["channel_id".to_string()],
    )
    .with_description("Get recent messages from a channel");

    ToolDefinition::new(
        "get_channel_history",
        "Retrieve recent message history from a specific channel. Requires channel_id.",
        schema,
    )
}

/// Create the search_messages tool definition
pub fn create_search_messages_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("query".to_string(), schemas::search_query());
    properties.insert(
        "channel_id".to_string(),
        schemas::channel_id(),
    );
    properties.insert(
        "limit".to_string(),
        schemas::limit(20),
    );

    let schema = ToolSchema::object(
        properties,
        vec!["query".to_string()],
    )
    .with_description("Search for messages matching a query");

    ToolDefinition::new(
        "search_messages",
        "Search for messages matching a query string. Optionally filter by channel_id.",
        schema,
    )
}

/// Output format for channel list
#[derive(Debug, Serialize)]
struct ChannelInfo {
    id: Uuid,
    name: String,
    display_name: String,
    channel_type: String,
    member_count: i64,
}

/// Output format for message
#[derive(Debug, Serialize)]
struct MessageInfo {
    id: Uuid,
    user_id: Uuid,
    username: String,
    message: String,
    created_at: String,
    thread_id: Option<Uuid>,
}

/// Execute list_channels tool
pub async fn execute_list_channels(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments.unwrap_or(serde_json::json!({}));
    let team_id = optional_uuid_arg(&args, "team_id");

    // In a real implementation, this would query the database
    // For now, return mock data if no DB connection
    let channels = if let Some(ref db) = context.db {
        sqlx::query_as::<_, (Uuid, String, Option<String>, String, i64)>(
            r#"
            SELECT 
                c.id,
                c.name,
                c.display_name,
                c.type::text as channel_type,
                COUNT(cm.user_id) as member_count
            FROM channels c
            LEFT JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.is_archived = false
            AND ($1::uuid IS NULL OR c.team_id = $1)
            AND EXISTS (
                SELECT 1 FROM channel_members 
                WHERE channel_id = c.id AND user_id = $2
            )
            GROUP BY c.id, c.name, c.display_name, c.type
            ORDER BY c.name
            "#,
        )
        .bind(team_id)
        .bind(context.user_id)
        .fetch_all(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?
        .into_iter()
        .map(|(id, name, display_name, channel_type, member_count)| ChannelInfo {
            id,
            name: name.clone(),
            display_name: display_name.unwrap_or(name),
            channel_type,
            member_count,
        })
        .collect()
    } else {
        // Mock data for testing
        vec![
            ChannelInfo {
                id: Uuid::new_v4(),
                name: "general".to_string(),
                display_name: "General".to_string(),
                channel_type: "public".to_string(),
                member_count: 42,
            },
            ChannelInfo {
                id: Uuid::new_v4(),
                name: "random".to_string(),
                display_name: "Random".to_string(),
                channel_type: "public".to_string(),
                member_count: 38,
            },
        ]
    };

    ToolCallResult::success_json(channels)
        .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
}

/// Execute get_channel_history tool
pub async fn execute_get_channel_history(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments.ok_or_else(|| McpError::InvalidToolParameters("Missing arguments".to_string()))?;
    let channel_id = require_uuid_arg(&args, "channel_id")?;
    let limit = optional_int_arg(&args, "limit", 50).clamp(1, 100);

    // Verify user has access to the channel
    if let Some(ref db) = context.db {
        let has_access: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)"
        )
        .bind(channel_id)
        .bind(context.user_id)
        .fetch_one(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?;

        if !has_access {
            return Err(McpError::ScopeViolation(
                "User does not have access to this channel".to_string(),
            ));
        }
    }

    // Fetch messages
    let messages = if let Some(ref db) = context.db {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, chrono::DateTime<chrono::Utc>, Option<Uuid>)>(
            r#"
            SELECT 
                p.id,
                p.user_id,
                u.username,
                p.message,
                p.created_at,
                p.thread_id
            FROM posts p
            JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1
            AND p.delete_at IS NULL
            ORDER BY p.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?
        .into_iter()
        .map(|(id, user_id, username, message, created_at, thread_id)| MessageInfo {
            id,
            user_id,
            username,
            message,
            created_at: created_at.to_rfc3339(),
            thread_id,
        })
        .collect()
    } else {
        // Mock data
        vec![
            MessageInfo {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                username: "alice".to_string(),
                message: "Hello everyone!".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                thread_id: None,
            },
        ]
    };

    ToolCallResult::success_json(messages)
        .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
}

/// Execute search_messages tool
pub async fn execute_search_messages(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments.ok_or_else(|| McpError::InvalidToolParameters("Missing arguments".to_string()))?;
    let query = optional_string_arg(&args, "query")
        .ok_or_else(|| McpError::InvalidToolParameters("Missing query parameter".to_string()))?;
    let channel_id = optional_uuid_arg(&args, "channel_id");
    let limit = optional_int_arg(&args, "limit", 20).clamp(1, 50);

    // Search messages
    let messages = if let Some(ref db) = context.db {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, chrono::DateTime<chrono::Utc>, Uuid, String)>(
            r#"
            SELECT 
                p.id,
                p.user_id,
                u.username,
                p.message,
                p.created_at,
                p.channel_id,
                c.name as channel_name
            FROM posts p
            JOIN users u ON p.user_id = u.id
            JOIN channels c ON p.channel_id = c.id
            WHERE p.message ILIKE $1
            AND p.delete_at IS NULL
            AND EXISTS (
                SELECT 1 FROM channel_members 
                WHERE channel_id = p.channel_id AND user_id = $2
            )
            AND ($3::uuid IS NULL OR p.channel_id = $3)
            ORDER BY p.created_at DESC
            LIMIT $4
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(context.user_id)
        .bind(channel_id)
        .bind(limit)
        .fetch_all(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?
        .into_iter()
        .map(|(id, user_id, username, message, created_at, ch_id, channel_name)| {
            serde_json::json!({
                "id": id,
                "user_id": user_id,
                "username": username,
                "message": message,
                "created_at": created_at.to_rfc3339(),
                "channel_id": ch_id,
                "channel_name": channel_name,
            })
        })
        .collect::<Vec<_>>()
    } else {
        // Mock data
        vec![
            serde_json::json!({
                "id": Uuid::new_v4(),
                "user_id": Uuid::new_v4(),
                "username": "alice",
                "message": format!("This message contains: {}", query),
                "created_at": chrono::Utc::now().to_rfc3339(),
                "channel_id": Uuid::new_v4(),
                "channel_name": "general",
            }),
        ]
    };

    ToolCallResult::success_json(messages)
        .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
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
    async fn test_list_channels_tool() {
        let context = create_test_context();
        let result = execute_list_channels(
            Some(serde_json::json!({})),
            &context,
        )
        .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_get_channel_history_missing_channel_id() {
        let context = create_test_context();
        let result = execute_get_channel_history(
            Some(serde_json::json!({})),
            &context,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_messages_missing_query() {
        let context = create_test_context();
        let result = execute_search_messages(
            Some(serde_json::json!({})),
            &context,
        )
        .await;

        assert!(result.is_err());
    }
}
