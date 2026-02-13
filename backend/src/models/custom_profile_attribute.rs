//! Custom Profile Attributes model for Mattermost mobile compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Custom profile field definition (schema for attributes)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomProfileField {
    pub id: Uuid,
    pub group_id: String,
    pub name: String,
    pub field_type: String,
    pub attrs: serde_json::Value,
    pub target_id: String,
    pub target_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Custom profile attribute value (actual value for a user)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomProfileAttribute {
    pub id: Uuid,
    pub field_id: Uuid,
    pub user_id: Uuid,
    pub value: String,
}

/// Response format for custom profile fields (Mattermost compatible)
#[derive(Debug, Clone, Serialize)]
pub struct CustomProfileFieldResponse {
    pub id: String,
    pub group_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<serde_json::Value>,
    pub target_id: String,
    pub target_type: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
}

impl From<CustomProfileField> for CustomProfileFieldResponse {
    fn from(f: CustomProfileField) -> Self {
        use crate::mattermost_compat::id::encode_mm_id;
        Self {
            id: encode_mm_id(f.id),
            group_id: f.group_id,
            name: f.name,
            field_type: f.field_type,
            attrs: if f.attrs.is_null() {
                None
            } else {
                Some(f.attrs)
            },
            target_id: f.target_id,
            target_type: f.target_type,
            create_at: f.created_at.timestamp_millis(),
            update_at: f.updated_at.timestamp_millis(),
            delete_at: f.deleted_at.map(|d| d.timestamp_millis()).unwrap_or(0),
        }
    }
}

/// Simple map of field_id -> value for user attributes
pub type UserCustomProfileAttributeSimple = HashMap<String, serde_json::Value>;
