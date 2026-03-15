//! MCP Configuration Module

use serde::Deserialize;

/// MCP (Model Context Protocol) configuration
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// Enable MCP server
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum number of tools allowed in a single request
    #[serde(default = "default_max_tools_per_request")]
    pub max_tools_per_request: u32,

    /// Approval request timeout in seconds
    #[serde(default = "default_approval_timeout_seconds")]
    pub approval_timeout_seconds: u64,

    /// Rate limit: requests per minute per user
    #[serde(default = "default_rate_limit_requests_per_minute")]
    pub rate_limit_requests_per_minute: u32,

    /// Enable audit logging for all MCP tool invocations
    #[serde(default = "default_enable_audit_logging")]
    pub enable_audit_logging: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_tools_per_request: default_max_tools_per_request(),
            approval_timeout_seconds: default_approval_timeout_seconds(),
            rate_limit_requests_per_minute: default_rate_limit_requests_per_minute(),
            enable_audit_logging: default_enable_audit_logging(),
        }
    }
}

fn default_enabled() -> bool {
    false // Disabled by default
}

fn default_max_tools_per_request() -> u32 {
    10
}

fn default_approval_timeout_seconds() -> u64 {
    300 // 5 minutes
}

fn default_rate_limit_requests_per_minute() -> u32 {
    60
}

fn default_enable_audit_logging() -> bool {
    true
}

impl McpConfig {
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) -> anyhow::Result<()> {
        if let Ok(val) = std::env::var("RUSTCHAT_MCP_ENABLED") {
            self.enabled = parse_bool_env(&val)?;
        }

        if let Ok(val) = std::env::var("RUSTCHAT_MCP_APPROVAL_TIMEOUT") {
            self.approval_timeout_seconds = val
                .parse::<u64>()
                .map_err(|e| anyhow::anyhow!("Invalid RUSTCHAT_MCP_APPROVAL_TIMEOUT: {}", e))?;
        }

        if let Ok(val) = std::env::var("RUSTCHAT_MCP_MAX_TOOLS_PER_REQUEST") {
            self.max_tools_per_request = val.parse::<u32>().map_err(|e| {
                anyhow::anyhow!("Invalid RUSTCHAT_MCP_MAX_TOOLS_PER_REQUEST: {}", e)
            })?;
        }

        if let Ok(val) = std::env::var("RUSTCHAT_MCP_RATE_LIMIT_PER_MINUTE") {
            self.rate_limit_requests_per_minute = val.parse::<u32>().map_err(|e| {
                anyhow::anyhow!("Invalid RUSTCHAT_MCP_RATE_LIMIT_PER_MINUTE: {}", e)
            })?;
        }

        if let Ok(val) = std::env::var("RUSTCHAT_MCP_ENABLE_AUDIT_LOGGING") {
            self.enable_audit_logging = parse_bool_env(&val)?;
        }

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_tools_per_request == 0 {
            anyhow::bail!("max_tools_per_request must be greater than 0");
        }

        if self.approval_timeout_seconds == 0 {
            anyhow::bail!("approval_timeout_seconds must be greater than 0");
        }

        if self.rate_limit_requests_per_minute == 0 {
            anyhow::bail!("rate_limit_requests_per_minute must be greater than 0");
        }

        Ok(())
    }
}

fn parse_bool_env(val: &str) -> anyhow::Result<bool> {
    match val.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(anyhow::anyhow!("Invalid boolean value: {}", val)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_tools_per_request, 10);
        assert_eq!(config.approval_timeout_seconds, 300);
        assert_eq!(config.rate_limit_requests_per_minute, 60);
        assert!(config.enable_audit_logging);
    }

    #[test]
    fn test_config_validation() {
        let config = McpConfig {
            max_tools_per_request: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = McpConfig {
            approval_timeout_seconds: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_bool_env() {
        assert!(parse_bool_env("true").unwrap());
        assert!(parse_bool_env("TRUE").unwrap());
        assert!(parse_bool_env("1").unwrap());
        assert!(parse_bool_env("yes").unwrap());
        assert!(!parse_bool_env("false").unwrap());
        assert!(!parse_bool_env("0").unwrap());
        assert!(parse_bool_env("invalid").is_err());
    }
}
