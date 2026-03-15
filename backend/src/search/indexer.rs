//! OpenSearch indexer for rustchat
//!
//! Provides document indexing, bulk operations, and index mapping definitions.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use opensearch::IndexParts;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

use crate::search::OpenSearchClient;

/// Post document for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostDocument {
    pub id: String,
    pub channel_id: String,
    pub user_id: String,
    pub root_post_id: Option<String>,
    pub message: String,
    pub message_analyzed: String,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    pub reply_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reply_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashtags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    pub has_attachments: bool,
    pub is_reply: bool,
}

impl PostDocument {
    /// Create a PostDocument from database post data
    #[allow(clippy::too_many_arguments)]
    pub fn from_post(
        post_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
        root_post_id: Option<Uuid>,
        message: String,
        is_pinned: bool,
        created_at: DateTime<Utc>,
        edited_at: Option<DateTime<Utc>>,
        reply_count: i64,
        last_reply_at: Option<DateTime<Utc>>,
        file_ids: &[Uuid],
    ) -> Self {
        // Extract hashtags and mentions from message
        let hashtags = extract_hashtags(&message);
        let mentions = extract_mentions(&message);

        Self {
            id: post_id.to_string(),
            channel_id: channel_id.to_string(),
            user_id: user_id.to_string(),
            root_post_id: root_post_id.map(|id| id.to_string()),
            message: message.clone(),
            message_analyzed: message, // Will be analyzed by OpenSearch
            is_pinned,
            created_at,
            edited_at,
            deleted_at: None,
            reply_count,
            last_reply_at,
            hashtags: if hashtags.is_empty() { None } else { Some(hashtags) },
            mentions: if mentions.is_empty() { None } else { Some(mentions) },
            has_attachments: !file_ids.is_empty(),
            is_reply: root_post_id.is_some(),
        }
    }
}

/// Extract hashtags from message text
fn extract_hashtags(message: &str) -> Vec<String> {
    extract_prefixed_tokens(message, '#')
}

/// Extract mentions from message text
fn extract_mentions(message: &str) -> Vec<String> {
    extract_prefixed_tokens(message, '@')
}

fn extract_prefixed_tokens(message: &str, prefix: char) -> Vec<String> {
    message
        .split_whitespace()
        .filter_map(|word| word.strip_prefix(prefix))
        .filter_map(normalize_token)
        .collect()
}

fn normalize_token(raw: &str) -> Option<String> {
    let mut normalized = String::new();
    for ch in raw.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            normalized.extend(ch.to_lowercase());
        } else {
            break;
        }
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Index operation result
#[derive(Debug, Clone)]
pub struct IndexResult {
    pub index: String,
    pub id: String,
    pub version: i64,
    pub result: String, // "created" or "updated"
}

/// Bulk operation result
#[derive(Debug, Clone)]
pub struct BulkResult {
    pub took: i64,
    pub errors: bool,
    pub items_processed: usize,
    pub items_failed: usize,
}

/// OpenSearch indexer
pub struct SearchIndexer {
    client: Arc<OpenSearchClient>,
}

impl SearchIndexer {
    /// Create a new indexer
    pub fn new(client: Arc<OpenSearchClient>) -> Arc<Self> {
        Arc::new(Self { client })
    }

    /// Index a single post document
    pub async fn index_post(&self, doc: &PostDocument) -> anyhow::Result<IndexResult> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let index = self.client.posts_index();

        let response = client
            .index(IndexParts::IndexId(&index, &doc.id))
            .body(json!(doc))
            .send()
            .await?;

        let status = response.status_code();
        if !status.is_success() {
            let error_body: serde_json::Value = response.json().await?;
            anyhow::bail!(
                "Failed to index document: {} - {:?}",
                status,
                error_body
            );
        }

        let body: serde_json::Value = response.json().await?;

        Ok(IndexResult {
            index: body
                .get("_index")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            id: body
                .get("_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: body.get("_version").and_then(|v| v.as_i64()).unwrap_or(0),
            result: body
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    /// Delete a post document from the index
    pub async fn delete_post(&self, post_id: &str) -> anyhow::Result<()> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let index = self.client.posts_index();

        let response = client
            .delete(opensearch::DeleteParts::IndexId(&index, post_id))
            .send()
            .await?;

        if !response.status_code().is_success() {
            // 404 is OK - document didn't exist
            if response.status_code().as_u16() != 404 {
                let error_body: serde_json::Value = response.json().await?;
                anyhow::bail!(
                    "Failed to delete document: {:?}",
                    error_body
                );
            }
        }

        Ok(())
    }

    /// Bulk index multiple documents
    /// 
    /// NOTE: This implementation indexes documents one by one.
    /// For production with high volume, use the OpenSearch bulk API directly.
    pub async fn bulk_index(&self, docs: &[PostDocument]) -> anyhow::Result<BulkResult> {
        if docs.is_empty() {
            return Ok(BulkResult {
                took: 0,
                errors: false,
                items_processed: 0,
                items_failed: 0,
            });
        }

        let start = std::time::Instant::now();
        let mut errors = false;
        let mut items_failed = 0;

        // Index documents one by one
        // For production, use the OpenSearch bulk API directly
        for doc in docs {
            match self.index_post(doc).await {
                Ok(_) => {}
                Err(e) => {
                    warn!(error = %e, doc_id = %doc.id, "Failed to index document");
                    errors = true;
                    items_failed += 1;
                }
            }
        }

        let took = start.elapsed().as_millis() as i64;
        let items_processed = docs.len();

        Ok(BulkResult {
            took,
            errors,
            items_processed,
            items_failed,
        })
    }

    /// Update a post document (partial update)
    pub async fn update_post(
        &self,
        post_id: &str,
        message: Option<String>,
        is_pinned: Option<bool>,
    ) -> anyhow::Result<IndexResult> {
        let client = self
            .client
            .client()
            .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

        let index = self.client.posts_index();

        // Build update script
        let mut doc = serde_json::Map::new();

        if let Some(msg) = message {
            doc.insert("message".to_string(), json!(msg.clone()));
            doc.insert("message_analyzed".to_string(), json!(msg));
            doc.insert("edited_at".to_string(), json!(chrono::Utc::now()));

            // Re-extract hashtags and mentions
            let hashtags = extract_hashtags(&msg);
            let mentions = extract_mentions(&msg);

            if !hashtags.is_empty() {
                doc.insert("hashtags".to_string(), json!(hashtags));
            }
            if !mentions.is_empty() {
                doc.insert("mentions".to_string(), json!(mentions));
            }
        }

        if let Some(pinned) = is_pinned {
            doc.insert("is_pinned".to_string(), json!(pinned));
        }

        let response = client
            .update(opensearch::UpdateParts::IndexId(&index, post_id))
            .body(json!({ "doc": doc }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_body: serde_json::Value = response.json().await?;
            anyhow::bail!("Failed to update document: {:?}", error_body);
        }

        let body: serde_json::Value = response.json().await?;

        Ok(IndexResult {
            index: body
                .get("_index")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            id: body
                .get("_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: body.get("_version").and_then(|v| v.as_i64()).unwrap_or(0),
            result: body
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

/// Index mapping for posts
pub fn posts_index_mapping() -> serde_json::Value {
    json!({
        "properties": {
            "id": {
                "type": "keyword"
            },
            "channel_id": {
                "type": "keyword"
            },
            "user_id": {
                "type": "keyword"
            },
            "root_post_id": {
                "type": "keyword"
            },
            "message": {
                "type": "text",
                "analyzer": "standard",
                "fields": {
                    "keyword": {
                        "type": "keyword",
                        "ignore_above": 32766
                    }
                }
            },
            "message_analyzed": {
                "type": "text",
                "analyzer": "standard"
            },
            "is_pinned": {
                "type": "boolean"
            },
            "created_at": {
                "type": "date"
            },
            "edited_at": {
                "type": "date"
            },
            "deleted_at": {
                "type": "date"
            },
            "reply_count": {
                "type": "integer"
            },
            "last_reply_at": {
                "type": "date"
            },
            "hashtags": {
                "type": "keyword"
            },
            "mentions": {
                "type": "keyword"
            },
            "has_attachments": {
                "type": "boolean"
            },
            "is_reply": {
                "type": "boolean"
            }
        }
    })
}

/// Index settings for posts
pub fn posts_index_settings(shards: u32, replicas: u32) -> serde_json::Value {
    json!({
        "number_of_shards": shards,
        "number_of_replicas": replicas,
        "refresh_interval": "1s",
        "analysis": {
            "analyzer": {
                "default": {
                    "type": "standard"
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hashtags() {
        let message = "Hello #world! This is a #test message with #multiple #hashtags";
        let hashtags = extract_hashtags(message);
        assert_eq!(hashtags, vec!["world", "test", "multiple", "hashtags"]);
    }

    #[test]
    fn test_extract_mentions() {
        let message = "Hey @alice and @bob, check this out! @charlie";
        let mentions = extract_mentions(message);
        assert_eq!(mentions, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn test_post_document_creation() {
        let doc = PostDocument::from_post(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Hello #world!".to_string(),
            false,
            chrono::Utc::now(),
            None,
            0,
            None,
            &[],
        );

        assert_eq!(doc.hashtags, Some(vec!["world".to_string()]));
        assert!(!doc.has_attachments);
        assert!(!doc.is_reply);
    }

    #[test]
    fn test_posts_index_mapping() {
        let mapping = posts_index_mapping();
        assert!(mapping.get("properties").is_some());
        assert!(mapping["properties"].get("message").is_some());
        assert!(mapping["properties"].get("hashtags").is_some());
    }
}
