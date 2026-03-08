use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    encode_mm_id, json, mm, parse_mm_or_uuid, reactions_for_posts, status_ok, ApiResult, AppError,
    AppState, MmAuthUser,
};

#[derive(Deserialize)]
pub(super) struct PostsUnreadQuery {
    #[serde(default = "default_limit")]
    limit_before: i32,
    #[serde(default = "default_limit")]
    limit_after: i32,
    #[serde(rename = "skipFetchThreads", default)]
    _skip_fetch_threads: bool,
    #[serde(rename = "collapsedThreads", default)]
    _collapsed_threads: bool,
    #[serde(rename = "collapsedThreadsExtended", default)]
    _collapsed_threads_extended: bool,
}

fn default_limit() -> i32 {
    60
}

fn clamp_unread_limits(query: &PostsUnreadQuery) -> (i64, i64) {
    (
        query.limit_before.clamp(0, 200) as i64,
        query.limit_after.clamp(1, 200) as i64,
    )
}

#[derive(Deserialize)]
pub(super) struct PostsUnreadPath {
    user_id: String,
    channel_id: String,
}

/// GET /api/v4/users/{user_id}/channels/{channel_id}/posts/unread
pub(super) async fn get_posts_around_unread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<PostsUnreadPath>,
    Query(query): Query<PostsUnreadQuery>,
) -> ApiResult<Json<mm::PostList>> {
    let user_id = crate::api::v4::users::resolve_user_id(&path.user_id, &auth)
        .map_err(|_| AppError::Forbidden("Cannot access another user's posts".to_string()))?;

    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    let _: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    let (limit_before, limit_after) = clamp_unread_limits(&query);

    let last_read_seq: Option<i64> = sqlx::query_scalar(
        "SELECT last_read_message_id FROM channel_reads WHERE user_id = $1 AND channel_id = $2",
    )
    .bind(user_id)
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    let last_read_seq = last_read_seq.unwrap_or(0);

    let mut posts: Vec<crate::models::post::PostResponse> = sqlx::query_as(
        r#"
        (
            SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
                   p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
                   p.reply_count::int8 as reply_count,
                   p.last_reply_at, p.seq,
                   u.username, u.avatar_url, u.email
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1 AND p.seq <= $2 AND p.deleted_at IS NULL
            ORDER BY p.seq DESC
            LIMIT $3
        )
        UNION ALL
        (
            SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
                   p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
                   p.reply_count::int8 as reply_count,
                   p.last_reply_at, p.seq,
                   u.username, u.avatar_url, u.email
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1 AND p.seq > $2 AND p.deleted_at IS NULL
            ORDER BY p.seq ASC
            LIMIT $4
        )
        ORDER BY seq DESC
        "#,
    )
    .bind(channel_id)
    .bind(last_read_seq)
    .bind(limit_before)
    .bind(limit_after)
    .fetch_all(&state.db)
    .await?;

    crate::services::posts::populate_files(&state, &mut posts).await?;

    let mut order = Vec::new();
    let mut posts_map: std::collections::HashMap<String, mm::Post> =
        std::collections::HashMap::new();
    let mut post_ids = Vec::new();
    let mut id_map = Vec::new();

    for p in posts {
        let id = encode_mm_id(p.id);
        post_ids.push(p.id);
        id_map.push((p.id, id.clone()));
        order.push(id.clone());
        posts_map.insert(id, p.into());
    }

    let reactions_map = reactions_for_posts(&state, &post_ids).await?;
    for (post_uuid, post_id) in id_map {
        if let Some(reactions) = reactions_map.get(&post_uuid) {
            if !reactions.is_empty() {
                if let Some(post) = posts_map.get_mut(&post_id) {
                    let mut metadata = post.metadata.take().unwrap_or_else(|| json!({}));
                    if let Some(obj) = metadata.as_object_mut() {
                        obj.insert("reactions".to_string(), json!(reactions));
                    }
                    post.metadata = Some(metadata);
                }
            }
        }
    }

    Ok(Json(mm::PostList {
        order,
        posts: posts_map,
        next_post_id: String::new(),
        prev_post_id: String::new(),
    }))
}

#[derive(Deserialize)]
pub(super) struct AckPath {
    user_id: String,
    post_id: String,
}

/// POST /api/v4/users/{user_id}/posts/{post_id}/ack - Acknowledge a post
pub(super) async fn save_acknowledgement_for_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<AckPath>,
) -> ApiResult<Json<mm::PostAcknowledgement>> {
    let user_id = crate::api::v4::users::resolve_user_id(&path.user_id, &auth)
        .map_err(|_| AppError::Forbidden("Cannot acknowledge for another user".to_string()))?;

    let post_id = parse_mm_or_uuid(&path.post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;

    let channel_id: Uuid =
        sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1 AND deleted_at IS NULL")
            .bind(post_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    let _: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO post_acknowledgements (user_id, post_id, acknowledged_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, post_id) DO UPDATE SET acknowledged_at = $3
        "#,
    )
    .bind(user_id)
    .bind(post_id)
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(Json(mm::PostAcknowledgement {
        user_id: encode_mm_id(user_id),
        post_id: encode_mm_id(post_id),
        acknowledged_at: now.timestamp_millis(),
    }))
}

/// DELETE /api/v4/users/{user_id}/posts/{post_id}/ack - Delete a post acknowledgement
pub(super) async fn delete_acknowledgement_for_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<AckPath>,
) -> ApiResult<impl IntoResponse> {
    let user_id = crate::api::v4::users::resolve_user_id(&path.user_id, &auth).map_err(|_| {
        AppError::Forbidden("Cannot delete acknowledgement for another user".to_string())
    })?;

    let post_id = parse_mm_or_uuid(&path.post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;

    let ack_time: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT acknowledged_at FROM post_acknowledgements WHERE user_id = $1 AND post_id = $2",
    )
    .bind(user_id)
    .bind(post_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    if let Some(ack_time) = ack_time {
        let now = chrono::Utc::now();
        let five_minutes = chrono::Duration::minutes(5);
        if now - ack_time > five_minutes {
            return Err(AppError::Forbidden(
                "Cannot delete acknowledgement after 5 minutes".to_string(),
            ));
        }
    } else {
        return Err(AppError::NotFound("Acknowledgement not found".to_string()));
    }

    sqlx::query("DELETE FROM post_acknowledgements WHERE user_id = $1 AND post_id = $2")
        .bind(user_id)
        .bind(post_id)
        .execute(&state.db)
        .await?;

    Ok(status_ok())
}

#[cfg(test)]
mod tests {
    use super::{clamp_unread_limits, PostsUnreadQuery};

    #[test]
    fn clamps_unread_limits() {
        let query = PostsUnreadQuery {
            limit_before: 500,
            limit_after: -10,
            _skip_fetch_threads: false,
            _collapsed_threads: false,
            _collapsed_threads_extended: false,
        };

        assert_eq!(clamp_unread_limits(&query), (200, 1));
    }
}
