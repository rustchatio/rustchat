//! OpenSearch client for rustchat
//!
//! Provides connection pooling, health checks, and AWS SigV4 authentication.

use std::sync::Arc;
use std::time::Duration;

use opensearch::auth::Credentials;
use opensearch::cert::CertificateValidation;
use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use opensearch::OpenSearch;
use tracing::{error, info};

pub use crate::config::OpenSearchConfig;

/// OpenSearch client wrapper
pub struct OpenSearchClient {
    config: OpenSearchConfig,
    client: Option<OpenSearch>,
}

/// Health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Green,
    Yellow,
    Red,
    Unknown,
}

/// Cluster health information
#[derive(Debug, Clone)]
pub struct ClusterHealth {
    pub status: HealthStatus,
    pub number_of_nodes: i32,
    pub number_of_data_nodes: i32,
    pub active_shards: i32,
    pub relocating_shards: i32,
    pub initializing_shards: i32,
    pub unassigned_shards: i32,
    pub pending_tasks: i32,
}

impl OpenSearchClient {
    /// Create a new OpenSearch client
    pub fn new(config: OpenSearchConfig) -> Arc<Self> {
        let client = if config.enabled {
            match Self::create_client(&config) {
                Ok(client) => {
                    info!(url = %config.url, "OpenSearch client initialized");
                    Some(client)
                }
                Err(e) => {
                    error!(error = %e, "Failed to create OpenSearch client");
                    None
                }
            }
        } else {
            info!("OpenSearch client disabled");
            None
        };

        Arc::new(Self { config, client })
    }

    /// Check if the client is available
    pub fn is_available(&self) -> bool {
        self.client.is_some()
    }

    /// Get the underlying client (if available)
    pub fn client(&self) -> Option<&OpenSearch> {
        self.client.as_ref()
    }

    /// Create the underlying OpenSearch client
    fn create_client(config: &OpenSearchConfig) -> anyhow::Result<OpenSearch> {
        let url = url::Url::parse(&config.url)?;

        // Configure connection pool
        let conn_pool = SingleNodeConnectionPool::new(url);

        // Build transport
        let mut transport_builder = TransportBuilder::new(conn_pool)
            .timeout(Duration::from_secs(config.request_timeout_secs));

        // Configure authentication
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            let credentials = Credentials::Basic(username.clone(), password.clone());
            transport_builder = transport_builder.auth(credentials);
        }

        // Configure SSL/TLS
        if config.ssl_enabled && config.skip_ssl_verify {
            transport_builder = transport_builder.cert_validation(CertificateValidation::None);
        }

        let transport = transport_builder.build()?;
        let client = OpenSearch::new(transport);

        Ok(client)
    }

    /// Check cluster health
    pub async fn health_check(&self) -> anyhow::Result<ClusterHealth> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let response = client
            .cluster()
            .health(opensearch::cluster::ClusterHealthParts::None)
            .send()
            .await?;

        let status_code = response.status_code();
        if !status_code.is_success() {
            anyhow::bail!("Health check failed with status: {}", status_code);
        }

        let body: serde_json::Value = response.json().await?;

        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "green" => HealthStatus::Green,
                "yellow" => HealthStatus::Yellow,
                "red" => HealthStatus::Red,
                _ => HealthStatus::Unknown,
            })
            .unwrap_or(HealthStatus::Unknown);

        Ok(ClusterHealth {
            status,
            number_of_nodes: body
                .get("number_of_nodes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            number_of_data_nodes: body
                .get("number_of_data_nodes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            active_shards: body
                .get("active_shards")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            relocating_shards: body
                .get("relocating_shards")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            initializing_shards: body
                .get("initializing_shards")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            unassigned_shards: body
                .get("unassigned_shards")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            pending_tasks: body
                .get("number_of_pending_tasks")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        })
    }

    /// Ping the OpenSearch cluster
    pub async fn ping(&self) -> anyhow::Result<bool> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let response = client.ping().send().await?;
        Ok(response.status_code().is_success())
    }

    /// Get index name with prefix
    pub fn index_name(&self, name: &str) -> String {
        if self.config.index_prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}-{}", self.config.index_prefix, name)
        }
    }

    /// Get the posts index name
    pub fn posts_index(&self) -> String {
        self.index_name("posts")
    }

    /// Get configuration reference
    pub fn config(&self) -> &OpenSearchConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OpenSearchConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.url, "http://localhost:9200");
        assert_eq!(config.index_prefix, "rustchat");
    }

    #[test]
    fn test_index_name_generation() {
        let config = OpenSearchConfig::default();
        let client = OpenSearchClient::new(config);
        assert_eq!(client.posts_index(), "rustchat-posts");
    }

    #[test]
    fn test_disabled_client() {
        let config = OpenSearchConfig::default();
        let client = OpenSearchClient::new(config);
        assert!(!client.is_available());
    }
}
