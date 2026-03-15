//! Search API endpoints
//!
//! Provides full-text search with OpenSearch (if available) or PostgreSQL FTS fallback.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::AppState;
use crate::auth::AuthUser;
use crate::error::{ApiResult, AppError};
use crate::models::Post;

/// Build search routes
pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search_messages))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub channel_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    #[serde(default)]
    pub use_opensearch: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub posts: Vec<Post>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub source: SearchSource,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchSource {
    Opensearch,
    Postgresql,
}

/// Full-text search for messages
///
/// Tries OpenSearch first if available and requested, falls back to PostgreSQL FTS
async fn search_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<SearchResult>> {
    if query.q.trim().is_empty() {
        return Err(AppError::Validation(
            "Search query cannot be empty".to_string(),
        ));
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Try OpenSearch if requested and available
    if query.use_opensearch {
        if let Some(ref search_client) = state.opensearch_client {
            match search_with_opensearch(
                search_client.clone(),
                &state.db,
                &query.q,
                query.channel_id,
                auth.user_id,
                page,
                per_page,
                offset,
            )
            .await
            {
                Ok(result) => return Ok(Json(result)),
                Err(e) => {
                    warn!(error = %e, "OpenSearch query failed, falling back to PostgreSQL");
                }
            }
        }
    }

    // Fallback to PostgreSQL FTS
    let result = search_with_postgresql(
        &state.db,
        &query.q,
        query.channel_id,
        auth.user_id,
        page,
        per_page,
        offset,
    )
    .await?;

    Ok(Json(result))
}

/// Search using OpenSearch
#[allow(clippy::too_many_arguments)]
async fn search_with_opensearch(
    search_client: Arc<crate::search::OpenSearchClient>,
    db: &sqlx::PgPool,
    query: &str,
    channel_id: Option<Uuid>,
    user_id: Uuid,
    page: i64,
    per_page: i64,
    offset: i64,
) -> anyhow::Result<SearchResult> {
    use opensearch::SearchParts;

    let client = search_client
        .client()
        .ok_or_else(|| anyhow::anyhow!("OpenSearch client not available"))?;

    let index = search_client.posts_index();

    // Build the search query
    let mut bool_query = serde_json::json!({
        "must": [
            {
                "multi_match": {
                    "query": query,
                    "fields": ["message^2", "message_analyzed"],
                    "type": "best_fields",
                    "fuzziness": "AUTO"
                }
            }
        ],
        "filter": [
            {
                "term": {
                    "deleted_at": null
                }
            }
        ]
    });

    // Add channel filter if specified
    if let Some(cid) = channel_id {
        bool_query["filter"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "term": {
                    "channel_id": cid.to_string()
                }
            }));
    }

    let search_body = serde_json::json!({
        "query": {
            "bool": bool_query
        },
        "sort": [
            { "_score": "desc" },
            { "created_at": "desc" }
        ],
        "from": offset,
        "size": per_page,
        "highlight": {
            "fields": {
                "message": {
                    "fragment_size": 150,
                    "number_of_fragments": 1
                }
            }
        }
    });

    debug!(index = %index, query = %query, "Searching with OpenSearch");

    let response = client
        .search(SearchParts::Index(&[&index]))
        .body(search_body)
        .send()
        .await?;

    if !response.status_code().is_success() {
        let error_body: serde_json::Value = response.json().await?;
        anyhow::bail!("OpenSearch query failed: {:?}", error_body);
    }

    let result: serde_json::Value = response.json().await?;

    let total = result
        .get("hits")
        .and_then(|h| h.get("total"))
        .and_then(|t| t.get("value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let hits = result
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .cloned()
        .unwrap_or_default();

    // Extract post IDs from search results
    let post_ids: Vec<Uuid> = hits
        .iter()
        .filter_map(|hit| {
            hit.get("_id")
                .and_then(|id| id.as_str())
                .and_then(|id| Uuid::parse_str(id).ok())
        })
        .collect();

    // Fetch full post data from database
    let posts = if post_ids.is_empty() {
        Vec::new()
    } else {
        fetch_posts_by_ids(db, &post_ids, user_id).await?
    };

    info!(
        query = %query,
        total = total,
        returned = posts.len(),
        "OpenSearch query completed"
    );

    Ok(SearchResult {
        posts,
        total,
        page,
        per_page,
        source: SearchSource::Opensearch,
    })
}

/// Search using PostgreSQL FTS
async fn search_with_postgresql(
    db: &sqlx::PgPool,
    query: &str,
    channel_id: Option<Uuid>,
    user_id: Uuid,
    page: i64,
    per_page: i64,
    offset: i64,
) -> Result<SearchResult, crate::error::AppError> {
    debug!(query = %query, "Searching with PostgreSQL FTS");

    let posts: Vec<Post> = if let Some(cid) = channel_id {
        sqlx::query_as(
            r#"
            SELECT p.* FROM posts p
            INNER JOIN channel_members cm ON cm.channel_id = p.channel_id AND cm.user_id = $1
            WHERE p.channel_id = $2
              AND p.deleted_at IS NULL
              AND to_tsvector('english', p.message) @@ plainto_tsquery('english', $3)
            ORDER BY ts_rank(to_tsvector('english', p.message), plainto_tsquery('english', $3)) DESC, p.created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(user_id)
        .bind(cid)
        .bind(query)
        .bind(per_page)
        .bind(offset)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT p.* FROM posts p
            INNER JOIN channel_members cm ON cm.channel_id = p.channel_id AND cm.user_id = $1
            WHERE p.deleted_at IS NULL
              AND to_tsvector('english', p.message) @@ plainto_tsquery('english', $2)
            ORDER BY ts_rank(to_tsvector('english', p.message), plainto_tsquery('english', $2)) DESC, p.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_id)
        .bind(query)
        .bind(per_page)
        .bind(offset)
        .fetch_all(db)
        .await?
    };

    // Get total count (approximate for pagination)
    let total = posts.len() as i64;

    info!(
        query = %query,
        returned = posts.len(),
        "PostgreSQL FTS query completed"
    );

    Ok(SearchResult {
        posts,
        total,
        page,
        per_page,
        source: SearchSource::Postgresql,
    })
}

/// Convert anyhow::Error to AppError
#[allow(dead_code)]
fn map_search_error(e: anyhow::Error) -> crate::error::AppError {
    crate::error::AppError::Internal(format!("Search error: {}", e))
}

/// Fetch posts by IDs, preserving order
async fn fetch_posts_by_ids(
    db: &sqlx::PgPool,
    post_ids: &[Uuid],
    user_id: Uuid,
) -> anyhow::Result<Vec<Post>> {
    if post_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Use unnest to pass array of IDs
    let posts: Vec<Post> = sqlx::query_as(
        r#"
        SELECT p.* FROM posts p
        INNER JOIN channel_members cm ON cm.channel_id = p.channel_id AND cm.user_id = $1
        WHERE p.id = ANY($2)
          AND p.deleted_at IS NULL
        ORDER BY array_position($2, p.id)
        "#,
    )
    .bind(user_id)
    .bind(post_ids)
    .fetch_all(db)
    .await?;

    Ok(posts)
}

/// Index a post in OpenSearch (called when posts are created/updated)
#[allow(dead_code)]
pub async fn index_post(
    indexer: Option<Arc<crate::search::SearchIndexer>>,
    post: &Post,
) -> anyhow::Result<()> {
    if let Some(idx) = indexer {
        use crate::search::indexer::PostDocument;

        let doc = PostDocument::from_post(
            post.id,
            post.channel_id,
            post.user_id,
            post.root_post_id,
            post.message.clone(),
            post.is_pinned,
            post.created_at,
            post.edited_at,
            post.reply_count,
            post.last_reply_at,
            &post.file_ids,
        );

        idx.index_post(&doc).await?;
        debug!(post_id = %post.id, "Indexed post in OpenSearch");
    }

    Ok(())
}

/// Delete a post from OpenSearch index
#[allow(dead_code)]
pub async fn delete_post_index(
    indexer: Option<Arc<crate::search::SearchIndexer>>,
    post_id: Uuid,
) -> anyhow::Result<()> {
    if let Some(idx) = indexer {
        idx.delete_post(&post_id.to_string()).await?;
        debug!(post_id = %post_id, "Deleted post from OpenSearch index");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_defaults() {
        let query_str = "test query";
        let query = SearchQuery {
            q: query_str.to_string(),
            channel_id: None,
            page: None,
            per_page: None,
            use_opensearch: false,
        };

        assert_eq!(query.q, "test query");
        assert!(query.channel_id.is_none());
        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
        assert!(!query.use_opensearch);
    }

    #[test]
    fn test_search_source_serialization() {
        let source = SearchSource::Opensearch;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"opensearch\"");

        let source = SearchSource::Postgresql;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"postgresql\"");
    }
}
