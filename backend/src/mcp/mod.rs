//! Model Context Protocol (MCP) module for rustchat
//!
//! Provides MCP server implementation for AI agent integration
//! following the Model Context Protocol specification.

pub mod capabilities;
pub mod protocol;
pub mod security;
pub mod server;
pub mod tools;

pub use capabilities::{ToolDefinition, ToolSchema};
pub use protocol::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpError, McpErrorCode, McpMessage,
    McpNotification, ServerCapabilities, ToolCallRequest, ToolCallResult, ToolListResult,
};
pub use security::{McpApprovalStore, McpAuditLog, McpSecurityContext};
pub use server::McpServer;

use crate::config::mcp::McpConfig;

/// MCP server state container
#[derive(Clone)]
pub struct McpState {
    pub config: McpConfig,
    pub server: McpServer,
}

impl McpState {
    pub fn new(config: McpConfig, server: McpServer) -> Self {
        Self { config, server }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Initialize MCP state with configuration
pub async fn init_mcp_state(config: McpConfig) -> anyhow::Result<McpState> {
    let server = McpServer::new(config.clone());
    Ok(McpState::new(config, server))
}

/// Default MCP protocol version
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// MCP server name
pub const MCP_SERVER_NAME: &str = "rustchat-mcp-server";

/// MCP server version
pub const MCP_SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
