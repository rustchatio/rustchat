//! Kafka configuration for rustchat
//!
//! Provides configuration for Kafka producer and consumer connections,
//! including bootstrap servers, topics, and consumer groups.

use serde::Deserialize;

/// Kafka configuration
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    /// Enable Kafka integration
    #[serde(default = "default_kafka_enabled")]
    pub enabled: bool,

    /// Bootstrap servers (comma-separated list)
    #[serde(default = "default_bootstrap_servers")]
    pub bootstrap_servers: String,

    /// Client ID for Kafka connections
    #[serde(default = "default_client_id")]
    pub client_id: String,

    /// Topic for post events
    #[serde(default = "default_posts_topic")]
    pub posts_topic: String,

    /// Consumer group ID
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,

    /// Message timeout in milliseconds
    #[serde(default = "default_message_timeout_ms")]
    pub message_timeout_ms: u64,

    /// Retry backoff in milliseconds
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,

    /// Maximum retries for message delivery
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable SSL/TLS connection
    #[serde(default)]
    pub ssl_enabled: bool,

    /// SSL certificate path (optional)
    #[serde(default)]
    pub ssl_ca_location: Option<String>,

    /// SSL certificate location (optional)
    #[serde(default)]
    pub ssl_certificate_location: Option<String>,

    /// SSL key location (optional)
    #[serde(default)]
    pub ssl_key_location: Option<String>,

    /// SASL mechanism (optional: PLAIN, SCRAM-SHA-256, SCRAM-SHA-512)
    #[serde(default)]
    pub sasl_mechanism: Option<String>,

    /// SASL username (optional)
    #[serde(default)]
    pub sasl_username: Option<String>,

    /// SASL password (optional)
    #[serde(default)]
    pub sasl_password: Option<String>,

    /// Fan-out threshold: channels with more members than this use Kafka
    #[serde(default = "default_fanout_threshold")]
    pub fanout_threshold: usize,

    /// Auto-commit interval for consumer in milliseconds
    #[serde(default = "default_auto_commit_interval_ms")]
    pub auto_commit_interval_ms: u64,

    /// Session timeout for consumer in milliseconds
    #[serde(default = "default_session_timeout_ms")]
    pub session_timeout_ms: u64,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            enabled: default_kafka_enabled(),
            bootstrap_servers: default_bootstrap_servers(),
            client_id: default_client_id(),
            posts_topic: default_posts_topic(),
            consumer_group: default_consumer_group(),
            message_timeout_ms: default_message_timeout_ms(),
            retry_backoff_ms: default_retry_backoff_ms(),
            max_retries: default_max_retries(),
            ssl_enabled: false,
            ssl_ca_location: None,
            ssl_certificate_location: None,
            ssl_key_location: None,
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            fanout_threshold: default_fanout_threshold(),
            auto_commit_interval_ms: default_auto_commit_interval_ms(),
            session_timeout_ms: default_session_timeout_ms(),
        }
    }
}

fn default_kafka_enabled() -> bool {
    false
}

fn default_bootstrap_servers() -> String {
    "localhost:9092".to_string()
}

fn default_client_id() -> String {
    format!("rustchat-{}", uuid::Uuid::new_v4())
}

fn default_posts_topic() -> String {
    "rustchat.posts".to_string()
}

fn default_consumer_group() -> String {
    "websocket-fanout".to_string()
}

fn default_message_timeout_ms() -> u64 {
    30000 // 30 seconds
}

fn default_retry_backoff_ms() -> u64 {
    1000 // 1 second
}

fn default_max_retries() -> u32 {
    3
}

fn default_fanout_threshold() -> usize {
    1000 // Use Kafka for channels with >1000 members
}

fn default_auto_commit_interval_ms() -> u64 {
    5000 // 5 seconds
}

fn default_session_timeout_ms() -> u64 {
    30000 // 30 seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = KafkaConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.bootstrap_servers, "localhost:9092");
        assert_eq!(config.posts_topic, "rustchat.posts");
        assert_eq!(config.consumer_group, "websocket-fanout");
        assert_eq!(config.fanout_threshold, 1000);
    }
}
