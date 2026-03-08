use axum::{
    body::Bytes,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::extractors::MmAuthUser;
use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::{CreatePost, FileInfo};
use crate::realtime::{EventType, WsBroadcast, WsEnvelope};
use crate::services::posts;

mod reactions;
mod search;
mod unread;

pub(crate) use reactions::reactions_for_posts;
use reactions::{add_reaction, get_reactions, remove_reaction, remove_reaction_for_user};
use search::{search_posts_all_teams, search_team_posts};
use unread::{
    delete_acknowledgement_for_post, get_posts_around_unread, save_acknowledgement_for_post,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/posts", post(create_post_handler))
        .route("/posts/ids", post(get_posts_by_ids))
        .route("/posts/ids/reactions", post(get_reactions_by_post_ids))
        .route(
            "/posts/{post_id}",
            get(get_post).put(update_post).delete(delete_post),
        )
        .route("/posts/{post_id}/files/info", get(get_post_files_info))
        .route("/posts/{post_id}/pin", post(pin_post))
        .route("/posts/{post_id}/unpin", post(unpin_post))
        .route("/posts/{post_id}/patch", put(patch_post))
        .route(
            "/posts/{post_id}/actions/{action_id}",
            post(handle_post_action),
        )
        .route("/posts/{post_id}/move", post(move_post))
        .route(
            "/posts/{post_id}/restore/{restore_version_id}",
            post(restore_post),
        )
        .route(
            "/posts/{post_id}/reveal",
            get(reveal_post).post(reveal_post),
        )
        .route("/posts/{post_id}/burn", delete(burn_post).post(burn_post))
        .route("/posts/rewrite", post(rewrite_post))
        .route(
            "/users/{user_id}/posts/{post_id}/set_unread",
            post(set_post_unread),
        )
        .route("/users/{user_id}/posts/flagged", get(get_flagged_posts))
        .route("/posts/{post_id}/ack", post(ack_post))
        .route("/reactions", post(add_reaction))
        .route(
            "/users/me/posts/{post_id}/reactions/{emoji_name}",
            delete(remove_reaction),
        )
        .route(
            "/users/{user_id}/posts/{post_id}/reactions/{emoji_name}",
            delete(remove_reaction_for_user),
        )
        .route("/posts/{post_id}/reactions", get(get_reactions))
        .route("/posts/{post_id}/thread", get(get_post_thread))
        .route("/posts/ephemeral", post(create_ephemeral_post))
        .route("/posts/schedule", post(create_scheduled_post))
        .route(
            "/posts/schedule/{scheduled_post_id}",
            put(update_scheduled_post).delete(delete_scheduled_post),
        )
        .route("/posts/scheduled/team/{team_id}", get(list_scheduled_posts))
        .route(
            "/users/{user_id}/posts/{post_id}/reminder",
            post(set_post_reminder),
        )
        .route("/posts/search", post(search_posts_all_teams))
        .route("/teams/{team_id}/posts/search", post(search_team_posts))
        .route(
            "/users/{user_id}/channels/{channel_id}/posts/unread",
            get(get_posts_around_unread),
        )
        .route(
            "/users/{user_id}/posts/{post_id}/ack",
            post(save_acknowledgement_for_post).delete(delete_acknowledgement_for_post),
        )
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub channel_id: String,
    pub message: String,
    #[serde(default)]
    pub root_id: String,
    #[serde(default)]
    pub file_ids: Vec<String>,
    #[serde(default)]
    pub props: serde_json::Value,
    #[serde(default)]
    pub pending_post_id: String,
}

async fn create_post_handler(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Post>> {
    let input: CreatePostRequest = parse_body(&headers, &body, "Invalid post body")?;
    let channel_id = parse_mm_or_uuid(&input.channel_id)
        .ok_or_else(|| AppError::Validation("Invalid channel_id".to_string()))?;

    let root_post_id = if !input.root_id.is_empty() {
        Some(
            parse_mm_or_uuid(&input.root_id)
                .ok_or_else(|| AppError::Validation("Invalid root_id".to_string()))?,
        )
    } else {
        None
    };

    let file_ids = input
        .file_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    let create_payload = CreatePost {
        message: input.message,
        root_post_id,
        props: Some(input.props),
        file_ids,
        client_msg_id: None,
    };

    let client_msg_id = if !input.pending_post_id.is_empty() {
        Some(input.pending_post_id)
    } else {
        None
    };

    let post_resp = posts::create_post(
        &state,
        auth.user_id,
        channel_id,
        create_payload,
        client_msg_id,
    )
    .await?;

    Ok(Json(post_resp.into()))
}

fn parse_body<T: serde::de::DeserializeOwned>(
    headers: &axum::http::HeaderMap,
    body: &Bytes,
    message: &str,
) -> ApiResult<T> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.starts_with("application/json") {
        serde_json::from_slice(body).map_err(|_| AppError::BadRequest(message.to_string()))
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        serde_urlencoded::from_bytes(body).map_err(|_| AppError::BadRequest(message.to_string()))
    } else {
        serde_json::from_slice(body)
            .or_else(|_| serde_urlencoded::from_bytes(body))
            .map_err(|_| AppError::BadRequest(message.to_string()))
    }
}

fn status_ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "OK"}))
}

async fn get_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<mm::Post>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let post: crate::models::post::PostResponse = sqlx::query_as(
        r#"
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM posts p
        LEFT JOIN users u ON p.user_id = u.id
        WHERE p.id = $1 AND p.deleted_at IS NULL
        "#,
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;

    let _: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(post.channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let mut mm_post: mm::Post = post.into();
    let reactions_map = reactions_for_posts(&state, &[post_id]).await?;
    if let Some(reactions) = reactions_map.get(&post_id) {
        if !reactions.is_empty() {
            let mut metadata = mm_post.metadata.clone().unwrap_or_else(|| json!({}));
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("reactions".to_string(), json!(reactions));
            }
            mm_post.metadata = Some(metadata);
        }
    }

    Ok(Json(mm_post))
}

async fn get_posts_by_ids(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::Post>>> {
    let input: Vec<String> = parse_body(&headers, &body, "Invalid post ids")?;
    if input.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let mut post_ids = Vec::new();
    for id in &input {
        let parsed = parse_mm_or_uuid(id)
            .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
        post_ids.push(parsed);
    }

    let posts: Vec<crate::models::post::PostResponse> = sqlx::query_as(
        r#"
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM posts p
        LEFT JOIN users u ON p.user_id = u.id
        JOIN channel_members cm ON cm.channel_id = p.channel_id AND cm.user_id = $2
        WHERE p.id = ANY($1) AND p.deleted_at IS NULL
        "#,
    )
    .bind(&post_ids)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let mut map = std::collections::HashMap::new();
    for post in posts {
        map.insert(post.id, mm::Post::from(post));
    }

    let mut ordered = Vec::new();
    for id in post_ids {
        if let Some(post) = map.remove(&id) {
            ordered.push(post);
        }
    }

    Ok(Json(ordered))
}

async fn get_reactions_by_post_ids(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<std::collections::HashMap<String, Vec<mm::Reaction>>>> {
    let input: Vec<String> = parse_body(&headers, &body, "Invalid post ids")?;
    if input.is_empty() {
        return Ok(Json(std::collections::HashMap::new()));
    }

    let mut post_ids = Vec::new();
    for id in &input {
        let parsed = parse_mm_or_uuid(id)
            .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
        post_ids.push(parsed);
    }

    let visible_ids: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT p.id
        FROM posts p
        JOIN channel_members cm ON cm.channel_id = p.channel_id AND cm.user_id = $2
        WHERE p.id = ANY($1) AND p.deleted_at IS NULL
        "#,
    )
    .bind(&post_ids)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let reactions_map = reactions_for_posts(&state, &visible_ids).await?;
    let mut output = std::collections::HashMap::new();
    for (post_id, reactions) in reactions_map {
        output.insert(encode_mm_id(post_id), reactions);
    }

    Ok(Json(output))
}

async fn get_post_files_info(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<Vec<mm::FileInfo>>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;

    let post: crate::models::post::Post = sqlx::query_as(
        r#"
        SELECT id, channel_id, user_id, root_post_id, message, props, file_ids,
               is_pinned, created_at, edited_at, deleted_at,
               reply_count::int8 as reply_count,
               last_reply_at, seq
        FROM posts WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;

    let _: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(post.channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    if post.file_ids.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let files: Vec<FileInfo> = sqlx::query_as("SELECT * FROM files WHERE id = ANY($1)")
        .bind(&post.file_ids)
        .fetch_all(&state.db)
        .await?;

    let mm_files: Vec<mm::FileInfo> = files.into_iter().map(|f| f.into()).collect();
    Ok(Json(mm_files))
}

async fn pin_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;

    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;

    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    sqlx::query("UPDATE posts SET is_pinned = true WHERE id = $1")
        .bind(post_id)
        .execute(&state.db)
        .await?;

    Ok(status_ok())
}

async fn unpin_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;

    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;

    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    sqlx::query("UPDATE posts SET is_pinned = false WHERE id = $1")
        .bind(post_id)
        .execute(&state.db)
        .await?;

    Ok(status_ok())
}

async fn get_post_thread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<mm::PostList>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    use std::collections::HashMap;

    // 1. Get the requested post
    let root_post: crate::models::post::PostResponse = sqlx::query_as(
        r#"
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM posts p
        LEFT JOIN users u ON p.user_id = u.id
        WHERE p.id = $1 AND p.deleted_at IS NULL
        "#,
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;

    // 2. Check permissions
    let _: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(root_post.channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // 3. Get replies
    let replies: Vec<crate::models::post::PostResponse> = sqlx::query_as(
        r#"
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM posts p
        LEFT JOIN users u ON p.user_id = u.id
        WHERE p.root_post_id = $1 AND p.deleted_at IS NULL
        ORDER BY p.created_at ASC
        "#,
    )
    .bind(post_id)
    .fetch_all(&state.db)
    .await?;

    // 4. Construct response
    let mut order = Vec::new();
    let mut posts_map: HashMap<String, mm::Post> = HashMap::new();
    let mut post_ids = Vec::new();
    let mut id_map = Vec::new();

    // Add root post
    let root_id = encode_mm_id(root_post.id);
    order.push(root_id.clone());
    let root_uuid = root_post.id;
    post_ids.push(root_uuid);
    id_map.push((root_uuid, root_id.clone()));
    posts_map.insert(root_id, root_post.into());

    // Add replies
    for r in replies {
        let id = encode_mm_id(r.id);
        post_ids.push(r.id);
        id_map.push((r.id, id.clone()));
        order.push(id.clone());
        posts_map.insert(id, r.into());
    }

    let reactions_map = reactions_for_posts(&state, &post_ids).await?;
    for (post_uuid, post_id) in id_map {
        if let Some(reactions) = reactions_map.get(&post_uuid) {
            if !reactions.is_empty() {
                if let Some(post) = posts_map.get_mut(&post_id) {
                    post.metadata = Some(json!({ "reactions": reactions }));
                }
            }
        }
    }

    Ok(Json(mm::PostList {
        order,
        posts: posts_map,
        next_post_id: "".to_string(),
        prev_post_id: "".to_string(),
    }))
}

async fn handle_post_action(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((post_id, _action_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid action body")?;

    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    Ok(status_ok())
}

#[derive(Deserialize)]
struct MovePostRequest {
    #[serde(rename = "channel_id")]
    _channel_id: String,
}

async fn move_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let _input: MovePostRequest = parse_body(&headers, &body, "Invalid move body")?;

    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    Ok(status_ok())
}

async fn restore_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((post_id, _restore_version_id)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;
    Ok(status_ok())
}

async fn reveal_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;
    Ok(status_ok())
}

async fn burn_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let channel_id: Uuid = sqlx::query_scalar("SELECT channel_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&state.db)
        .await?;
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;
    Ok(status_ok())
}

#[derive(Deserialize)]
struct RewriteRequest {
    message: String,
}

async fn rewrite_post(
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let input: RewriteRequest = parse_body(&headers, &body, "Invalid rewrite body")?;
    Ok(Json(serde_json::json!({"rewritten_text": input.message})))
}

#[derive(Deserialize)]
struct SetUnreadPath {
    user_id: String,
    post_id: String,
}

#[derive(Deserialize, Default)]
struct SetPostUnreadRequest {
    #[serde(default)]
    collapsed_threads_supported: bool,
}

#[derive(sqlx::FromRow)]
struct ChannelUnreadComputation {
    total_msg_count: i64,
    total_msg_count_root: i64,
    unread_msg_count: i64,
    unread_msg_count_root: i64,
    mention_count: i64,
    mention_count_root: i64,
    urgent_mention_count: i64,
}

#[derive(sqlx::FromRow)]
struct ThreadSnapshotRow {
    id: Uuid,
    channel_id: Uuid,
    user_id: Uuid,
    message: String,
    created_at: chrono::DateTime<chrono::Utc>,
    reply_count: i64,
    last_reply_at: Option<chrono::DateTime<chrono::Utc>>,
    following: bool,
    last_read_at: Option<chrono::DateTime<chrono::Utc>>,
    mention_count: i32,
    unread_replies_count: i32,
}

async fn is_crt_enabled_for_user(state: &AppState, user_id: Uuid) -> ApiResult<bool> {
    if !state.config.unread.collapsed_threads_enabled {
        return Ok(false);
    }

    let pref_value: Option<String> = sqlx::query_scalar(
        r#"
        SELECT value
        FROM mattermost_preferences
        WHERE user_id = $1
          AND category = 'display_settings'
          AND name = 'collapsed_reply_threads'
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let enabled = pref_value
        .as_deref()
        .map(|v| {
            let normalized = v.trim().to_ascii_lowercase();
            normalized == "on" || normalized == "true" || normalized == "1"
        })
        .unwrap_or(true);

    Ok(enabled)
}

async fn compute_channel_unread_from_post(
    state: &AppState,
    channel_id: Uuid,
    last_read_id: i64,
    username: &str,
) -> ApiResult<ChannelUnreadComputation> {
    let mut stats: ChannelUnreadComputation = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE p.deleted_at IS NULL)::BIGINT AS total_msg_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.root_post_id IS NULL
            )::BIGINT AS total_msg_count_root,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
            )::BIGINT AS unread_msg_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND p.root_post_id IS NULL
            )::BIGINT AS unread_msg_count_root,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND (
                      p.message LIKE '%@' || $3 || '%'
                      OR p.message LIKE '%@all%'
                      OR p.message LIKE '%@channel%'
                  )
            )::BIGINT AS mention_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND p.root_post_id IS NULL
                  AND (
                      p.message LIKE '%@' || $3 || '%'
                      OR p.message LIKE '%@all%'
                      OR p.message LIKE '%@channel%'
                  )
            )::BIGINT AS mention_count_root,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND (
                      p.message LIKE '%@' || $3 || '%'
                      OR p.message LIKE '%@all%'
                      OR p.message LIKE '%@channel%'
                  )
                  AND p.message LIKE '%@here%'
            )::BIGINT AS urgent_mention_count
        FROM posts p
        WHERE p.channel_id = $1
        "#,
    )
    .bind(channel_id)
    .bind(last_read_id)
    .bind(username)
    .fetch_one(&state.db)
    .await?;

    if !state.config.unread.post_priority_enabled {
        stats.urgent_mention_count = 0;
    }

    Ok(stats)
}

async fn fetch_thread_snapshot_for_user(
    state: &AppState,
    thread_root_id: Uuid,
    user_id: Uuid,
) -> ApiResult<Option<mm::Thread>> {
    let row: Option<ThreadSnapshotRow> = sqlx::query_as(
        r#"
        SELECT
            p.id,
            p.channel_id,
            p.user_id,
            p.message,
            p.created_at,
            p.reply_count::int8 AS reply_count,
            p.last_reply_at,
            COALESCE(tm.following, false) AS following,
            tm.last_read_at,
            COALESCE(tm.mention_count, 0)::int4 AS mention_count,
            COALESCE(tm.unread_replies_count, 0)::int4 AS unread_replies_count
        FROM posts p
        LEFT JOIN thread_memberships tm
               ON tm.post_id = p.id
              AND tm.user_id = $2
        WHERE p.id = $1
          AND p.root_post_id IS NULL
          AND p.deleted_at IS NULL
        "#,
    )
    .bind(thread_root_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    Ok(row.map(|t| mm::Thread {
        id: encode_mm_id(t.id),
        reply_count: t.reply_count,
        last_reply_at: t.last_reply_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        last_viewed_at: t.last_read_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        participants: vec![],
        post: mm::PostInThread {
            id: encode_mm_id(t.id),
            channel_id: encode_mm_id(t.channel_id),
            user_id: encode_mm_id(t.user_id),
            message: t.message,
            create_at: t.created_at.timestamp_millis(),
        },
        unread_replies: i64::from(t.unread_replies_count),
        unread_mentions: i64::from(t.mention_count),
        is_following: Some(t.following),
    }))
}

async fn set_post_unread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<SetUnreadPath>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::ChannelUnreadAt>> {
    let user_id = super::users::resolve_user_id(&path.user_id, &auth)
        .map_err(|_| AppError::Forbidden("Cannot access another user's posts".to_string()))?;
    let post_id = parse_mm_or_uuid(&path.post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let request: SetPostUnreadRequest = if body.is_empty() {
        SetPostUnreadRequest::default()
    } else {
        parse_body(&headers, &body, "Invalid set unread body")?
    };

    let (channel_id, team_id, seq, root_post_id, post_created_at): (
        Uuid,
        Uuid,
        i64,
        Option<Uuid>,
        chrono::DateTime<chrono::Utc>,
    ) = sqlx::query_as(
        r#"
        SELECT p.channel_id, c.team_id, p.seq, p.root_post_id, p.created_at
        FROM posts p
        JOIN channels c ON p.channel_id = c.id
        WHERE p.id = $1 AND p.deleted_at IS NULL
        "#,
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;

    let _: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a member of this channel".to_string()))?;

    let last_read_id = if seq > 0 { seq - 1 } else { 0 };
    let mark_view_at = post_created_at - chrono::Duration::milliseconds(1);
    let crt_enabled_for_user = is_crt_enabled_for_user(&state, user_id).await?;
    let crt_supported_request = request.collapsed_threads_supported && crt_enabled_for_user;
    let is_reply = root_post_id.is_some();

    sqlx::query(
        r#"
        INSERT INTO channel_reads (user_id, channel_id, last_read_message_id, last_read_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id, channel_id)
        DO UPDATE SET last_read_message_id = $3, last_read_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(channel_id)
    .bind(last_read_id)
    .execute(&state.db)
    .await?;

    let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    let mut stats =
        compute_channel_unread_from_post(&state, channel_id, last_read_id, &username).await?;

    // CRT unsupported + reply follows Mattermost behavior:
    // unread root/urgent counters for the channel are intentionally zeroed.
    let set_unread_count_root = !is_reply || crt_supported_request;
    if !set_unread_count_root {
        stats.unread_msg_count_root = 0;
        stats.mention_count_root = 0;
        stats.urgent_mention_count = 0;
    }

    if is_reply && !crt_supported_request && state.config.unread.thread_auto_follow {
        let thread_root_id = root_post_id.unwrap_or(post_id);
        let (thread_unread_replies, thread_unread_mentions): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.created_at > $3
                )::BIGINT AS unread_replies_count,
                COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.created_at > $3
                      AND (
                          p.message LIKE '%@' || $2 || '%'
                          OR p.message LIKE '%@all%'
                          OR p.message LIKE '%@channel%'
                      )
                )::BIGINT AS mention_count
            FROM posts p
            WHERE p.root_post_id = $1
            "#,
        )
        .bind(thread_root_id)
        .bind(&username)
        .bind(mark_view_at)
        .fetch_one(&state.db)
        .await?;

        let unread_replies_count = i32::try_from(thread_unread_replies).unwrap_or(i32::MAX);
        let mention_count = i32::try_from(thread_unread_mentions).unwrap_or(i32::MAX);

        sqlx::query(
            r#"
            INSERT INTO thread_memberships
                (user_id, post_id, following, last_read_at, mention_count, unread_replies_count, updated_at)
            VALUES
                ($1, $2, true, $3, $4, $5, NOW())
            ON CONFLICT (user_id, post_id)
            DO UPDATE SET
                following = true,
                last_read_at = $3,
                mention_count = $4,
                unread_replies_count = $5,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(thread_root_id)
        .bind(mark_view_at)
        .bind(mention_count)
        .bind(unread_replies_count)
        .execute(&state.db)
        .await?;

        // Match Mattermost: only send thread_updated when user CRT is enabled but request
        // came from a CRT-unsupported client.
        if crt_enabled_for_user && !request.collapsed_threads_supported {
            if let Some(thread) =
                fetch_thread_snapshot_for_user(&state, thread_root_id, user_id).await?
            {
                if let Ok(payload) = serde_json::to_string(&thread) {
                    let thread_updated = WsEnvelope::event(
                        EventType::ThreadUpdated,
                        serde_json::json!({ "thread": payload }),
                        None,
                    )
                    .with_broadcast(WsBroadcast {
                        channel_id: None,
                        team_id: Some(team_id),
                        user_id: Some(user_id),
                        exclude_user_id: None,
                    });
                    state.ws_hub.broadcast(thread_updated).await;
                }
            }
        }
    }

    let msg_count = (stats.total_msg_count - stats.unread_msg_count).max(0);
    let msg_count_root = (stats.total_msg_count_root - stats.unread_msg_count_root).max(0);
    let mention_count = stats.mention_count.max(0);
    let mention_count_root = stats.mention_count_root.max(0);
    let urgent_mention_count = stats.urgent_mention_count.max(0);

    sqlx::query(
        r#"
        UPDATE channel_members
        SET last_viewed_at = $3,
            manually_unread = true,
            msg_count = $4,
            mention_count = $5,
            msg_count_root = $6,
            mention_count_root = $7,
            urgent_mention_count = $8,
            last_update_at = NOW()
        WHERE channel_id = $1 AND user_id = $2
        "#,
    )
    .bind(channel_id)
    .bind(user_id)
    .bind(mark_view_at)
    .bind(msg_count)
    .bind(mention_count)
    .bind(msg_count_root)
    .bind(mention_count_root)
    .bind(urgent_mention_count)
    .execute(&state.db)
    .await?;

    let payload = mm::ChannelUnreadAt {
        team_id: encode_mm_id(team_id),
        user_id: encode_mm_id(user_id),
        channel_id: encode_mm_id(channel_id),
        msg_count,
        mention_count,
        mention_count_root,
        urgent_mention_count,
        msg_count_root,
        last_viewed_at: mark_view_at.timestamp_millis(),
    };

    if state.config.unread.post_unread_ws_enabled {
        let broadcast = WsEnvelope::event(
            EventType::PostUnread,
            serde_json::json!({
                "team_id": payload.team_id,
                "user_id": payload.user_id,
                "channel_id": payload.channel_id,
                "msg_count": payload.msg_count,
                "msg_count_root": payload.msg_count_root,
                "mention_count": payload.mention_count,
                "mention_count_root": payload.mention_count_root,
                "urgent_mention_count": payload.urgent_mention_count,
                "last_viewed_at": payload.last_viewed_at,
                "post_id": encode_mm_id(post_id),
            }),
            Some(channel_id),
        )
        .with_broadcast(WsBroadcast {
            channel_id: Some(channel_id),
            team_id: Some(team_id),
            user_id: Some(user_id),
            exclude_user_id: None,
        });
        state.ws_hub.broadcast(broadcast).await;
    }

    Ok(Json(payload))
}

async fn get_flagged_posts(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<mm::PostList>> {
    let user_id = if user_id == "me" {
        auth.user_id
    } else {
        let parsed = parse_mm_or_uuid(&user_id)
            .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
        if !auth.can_access_owned(parsed, &permissions::USER_MANAGE) {
            return Err(AppError::Forbidden(
                "Cannot access another user's posts".to_string(),
            ));
        }
        parsed
    };

    let posts: Vec<crate::models::post::PostResponse> = sqlx::query_as(
        r#"
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM saved_posts s
        JOIN posts p ON s.post_id = p.id
        LEFT JOIN users u ON p.user_id = u.id
        WHERE s.user_id = $1 AND p.deleted_at IS NULL
        ORDER BY s.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

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
                    post.metadata = Some(json!({ "reactions": reactions }));
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

async fn delete_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let (post_user_id, post_channel_id): (Uuid, Uuid) =
        sqlx::query_as("SELECT user_id, channel_id FROM posts WHERE id = $1")
            .bind(post_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if post_user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot delete others' posts".to_string(),
        ));
    }

    let deleted_post: crate::models::post::PostResponse = sqlx::query_as(
        r#"
        WITH updated_post AS (
            UPDATE posts SET deleted_at = NOW() WHERE id = $1
            RETURNING *
        )
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM updated_post p
        LEFT JOIN users u ON p.user_id = u.id
        "#,
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;

    let broadcast = WsEnvelope::event(
        EventType::MessageDeleted,
        deleted_post,
        Some(post_channel_id),
    )
    .with_broadcast(WsBroadcast {
        channel_id: Some(post_channel_id),
        team_id: None,
        user_id: None,
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(
        serde_json::json!({"status": "OK", "id": encode_mm_id(post_id)}),
    ))
}

#[derive(Deserialize)]
struct PatchPostRequest {
    message: String,
}

#[derive(Deserialize)]
struct UpdatePostRequest {
    id: String,
    #[serde(default)]
    message: String,
}

async fn update_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
    Json(input): Json<UpdatePostRequest>,
) -> ApiResult<Json<mm::Post>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    let body_post_id = parse_mm_or_uuid(&input.id)
        .ok_or_else(|| AppError::BadRequest("Invalid id".to_string()))?;

    if post_id != body_post_id {
        return Err(AppError::BadRequest("Invalid id".to_string()));
    }

    update_post_message(&state, auth.user_id, post_id, input.message).await
}

async fn patch_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(post_id): Path<String>,
    Json(input): Json<PatchPostRequest>,
) -> ApiResult<Json<mm::Post>> {
    let post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;
    update_post_message(&state, auth.user_id, post_id, input.message).await
}

async fn update_post_message(
    state: &AppState,
    acting_user_id: Uuid,
    post_id: Uuid,
    message: String,
) -> ApiResult<Json<mm::Post>> {
    let (post_user_id, post_channel_id, post_created_at, original_message): (
        Uuid,
        Uuid,
        chrono::DateTime<Utc>,
        String,
    ) = sqlx::query_as("SELECT user_id, channel_id, created_at, message FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if post_user_id != acting_user_id {
        return Err(AppError::Forbidden("Cannot edit others' posts".to_string()));
    }

    if message != original_message {
        let post_edit_time_limit_seconds = load_post_edit_time_limit_seconds(state).await?;
        if post_edit_time_limit_seconds == 0 {
            return Err(AppError::BadRequest(
                "Message editing is disabled by server policy".to_string(),
            ));
        }
        if post_edit_time_limit_seconds > 0 {
            let post_age_seconds = Utc::now()
                .signed_duration_since(post_created_at)
                .num_seconds();
            if post_age_seconds >= post_edit_time_limit_seconds {
                return Err(AppError::BadRequest(format!(
                    "Message edit window expired after {} seconds",
                    post_edit_time_limit_seconds
                )));
            }
        }
    }

    let updated: crate::models::post::PostResponse = sqlx::query_as(
        r#"
        WITH updated_post AS (
            UPDATE posts SET message = $1, edited_at = NOW()
            WHERE id = $2
            RETURNING *
        )
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM updated_post p
        LEFT JOIN users u ON p.user_id = u.id
        "#,
    )
    .bind(message)
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;

    let broadcast = WsEnvelope::event(
        EventType::MessageUpdated,
        updated.clone(),
        Some(post_channel_id),
    )
    .with_broadcast(WsBroadcast {
        channel_id: Some(post_channel_id),
        team_id: None,
        user_id: None,
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(updated.into()))
}

async fn load_post_edit_time_limit_seconds(state: &AppState) -> ApiResult<i64> {
    let limit = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE((site->>'post_edit_time_limit_seconds')::bigint, -1) FROM server_config WHERE id = 'default'",
    )
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(-1);
    Ok(limit)
}

/// POST /posts/{post_id}/ack - Acknowledge a post (push notification receipt)
#[derive(Deserialize)]
#[allow(dead_code)]
struct AckPostRequest {
    #[serde(default)]
    post_id: String,
}

async fn ack_post(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate the post ID
    let _post_id = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post_id".to_string()))?;

    // Acknowledgments are typically used for:
    // 1. Confirming push notification receipt
    // 2. Analytics/delivery tracking
    // For now, we just return success - can be extended to track delivery status

    Ok(Json(serde_json::json!({"status": "OK"})))
}

#[derive(serde::Deserialize)]
pub struct CreateScheduledPostRequest {
    pub channel_id: String,
    pub message: String,
    #[serde(default)]
    pub root_id: String,
    #[serde(default)]
    pub props: serde_json::Value,
    #[serde(default)]
    pub file_ids: Vec<String>,
    pub scheduled_at: i64,
}

async fn list_scheduled_posts(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id_str): Path<String>,
) -> ApiResult<Json<Vec<mm::ScheduledPost>>> {
    let team_id = parse_mm_or_uuid(&team_id_str)
        .ok_or_else(|| AppError::Validation("Invalid team_id".to_string()))?;

    #[allow(clippy::type_complexity)]
    let rows: Vec<(Uuid, Uuid, Uuid, Option<Uuid>, String, serde_json::Value, Vec<Uuid>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT id, user_id, channel_id, root_id, message, props, file_ids, scheduled_at, created_at, updated_at
        FROM scheduled_posts
        WHERE user_id = $1 AND channel_id IN (SELECT id FROM channels WHERE team_id = $2)
        AND state = 'pending'
        "#
    )
    .bind(auth.user_id)
    .bind(team_id)
    .fetch_all(&state.db)
    .await?;

    let posts = rows
        .into_iter()
        .map(|r| mm::ScheduledPost {
            id: encode_mm_id(r.0),
            user_id: encode_mm_id(r.1),
            channel_id: encode_mm_id(r.2),
            root_id: r.3.map(encode_mm_id).unwrap_or_default(),
            message: r.4,
            props: r.5,
            file_ids: r.6.into_iter().map(encode_mm_id).collect(),
            scheduled_at: r.7.timestamp_millis(),
            create_at: r.8.timestamp_millis(),
            update_at: r.9.timestamp_millis(),
        })
        .collect();

    Ok(Json(posts))
}

async fn create_scheduled_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<CreateScheduledPostRequest>,
) -> ApiResult<Json<mm::ScheduledPost>> {
    let channel_id = parse_mm_or_uuid(&input.channel_id)
        .ok_or_else(|| AppError::Validation("Invalid channel_id".to_string()))?;

    let root_id = if !input.root_id.is_empty() {
        Some(
            parse_mm_or_uuid(&input.root_id)
                .ok_or_else(|| AppError::Validation("Invalid root_id".to_string()))?,
        )
    } else {
        None
    };

    let file_ids = input
        .file_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect::<Vec<_>>();
    let scheduled_at = chrono::DateTime::from_timestamp_millis(input.scheduled_at)
        .ok_or_else(|| AppError::Validation("Invalid scheduled_at".to_string()))?;

    let row: (Uuid, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        r#"
        INSERT INTO scheduled_posts (user_id, channel_id, root_id, message, props, file_ids, scheduled_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, created_at, updated_at
        "#
    )
    .bind(auth.user_id)
    .bind(channel_id)
    .bind(root_id)
    .bind(&input.message)
    .bind(&input.props)
    .bind(&file_ids)
    .bind(scheduled_at)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(mm::ScheduledPost {
        id: encode_mm_id(row.0),
        user_id: encode_mm_id(auth.user_id),
        channel_id: input.channel_id,
        root_id: input.root_id,
        message: input.message,
        props: input.props,
        file_ids: input.file_ids,
        scheduled_at: input.scheduled_at,
        create_at: row.1.timestamp_millis(),
        update_at: row.2.timestamp_millis(),
    }))
}

#[derive(Deserialize)]
struct UpdateScheduledPostRequest {
    id: String,
    channel_id: String,
    user_id: String,
    message: String,
    scheduled_at: i64,
    #[serde(default)]
    root_id: String,
    #[serde(default)]
    props: serde_json::Value,
    #[serde(default)]
    file_ids: Vec<String>,
}

async fn update_scheduled_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(scheduled_post_id): Path<String>,
    Json(input): Json<UpdateScheduledPostRequest>,
) -> ApiResult<Json<mm::ScheduledPost>> {
    if input.id != scheduled_post_id {
        return Err(AppError::BadRequest(
            "Scheduled post id mismatch".to_string(),
        ));
    }

    let scheduled_id = parse_mm_or_uuid(&scheduled_post_id)
        .ok_or_else(|| AppError::Validation("Invalid scheduled_post_id".to_string()))?;
    let channel_id = parse_mm_or_uuid(&input.channel_id)
        .ok_or_else(|| AppError::Validation("Invalid channel_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&input.user_id)
        .ok_or_else(|| AppError::Validation("Invalid user_id".to_string()))?;

    if user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot update another user's scheduled post".to_string(),
        ));
    }

    let root_id = if !input.root_id.is_empty() {
        Some(
            parse_mm_or_uuid(&input.root_id)
                .ok_or_else(|| AppError::Validation("Invalid root_id".to_string()))?,
        )
    } else {
        None
    };

    let file_ids = input
        .file_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect::<Vec<_>>();
    let scheduled_at = chrono::DateTime::from_timestamp_millis(input.scheduled_at)
        .ok_or_else(|| AppError::Validation("Invalid scheduled_at".to_string()))?;

    let row: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        r#"
        UPDATE scheduled_posts
        SET channel_id = $1,
            root_id = $2,
            message = $3,
            props = $4,
            file_ids = $5,
            scheduled_at = $6,
            updated_at = NOW()
        WHERE id = $7 AND user_id = $8
        RETURNING created_at, updated_at
        "#,
    )
    .bind(channel_id)
    .bind(root_id)
    .bind(&input.message)
    .bind(&input.props)
    .bind(&file_ids)
    .bind(scheduled_at)
    .bind(scheduled_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Scheduled post not found".to_string()))?;

    Ok(Json(mm::ScheduledPost {
        id: scheduled_post_id,
        user_id: input.user_id,
        channel_id: input.channel_id,
        root_id: input.root_id,
        message: input.message,
        props: input.props,
        file_ids: input.file_ids,
        scheduled_at: input.scheduled_at,
        create_at: row.0.timestamp_millis(),
        update_at: row.1.timestamp_millis(),
    }))
}

async fn delete_scheduled_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(scheduled_post_id): Path<String>,
) -> ApiResult<Json<mm::ScheduledPost>> {
    let scheduled_id = parse_mm_or_uuid(&scheduled_post_id)
        .ok_or_else(|| AppError::Validation("Invalid scheduled_post_id".to_string()))?;

    // Get the scheduled post details before deleting
    #[allow(clippy::type_complexity)]
    let row: Option<(Uuid, Uuid, String, String, serde_json::Value, Vec<Uuid>, i64, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        DELETE FROM scheduled_posts
        WHERE id = $1 AND user_id = $2 AND processed_at = 0
        RETURNING channel_id, user_id, root_id::text, message, props, file_ids, scheduled_at, create_at, update_at
        "#,
    )
    .bind(scheduled_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Scheduled post not found".to_string()))?;

    Ok(Json(mm::ScheduledPost {
        id: scheduled_post_id,
        user_id: encode_mm_id(row.1),
        channel_id: encode_mm_id(row.0),
        root_id: row.2.clone(),
        message: row.3.clone(),
        props: row.4.clone(),
        file_ids: row.5.iter().map(|id| encode_mm_id(*id)).collect(),
        scheduled_at: row.6,
        create_at: row.7.timestamp_millis(),
        update_at: row.8.timestamp_millis(),
    }))
}

#[derive(serde::Deserialize)]
pub struct EphemeralPostRequest {
    pub user_id: String,
    pub post: CreatePostRequest,
}

async fn create_ephemeral_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<EphemeralPostRequest>,
) -> ApiResult<Json<mm::Post>> {
    let target_user_id = parse_mm_or_uuid(&input.user_id)
        .ok_or_else(|| AppError::Validation("Invalid user_id".to_string()))?;

    if target_user_id != auth.user_id && input.user_id != "me" {
        return Err(AppError::Forbidden(
            "Cannot send ephemeral post to others".to_string(),
        ));
    }

    let channel_id = parse_mm_or_uuid(&input.post.channel_id)
        .ok_or_else(|| AppError::Validation("Invalid channel_id".to_string()))?;

    let post_id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp_millis();

    let ephemeral_post = mm::Post {
        id: encode_mm_id(post_id),
        create_at: now,
        update_at: now,
        delete_at: 0,
        edit_at: 0,
        user_id: encode_mm_id(auth.user_id),
        channel_id: input.post.channel_id,
        root_id: input.post.root_id,
        original_id: "".to_string(),
        message: input.post.message,
        post_type: "ephemeral".to_string(),
        props: input.post.props,
        hashtags: "".to_string(),
        file_ids: input.post.file_ids,
        pending_post_id: input.post.pending_post_id,
        metadata: None,
    };

    let broadcast = WsEnvelope::event(
        EventType::EphemeralMessage,
        ephemeral_post.clone(),
        Some(channel_id),
    )
    .with_broadcast(WsBroadcast {
        channel_id: Some(channel_id),
        team_id: None,
        user_id: Some(auth.user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(ephemeral_post))
}

#[derive(serde::Deserialize)]
pub struct PostReminderRequest {
    pub target_at: i64,
}

async fn set_post_reminder(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((user_id_str, post_id_str)): Path<(String, String)>,
    Json(input): Json<PostReminderRequest>,
) -> ApiResult<impl axum::response::IntoResponse> {
    let target_user_id = parse_mm_or_uuid(&user_id_str)
        .ok_or_else(|| AppError::Validation("Invalid user_id".to_string()))?;

    if target_user_id != auth.user_id && user_id_str != "me" {
        return Err(AppError::Forbidden(
            "Cannot set reminder for others".to_string(),
        ));
    }

    let post_id = parse_mm_or_uuid(&post_id_str)
        .ok_or_else(|| AppError::Validation("Invalid post_id".to_string()))?;

    let target_at = chrono::DateTime::from_timestamp_millis(input.target_at)
        .ok_or_else(|| AppError::Validation("Invalid target_at".to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO post_reminders (user_id, post_id, target_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, post_id) DO UPDATE SET target_at = $3
        "#,
    )
    .bind(auth.user_id)
    .bind(post_id)
    .bind(target_at)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}
