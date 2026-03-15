//! MCP Tools Module
//!
//! Defines MCP tools for channel, user, message, and file operations.

use serde_json::Value;

use super::capabilities::ToolRegistry;
use super::protocol::{McpError, ToolCallResult};
use super::security::McpSecurityContext;

mod channels;
mod files;
mod messages;
mod users;

pub use channels::*;
pub use files::*;
pub use messages::*;
pub use users::*;

/// Create the default tool registry with all MCP tools
pub fn create_default_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Register channel tools
    registry.register(channels::create_list_channels_tool());
    registry.register(channels::create_get_channel_history_tool());
    registry.register(channels::create_search_messages_tool());

    // Register user tools
    registry.register(users::create_get_user_profile_tool());
    registry.register(users::create_list_team_members_tool());

    // Register message tools
    registry.register(messages::create_post_message_tool());

    // Register file tools
    registry.register(files::create_list_files_tool());
    registry.register(files::create_get_file_info_tool());

    registry
}

/// Handle channel-related tools
pub async fn handle_channel_tool(
    tool_name: &str,
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    match tool_name {
        "list_channels" => channels::execute_list_channels(arguments, context).await,
        "get_channel_history" => channels::execute_get_channel_history(arguments, context).await,
        "search_messages" => channels::execute_search_messages(arguments, context).await,
        _ => Err(McpError::ToolNotFound(tool_name.to_string())),
    }
}

/// Handle user-related tools
pub async fn handle_user_tool(
    tool_name: &str,
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    match tool_name {
        "get_user_profile" => users::execute_get_user_profile(arguments, context).await,
        "list_team_members" => users::execute_list_team_members(arguments, context).await,
        _ => Err(McpError::ToolNotFound(tool_name.to_string())),
    }
}

/// Handle message-related tools
pub async fn handle_message_tool(
    tool_name: &str,
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    match tool_name {
        "post_message" => messages::execute_post_message(arguments, context).await,
        _ => Err(McpError::ToolNotFound(tool_name.to_string())),
    }
}

/// Handle file-related tools
pub async fn handle_file_tool(
    tool_name: &str,
    arguments: Option<Value>,
    context: &McpSecurityContext,
) -> Result<ToolCallResult, McpError> {
    match tool_name {
        "list_files" => files::execute_list_files(arguments, context).await,
        "get_file_info" => files::execute_get_file_info(arguments, context).await,
        _ => Err(McpError::ToolNotFound(tool_name.to_string())),
    }
}

/// Helper function to extract a required string argument
pub fn require_string_arg(args: &Value, name: &str) -> Result<String, McpError> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            McpError::InvalidToolParameters(format!("Missing required argument: {}", name))
        })
}

/// Helper function to extract an optional string argument
pub fn optional_string_arg(args: &Value, name: &str) -> Option<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper function to extract an optional integer argument with default
pub fn optional_int_arg(args: &Value, name: &str, default: i64) -> i64 {
    args.get(name).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// Helper function to extract an optional UUID argument
pub fn optional_uuid_arg(args: &Value, name: &str) -> Option<uuid::Uuid> {
    args.get(name)
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
}

/// Helper function to require a UUID argument
pub fn require_uuid_arg(args: &Value, name: &str) -> Result<uuid::Uuid, McpError> {
    args.get(name)
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            McpError::InvalidToolParameters(format!("Missing or invalid UUID argument: {}", name))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tool_registry() {
        let registry = create_default_tool_registry();
        assert!(registry.contains("list_channels"));
        assert!(registry.contains("get_channel_history"));
        assert!(registry.contains("search_messages"));
        assert!(registry.contains("get_user_profile"));
        assert!(registry.contains("list_team_members"));
        assert!(registry.contains("post_message"));
        assert!(registry.contains("list_files"));
        assert!(registry.contains("get_file_info"));
    }

    #[test]
    fn test_require_string_arg() {
        let args = serde_json::json!({
            "name": "test",
            "count": 42
        });

        assert_eq!(require_string_arg(&args, "name").unwrap(), "test");
        assert!(require_string_arg(&args, "missing").is_err());
        assert!(require_string_arg(&args, "count").is_err()); // Not a string
    }

    #[test]
    fn test_optional_args() {
        let args = serde_json::json!({
            "name": "test",
            "limit": 50
        });

        assert_eq!(optional_string_arg(&args, "name"), Some("test".to_string()));
        assert_eq!(optional_string_arg(&args, "missing"), None);
        assert_eq!(optional_int_arg(&args, "limit", 10), 50);
        assert_eq!(optional_int_arg(&args, "missing", 10), 10);
    }
}
