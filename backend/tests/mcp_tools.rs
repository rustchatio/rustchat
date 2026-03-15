//! MCP Tools Tests
//!
//! Tests for MCP tool implementations.

use rustchat::mcp::capabilities::schemas;
use rustchat::mcp::protocol::ToolCallResult;
use rustchat::mcp::tools::{
    create_get_channel_history_tool, create_get_file_info_tool, create_get_user_profile_tool,
    create_list_channels_tool, create_list_files_tool, create_list_team_members_tool,
    create_post_message_tool, create_search_messages_tool, handle_channel_tool, optional_int_arg,
    optional_string_arg, optional_uuid_arg, require_string_arg, require_uuid_arg,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_list_channels_tool_definition() {
    let tool = create_list_channels_tool();
    assert_eq!(tool.name, "list_channels");
    assert!(!tool.requires_approval);
    assert!(tool.input_schema.properties.is_some());
}

#[test]
fn test_get_channel_history_tool_definition() {
    let tool = create_get_channel_history_tool();
    assert_eq!(tool.name, "get_channel_history");
    assert!(!tool.requires_approval);

    let schema = tool.input_schema;
    assert!(schema
        .properties
        .as_ref()
        .unwrap()
        .contains_key("channel_id"));
    assert!(schema.properties.as_ref().unwrap().contains_key("limit"));
}

#[test]
fn test_search_messages_tool_definition() {
    let tool = create_search_messages_tool();
    assert_eq!(tool.name, "search_messages");
    assert!(!tool.requires_approval);
}

#[test]
fn test_get_user_profile_tool_definition() {
    let tool = create_get_user_profile_tool();
    assert_eq!(tool.name, "get_user_profile");
    assert!(!tool.requires_approval);
}

#[test]
fn test_list_team_members_tool_definition() {
    let tool = create_list_team_members_tool();
    assert_eq!(tool.name, "list_team_members");
    assert!(!tool.requires_approval);
}

#[test]
fn test_post_message_tool_definition() {
    let tool = create_post_message_tool();
    assert_eq!(tool.name, "post_message");
    assert!(tool.requires_approval); // This tool requires user approval
}

#[test]
fn test_list_files_tool_definition() {
    let tool = create_list_files_tool();
    assert_eq!(tool.name, "list_files");
    assert!(!tool.requires_approval);
}

#[test]
fn test_get_file_info_tool_definition() {
    let tool = create_get_file_info_tool();
    assert_eq!(tool.name, "get_file_info");
    assert!(!tool.requires_approval);
}

#[test]
fn test_optional_string_arg() {
    let args = json!({
        "name": "test",
        "count": 42
    });

    assert_eq!(optional_string_arg(&args, "name"), Some("test".to_string()));
    assert_eq!(optional_string_arg(&args, "missing"), None);
    assert_eq!(optional_string_arg(&args, "count"), None); // Not a string
}

#[test]
fn test_require_string_arg() {
    let args = json!({
        "required": "value",
        "number": 42
    });

    assert_eq!(require_string_arg(&args, "required").unwrap(), "value");
    assert!(require_string_arg(&args, "missing").is_err());
    assert!(require_string_arg(&args, "number").is_err()); // Not a string
}

#[test]
fn test_optional_int_arg() {
    let args = json!({
        "limit": 50,
        "name": "test"
    });

    assert_eq!(optional_int_arg(&args, "limit", 10), 50);
    assert_eq!(optional_int_arg(&args, "missing", 10), 10);
    assert_eq!(optional_int_arg(&args, "name", 10), 10); // Not an integer
}

#[test]
fn test_optional_uuid_arg() {
    let uuid = Uuid::new_v4();
    let args = json!({
        "id": uuid.to_string(),
        "invalid": "not-a-uuid"
    });

    assert_eq!(optional_uuid_arg(&args, "id"), Some(uuid));
    assert_eq!(optional_uuid_arg(&args, "missing"), None);
    assert_eq!(optional_uuid_arg(&args, "invalid"), None);
}

#[test]
fn test_require_uuid_arg() {
    let uuid = Uuid::new_v4();
    let args = json!({
        "id": uuid.to_string(),
        "invalid": "not-a-uuid"
    });

    assert_eq!(require_uuid_arg(&args, "id").unwrap(), uuid);
    assert!(require_uuid_arg(&args, "missing").is_err());
    assert!(require_uuid_arg(&args, "invalid").is_err());
}

#[test]
fn test_schema_helpers() {
    let uuid_prop = schemas::uuid_property("A UUID field");
    assert_eq!(uuid_prop.property_type, "string");
    assert_eq!(uuid_prop.format, Some("uuid".to_string()));

    let channel_prop = schemas::channel_id();
    assert_eq!(channel_prop.property_type, "string");

    let limit_prop = schemas::limit(50);
    assert_eq!(limit_prop.property_type, "integer");
    assert_eq!(limit_prop.default, Some(json!(50)));
}

#[tokio::test]
async fn test_execute_list_channels() {
    use rustchat::mcp::security::{McpApprovalStore, McpAuditLog, McpSecurityContext};

    let context = McpSecurityContext::new(
        Uuid::new_v4(),
        McpApprovalStore::new_mock(),
        McpAuditLog::new_mock(),
    );

    let result: Result<ToolCallResult, _> =
        handle_channel_tool("list_channels", Some(json!({})), &context).await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.is_error);
}

#[tokio::test]
async fn test_execute_get_channel_history_missing_id() {
    use rustchat::mcp::security::{McpApprovalStore, McpAuditLog, McpSecurityContext};

    let context = McpSecurityContext::new(
        Uuid::new_v4(),
        McpApprovalStore::new_mock(),
        McpAuditLog::new_mock(),
    );

    let result: Result<ToolCallResult, _> =
        handle_channel_tool("get_channel_history", Some(json!({})), &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_get_channel_history_with_id() {
    use rustchat::mcp::security::{McpApprovalStore, McpAuditLog, McpSecurityContext};

    let context = McpSecurityContext::new(
        Uuid::new_v4(),
        McpApprovalStore::new_mock(),
        McpAuditLog::new_mock(),
    );

    let channel_id = Uuid::new_v4();
    let result: Result<ToolCallResult, _> = handle_channel_tool(
        "get_channel_history",
        Some(json!({
            "channel_id": channel_id.to_string(),
            "limit": 10
        })),
        &context,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_search_messages_missing_query() {
    use rustchat::mcp::security::{McpApprovalStore, McpAuditLog, McpSecurityContext};

    let context = McpSecurityContext::new(
        Uuid::new_v4(),
        McpApprovalStore::new_mock(),
        McpAuditLog::new_mock(),
    );

    let result: Result<ToolCallResult, _> =
        handle_channel_tool("search_messages", Some(json!({})), &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_search_messages_with_query() {
    use rustchat::mcp::security::{McpApprovalStore, McpAuditLog, McpSecurityContext};

    let context = McpSecurityContext::new(
        Uuid::new_v4(),
        McpApprovalStore::new_mock(),
        McpAuditLog::new_mock(),
    );

    let result: Result<ToolCallResult, _> = handle_channel_tool(
        "search_messages",
        Some(json!({
            "query": "hello",
            "limit": 5
        })),
        &context,
    )
    .await;

    assert!(result.is_ok());
}

#[test]
fn test_all_tools_have_descriptions() {
    let tools = vec![
        create_list_channels_tool(),
        create_get_channel_history_tool(),
        create_search_messages_tool(),
        create_get_user_profile_tool(),
        create_list_team_members_tool(),
        create_post_message_tool(),
        create_list_files_tool(),
        create_get_file_info_tool(),
    ];

    for tool in tools {
        assert!(!tool.name.is_empty(), "Tool must have a name");
        assert!(!tool.description.is_empty(), "Tool must have a description");
        assert!(
            !tool.input_schema.schema_type.is_empty(),
            "Tool must have a schema type"
        );
    }
}
