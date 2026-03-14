use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::auth::policy::permissions;
use crate::auth::{hash_password, AuthUser};
use crate::error::{ApiResult, AppError};
use crate::models::{ChangePassword, UpdateUser, User, UserResponse};

/// User status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatusResponse {
    pub user_id: String,
    pub status: String,
    pub manual: bool,
    pub last_activity_at: i64,
}

/// Update status request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub dnd_end_time: Option<i64>,
}


/// Build users routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", get(get_user).put(update_user))
        .route("/{id}/password", axum::routing::post(change_password))
        .route(
            "/me/status",
            get(get_my_status)
                .put(update_my_status)
                .delete(delete_my_status),
        )
        .route("/{id}/status", get(get_user_status))
        .route("/status/ids", axum::routing::post(get_statuses_by_ids))
        // Thread routes (Mattermost-compatible)
        .route("/{user_id}/teams/{team_id}/threads/{thread_id}/read/{timestamp}", axum::routing::put(mark_thread_as_read))
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub _org_id: Option<Uuid>,
    pub q: Option<String>,
}

/// List users (requires admin or same org)
async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListUsersQuery>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;
    let search_term = query.q.map(|s| format!("%{}%", s));

    let users: Vec<User> = if auth.has_permission(&permissions::ADMIN_FULL) {
        // System admin can see all users
        if let Some(term) = search_term {
            sqlx::query_as(
                "SELECT * FROM users WHERE deleted_at IS NULL AND (username ILIKE $1 OR display_name ILIKE $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(term)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
        } else {
            sqlx::query_as("SELECT * FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(&state.db)
                .await?
        }
    } else if let Some(org_id) = auth.org_id {
        // Regular users see their org members
        if let Some(term) = search_term {
            sqlx::query_as(
                "SELECT * FROM users WHERE org_id = $1 AND deleted_at IS NULL AND (username ILIKE $2 OR display_name ILIKE $2) ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(org_id)
            .bind(term)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
        } else {
            sqlx::query_as(
                "SELECT * FROM users WHERE org_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(org_id)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
        }
    } else {
        return Err(AppError::Forbidden("No organization access".to_string()));
    };

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

/// Get a specific user
async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserResponse>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Check access: same user, same org, or system admin
    let can_view = auth.user_id == user.id
        || auth.has_permission(&permissions::ADMIN_FULL)
        || (auth.org_id.is_some() && auth.org_id == user.org_id);

    if !can_view {
        return Err(AppError::Forbidden("Cannot view this user".to_string()));
    }

    Ok(Json(UserResponse::from(user)))
}

/// Update a user profile
async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUser>,
) -> ApiResult<Json<UserResponse>> {
    // Only the user themselves or an admin can update
    if !auth.can_access_owned(id, &permissions::USER_MANAGE) {
        return Err(AppError::Forbidden("Cannot update this user".to_string()));
    }

    let has_updates = input.username.is_some()
        || input.display_name.is_some()
        || input.first_name.is_some()
        || input.last_name.is_some()
        || input.nickname.is_some()
        || input.position.is_some()
        || input.avatar_url.is_some()
        || input.custom_status.is_some();

    if !has_updates {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    // For simplicity, we'll do separate optional updates
    if let Some(ref username) = input.username {
        sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(username)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref custom_status) = input.custom_status {
        sqlx::query("UPDATE users SET custom_status = $1 WHERE id = $2")
            .bind(custom_status)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref display_name) = input.display_name {
        sqlx::query("UPDATE users SET display_name = $1 WHERE id = $2")
            .bind(display_name)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref first_name) = input.first_name {
        sqlx::query("UPDATE users SET first_name = $1 WHERE id = $2")
            .bind(first_name)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref last_name) = input.last_name {
        sqlx::query("UPDATE users SET last_name = $1 WHERE id = $2")
            .bind(last_name)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref nickname) = input.nickname {
        sqlx::query("UPDATE users SET nickname = $1 WHERE id = $2")
            .bind(nickname)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref position) = input.position {
        sqlx::query("UPDATE users SET position = $1 WHERE id = $2")
            .bind(position)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref avatar_url) = input.avatar_url {
        sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
            .bind(avatar_url)
            .bind(id)
            .execute(&state.db)
            .await?;
    }

    // Fetch updated user
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    // Broadcast update
    let user_response = UserResponse::from(user.clone());
    let event = crate::realtime::events::WsEnvelope::event(
        crate::realtime::events::EventType::UserUpdated,
        user_response.clone(),
        None,
    )
    .with_broadcast(crate::realtime::events::WsBroadcast {
        team_id: None,
        channel_id: None,
        user_id: None, // Broadcast to everyone
        exclude_user_id: Some(auth.user_id),
    });
    state.ws_hub.broadcast(event).await;

    Ok(Json(user_response))
}

/// Change user password
async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ChangePassword>,
) -> ApiResult<Json<serde_json::Value>> {
    // Only the user themselves can change their password
    if auth.user_id != id {
        return Err(AppError::Forbidden(
            "Cannot change password for another user".to_string(),
        ));
    }

    // Fetch user with current password hash (still needed to ensure user exists and for other potential logic)
    let _user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    // Validate new password complexity
    let config = crate::services::auth_config::get_password_rules(&state.db).await?;
    crate::services::auth_config::validate_password(&input.new_password, &config)?;

    // Hash and update
    let new_hash = hash_password(&input.new_password)?;

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_hash)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(
        serde_json::json!({ "status": "success", "message": "Password updated successfully" }),
    ))
}

/// Get current user's status
/// GET /api/v1/users/me/status
async fn get_my_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<UserStatusResponse>> {
    get_user_status_by_id(&state, auth.user_id).await
}

/// Get user status by ID
/// GET /api/v1/users/{id}/status
async fn get_user_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserStatusResponse>> {
    get_user_status_by_id(&state, id).await
}

/// Internal: Get user status by UUID
async fn get_user_status_by_id(
    state: &AppState,
    user_id: Uuid,
) -> ApiResult<Json<UserStatusResponse>> {
    let result: (String, bool, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        r#"
        SELECT presence, COALESCE(presence_manual, false), last_login_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    let (presence, manual, last_login) = result;

    Ok(Json(UserStatusResponse {
        user_id: user_id.to_string(),
        status: if presence.is_empty() {
            "offline".to_string()
        } else {
            presence
        },
        manual,
        last_activity_at: last_login.map(|t| t.timestamp_millis()).unwrap_or(0),
    }))
}

/// Update my status
/// PUT /api/v1/users/me/status
async fn update_my_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateStatusRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate status
    let valid_statuses = ["online", "away", "dnd", "offline"];
    if !valid_statuses.contains(&body.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid status: {}. Must be one of: online, away, dnd, offline",
            body.status
        )));
    }

    // Determine if this is a manual status
    let manual = body.status != "online";

    // Update user presence
    sqlx::query(
        r#"
        UPDATE users
        SET presence = $2, presence_manual = $3
        WHERE id = $1
        "#,
    )
    .bind(auth.user_id)
    .bind(&body.status)
    .bind(manual)
    .execute(&state.db)
    .await?;

    // Broadcast status change via WebSocket
    let event = crate::realtime::events::WsEnvelope::event(
        crate::realtime::events::EventType::UserPresence,
        serde_json::json!({
            "user_id": auth.user_id.to_string(),
            "status": &body.status,
            "manual": manual,
        }),
        None,
    );
    state.ws_hub.broadcast(event).await;

    Ok(Json(serde_json::json!({"status": body.status})))
}

/// Delete/clear my status (custom status text/emoji)
/// DELETE /api/v1/users/me/status
async fn delete_my_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    sqlx::query(
        "UPDATE users SET status_text = NULL, status_emoji = NULL, status_expires_at = NULL, updated_at = NOW() WHERE id = $1"
    )
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"status": "cleared"})))
}

/// Get statuses for multiple users
/// POST /api/v1/users/status/ids
/// Accepts both: ["id1", "id2"] and {"user_ids": ["id1", "id2"]}
async fn get_statuses_by_ids(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    // Extract user_ids from either array format or object format
    let user_ids: Vec<String> = if let Some(arr) = body.as_array() {
        // Direct array format: ["id1", "id2"]
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else if let Some(obj) = body.as_object() {
        // Object format: {"user_ids": ["id1", "id2"]}
        obj.get("user_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let user_ids: Vec<Uuid> = user_ids
        .iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect();

    if user_ids.is_empty() {
        return Ok(Json(serde_json::json!({})));
    }

    let statuses: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, presence
        FROM users
        WHERE id = ANY($1)
        "#,
    )
    .bind(&user_ids)
    .fetch_all(&state.db)
    .await?;

    // Build response map - return format expected by frontend
    // Frontend expects: [{"user_id": "...", "status": "...", ...}]
    let result: Vec<serde_json::Value> = statuses
        .into_iter()
        .map(|(user_id, status)| {
            serde_json::json!({
                "user_id": user_id.to_string(),
                "status": status,
                "manual": false,
                "last_activity_at": 0
            })
        })
        .collect();

    Ok(Json(serde_json::json!(result)))
}


/// Mark a thread as read
/// PUT /users/{user_id}/teams/{team_id}/threads/{thread_id}/read/{timestamp}
async fn mark_thread_as_read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, _team_id, thread_id, timestamp)): Path<(String, Uuid, Uuid, i64)>,
) -> ApiResult<Json<serde_json::Value>> {
    // Handle "me" as current user
    let target_user_id = if user_id == "me" {
        auth.user_id
    } else {
        user_id.parse::<Uuid>()
            .map_err(|_| AppError::BadRequest("Invalid user_id".to_string()))?
    };

    // Users can only mark their own threads as read
    if target_user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot mark thread as read for other users".to_string(),
        ));
    }

    let read_at = chrono::DateTime::from_timestamp_millis(timestamp)
        .unwrap_or_else(chrono::Utc::now);

    // Upsert thread membership with read time
    sqlx::query(r#"
        INSERT INTO thread_memberships (user_id, post_id, last_read_at, unread_replies_count, mention_count)
        VALUES ($1, $2, $3, 0, 0)
        ON CONFLICT (user_id, post_id) DO UPDATE SET
            last_read_at = $3,
            unread_replies_count = 0,
            mention_count = 0,
            updated_at = NOW()
    "#)
    .bind(target_user_id)
    .bind(thread_id)
    .bind(read_at)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}
