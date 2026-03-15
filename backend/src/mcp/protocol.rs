//! MCP Protocol types for JSON-RPC 2.0 message handling
//!
//! Implements the Model Context Protocol specification types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::String(Uuid::new_v4().to_string())),
            method: method.into(),
            params,
        }
    }

    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<JsonRpcId>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<JsonRpcId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC request ID (string, number, or null)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    String(String),
    Number(i64),
    Null,
}

impl fmt::Display for JsonRpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonRpcId::String(s) => write!(f, "{}", s),
            JsonRpcId::Number(n) => write!(f, "{}", n),
            JsonRpcId::Null => write!(f, "null"),
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message, self.code)
    }
}

impl std::error::Error for JsonRpcError {}

impl JsonRpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    // Standard JSON-RPC error codes
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request")
    }

    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found")
    }

    pub fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params")
    }

    pub fn internal_error() -> Self {
        Self::new(-32603, "Internal error")
    }
}

/// MCP-specific error types
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Invalid protocol version: {0}")]
    InvalidProtocolVersion(String),

    #[error("Missing required capability: {0}")]
    MissingCapability(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    #[error("User approval required")]
    ApprovalRequired,

    #[error("User denied approval")]
    ApprovalDenied,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid tool parameters: {0}")]
    InvalidToolParameters(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Scope violation: {0}")]
    ScopeViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpc(#[from] JsonRpcError),
}

/// MCP error codes for JSON-RPC responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum McpErrorCode {
    // Standard JSON-RPC errors
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,

    // MCP-specific errors (starting from -32000)
    InvalidProtocolVersion = -32001,
    MissingCapability = -32002,
    ToolNotFound = -32003,
    ToolExecutionFailed = -32004,
    ApprovalRequired = -32005,
    ApprovalDenied = -32006,
    RateLimitExceeded = -32007,
    InvalidToolParameters = -32008,
    Unauthorized = -32009,
    ScopeViolation = -32010,
}

impl McpErrorCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<&McpError> for JsonRpcError {
    fn from(err: &McpError) -> Self {
        let (code, message) = match err {
            McpError::InvalidProtocolVersion(msg) => (
                McpErrorCode::InvalidProtocolVersion.as_i32(),
                format!("Invalid protocol version: {}", msg),
            ),
            McpError::MissingCapability(cap) => (
                McpErrorCode::MissingCapability.as_i32(),
                format!("Missing required capability: {}", cap),
            ),
            McpError::ToolNotFound(name) => (
                McpErrorCode::ToolNotFound.as_i32(),
                format!("Tool not found: {}", name),
            ),
            McpError::ToolExecutionFailed(msg) => (
                McpErrorCode::ToolExecutionFailed.as_i32(),
                format!("Tool execution failed: {}", msg),
            ),
            McpError::ApprovalRequired => (
                McpErrorCode::ApprovalRequired.as_i32(),
                "User approval required".to_string(),
            ),
            McpError::ApprovalDenied => (
                McpErrorCode::ApprovalDenied.as_i32(),
                "User denied approval".to_string(),
            ),
            McpError::RateLimitExceeded => (
                McpErrorCode::RateLimitExceeded.as_i32(),
                "Rate limit exceeded".to_string(),
            ),
            McpError::InvalidToolParameters(msg) => (
                McpErrorCode::InvalidToolParameters.as_i32(),
                format!("Invalid tool parameters: {}", msg),
            ),
            McpError::Unauthorized => (
                McpErrorCode::Unauthorized.as_i32(),
                "Unauthorized access".to_string(),
            ),
            McpError::ScopeViolation(msg) => (
                McpErrorCode::ScopeViolation.as_i32(),
                format!("Scope violation: {}", msg),
            ),
            McpError::Internal(msg) => (
                McpErrorCode::InternalError.as_i32(),
                format!("Internal error: {}", msg),
            ),
            McpError::JsonRpc(e) => return e.clone(),
        };

        JsonRpcError::new(code, message)
    }
}

/// MCP message envelope (request or notification)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum McpMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(McpNotification),
    Batch(Vec<McpMessage>),
}

/// MCP notification (request without id)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// MCP Initialize request parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InitializeRequest {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: Implementation,
}

/// MCP Initialize response result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
}

/// Client capabilities
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub experimental: Option<Value>,
    #[serde(default)]
    pub sampling: Option<Value>,
}

/// Server capabilities
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ServerCapabilities {
    #[serde(default)]
    pub experimental: Option<Value>,
    #[serde(default)]
    pub logging: Option<Value>,
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
}

/// Prompts capability
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PromptsCapability {
    pub list_changed: bool,
}

/// Resources capability
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcesCapability {
    pub subscribe: bool,
    pub list_changed: bool,
}

/// Tools capability
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsCapability {
    pub list_changed: bool,
}

/// Implementation info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

/// Tools/list result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolListResult {
    pub tools: Vec<ToolInfo>,
}

/// Tool information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: Option<Value>,
}

/// Tool call request parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallRequest {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

/// Tool call result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(default)]
    pub is_error: bool,
}

/// Tool content item
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

/// Resource content for tool results
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub blob: Option<String>,
}

impl ToolCallResult {
    pub fn success_text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: text.into() }],
            is_error: false,
        }
    }

    pub fn error_text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: text.into() }],
            is_error: true,
        }
    }

    pub fn success_json<T: Serialize>(value: T) -> Result<Self, serde_json::Error> {
        let text = serde_json::to_string(&value)?;
        Ok(Self::success_text(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request =
            JsonRpcRequest::new("initialize", Some(serde_json::json!({"version": "1.0"})));
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("jsonrpc"));
        assert!(json.contains("2.0"));
        assert!(json.contains("initialize"));
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse::success(
            Some(JsonRpcId::String("123".to_string())),
            serde_json::json!({"result": "ok"}),
        );
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("jsonrpc"));
        assert!(json.contains("result"));
    }

    #[test]
    fn test_json_rpc_error_serialization() {
        let error = JsonRpcError::method_not_found();
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("-32601"));
        assert!(json.contains("Method not found"));
    }

    #[test]
    fn test_mcp_error_conversion() {
        let mcp_err = McpError::ToolNotFound("test_tool".to_string());
        let json_err: JsonRpcError = (&mcp_err).into();
        assert_eq!(json_err.code, McpErrorCode::ToolNotFound.as_i32());
    }

    #[test]
    fn test_tool_call_result() {
        let result = ToolCallResult::success_text("Hello, world!");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);

        if let ToolContent::Text { text } = &result.content[0] {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }
}
