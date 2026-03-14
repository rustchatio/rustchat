//! MCP Protocol Tests
//!
//! Tests for JSON-RPC message parsing and MCP protocol handling.

use rustchat::mcp::protocol::{
    ClientCapabilities, Implementation, InitializeRequest, InitializeResult, JsonRpcError,
    JsonRpcId, JsonRpcRequest, JsonRpcResponse, McpError, McpErrorCode, ServerCapabilities,
    ToolCallRequest, ToolCallResult, ToolInfo, ToolListResult, ToolContent,
};
use serde_json::json;

#[test]
fn test_json_rpc_request_parsing() {
    let json = json!({
        "jsonrpc": "2.0",
        "id": "123",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05"
        }
    });

    let request: JsonRpcRequest = serde_json::from_value(json).unwrap();
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "initialize");
    assert!(request.params.is_some());
}

#[test]
fn test_json_rpc_request_with_numeric_id() {
    let json = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "tools/list"
    });

    let request: JsonRpcRequest = serde_json::from_value(json).unwrap();
    match request.id {
        Some(JsonRpcId::Number(n)) => assert_eq!(n, 42),
        _ => panic!("Expected numeric ID"),
    }
}

#[test]
fn test_json_rpc_notification_no_id() {
    let json = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    let request: JsonRpcRequest = serde_json::from_value(json).unwrap();
    assert!(request.id.is_none());
}

#[test]
fn test_json_rpc_response_success() {
    let response = JsonRpcResponse::success(
        Some(JsonRpcId::String("123".to_string())),
        json!({"result": "ok"}),
    );

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_json_rpc_response_error() {
    let error = JsonRpcError::method_not_found();
    let response = JsonRpcResponse::error(Some(JsonRpcId::String("123".to_string())), error);

    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.error.as_ref().unwrap().code, -32601);
}

#[test]
fn test_json_rpc_error_types() {
    assert_eq!(JsonRpcError::parse_error().code, -32700);
    assert_eq!(JsonRpcError::invalid_request().code, -32600);
    assert_eq!(JsonRpcError::method_not_found().code, -32601);
    assert_eq!(JsonRpcError::invalid_params().code, -32602);
    assert_eq!(JsonRpcError::internal_error().code, -32603);
}

#[test]
fn test_json_rpc_error_with_data() {
    let error = JsonRpcError::invalid_params()
        .with_data(json!({"field": "missing", "reason": "required"}));
    
    assert_eq!(error.code, -32602);
    assert!(error.data.is_some());
}

#[test]
fn test_initialize_request_serialization() {
    let request = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["protocol_version"], "2024-11-05");
    assert_eq!(json["client_info"]["name"], "test-client");
}

#[test]
fn test_initialize_result_serialization() {
    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities::default(),
        server_info: Implementation {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["protocol_version"], "2024-11-05");
    assert_eq!(json["server_info"]["name"], "test-server");
}

#[test]
fn test_tool_list_result() {
    let result = ToolListResult {
        tools: vec![
            ToolInfo {
                name: "list_channels".to_string(),
                description: "List channels".to_string(),
                input_schema: Some(json!({"type": "object"})),
            },
        ],
    };

    let json = serde_json::to_value(&result).unwrap();
    assert!(json["tools"].is_array());
    assert_eq!(json["tools"].as_array().unwrap().len(), 1);
}

#[test]
fn test_tool_call_request() {
    let request = ToolCallRequest {
        name: "post_message".to_string(),
        arguments: Some(json!({
            "channel_id": "123",
            "message": "Hello"
        })),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["name"], "post_message");
    assert!(json["arguments"].is_object());
}

#[test]
fn test_tool_call_result() {
    let result = ToolCallResult::success_text("Operation completed");
    
    assert!(!result.is_error);
    assert_eq!(result.content.len(), 1);
    
    match &result.content[0] {
        ToolContent::Text { text } => assert_eq!(text, "Operation completed"),
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_tool_call_result_error() {
    let result = ToolCallResult::error_text("Something went wrong");
    
    assert!(result.is_error);
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_tool_call_result_json() {
    let data = json!({"id": "123", "status": "ok"});
    let result = ToolCallResult::success_json(data).unwrap();
    
    assert!(!result.is_error);
    
    match &result.content[0] {
        ToolContent::Text { text } => {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(parsed["id"], "123");
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_mcp_error_conversion() {
    let errors = vec![
        (McpError::ToolNotFound("test".to_string()), McpErrorCode::ToolNotFound),
        (McpError::ApprovalRequired, McpErrorCode::ApprovalRequired),
        (McpError::ApprovalDenied, McpErrorCode::ApprovalDenied),
        (McpError::RateLimitExceeded, McpErrorCode::RateLimitExceeded),
        (McpError::Unauthorized, McpErrorCode::Unauthorized),
        (McpError::ScopeViolation("test".to_string()), McpErrorCode::ScopeViolation),
    ];

    for (mcp_err, expected_code) in errors {
        let json_err: JsonRpcError = (&mcp_err).into();
        assert_eq!(json_err.code, expected_code.as_i32(), 
            "Error {:?} should have code {}", mcp_err, expected_code.as_i32());
    }
}

#[test]
fn test_mcp_error_display() {
    let err = McpError::ToolNotFound("my_tool".to_string());
    assert!(err.to_string().contains("my_tool"));
    
    let err = McpError::InvalidProtocolVersion("1.0".to_string());
    assert!(err.to_string().contains("1.0"));
}

#[test]
fn test_json_rpc_id_display() {
    assert_eq!(JsonRpcId::String("test".to_string()).to_string(), "test");
    assert_eq!(JsonRpcId::Number(42).to_string(), "42");
    assert_eq!(JsonRpcId::Null.to_string(), "null");
}

#[test]
fn test_json_rpc_id_equality() {
    assert_eq!(
        JsonRpcId::String("test".to_string()),
        JsonRpcId::String("test".to_string())
    );
    assert_eq!(JsonRpcId::Number(42), JsonRpcId::Number(42));
    assert_eq!(JsonRpcId::Null, JsonRpcId::Null);
    
    assert_ne!(
        JsonRpcId::String("test".to_string()),
        JsonRpcId::String("other".to_string())
    );
}

#[test]
fn test_batch_message_parsing() {
    use rustchat::mcp::protocol::McpMessage;
    
    let json = json!([
        {"jsonrpc": "2.0", "id": 1, "method": "tools/list"},
        {"jsonrpc": "2.0", "id": 2, "method": "ping"}
    ]);

    // Note: Batch handling is a future enhancement
    // This test documents the expected behavior
    let result: Result<Vec<JsonRpcRequest>, _> = serde_json::from_value(json);
    assert!(result.is_ok());
    let requests = result.unwrap();
    assert_eq!(requests.len(), 2);
}
