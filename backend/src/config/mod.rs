//! Configuration module for rustchat
//!
//! Supports loading configuration from environment variables and .env files.

use serde::Deserialize;

/// Application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Runtime environment (development, staging, production)
    #[serde(default = "default_environment")]
    pub environment: String,

    /// Server host address
    #[serde(default = "default_host")]
    pub server_host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub server_port: u16,

    /// PostgreSQL database URL
    pub database_url: String,

    /// Redis connection URL
    #[serde(default = "default_redis_url")]
    pub redis_url: String,

    /// JWT secret key
    pub jwt_secret: String,

    /// Encryption key for sensitive data
    pub encryption_key: String,

    /// JWT token expiry in hours
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_hours: u64,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// S3 endpoint URL
    #[serde(default)]
    pub s3_endpoint: Option<String>,

    /// Public S3 endpoint URL (for presigned URLs returned to clients)
    #[serde(default)]
    pub s3_public_endpoint: Option<String>,

    /// S3 bucket name
    #[serde(default = "default_s3_bucket")]
    pub s3_bucket: String,

    /// S3 access key
    #[serde(default)]
    pub s3_access_key: Option<String>,

    /// S3 secret key
    #[serde(default)]
    pub s3_secret_key: Option<String>,

    /// S3 region
    #[serde(default = "default_s3_region")]
    pub s3_region: String,

    /// Initial admin email
    #[serde(default)]
    pub admin_user: Option<String>,

    /// Initial admin password
    #[serde(default)]
    pub admin_password: Option<String>,

    /// Comma-separated CORS origin allowlist.
    /// Example: "https://chat.example.com,https://admin.example.com"
    #[serde(default)]
    pub cors_allowed_origins: Option<String>,

    /// Calls plugin configuration
    #[serde(default)]
    pub calls: CallsConfig,
}

/// Calls plugin configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CallsConfig {
    /// Enable Calls plugin
    #[serde(default = "default_calls_enabled")]
    pub enabled: bool,

    /// RTC UDP port for WebRTC
    #[serde(default = "default_calls_udp_port")]
    pub udp_port: u16,

    /// RTC TCP port for WebRTC (if needed for firewall traversal)
    #[serde(default = "default_calls_tcp_port")]
    pub tcp_port: u16,

    /// ICE host override (public IP or hostname for NAT)
    #[serde(default)]
    pub ice_host_override: Option<String>,

    /// TURN server enabled (from TURN_SERVER_ENABLED env var)
    #[serde(default = "default_turn_server_enabled")]
    pub turn_server_enabled: bool,

    /// TURN server URL (from TURN_SERVER_URL env var)
    #[serde(default = "default_turn_server_url")]
    pub turn_server_url: String,

    /// TURN server username (from TURN_SERVER_USERNAME env var)
    #[serde(default)]
    pub turn_server_username: String,

    /// TURN server credential (from TURN_SERVER_CREDENTIAL env var)
    #[serde(default)]
    pub turn_server_credential: String,

    /// TURN credentials TTL in minutes (for REST API style generation)
    #[serde(default = "default_turn_ttl")]
    pub turn_ttl_minutes: u64,

    /// STUN server URLs
    #[serde(default = "default_stun_servers")]
    pub stun_servers: Vec<String>,

    /// Call state backend mode: memory, redis, auto
    #[serde(default = "default_calls_state_backend")]
    pub state_backend: String,
}

impl Default for CallsConfig {
    fn default() -> Self {
        Self {
            enabled: default_calls_enabled(),
            udp_port: default_calls_udp_port(),
            tcp_port: default_calls_tcp_port(),
            ice_host_override: None,
            turn_server_enabled: default_turn_server_enabled(),
            turn_server_url: default_turn_server_url(),
            turn_server_username: String::new(),
            turn_server_credential: String::new(),
            turn_ttl_minutes: default_turn_ttl(),
            stun_servers: default_stun_servers(),
            state_backend: default_calls_state_backend(),
        }
    }
}

fn default_calls_enabled() -> bool {
    false // Disabled by default
}

fn default_calls_udp_port() -> u16 {
    8443
}

fn default_calls_tcp_port() -> u16 {
    8443
}

fn default_turn_server_enabled() -> bool {
    true // Enabled by default
}

fn default_turn_server_url() -> String {
    "turn:turn.kubedo.io:3478".to_string()
}

fn default_turn_ttl() -> u64 {
    1440 // 24 hours
}

fn default_stun_servers() -> Vec<String> {
    vec!["stun:stun.l.google.com:19302".to_string()]
}

fn default_calls_state_backend() -> String {
    "auto".to_string()
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

fn default_jwt_expiry() -> u64 {
    24
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_s3_bucket() -> String {
    "rustchat".to_string()
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> anyhow::Result<Self> {
        let mut builder = config::Config::builder();

        // Load RUSTCHAT_ prefixed variables
        builder = builder.add_source(
            config::Environment::default()
                .prefix("RUSTCHAT")
                .try_parsing(true),
        );

        // Load TURN_SERVER_ prefixed variables (for backwards compatibility with Mattermost-style env vars)
        builder = builder.add_source(
            config::Environment::default()
                .prefix("TURN_SERVER")
                .try_parsing(true)
                .separator("_"),
        );

        let config = builder.build()?;
        let settings: Config = config.try_deserialize()?;
        Ok(settings)
    }

    pub fn is_production(&self) -> bool {
        matches!(
            self.environment.trim().to_ascii_lowercase().as_str(),
            "prod" | "production"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_host(), "0.0.0.0");
        assert_eq!(default_port(), 3000);
        assert_eq!(default_log_level(), "info");
    }
}
