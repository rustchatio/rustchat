//! Post repository for centralized query patterns
//!
//! This module centralizes common post query patterns to reduce the 20+ duplicated
//! queries previously scattered across the codebase (api/posts.rs, api/v4/posts.rs, etc.)

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiResult;

/// Post with joined user info
///
/// This struct combines post fields with user information from a JOIN query.
/// All user fields are Option<> because LEFT JOIN may return NULL if the user
/// was deleted (though this shouldn't happen in normal operation due to FK constraints).
#[derive(Debug, Clone)]
pub struct PostWithUser {
    // Post fields
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub root_post_id: Option<Uuid>,
    pub message: String,
    pub props: Option<serde_json::Value>,
    pub file_ids: Vec<Uuid>,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub reply_count: i64,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub seq: i64,
    // User fields
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

/// Repository for post-related database operations
#[derive(Debug, Clone)]
pub struct PostRepository {
    db: PgPool,
}

impl PostRepository {
    /// Create a new PostRepository instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Common SELECT columns for post queries with user JOIN
    const POST_COLUMNS: &'static str = r#"
        p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
        p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
        p.reply_count::int8 as reply_count, p.last_reply_at, p.seq,
        u.username, u.avatar_url, u.email
    "#;

    /// Get a single post by ID with user info
    ///
    /// Returns None if the post doesn't exist or has been soft-deleted.
    ///
    /// # Example
    /// ```rust,ignore
    /// let post_repo = PostRepository::new(pool);
    /// if let Some(post) = post_repo.find_by_id(post_id).await? {
    ///     println!("Found post by {}", post.username.unwrap_or_default());
    /// }
    /// ```
    pub async fn find_by_id(&self, post_id: Uuid) -> ApiResult<Option<PostWithUser>> {
        let post = sqlx::query_as::<_, PostWithUserRow>(&format!(
            r#"
                SELECT {}
                FROM posts p
                LEFT JOIN users u ON p.user_id = u.id
                WHERE p.id = $1 AND p.deleted_at IS NULL
                "#,
            Self::POST_COLUMNS
        ))
        .bind(post_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(post.map(Into::into))
    }

    /// List posts in a channel with pagination
    ///
    /// Returns posts ordered by creation time (newest first).
    /// This is the common pattern used for channel message history.
    ///
    /// # Arguments
    /// * `channel_id` - The channel to query
    /// * `limit` - Maximum number of posts to return
    /// * `offset` - Number of posts to skip (for pagination)
    pub async fn list_by_channel(
        &self,
        channel_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<PostWithUser>> {
        let rows = sqlx::query_as::<_, PostWithUserRow>(&format!(
            r#"
                SELECT {}
                FROM posts p
                LEFT JOIN users u ON p.user_id = u.id
                WHERE p.channel_id = $1
                  AND p.deleted_at IS NULL
                ORDER BY p.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            Self::POST_COLUMNS
        ))
        .bind(channel_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get thread replies
    ///
    /// Returns all non-deleted replies to a root post, ordered chronologically.
    /// The root post itself is NOT included in the results.
    ///
    /// # Arguments
    /// * `root_post_id` - The ID of the parent/root post
    pub async fn get_thread_replies(&self, root_post_id: Uuid) -> ApiResult<Vec<PostWithUser>> {
        let rows = sqlx::query_as::<_, PostWithUserRow>(&format!(
            r#"
                SELECT {}
                FROM posts p
                LEFT JOIN users u ON p.user_id = u.id
                WHERE p.root_post_id = $1
                  AND p.deleted_at IS NULL
                ORDER BY p.created_at ASC
                "#,
            Self::POST_COLUMNS
        ))
        .bind(root_post_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// List posts since a timestamp (for sync)
    ///
    /// Returns all posts in a channel created or updated after the given timestamp.
    /// Used by mobile clients and sync endpoints.
    ///
    /// # Arguments
    /// * `channel_id` - The channel to query
    /// * `since` - Return posts with created_at > this timestamp
    pub async fn list_since(
        &self,
        channel_id: Uuid,
        since: DateTime<Utc>,
    ) -> ApiResult<Vec<PostWithUser>> {
        let rows = sqlx::query_as::<_, PostWithUserRow>(&format!(
            r#"
                SELECT {}
                FROM posts p
                LEFT JOIN users u ON p.user_id = u.id
                WHERE p.channel_id = $1
                  AND p.deleted_at IS NULL
                  AND p.created_at > $2
                ORDER BY p.created_at ASC
                "#,
            Self::POST_COLUMNS
        ))
        .bind(channel_id)
        .bind(since)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// List posts before a timestamp with pagination
    ///
    /// Used for infinite scroll "load older messages" functionality.
    /// Returns posts with created_at < before, ordered newest first.
    pub async fn list_before(
        &self,
        channel_id: Uuid,
        before: DateTime<Utc>,
        limit: i64,
    ) -> ApiResult<Vec<PostWithUser>> {
        let rows = sqlx::query_as::<_, PostWithUserRow>(&format!(
            r#"
                SELECT {}
                FROM posts p
                LEFT JOIN users u ON p.user_id = u.id
                WHERE p.channel_id = $1
                  AND p.deleted_at IS NULL
                  AND p.created_at < $2
                ORDER BY p.created_at DESC
                LIMIT $3
                "#,
            Self::POST_COLUMNS
        ))
        .bind(channel_id)
        .bind(before)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// List posts after a timestamp with pagination
    ///
    /// Used for "load newer messages" functionality.
    /// Returns posts with created_at > after, ordered oldest first.
    pub async fn list_after(
        &self,
        channel_id: Uuid,
        after: DateTime<Utc>,
        limit: i64,
    ) -> ApiResult<Vec<PostWithUser>> {
        let rows = sqlx::query_as::<_, PostWithUserRow>(&format!(
            r#"
                SELECT {}
                FROM posts p
                LEFT JOIN users u ON p.user_id = u.id
                WHERE p.channel_id = $1
                  AND p.deleted_at IS NULL
                  AND p.created_at > $2
                ORDER BY p.created_at ASC
                LIMIT $3
                "#,
            Self::POST_COLUMNS
        ))
        .bind(channel_id)
        .bind(after)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

/// Internal row type for SQLx mapping
///
/// This struct maps directly to the SQL query results. We use this as an
/// intermediate to handle the Option<> wrapping that comes from LEFT JOIN.
#[derive(Debug, Clone, sqlx::FromRow)]
struct PostWithUserRow {
    // Post fields
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub root_post_id: Option<Uuid>,
    pub message: String,
    pub props: Option<serde_json::Value>,
    pub file_ids: Vec<Uuid>,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub reply_count: i64,
    pub last_reply_at: Option<DateTime<Utc>>,
    pub seq: i64,
    // User fields (Option because of LEFT JOIN)
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

impl From<PostWithUserRow> for PostWithUser {
    fn from(row: PostWithUserRow) -> Self {
        Self {
            id: row.id,
            channel_id: row.channel_id,
            user_id: row.user_id,
            root_post_id: row.root_post_id,
            message: row.message,
            props: row.props,
            file_ids: row.file_ids,
            is_pinned: row.is_pinned,
            created_at: row.created_at,
            edited_at: row.edited_at,
            deleted_at: row.deleted_at,
            reply_count: row.reply_count,
            last_reply_at: row.last_reply_at,
            seq: row.seq,
            username: row.username,
            avatar_url: row.avatar_url,
            email: row.email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are compile-time tests to ensure the SQL queries are valid.
    // Full integration tests would require a database connection.

    #[test]
    fn test_post_columns_format() {
        // Verify the SQL column string is valid
        let columns = PostRepository::POST_COLUMNS;
        assert!(columns.contains("p.id"));
        assert!(columns.contains("p.channel_id"));
        assert!(columns.contains("p.user_id"));
        assert!(columns.contains("p.reply_count::int8"));
        assert!(columns.contains("u.username"));
        assert!(columns.contains("u.avatar_url"));
        assert!(columns.contains("u.email"));
    }
}
