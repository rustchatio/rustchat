use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};

/// Bookmark with optional file info for API responses
#[derive(serde::Serialize)]
pub struct ChannelBookmarkResponse {
    id: String,
    create_at: i64,
    update_at: i64,
    delete_at: i64,
    channel_id: String,
    owner_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_id: Option<String>,
    display_name: String,
    sort_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<String>,
    #[serde(rename = "type")]
    bookmark_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<serde_json::Value>,
}

#[derive(sqlx::FromRow)]
struct BookmarkRow {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    channel_id: Uuid,
    owner_id: Uuid,
    file_id: Option<Uuid>,
    display_name: String,
    sort_order: i64,
    link_url: Option<String>,
    image_url: Option<String>,
    emoji: Option<String>,
    bookmark_type: String,
    original_id: Option<Uuid>,
    parent_id: Option<Uuid>,
}

impl From<BookmarkRow> for ChannelBookmarkResponse {
    fn from(row: BookmarkRow) -> Self {
        Self {
            id: encode_mm_id(row.id),
            create_at: row.created_at.timestamp_millis(),
            update_at: row.updated_at.timestamp_millis(),
            delete_at: row.deleted_at.map(|t| t.timestamp_millis()).unwrap_or(0),
            channel_id: encode_mm_id(row.channel_id),
            owner_id: encode_mm_id(row.owner_id),
            file_id: row.file_id.map(encode_mm_id),
            display_name: row.display_name,
            sort_order: row.sort_order,
            link_url: row.link_url,
            image_url: row.image_url,
            emoji: row.emoji,
            bookmark_type: row.bookmark_type,
            original_id: row.original_id.map(encode_mm_id),
            parent_id: row.parent_id.map(encode_mm_id),
            file: None, // TODO: join with files table if needed
        }
    }
}

#[derive(Deserialize)]
pub struct BookmarksQuery {
    bookmarks_since: Option<i64>,
}

/// GET /api/v4/channels/{channel_id}/bookmarks
pub(super) async fn get_channel_bookmarks(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Query(query): Query<BookmarksQuery>,
) -> ApiResult<Json<Vec<ChannelBookmarkResponse>>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Verify channel membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this channel".to_string()));
    }

    let since = query.bookmarks_since.unwrap_or(0);
    
    let bookmarks: Vec<BookmarkRow> = sqlx::query_as(
        r#"
        SELECT id, created_at, updated_at, deleted_at, channel_id, owner_id, file_id,
               display_name, sort_order, link_url, image_url, emoji, bookmark_type,
               original_id, parent_id
        FROM channel_bookmarks
        WHERE channel_id = $1
          AND ($2 <= 0 OR updated_at >= to_timestamp($2::double precision / 1000.0))
          AND deleted_at IS NULL
        ORDER BY sort_order ASC, created_at ASC
        "#,
    )
    .bind(channel_id)
    .bind(since)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(bookmarks.into_iter().map(Into::into).collect()))
}

#[derive(Deserialize)]
pub struct CreateBookmarkRequest {
    display_name: String,
    #[serde(rename = "type")]
    bookmark_type: String,
    link_url: Option<String>,
    image_url: Option<String>,
    emoji: Option<String>,
    file_id: Option<String>,
}

/// POST /api/v4/channels/{channel_id}/bookmarks
pub(super) async fn create_channel_bookmark(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    _headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ChannelBookmarkResponse>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Verify channel membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this channel".to_string()));
    }

    let req: CreateBookmarkRequest = serde_json::from_slice(&body)
        .map_err(|_| AppError::BadRequest("Invalid bookmark body".to_string()))?;

    // Validate bookmark type
    if req.bookmark_type != "link" && req.bookmark_type != "file" {
        return Err(AppError::BadRequest("Type must be 'link' or 'file'".to_string()));
    }

    // Validate link URL for link type
    if req.bookmark_type == "link" && req.link_url.is_none() {
        return Err(AppError::BadRequest("Link URL required for link bookmarks".to_string()));
    }

    let file_id = req.file_id.as_ref().and_then(|id| parse_mm_or_uuid(id));
    
    // Get max sort order for this channel
    let max_order: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM channel_bookmarks WHERE channel_id = $1",
    )
    .bind(channel_id)
    .fetch_one(&state.db)
    .await?;
    
    let sort_order = max_order.unwrap_or(0) + 1;
    let now = Utc::now();

    let bookmark: BookmarkRow = sqlx::query_as(
        r#"
        INSERT INTO channel_bookmarks (
            channel_id, owner_id, file_id, display_name, sort_order,
            link_url, image_url, emoji, bookmark_type, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        RETURNING id, created_at, updated_at, deleted_at, channel_id, owner_id, file_id,
                  display_name, sort_order, link_url, image_url, emoji, bookmark_type,
                  original_id, parent_id
        "#,
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(file_id)
    .bind(&req.display_name)
    .bind(sort_order)
    .bind(&req.link_url)
    .bind(&req.image_url)
    .bind(&req.emoji)
    .bind(&req.bookmark_type)
    .bind(now)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(bookmark.into()))
}

#[derive(Deserialize)]
pub struct PatchBookmarkRequest {
    display_name: Option<String>,
    link_url: Option<String>,
    image_url: Option<String>,
    emoji: Option<String>,
    file_id: Option<String>,
    sort_order: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct UpdateBookmarkResponse {
    updated: ChannelBookmarkResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    deleted: Option<ChannelBookmarkResponse>,
}

/// PATCH /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}
pub(super) async fn patch_channel_bookmark(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, bookmark_id)): Path<(String, String)>,
    _headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<UpdateBookmarkResponse>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    let bookmark_id = parse_mm_or_uuid(&bookmark_id)
        .ok_or_else(|| AppError::BadRequest("Invalid bookmark_id".to_string()))?;

    // Verify channel membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this channel".to_string()));
    }

    let req: PatchBookmarkRequest = serde_json::from_slice(&body)
        .map_err(|_| AppError::BadRequest("Invalid patch body".to_string()))?;

    let file_id = req.file_id.as_ref().and_then(|id| parse_mm_or_uuid(id));

    let bookmark: BookmarkRow = sqlx::query_as(
        r#"
        UPDATE channel_bookmarks SET
            display_name = COALESCE($3, display_name),
            link_url = COALESCE($4, link_url),
            image_url = COALESCE($5, image_url),
            emoji = COALESCE($6, emoji),
            file_id = COALESCE($7, file_id),
            sort_order = COALESCE($8, sort_order),
            updated_at = NOW()
        WHERE id = $1 AND channel_id = $2 AND deleted_at IS NULL
        RETURNING id, created_at, updated_at, deleted_at, channel_id, owner_id, file_id,
                  display_name, sort_order, link_url, image_url, emoji, bookmark_type,
                  original_id, parent_id
        "#,
    )
    .bind(bookmark_id)
    .bind(channel_id)
    .bind(&req.display_name)
    .bind(&req.link_url)
    .bind(&req.image_url)
    .bind(&req.emoji)
    .bind(file_id)
    .bind(req.sort_order)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Bookmark not found".to_string()))?;

    Ok(Json(UpdateBookmarkResponse {
        updated: bookmark.into(),
        deleted: None,
    }))
}

/// POST /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}/sort_order
pub(super) async fn update_channel_bookmark_sort_order(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, bookmark_id)): Path<(String, String)>,
    _headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<ChannelBookmarkResponse>>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    let bookmark_id = parse_mm_or_uuid(&bookmark_id)
        .ok_or_else(|| AppError::BadRequest("Invalid bookmark_id".to_string()))?;

    // Verify channel membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this channel".to_string()));
    }

    let new_order: i64 = serde_json::from_slice(&body)
        .map_err(|_| AppError::BadRequest("Invalid sort order".to_string()))?;

    sqlx::query(
        "UPDATE channel_bookmarks SET sort_order = $3, updated_at = NOW() WHERE id = $1 AND channel_id = $2",
    )
    .bind(bookmark_id)
    .bind(channel_id)
    .bind(new_order)
    .execute(&state.db)
    .await?;

    // Return all bookmarks for this channel
    let bookmarks: Vec<BookmarkRow> = sqlx::query_as(
        r#"
        SELECT id, created_at, updated_at, deleted_at, channel_id, owner_id, file_id,
               display_name, sort_order, link_url, image_url, emoji, bookmark_type,
               original_id, parent_id
        FROM channel_bookmarks
        WHERE channel_id = $1 AND deleted_at IS NULL
        ORDER BY sort_order ASC, created_at ASC
        "#,
    )
    .bind(channel_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(bookmarks.into_iter().map(Into::into).collect()))
}

/// DELETE /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}
pub(super) async fn delete_channel_bookmark(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, bookmark_id)): Path<(String, String)>,
) -> ApiResult<Json<ChannelBookmarkResponse>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    let bookmark_id = parse_mm_or_uuid(&bookmark_id)
        .ok_or_else(|| AppError::BadRequest("Invalid bookmark_id".to_string()))?;

    // Verify channel membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this channel".to_string()));
    }

    // Soft delete
    let bookmark: BookmarkRow = sqlx::query_as(
        r#"
        UPDATE channel_bookmarks SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND channel_id = $2 AND deleted_at IS NULL
        RETURNING id, created_at, updated_at, deleted_at, channel_id, owner_id, file_id,
                  display_name, sort_order, link_url, image_url, emoji, bookmark_type,
                  original_id, parent_id
        "#,
    )
    .bind(bookmark_id)
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Bookmark not found".to_string()))?;

    Ok(Json(bookmark.into()))
}

/// POST /api/v4/channels/group/search
pub(super) async fn search_group_channels(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Json(_query): Json<serde_json::Value>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// PUT /api/v4/channels/{channel_id}/scheme
pub(super) async fn update_channel_scheme(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
    Json(_patch): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// GET /api/v4/channels/{channel_id}/members_minus_group_members
pub(super) async fn get_channel_members_minus_group_members(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/member_counts_by_group
pub(super) async fn get_channel_member_counts_by_group(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/moderations
pub(super) async fn get_channel_moderations(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// PUT /api/v4/channels/{channel_id}/moderations/patch
pub(super) async fn patch_channel_moderations(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
    Json(_patch): Json<serde_json::Value>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/common_teams
pub(super) async fn get_channel_common_teams(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/groups
pub(super) async fn get_channel_groups(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/access_control/attributes
pub(super) async fn get_channel_access_control_attributes(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({})))
}
