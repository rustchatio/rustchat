use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use super::{encode_mm_id, parse_mm_or_uuid, ApiResult, AppState, MmAuthUser};

/// GET /channels/{channel_id}/unread - Get unread counts for a channel
pub async fn get_channel_unread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let member: Option<crate::models::ChannelMember> =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?;

    let _member = member.ok_or_else(|| {
        crate::error::AppError::Forbidden("Not a member of this channel".to_string())
    })?;

    let team_id: uuid::Uuid = sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_one(&state.db)
        .await?;

    let username: Option<String> = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_optional(&state.db)
        .await?;
    let username = username.unwrap_or_default();

    let last_read_message_id: i64 = sqlx::query_scalar(
        "SELECT COALESCE(last_read_message_id, 0) FROM channel_reads WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(0);

    let (msg_count, mention_count, mention_count_root, mut urgent_mention_count, msg_count_root): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
            )::BIGINT AS msg_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND (p.message LIKE '%@' || $3 || '%' OR p.message LIKE '%@all%' OR p.message LIKE '%@channel%')
            )::BIGINT AS mention_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND p.root_post_id IS NULL
                  AND (p.message LIKE '%@' || $3 || '%' OR p.message LIKE '%@all%' OR p.message LIKE '%@channel%')
            )::BIGINT AS mention_count_root,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND (p.message LIKE '%@' || $3 || '%' OR p.message LIKE '%@all%' OR p.message LIKE '%@channel%')
                  AND p.message LIKE '%@here%'
            )::BIGINT AS urgent_mention_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND p.root_post_id IS NULL
            )::BIGINT AS msg_count_root
        FROM posts p
        WHERE p.channel_id = $1
        "#,
    )
    .bind(channel_id)
    .bind(last_read_message_id)
    .bind(&username)
    .fetch_one(&state.db)
    .await?;

    if !state.config.unread.post_priority_enabled {
        urgent_mention_count = 0;
    }

    Ok(Json(serde_json::json!({
        "team_id": encode_mm_id(team_id),
        "channel_id": encode_mm_id(channel_id),
        "msg_count": msg_count,
        "mention_count": mention_count,
        "mention_count_root": mention_count_root,
        "msg_count_root": msg_count_root,
        "urgent_mention_count": urgent_mention_count
    })))
}

/// POST /channels/{channel_id}/members/{user_id}/read - Mark channel as read
pub async fn mark_channel_as_read(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, user_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let target_user_id = if user_id == "me" {
        auth.user_id
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?
    };

    // Users can only mark their own channels as read
    if target_user_id != auth.user_id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot mark channel as read for other users".to_string(),
        ));
    }

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // Update last_viewed_at to mark all messages as read
    sqlx::query(
        "UPDATE channel_members SET last_viewed_at = NOW(), manually_unread = false, last_update_at = NOW() WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    // Also update channel_reads table
    sqlx::query(
        r#"
        INSERT INTO channel_reads (user_id, channel_id, last_read_message_id, last_read_at)
        VALUES ($1, $2, (SELECT MAX(seq) FROM posts WHERE channel_id = $2), NOW())
        ON CONFLICT (user_id, channel_id)
        DO UPDATE SET last_read_message_id = EXCLUDED.last_read_message_id, last_read_at = NOW()
        "#,
    )
    .bind(auth.user_id)
    .bind(channel_id)
    .execute(&state.db)
    .await?;

    // Broadcast channel viewed event
    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::ChannelViewed,
        serde_json::json!({
            "channel_id": encode_mm_id(channel_id),
        }),
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: None,
        team_id: None,
        user_id: Some(auth.user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(json!({"status": "OK"})))
}

/// POST /channels/{channel_id}/members/{user_id}/set_unread - Mark channel as unread
/// This sets the last_viewed_at to a past time to create unread state
pub async fn mark_channel_as_unread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, user_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let target_user_id = if user_id == "me" {
        auth.user_id
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?
    };

    // Users can only mark their own channels as unread
    if target_user_id != auth.user_id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot mark channel as unread for other users".to_string(),
        ));
    }

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // Get the oldest post in the channel to set as unread point
    // Or use a time far in the past to ensure all messages are marked as unread
    let oldest_post_time: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT MIN(created_at) FROM posts WHERE channel_id = $1 AND deleted_at IS NULL",
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?;

    // Set last_viewed_at to the oldest post time, or epoch if no posts
    let mark_time = oldest_post_time.unwrap_or(chrono::DateTime::UNIX_EPOCH);

    sqlx::query(
        "UPDATE channel_members SET last_viewed_at = $3, manually_unread = true, last_update_at = NOW() WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(mark_time)
    .execute(&state.db)
    .await?;

    // Also update channel_reads table
    sqlx::query(
        "UPDATE channel_reads SET last_read_message_id = 0, last_read_at = $3 WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(mark_time)
    .execute(&state.db)
    .await?;

    // Broadcast unread update
    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::ChannelUnread,
        serde_json::json!({
            "channel_id": encode_mm_id(channel_id),
            "user_id": encode_mm_id(auth.user_id),
            "unread_count": 1,
        }),
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: None,
        team_id: None,
        user_id: Some(auth.user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(json!({
        "channel_id": encode_mm_id(channel_id),
        "user_id": encode_mm_id(auth.user_id),
        "status": "OK"
    })))
}
