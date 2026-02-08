//! Scheduled Posts model for Mattermost mobile compatibility

use serde::{Deserialize, Serialize};
use uuid::Uuid;


/// Scheduled post record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledPost {
    pub id: Uuid,
    pub create_at: i64,
    pub update_at: i64,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub root_id: Option<Uuid>,
    pub message: String,
    pub props: serde_json::Value,
    pub file_ids: Vec<Uuid>,
    pub priority: Option<serde_json::Value>,
    pub scheduled_at: i64,
    pub processed_at: i64,
    pub error_code: String,
}

/// Request to create a new scheduled post
#[derive(Debug, Clone, Deserialize)]
pub struct CreateScheduledPostRequest {
    #[serde(default)]
    pub channel_id: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub root_id: Option<String>,
    #[serde(default)]
    pub file_ids: Vec<String>,
    #[serde(default)]
    pub props: Option<serde_json::Value>,
    #[serde(default)]
    pub priority: Option<serde_json::Value>,
    pub scheduled_at: i64,
}

/// Request to update an existing scheduled post
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateScheduledPostRequest {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub props: Option<serde_json::Value>,
    #[serde(default)]
    pub file_ids: Option<Vec<String>>,
    #[serde(default)]
    pub priority: Option<serde_json::Value>,
    #[serde(default)]
    pub scheduled_at: Option<i64>,
}

/// Response format for scheduled posts API (Mattermost compatible)
#[derive(Debug, Clone, Serialize)]
pub struct ScheduledPostResponse {
    pub id: String,
    pub create_at: i64,
    pub update_at: i64,
    pub user_id: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_id: Option<String>,
    pub message: String,
    pub props: serde_json::Value,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<serde_json::Value>,
    pub scheduled_at: i64,
    pub processed_at: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error_code: String,
}

/// Response for fetching scheduled posts (matches Mattermost FetchScheduledPostsResponse)
#[derive(Debug, Clone, Serialize)]
pub struct FetchScheduledPostsResponse {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scheduled_posts: Vec<ScheduledPostResponse>,
}

impl From<ScheduledPost> for ScheduledPostResponse {
    fn from(sp: ScheduledPost) -> Self {
        use crate::mattermost_compat::id::encode_mm_id;
        
        Self {
            id: encode_mm_id(sp.id),
            create_at: sp.create_at,
            update_at: sp.update_at,
            user_id: encode_mm_id(sp.user_id),
            channel_id: encode_mm_id(sp.channel_id),
            root_id: sp.root_id.map(encode_mm_id),
            message: sp.message,
            props: sp.props,
            file_ids: sp.file_ids.into_iter().map(encode_mm_id).collect(),
            priority: sp.priority,
            scheduled_at: sp.scheduled_at,
            processed_at: sp.processed_at,
            error_code: sp.error_code,
        }
    }
}
