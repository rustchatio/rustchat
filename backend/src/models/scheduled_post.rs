//! Scheduled Post model for Mattermost mobile compatibility
//! Matches existing scheduled_posts table from 20260130000003_advanced_messaging.sql

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Scheduled post stored in DB (matches existing table schema)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledPost {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub root_id: Option<Uuid>,
    pub message: String,
    pub props: serde_json::Value,
    pub file_ids: Vec<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub state: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a scheduled post
#[derive(Debug, Clone, Deserialize)]
pub struct CreateScheduledPostRequest {
    pub channel_id: String,
    pub message: String,
    #[serde(default)]
    pub root_id: Option<String>,
    #[serde(default)]
    pub file_ids: Option<Vec<String>>,
    pub scheduled_at: i64, // Mattermost uses epoch milliseconds
    #[serde(default)]
    pub props: Option<serde_json::Value>,
}

/// Request to update a scheduled post
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateScheduledPostRequest {
    pub message: Option<String>,
    pub scheduled_at: Option<i64>,
    #[serde(default)]
    pub file_ids: Option<Vec<String>>,
    #[serde(default)]
    pub props: Option<serde_json::Value>,
}

/// Response format for scheduled posts (Mattermost compatible)
#[derive(Debug, Clone, Serialize)]
pub struct ScheduledPostResponse {
    pub id: String,
    pub user_id: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_id: Option<String>,
    pub message: String,
    pub file_ids: Vec<String>,
    pub scheduled_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

impl ScheduledPost {
    pub fn to_response(&self) -> ScheduledPostResponse {
        use crate::mattermost_compat::id::encode_mm_id;
        ScheduledPostResponse {
            id: encode_mm_id(self.id),
            user_id: encode_mm_id(self.user_id),
            channel_id: encode_mm_id(self.channel_id),
            root_id: self.root_id.map(encode_mm_id),
            message: self.message.clone(),
            file_ids: self.file_ids.iter().map(|id| encode_mm_id(*id)).collect(),
            scheduled_at: self.scheduled_at.timestamp_millis(),
            props: if self.props.is_null() {
                None
            } else {
                Some(self.props.clone())
            },
            error_code: self.error_message.clone(),
        }
    }
}

/// Response for fetching scheduled posts
#[derive(Debug, Clone, Serialize)]
pub struct FetchScheduledPostsResponse {
    pub scheduled_posts: Vec<ScheduledPostResponse>,
}
