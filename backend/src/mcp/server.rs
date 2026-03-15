//! MCP Server implementation
//!
//! Handles MCP protocol requests and routes them to appropriate handlers.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::capabilities::{default_server_capabilities, ToolRegistry};
use super::protocol::{
    ClientCapabilities, Implementation, InitializeRequest, InitializeResult, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, McpError, McpNotification, ServerCapabilities,
    ToolCallRequest, ToolCallResult, ToolInfo, ToolListResult,
};
use super::tools::create_default_tool_registry;
use super::{MCP_PROTOCOL_VERSION, MCP_SERVER_NAME, MCP_SERVER_VERSION};
use crate::config::mcp::McpConfig;
use crate::mcp::security::McpSecurityContext;

/// MCP Server state
#[derive(Clone)]
pub struct McpServer {
    inner: Arc<RwLock<McpServerInner>>,
}

struct McpServerInner {
    config: McpConfig,
    capabilities: ServerCapabilities,
    tool_registry: ToolRegistry,
    #[allow(dead_code)]
    initialized: bool,
    sessions: HashMap<String, McpSession>,
}

/// MCP Session state
#[derive(Debug, Clone)]
pub struct McpSession {
    pub id: String,
    pub user_id: Uuid,
    pub initialized: bool,
    pub client_capabilities: Option<ClientCapabilities>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl McpSession {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            initialized: false,
            client_capabilities: None,
            created_at: chrono::Utc::now(),
        }
    }
}

impl McpServer {
    /// Create a new MCP server with default configuration
    pub fn new(config: McpConfig) -> Self {
        let inner = McpServerInner {
            config,
            capabilities: default_server_capabilities(),
            tool_registry: create_default_tool_registry(),
            initialized: false,
            sessions: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Handle a JSON-RPC request and return a response
    pub async fn handle_request(
        &self,
        request: JsonRpcRequest,
        security_context: &McpSecurityContext,
    ) -> JsonRpcResponse {
        let id = request.id.clone();

        // Check if MCP is enabled
        if !self.is_enabled().await {
            return JsonRpcResponse::error(id, JsonRpcError::new(-32099, "MCP server is disabled"));
        }

        // Route the request based on method
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request, security_context).await,
            "initialized" => self.handle_initialized(request, security_context).await,
            "tools/list" => self.handle_tools_list(request, security_context).await,
            "tools/call" => self.handle_tool_call(request, security_context).await,
            "ping" => Ok(serde_json::json!({})),
            _ => {
                return JsonRpcResponse::error(id, JsonRpcError::method_not_found());
            }
        };

        match result {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err(e) => {
                let json_err: JsonRpcError = (&e).into();
                JsonRpcResponse::error(id, json_err)
            }
        }
    }

    /// Handle notification (no response expected)
    pub async fn handle_notification(
        &self,
        notification: McpNotification,
        _security_context: &McpSecurityContext,
    ) -> Result<(), McpError> {
        match notification.method.as_str() {
            "notifications/initialized" => {
                tracing::info!("MCP client sent initialized notification");
                Ok(())
            }
            "notifications/cancelled" => {
                tracing::debug!("MCP request cancelled by client");
                Ok(())
            }
            _ => {
                tracing::warn!("Unknown MCP notification: {}", notification.method);
                Ok(())
            }
        }
    }

    /// Handle initialize request
    async fn handle_initialize(
        &self,
        request: JsonRpcRequest,
        security_context: &McpSecurityContext,
    ) -> Result<Value, McpError> {
        let params: InitializeRequest =
            serde_json::from_value(request.params.unwrap_or(serde_json::json!({})))
                .map_err(|e| McpError::InvalidToolParameters(e.to_string()))?;

        // Validate protocol version
        if params.protocol_version != MCP_PROTOCOL_VERSION {
            return Err(McpError::InvalidProtocolVersion(format!(
                "Server supports {}, client requested {}",
                MCP_PROTOCOL_VERSION, params.protocol_version
            )));
        }

        // Create or update session
        let session = McpSession::new(security_context.user_id);
        let session_id = session.id.clone();

        {
            let mut inner = self.inner.write().await;
            inner.sessions.insert(session_id.clone(), session);
        }

        tracing::info!(
            session_id = %session_id,
            user_id = %security_context.user_id,
            client_name = %params.client_info.name,
            client_version = %params.client_info.version,
            "MCP session initialized"
        );

        let inner = self.inner.read().await;
        let result = InitializeResult {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: inner.capabilities.clone(),
            server_info: Implementation {
                name: MCP_SERVER_NAME.to_string(),
                version: MCP_SERVER_VERSION.to_string(),
            },
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::Internal(format!("Serialization error: {}", e)))
    }

    /// Handle initialized notification response
    async fn handle_initialized(
        &self,
        _request: JsonRpcRequest,
        _security_context: &McpSecurityContext,
    ) -> Result<Value, McpError> {
        // Mark session as initialized
        Ok(serde_json::json!({}))
    }

    /// Handle tools/list request
    async fn handle_tools_list(
        &self,
        _request: JsonRpcRequest,
        _security_context: &McpSecurityContext,
    ) -> Result<Value, McpError> {
        let inner = self.inner.read().await;

        let tools: Vec<ToolInfo> = inner
            .tool_registry
            .list()
            .iter()
            .map(|def| ToolInfo {
                name: def.name.clone(),
                description: def.description.clone(),
                input_schema: Some(
                    serde_json::to_value(&def.input_schema)
                        .unwrap_or_else(|_| serde_json::json!({})),
                ),
            })
            .collect();

        let result = ToolListResult { tools };

        serde_json::to_value(result)
            .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
    }

    /// Handle tools/call request
    async fn handle_tool_call(
        &self,
        request: JsonRpcRequest,
        security_context: &McpSecurityContext,
    ) -> Result<Value, McpError> {
        let params: ToolCallRequest =
            serde_json::from_value(request.params.unwrap_or(serde_json::json!({})))
                .map_err(|e| McpError::InvalidToolParameters(e.to_string()))?;

        // Get tool definition
        let tool_def = {
            let inner = self.inner.read().await;
            inner
                .tool_registry
                .get(&params.name)
                .cloned()
                .ok_or_else(|| McpError::ToolNotFound(params.name.clone()))?
        };

        // Check if tool requires approval
        if tool_def.requires_approval {
            // Check if already approved
            let approval_store = &security_context.approval_store;
            let approval_key = format!("mcp:approval:{}:{}", security_context.user_id, params.name);

            let is_approved = approval_store
                .is_approved(&approval_key)
                .await
                .map_err(|e| McpError::Internal(e.to_string()))?;

            if !is_approved {
                // Store pending approval
                let arguments = params.arguments.clone().unwrap_or(serde_json::json!({}));
                approval_store
                    .request_approval(
                        &approval_key,
                        &params.name,
                        &security_context.user_id,
                        arguments,
                    )
                    .await
                    .map_err(|e| McpError::Internal(e.to_string()))?;

                return Err(McpError::ApprovalRequired);
            }
        }

        // Execute the tool
        let tool_result = self
            .execute_tool(&params.name, params.arguments, security_context)
            .await?;

        serde_json::to_value(tool_result)
            .map_err(|e| McpError::ToolExecutionFailed(format!("Serialization error: {}", e)))
    }

    /// Execute a tool with the given arguments
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Option<Value>,
        security_context: &McpSecurityContext,
    ) -> Result<ToolCallResult, McpError> {
        use super::tools::{
            handle_channel_tool, handle_file_tool, handle_message_tool, handle_user_tool,
        };

        match tool_name {
            // Channel tools
            "list_channels" | "get_channel_history" | "search_messages" => {
                handle_channel_tool(tool_name, arguments, security_context).await
            }
            // User tools
            "get_user_profile" | "list_team_members" => {
                handle_user_tool(tool_name, arguments, security_context).await
            }
            // Message tools
            "post_message" => handle_message_tool(tool_name, arguments, security_context).await,
            // File tools
            "list_files" | "get_file_info" => {
                handle_file_tool(tool_name, arguments, security_context).await
            }
            _ => Err(McpError::ToolNotFound(tool_name.to_string())),
        }
    }

    /// Check if MCP server is enabled
    async fn is_enabled(&self) -> bool {
        let inner = self.inner.read().await;
        inner.config.enabled
    }

    /// Get server capabilities
    pub async fn get_capabilities(&self) -> ServerCapabilities {
        let inner = self.inner.read().await;
        inner.capabilities.clone()
    }

    /// Get tool registry (for testing)
    #[cfg(test)]
    pub async fn get_tool_registry(&self) -> ToolRegistry {
        let inner = self.inner.read().await;
        inner.tool_registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::security::{McpApprovalStore, McpAuditLog};

    fn create_test_security_context() -> McpSecurityContext {
        McpSecurityContext {
            user_id: Uuid::new_v4(),
            org_id: None,
            approval_store: McpApprovalStore::new_mock(),
            audit_log: McpAuditLog::new_mock(),
            redis: None,
            db: None,
        }
    }

    #[tokio::test]
    async fn test_server_initialization() {
        let config = McpConfig {
            enabled: true,
            max_tools_per_request: 10,
            approval_timeout_seconds: 300,
            rate_limit_requests_per_minute: 60,
            enable_audit_logging: true,
        };

        let server = McpServer::new(config);
        let capabilities = server.get_capabilities().await;

        assert!(capabilities.tools.is_some());
    }

    #[tokio::test]
    async fn test_initialize_request() {
        let config = McpConfig {
            enabled: true,
            max_tools_per_request: 10,
            approval_timeout_seconds: 300,
            rate_limit_requests_per_minute: 60,
            enable_audit_logging: true,
        };

        let server = McpServer::new(config);
        let security_context = create_test_security_context();

        let request = JsonRpcRequest::new(
            "initialize",
            Some(serde_json::json!({
                "protocol_version": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "client_info": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        );

        let response = server.handle_request(request, &security_context).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_disabled_server() {
        let config = McpConfig {
            enabled: false,
            max_tools_per_request: 10,
            approval_timeout_seconds: 300,
            rate_limit_requests_per_minute: 60,
            enable_audit_logging: true,
        };

        let server = McpServer::new(config);
        let security_context = create_test_security_context();

        let request = JsonRpcRequest::new("initialize", None);
        let response = server.handle_request(request, &security_context).await;

        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let config = McpConfig {
            enabled: true,
            max_tools_per_request: 10,
            approval_timeout_seconds: 300,
            rate_limit_requests_per_minute: 60,
            enable_audit_logging: true,
        };

        let server = McpServer::new(config);
        let security_context = create_test_security_context();

        let request = JsonRpcRequest::new("unknown_method", None);
        let response = server.handle_request(request, &security_context).await;

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601); // Method not found
    }
}
