//! Activity feed API endpoints

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::parse_mm_or_uuid;
use crate::models::{ActivityFeedResponse, ActivityQuery, MarkReadRequest};
use crate::services::activity;

/// Query parameters for GET /users/{user_id}/activity
#[derive(Debug, Deserialize, Default)]
pub struct GetActivityParams {
    pub cursor: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(rename = "type")]
    pub activity_type: Option<String>,
    #[serde(default)]
    pub unread_only: bool,
}

fn default_limit() -> i64 {
    50
}

/// Resolves the target user ID, enforcing that callers can only access their own activity feed.
/// Unlike the shared resolve_user_id, this intentionally does NOT grant admin bypass — activity
/// feeds contain personal notification data that admins have no legitimate reason to read.
fn resolve_activity_user_id(user_id_str: &str, auth: &MmAuthUser) -> ApiResult<uuid::Uuid> {
    if user_id_str == "me" {
        return Ok(auth.user_id);
    }
    let user_id = parse_mm_or_uuid(user_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;
    if user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot access another user's activity feed".to_string(),
        ));
    }
    Ok(user_id)
}

/// Build the activity router - called from users.rs router function
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/{user_id}/activity", get(get_activity_feed))
        .route("/users/{user_id}/activity/read", post(mark_read))
        .route("/users/{user_id}/activity/read-all", post(mark_all_read))
}

/// GET /api/v4/users/{user_id}/activity
async fn get_activity_feed(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<GetActivityParams>,
    auth: MmAuthUser,
) -> ApiResult<Json<ActivityFeedResponse>> {
    let target_id = resolve_activity_user_id(&user_id, &auth)?;

    let query = ActivityQuery {
        cursor: params.cursor,
        limit: params.limit,
        activity_type: params.activity_type,
        unread_only: params.unread_only,
    };

    let response = activity::get_activities(&state, target_id, query).await?;
    Ok(Json(response))
}

/// POST /api/v4/users/{user_id}/activity/read
async fn mark_read(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    auth: MmAuthUser,
    Json(payload): Json<MarkReadRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let target_id = resolve_activity_user_id(&user_id, &auth)?;
    let updated = activity::mark_activities_read(&state, target_id, payload.activity_ids).await?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}

/// POST /api/v4/users/{user_id}/activity/read-all
async fn mark_all_read(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    auth: MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let target_id = resolve_activity_user_id(&user_id, &auth)?;
    let updated = activity::mark_all_read(&state, target_id).await?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}
