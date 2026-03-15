//! MCP User Tools
//!
//! Tools for getting user profiles and listing team members.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::{optional_string_arg, optional_uuid_arg};
use crate::mcp::capabilities::schemas;
use crate::mcp::capabilities::{ToolDefinition, ToolSchema};
use crate::mcp::protocol::{McpError, ToolCallResult};
use crate::mcp::security::McpSecurityContext;

/// Create the get_user_profile tool definition
pub fn create_get_user_profile_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("user_id".to_string(), schemas::user_id());
    properties.insert("username".to_string(), schemas::username());

    let schema = ToolSchema::object(
        properties,
        vec![], // Either user_id or username is accepted
    )
    .with_description("Get user profile information by user_id or username");

    ToolDefinition::new(
        "get_user_profile",
        "Retrieve user profile information. Provide either user_id or username.",
        schema,
    )
}

/// Create the list_team_members tool definition
pub fn create_list_team_members_tool() -> ToolDefinition {
    let mut properties = HashMap::new();
    properties.insert("team_id".to_string(), schemas::team_id());

    let schema = ToolSchema::object(properties, vec!["team_id".to_string()])
        .with_description("List all members of a team with their roles");

    ToolDefinition::new(
        "list_team_members",
        "List all members of a team with their roles. Requires team_id.",
        schema,
    )
}

/// Output format for user profile
#[derive(Debug, Serialize)]
struct UserProfile {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_bot: Option<bool>,
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_activity_at: Option<String>,
}

/// Output format for team member
#[derive(Debug, Serialize)]
struct TeamMember {
    user_id: Uuid,
    username: String,
    display_name: Option<String>,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    joined_at: Option<String>,
}

/// Execute get_user_profile tool
pub async fn execute_get_user_profile(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments.unwrap_or(serde_json::json!({}));
    let user_id = optional_uuid_arg(&args, "user_id");
    let username = optional_string_arg(&args, "username");

    // Must provide either user_id or username
    if user_id.is_none() && username.is_none() {
        return Err(McpError::InvalidToolParameters(
            "Either user_id or username must be provided".to_string(),
        ));
    }

    // Fetch user profile
    let profile = if let Some(ref db) = context.db {
        let query = if let Some(id) = user_id {
            sqlx::query_as::<
                _,
                (
                    Uuid,
                    String,
                    Option<String>,
                    String,
                    String,
                    Option<String>,
                    chrono::DateTime<chrono::Utc>,
                ),
            >(
                r#"
                SELECT 
                    id,
                    username,
                    display_name,
                    email,
                    role,
                    timezone,
                    created_at
                FROM users
                WHERE id = $1 AND deleted_at IS NULL
                "#,
            )
            .bind(id)
            .fetch_optional(db)
            .await
        } else if let Some(name) = username {
            sqlx::query_as::<
                _,
                (
                    Uuid,
                    String,
                    Option<String>,
                    String,
                    String,
                    Option<String>,
                    chrono::DateTime<chrono::Utc>,
                ),
            >(
                r#"
                SELECT 
                    id,
                    username,
                    display_name,
                    email,
                    role,
                    timezone,
                    created_at
                FROM users
                WHERE username = $1 AND deleted_at IS NULL
                "#,
            )
            .bind(name)
            .fetch_optional(db)
            .await
        } else {
            unreachable!()
        }
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?;

        match query {
            Some((id, username, display_name, email, role, timezone, created_at)) => UserProfile {
                id,
                username,
                display_name,
                email: mask_email(&email), // Respect privacy
                role,
                status: Some("online".to_string()), // Would need real status lookup
                timezone,
                locale: Some("en".to_string()),
                avatar_url: None,
                is_bot: Some(false),
                created_at: Some(created_at.to_rfc3339()),
                last_activity_at: None,
            },
            None => {
                return Err(McpError::ToolExecutionFailed("User not found".to_string()));
            }
        }
    } else {
        // Mock data
        UserProfile {
            id: user_id.unwrap_or_else(Uuid::new_v4),
            username: username.unwrap_or_else(|| "alice".to_string()),
            display_name: Some("Alice Smith".to_string()),
            email: "a***@example.com".to_string(),
            role: "member".to_string(),
            status: Some("online".to_string()),
            timezone: Some("America/New_York".to_string()),
            locale: Some("en".to_string()),
            avatar_url: None,
            is_bot: Some(false),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_activity_at: None,
        }
    };

    ToolCallResult::success_json(profile)
        .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
}

/// Execute list_team_members tool
pub async fn execute_list_team_members(
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    let args = arguments
        .ok_or_else(|| McpError::InvalidToolParameters("Missing arguments".to_string()))?;
    let team_id = optional_uuid_arg(&args, "team_id")
        .ok_or_else(|| McpError::InvalidToolParameters("Missing team_id parameter".to_string()))?;

    // Verify user is a member of the team
    if let Some(ref db) = context.db {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(team_id)
        .bind(context.user_id)
        .fetch_one(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?;

        if !is_member {
            return Err(McpError::ScopeViolation(
                "User is not a member of this team".to_string(),
            ));
        }
    }

    // Fetch team members
    let members = if let Some(ref db) = context.db {
        sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                Option<String>,
                String,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT 
                u.id,
                u.username,
                u.display_name,
                tm.role,
                tm.created_at as joined_at
            FROM team_members tm
            JOIN users u ON tm.user_id = u.id
            WHERE tm.team_id = $1
            AND u.deleted_at IS NULL
            ORDER BY u.username
            "#,
        )
        .bind(team_id)
        .fetch_all(db)
        .await
        .map_err(|e| McpError::ToolExecutionFailed(format!("Database error: {}", e)))?
        .into_iter()
        .map(
            |(user_id, username, display_name, role, joined_at)| TeamMember {
                user_id,
                username,
                display_name,
                role,
                joined_at: Some(joined_at.to_rfc3339()),
            },
        )
        .collect()
    } else {
        // Mock data
        vec![
            TeamMember {
                user_id: Uuid::new_v4(),
                username: "alice".to_string(),
                display_name: Some("Alice Smith".to_string()),
                role: "team_admin".to_string(),
                joined_at: Some(chrono::Utc::now().to_rfc3339()),
            },
            TeamMember {
                user_id: Uuid::new_v4(),
                username: "bob".to_string(),
                display_name: Some("Bob Jones".to_string()),
                role: "member".to_string(),
                joined_at: Some(chrono::Utc::now().to_rfc3339()),
            },
        ]
    };

    ToolCallResult::success_json(members)
        .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
}

/// Mask email for privacy (show first character and domain)
fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos..];
        if local.len() > 1 {
            format!("{}***{}", &local[..1], domain)
        } else {
            format!("***{}", domain)
        }
    } else {
        "***".to_string()
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
    async fn test_get_user_profile_missing_args() {
        let context = create_test_context();
        let result = execute_get_user_profile(Some(serde_json::json!({})), &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_user_profile_with_username() {
        let context = create_test_context();
        let result =
            execute_get_user_profile(Some(serde_json::json!({"username": "alice"})), &context)
                .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_list_team_members_missing_team_id() {
        let context = create_test_context();
        let result = execute_list_team_members(Some(serde_json::json!({})), &context).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("alice@example.com"), "a***@example.com");
        assert_eq!(mask_email("a@example.com"), "***@example.com");
        assert_eq!(mask_email("invalid"), "***");
    }
}
