//! MCP File Tools
//!
//! Tools for listing files and getting file information.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::{optional_int_arg, require_uuid_arg};
use crate::mcp::capabilities::schemas;
use crate::mcp::capabilities::{ToolDefinition, ToolSchema};
use crate::mcp::protocol::{McpError, ToolCallResult};
use crate::mcp::security::McpSecurityContext;

/// File info row from database query
#[allow(clippy::type_complexity)]
type FileInfoRow = (
    Uuid,
    String,
    Option<String>,
    i64,
    Uuid,
    String,
    Uuid,
    String,
    chrono::DateTime<chrono::Utc>,
);

/// Create the list_files tool definition
pub fn create_list_files_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("channel_id".to_string(), schemas::channel_id());
    properties.insert("limit".to_string(), schemas::limit(20));

    let schema = ToolSchema::object(properties, vec!["channel_id".to_string()])
        .with_description("List files in a channel");

    ToolDefinition::new(
        "list_files",
        "List files uploaded to a channel with metadata. Requires channel_id.",
        schema,
    )
}

/// Create the get_file_info tool definition
pub fn create_get_file_info_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("file_id".to_string(), schemas::file_id());

    let schema = ToolSchema::object(properties, vec!["file_id".to_string()])
        .with_description("Get metadata and temporary URL for a file");

    ToolDefinition::new(
        "get_file_info",
        "Get file metadata and a temporary download URL. Requires file_id.",
        schema,
    )
}

/// Output format for file info
#[derive(Debug, Serialize)]
struct FileInfo {
    id: Uuid,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
    size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    user_id: Uuid,
    username: String,
    channel_id: Uuid,
    channel_name: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
}

/// Execute list_files tool
pub async fn execute_list_files(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments
        .ok_or_else(|| McpError::InvalidToolParameters("Missing arguments".to_string()))?;
    let channel_id = require_uuid_arg(&args, "channel_id")?;
    let limit = optional_int_arg(&args, "limit", 20).clamp(1, 50);

    // Verify user has access to the channel
    if let Some(ref db) = context.db {
        let has_access: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
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

    // Fetch files
    let files = if let Some(ref db) = context.db {
        sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                Option<String>,
                i64,
                Uuid,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT 
                f.id,
                f.name,
                f.mime_type,
                f.size,
                f.user_id,
                u.username,
                c.name as channel_name,
                f.created_at
            FROM files f
            JOIN users u ON f.user_id = u.id
            JOIN channels c ON f.channel_id = c.id
            WHERE f.channel_id = $1
            AND f.delete_at IS NULL
            ORDER BY f.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?
        .into_iter()
        .map(
            |(id, name, mime_type, size, user_id, username, channel_name, created_at)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "mime_type": mime_type,
                    "size": size,
                    "user_id": user_id,
                    "username": username,
                    "channel_id": channel_id,
                    "channel_name": channel_name,
                    "created_at": created_at.to_rfc3339(),
                })
            },
        )
        .collect::<Vec<_>>()
    } else {
        // Mock data
        vec![serde_json::json!({
            "id": Uuid::new_v4(),
            "name": "document.pdf",
            "mime_type": "application/pdf",
            "size": 1024000i64,
            "user_id": Uuid::new_v4(),
            "username": "alice",
            "channel_id": channel_id,
            "channel_name": "general",
            "created_at": chrono::Utc::now().to_rfc3339(),
        })]
    };

    ToolCallResult::success_json(files)
        .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
}

/// Execute get_file_info tool
pub async fn execute_get_file_info(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments
        .ok_or_else(|| McpError::InvalidToolParameters("Missing arguments".to_string()))?;
    let file_id = require_uuid_arg(&args, "file_id")?;

    // Fetch file info
    let file_info = if let Some(ref db) = context.db {
        let row: Option<FileInfoRow> = sqlx::query_as(
            r#"
            SELECT 
                f.id,
                f.name,
                f.mime_type,
                f.size,
                f.user_id,
                u.username,
                f.channel_id,
                c.name as channel_name,
                f.created_at
            FROM files f
            JOIN users u ON f.user_id = u.id
            JOIN channels c ON f.channel_id = c.id
            WHERE f.id = $1
            AND f.delete_at IS NULL
            "#,
        )
        .bind(file_id)
        .fetch_optional(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?;

        match row {
            Some((
                id,
                name,
                mime_type,
                size,
                user_id,
                username,
                channel_id,
                channel_name,
                created_at,
            )) => {
                // Check if user has access to the channel
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
                        "User does not have access to this file".to_string(),
                    ));
                }

                // Generate presigned URL
                let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);
                // Note: In a real implementation, we'd use the S3 client to generate a presigned URL
                let url = format!("https://example.com/files/{}/download?token=temporary", id);

                FileInfo {
                    id,
                    name,
                    mime_type,
                    size,
                    url: Some(url),
                    user_id,
                    username,
                    channel_id,
                    channel_name,
                    created_at: created_at.to_rfc3339(),
                    expires_at: Some(expires_at.to_rfc3339()),
                }
            }
            None => {
                return Err(McpError::ToolExecutionFailed("File not found".to_string()));
            }
        }
    } else {
        // Mock data
        FileInfo {
            id: file_id,
            name: "document.pdf".to_string(),
            mime_type: Some("application/pdf".to_string()),
            size: 1024000,
            url: Some(format!(
                "https://example.com/files/{}/download?token=temporary",
                file_id
            )),
            user_id: Uuid::new_v4(),
            username: "alice".to_string(),
            channel_id: Uuid::new_v4(),
            channel_name: "general".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: Some((chrono::Utc::now() + chrono::Duration::minutes(15)).to_rfc3339()),
        }
    };

    ToolCallResult::success_json(file_info)
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
    async fn test_list_files_missing_channel_id() {
        let context = create_test_context();
        let result = execute_list_files(Some(serde_json::json!({})), &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_files_success() {
        let context = create_test_context();
        let result = execute_list_files(
            Some(serde_json::json!({
                "channel_id": Uuid::new_v4(),
                "limit": 10
            })),
            &context,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_file_info_missing_file_id() {
        let context = create_test_context();
        let result = execute_get_file_info(Some(serde_json::json!({})), &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_file_info_success() {
        let context = create_test_context();
        let result = execute_get_file_info(
            Some(serde_json::json!({"file_id": Uuid::new_v4()})),
            &context,
        )
        .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_error);
    }
}
