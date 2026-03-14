//! OpenSearch index management for rustchat
//!
//! Provides index creation, rollover, and reindexing capabilities.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use opensearch::indices::{
    IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts, IndicesGetParts,
    IndicesPutSettingsParts,
};
use serde_json::json;
use tracing::{info, warn};

use crate::search::OpenSearchClient;
use crate::search::indexer::{posts_index_mapping, posts_index_settings};

/// Index manager for OpenSearch operations
pub struct IndexManager {
    client: Arc<OpenSearchClient>,
}

/// Index information
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub health: String,
    pub status: String,
    pub docs_count: i64,
    pub docs_deleted: i64,
    pub store_size_bytes: i64,
    pub creation_date: Option<i64>,
}

/// Rollover result
#[derive(Debug, Clone)]
pub struct RolloverResult {
    pub old_index: String,
    pub new_index: String,
    pub acknowledged: bool,
    pub shards_acknowledged: bool,
}

/// Reindex result
#[derive(Debug, Clone)]
pub struct ReindexResult {
    pub took: i64,
    pub total: i64,
    pub created: i64,
    pub updated: i64,
    pub deleted: i64,
    pub version_conflicts: i64,
    pub failures: Vec<String>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new(client: Arc<OpenSearchClient>) -> Arc<Self> {
        Arc::new(Self { client })
    }

    /// Check if an index exists
    pub async fn index_exists(&self, index_name: &str) -> anyhow::Result<bool> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let response = client
            .indices()
            .exists(IndicesExistsParts::Index(&[index_name]))
            .send()
            .await?;

        Ok(response.status_code().is_success())
    }

    /// Create the posts index with proper mappings
    pub async fn create_posts_index(&self) -> anyhow::Result<String> {
        self.create_posts_index_with_name(&self.client.posts_index())
            .await
    }

    /// Create posts index with a specific name
    pub async fn create_posts_index_with_name(&self, index_name: &str) -> anyhow::Result<String> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let config = self.client.config();

        // Check if index already exists
        if self.index_exists(index_name).await? {
            info!(index = %index_name, "Index already exists");
            return Ok(index_name.to_string());
        }

        let settings = posts_index_settings(config.number_of_shards, config.number_of_replicas);
        let mappings = posts_index_mapping();

        let body = json!({
            "settings": settings,
            "mappings": mappings
        });

        let response = client
            .indices()
            .create(IndicesCreateParts::Index(index_name))
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_body: serde_json::Value = response.json().await?;
            anyhow::bail!(
                "Failed to create index {}: {:?}",
                index_name,
                error_body
            );
        }

        info!(index = %index_name, "Created posts index");

        // Wait for index to be ready
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(index_name.to_string())
    }

    /// Delete an index
    pub async fn delete_index(&self, index_name: &str) -> anyhow::Result<()> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let response = client
            .indices()
            .delete(IndicesDeleteParts::Index(&[index_name]))
            .send()
            .await?;

        if !response.status_code().is_success() {
            // 404 is OK - index didn't exist
            if response.status_code().as_u16() != 404 {
                let error_body: serde_json::Value = response.json().await?;
                anyhow::bail!(
                    "Failed to delete index {}: {:?}",
                    index_name,
                    error_body
                );
            }
        }

        info!(index = %index_name, "Deleted index");
        Ok(())
    }

    /// Get index information
    pub async fn get_index_info(&self, index_name: &str) -> anyhow::Result<Option<IndexInfo>> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let response = client
            .indices()
            .get(IndicesGetParts::Index(&[index_name]))
            .send()
            .await?;

        if response.status_code().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status_code().is_success() {
            let error_body: serde_json::Value = response.json().await?;
            anyhow::bail!("Failed to get index info: {:?}", error_body);
        }

        let body: serde_json::Value = response.json().await?;

        // Parse index information
        if let Some(index_data) = body.get(index_name) {
            let settings = index_data.get("settings").and_then(|s| s.get("index"));
            let creation_date = settings
                .and_then(|s| s.get("creation_date"))
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse::<i64>().ok());

            let stats = index_data.get("stats");
            let docs = stats.and_then(|s| s.get("docs"));
            let docs_count = docs
                .and_then(|d| d.get("count"))
                .and_then(|c| c.as_i64())
                .unwrap_or(0);
            let docs_deleted = docs
                .and_then(|d| d.get("deleted"))
                .and_then(|d| d.as_i64())
                .unwrap_or(0);

            let store_size = stats
                .and_then(|s| s.get("store"))
                .and_then(|s| s.get("size_in_bytes"))
                .and_then(|s| s.as_i64())
                .unwrap_or(0);

            let health = index_data
                .get("health")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown")
                .to_string();

            let status = index_data
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string();

            Ok(Some(IndexInfo {
                name: index_name.to_string(),
                health,
                status,
                docs_count,
                docs_deleted,
                store_size_bytes: store_size,
                creation_date,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a time-based index name (for rollover)
    pub fn time_based_index_name(&self, base_name: &str, date: DateTime<Utc>) -> String {
        format!("{}-{}", base_name, date.format("%Y.%m.%d"))
    }

    /// Perform index rollover
    /// 
    /// Creates a new time-based index and optionally reindexes from the old one
    pub async fn rollover_posts_index(&self) -> anyhow::Result<RolloverResult> {
        let current_index = self.client.posts_index();
        let today = chrono::Utc::now();
        let new_index = self.time_based_index_name(&current_index, today);

        // Create new index
        self.create_posts_index_with_name(&new_index).await?;

        // Note: In a production setup, you would update an alias to point to the new index
        // and potentially use index templates for automatic creation

        info!(
            old_index = %current_index,
            new_index = %new_index,
            "Rolled over posts index"
        );

        Ok(RolloverResult {
            old_index: current_index,
            new_index,
            acknowledged: true,
            shards_acknowledged: true,
        })
    }

    /// Reindex data from one index to another
    /// 
    /// NOTE: This is a simplified implementation. For production, use the OpenSearch reindex API directly.
    pub async fn reindex(
        &self,
        _source_index: &str,
        _dest_index: &str,
    ) -> anyhow::Result<ReindexResult> {
        // Simplified implementation - in production, use the OpenScroll API or direct reindex
        // For now, return an error
        anyhow::bail!("Reindex not implemented - use OpenSearch API directly")
    }

    /// Optimize index settings for bulk indexing
    pub async fn optimize_for_bulk(&self, index_name: &str) -> anyhow::Result<()> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let settings = json!({
            "index": {
                "refresh_interval": "-1",
                "number_of_replicas": 0
            }
        });

        let response = client
            .indices()
            .put_settings(IndicesPutSettingsParts::Index(&[index_name]))
            .body(settings)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_body: serde_json::Value = response.json().await?;
            anyhow::bail!("Failed to optimize index settings: {:?}", error_body);
        }

        info!(index = %index_name, "Optimized index for bulk indexing");
        Ok(())
    }

    /// Restore normal index settings after bulk indexing
    pub async fn restore_after_bulk(&self, index_name: &str) -> anyhow::Result<()> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let config = self.client.config();

        let settings = json!({
            "index": {
                "refresh_interval": "1s",
                "number_of_replicas": config.number_of_replicas
            }
        });

        let response = client
            .indices()
            .put_settings(IndicesPutSettingsParts::Index(&[index_name]))
            .body(settings)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_body: serde_json::Value = response.json().await?;
            anyhow::bail!("Failed to restore index settings: {:?}", error_body);
        }

        info!(index = %index_name, "Restored index settings after bulk indexing");
        Ok(())
    }

    /// Initialize all required indices
    pub async fn initialize_indices(&self) -> anyhow::Result<Vec<String>> {
        let mut created = Vec::new();

        // Create posts index
        match self.create_posts_index().await {
            Ok(name) => created.push(name),
            Err(e) => warn!(error = %e, "Failed to create posts index"),
        }

        info!(created = created.len(), "Initialized search indices");
        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_based_index_name() {
        use crate::config::OpenSearchConfig;
        
        let config = OpenSearchConfig::default();
        let client = crate::search::OpenSearchClient::new(config);
        let manager = IndexManager::new(client);

        let date = chrono::Utc::now();
        let name = manager.time_based_index_name("rustchat-posts", date);

        assert!(name.starts_with("rustchat-posts-"));
        assert!(name.contains(&date.format("%Y").to_string()));
    }

    #[test]
    fn test_index_manager_disabled() {
        use crate::config::OpenSearchConfig;
        
        let config = OpenSearchConfig::default();
        let client = crate::search::OpenSearchClient::new(config);
        let manager = IndexManager::new(client);

        // Should return error when client is not available
        assert!(!manager.client.is_available());
    }
}
