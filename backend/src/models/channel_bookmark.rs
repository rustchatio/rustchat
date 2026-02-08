//! Channel Bookmarks model for Mattermost compatibility

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Channel bookmark types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BookmarkType {
    Link,
    File,
}

impl Default for BookmarkType {
    fn default() -> Self {
        Self::Link
    }
}

/// Database row for channel bookmark
#[derive(Debug, Clone, FromRow)]
pub struct ChannelBookmark {
    pub id: Uuid,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub channel_id: Uuid,
    pub owner_id: Uuid,
    pub file_id: Option<Uuid>,
    pub display_name: String,
    pub sort_order: i64,
    pub link_url: Option<String>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
    pub bookmark_type: String,
    pub original_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
}

/// Create bookmark request
#[derive(Debug, Deserialize)]
pub struct CreateBookmarkRequest {
    pub display_name: String,
    #[serde(rename = "type")]
    pub bookmark_type: BookmarkType,
    pub link_url: Option<String>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
    pub file_id: Option<String>,
}

/// Update bookmark request
#[derive(Debug, Deserialize)]
pub struct UpdateBookmarkRequest {
    pub display_name: Option<String>,
    pub link_url: Option<String>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
    pub file_id: Option<String>,
    pub sort_order: Option<i64>,
}
