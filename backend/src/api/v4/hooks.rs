use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/hooks/incoming",
            get(list_incoming_hooks).post(create_incoming_hook),
        )
        .route(
            "/hooks/incoming/{hook_id}",
            get(get_incoming_hook)
                .put(update_incoming_hook)
                .delete(delete_incoming_hook),
        )
        .route(
            "/hooks/outgoing",
            get(list_outgoing_hooks).post(create_outgoing_hook),
        )
        .route(
            "/hooks/outgoing/{hook_id}",
            get(get_outgoing_hook)
                .put(update_outgoing_hook)
                .delete(delete_outgoing_hook),
        )
        .route(
            "/hooks/outgoing/{hook_id}/regen_token",
            post(regen_outgoing_hook_token),
        )
        // Public incoming webhook endpoint (no auth required)
        .route("/hooks/{token}", post(execute_incoming_hook))
}
use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::{IncomingWebhook, OutgoingWebhook};
use axum::extract::Path;
use uuid::Uuid;

/// Check if user can manage a webhook (creator or system admin)
async fn can_manage_incoming_hook(
    state: &AppState,
    hook_id: Uuid,
    auth: &MmAuthUser,
) -> ApiResult<bool> {
    if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Ok(true);
    }
    let creator_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT creator_id FROM incoming_webhooks WHERE id = $1"
    )
    .bind(hook_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(creator_id == Some(auth.user_id))
}

async fn can_manage_outgoing_hook(
    state: &AppState,
    hook_id: Uuid,
    auth: &MmAuthUser,
) -> ApiResult<bool> {
    if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Ok(true);
    }
    let creator_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT creator_id FROM outgoing_webhooks WHERE id = $1"
    )
    .bind(hook_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(creator_id == Some(auth.user_id))
}

#[derive(serde::Deserialize)]
pub struct CreateIncomingRequest {
    pub channel_id: String,
    pub display_name: String,
    pub description: String,
}

#[derive(serde::Deserialize)]
pub struct CreateOutgoingRequest {
    pub team_id: String,
    pub channel_id: Option<String>,
    pub display_name: String,
    pub description: String,
    pub trigger_words: Vec<String>,
    #[serde(rename = "trigger_when")]
    pub _trigger_when: i32,
    pub callback_urls: Vec<String>,
    pub content_type: String,
}

#[derive(serde::Deserialize)]
pub struct HooksQuery {
    pub team_id: Option<String>,
    #[serde(rename = "channel_id")]
    pub _channel_id: Option<String>,
    #[serde(default)]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_per_page() -> i64 {
    50
}

pub async fn create_incoming_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<CreateIncomingRequest>,
) -> ApiResult<Json<mm::IncomingWebhook>> {
    let channel_id = parse_mm_or_uuid(&input.channel_id)
        .ok_or_else(|| AppError::Validation("Invalid channel_id".to_string()))?;

    // Get team_id for the channel and verify the caller is a member
    let team_id: Uuid = sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_one(&state.db)
        .await?;

    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;
    if !is_member {
        return Err(AppError::Forbidden(
            "Not a member of this channel".to_string(),
        ));
    }

    let hook: IncomingWebhook = sqlx::query_as(
        r#"
        INSERT INTO incoming_webhooks (team_id, channel_id, creator_id, display_name, description, token, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(team_id)
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(&input.display_name)
    .bind(&input.description)
    .bind(Uuid::new_v4().to_string()) // In-situ token generation
    .bind(true)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(map_incoming_hook(hook)))
}

pub async fn list_incoming_hooks(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<HooksQuery>,
) -> ApiResult<Json<Vec<mm::IncomingWebhook>>> {
    // List only hooks the user can manage: created by them or all if system admin
    let hooks: Vec<IncomingWebhook> = if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        if let Some(ref tid_str) = query.team_id {
            let tid = parse_mm_or_uuid(tid_str);
            if let Some(tid) = tid {
                sqlx::query_as("SELECT * FROM incoming_webhooks WHERE team_id = $1 LIMIT $2 OFFSET $3")
                    .bind(tid)
                    .bind(query.per_page)
                    .bind(query.page * query.per_page)
                    .fetch_all(&state.db).await?
            } else {
                vec![]
            }
        } else {
            sqlx::query_as("SELECT * FROM incoming_webhooks LIMIT $1 OFFSET $2")
                .bind(query.per_page)
                .bind(query.page * query.per_page)
                .fetch_all(&state.db).await?
        }
    } else {
        // Regular users can only see hooks they created
        if let Some(ref tid_str) = query.team_id {
            let tid = parse_mm_or_uuid(tid_str);
            if let Some(tid) = tid {
                sqlx::query_as("SELECT * FROM incoming_webhooks WHERE creator_id = $1 AND team_id = $2 LIMIT $3 OFFSET $4")
                    .bind(auth.user_id)
                    .bind(tid)
                    .bind(query.per_page)
                    .bind(query.page * query.per_page)
                    .fetch_all(&state.db).await?
            } else {
                vec![]
            }
        } else {
            sqlx::query_as("SELECT * FROM incoming_webhooks WHERE creator_id = $1 LIMIT $2 OFFSET $3")
                .bind(auth.user_id)
                .bind(query.per_page)
                .bind(query.page * query.per_page)
                .fetch_all(&state.db).await?
        }
    };

    Ok(Json(hooks.into_iter().map(map_incoming_hook).collect()))
}

pub async fn create_outgoing_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<CreateOutgoingRequest>,
) -> ApiResult<Json<mm::OutgoingWebhook>> {
    // Validate callback URLs to prevent SSRF
    for url in &input.callback_urls {
        if !crate::services::webhooks::is_valid_callback_url(url) {
            return Err(AppError::BadRequest(format!("Invalid callback URL (SSRF risk): {}", url)));
        }
    }

    let team_id = parse_mm_or_uuid(&input.team_id)
        .ok_or_else(|| AppError::Validation("Invalid team_id".to_string()))?;

    let channel_id = input.channel_id.and_then(|id| parse_mm_or_uuid(&id));

    let hook: OutgoingWebhook = sqlx::query_as(
        r#"
        INSERT INTO outgoing_webhooks (team_id, channel_id, creator_id, display_name, description, trigger_words, trigger_when, callback_urls, content_type, token, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#
    )
    .bind(team_id)
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(&input.display_name)
    .bind(&input.description)
    .bind(&input.trigger_words)
    .bind("first_word") // Simplified mapping for trigger_when
    .bind(&input.callback_urls)
    .bind(&input.content_type)
    .bind(Uuid::new_v4().to_string())
    .bind(true)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(map_outgoing_hook(hook)))
}

pub async fn list_outgoing_hooks(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<HooksQuery>,
) -> ApiResult<Json<Vec<mm::OutgoingWebhook>>> {
    // List only hooks the user can manage: created by them or all if system admin
    let hooks: Vec<OutgoingWebhook> = if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        if let Some(ref tid_str) = query.team_id {
            let tid = parse_mm_or_uuid(tid_str);
            if let Some(tid) = tid {
                sqlx::query_as("SELECT * FROM outgoing_webhooks WHERE team_id = $1 LIMIT $2 OFFSET $3")
                    .bind(tid)
                    .bind(query.per_page)
                    .bind(query.page * query.per_page)
                    .fetch_all(&state.db).await?
            } else {
                vec![]
            }
        } else {
            sqlx::query_as("SELECT * FROM outgoing_webhooks LIMIT $1 OFFSET $2")
                .bind(query.per_page)
                .bind(query.page * query.per_page)
                .fetch_all(&state.db).await?
        }
    } else {
        // Regular users can only see hooks they created
        if let Some(ref tid_str) = query.team_id {
            let tid = parse_mm_or_uuid(tid_str);
            if let Some(tid) = tid {
                sqlx::query_as("SELECT * FROM outgoing_webhooks WHERE creator_id = $1 AND team_id = $2 LIMIT $3 OFFSET $4")
                    .bind(auth.user_id)
                    .bind(tid)
                    .bind(query.per_page)
                    .bind(query.page * query.per_page)
                    .fetch_all(&state.db).await?
            } else {
                vec![]
            }
        } else {
            sqlx::query_as("SELECT * FROM outgoing_webhooks WHERE creator_id = $1 LIMIT $2 OFFSET $3")
                .bind(auth.user_id)
                .bind(query.per_page)
                .bind(query.page * query.per_page)
                .fetch_all(&state.db).await?
        }
    };

    Ok(Json(hooks.into_iter().map(map_outgoing_hook).collect()))
}

fn map_incoming_hook(h: IncomingWebhook) -> mm::IncomingWebhook {
    mm::IncomingWebhook {
        id: encode_mm_id(h.id),
        create_at: h.created_at.timestamp_millis(),
        update_at: h.updated_at.timestamp_millis(),
        delete_at: 0,
        user_id: encode_mm_id(h.creator_id),
        channel_id: encode_mm_id(h.channel_id),
        team_id: encode_mm_id(h.team_id),
        display_name: h.display_name.unwrap_or_default(),
        description: h.description.unwrap_or_default(),
    }
}

fn map_outgoing_hook(h: OutgoingWebhook) -> mm::OutgoingWebhook {
    mm::OutgoingWebhook {
        id: encode_mm_id(h.id),
        create_at: h.created_at.timestamp_millis(),
        update_at: h.updated_at.timestamp_millis(),
        delete_at: 0,
        creator_id: encode_mm_id(h.creator_id),
        channel_id: h.channel_id.map(encode_mm_id).unwrap_or_default(),
        team_id: encode_mm_id(h.team_id),
        trigger_words: h.trigger_words,
        trigger_when: 0,
        callback_urls: h.callback_urls,
        display_name: h.display_name.unwrap_or_default(),
        description: h.description.unwrap_or_default(),
        content_type: h.content_type.unwrap_or_default(),
    }
}

async fn get_incoming_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
) -> ApiResult<Json<mm::IncomingWebhook>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_incoming_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot access this webhook".to_string()));
    }

    let hook: IncomingWebhook = sqlx::query_as("SELECT * FROM incoming_webhooks WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::NotFound("Webhook not found".to_string()))?;

    Ok(Json(map_incoming_hook(hook)))
}

#[derive(serde::Deserialize)]
pub struct UpdateIncomingRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "channel_id")]
    pub _channel_id: Option<String>,
}

async fn update_incoming_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
    Json(input): Json<UpdateIncomingRequest>,
) -> ApiResult<Json<mm::IncomingWebhook>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_incoming_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot modify this webhook".to_string()));
    }

    let hook: IncomingWebhook = sqlx::query_as(
        r#"UPDATE incoming_webhooks SET
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            updated_at = NOW()
           WHERE id = $1 RETURNING *"#,
    )
    .bind(id)
    .bind(&input.display_name)
    .bind(&input.description)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("Webhook not found".to_string()))?;

    Ok(Json(map_incoming_hook(hook)))
}

async fn delete_incoming_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_incoming_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot delete this webhook".to_string()));
    }

    sqlx::query("DELETE FROM incoming_webhooks WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn get_outgoing_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
) -> ApiResult<Json<mm::OutgoingWebhook>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_outgoing_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot access this webhook".to_string()));
    }

    let hook: OutgoingWebhook = sqlx::query_as("SELECT * FROM outgoing_webhooks WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::NotFound("Webhook not found".to_string()))?;

    Ok(Json(map_outgoing_hook(hook)))
}

#[derive(serde::Deserialize)]
pub struct UpdateOutgoingRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub trigger_words: Option<Vec<String>>,
    pub callback_urls: Option<Vec<String>>,
}

async fn update_outgoing_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
    Json(input): Json<UpdateOutgoingRequest>,
) -> ApiResult<Json<mm::OutgoingWebhook>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_outgoing_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot modify this webhook".to_string()));
    }

    // Validate callback URLs if provided
    if let Some(ref urls) = input.callback_urls {
        for url in urls {
            if !crate::services::webhooks::is_valid_callback_url(url) {
                return Err(AppError::BadRequest(format!("Invalid callback URL (SSRF risk): {}", url)));
            }
        }
    }

    let hook: OutgoingWebhook = sqlx::query_as(
        r#"UPDATE outgoing_webhooks SET
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            trigger_words = COALESCE($4, trigger_words),
            callback_urls = COALESCE($5, callback_urls),
            updated_at = NOW()
           WHERE id = $1 RETURNING *"#,
    )
    .bind(id)
    .bind(&input.display_name)
    .bind(&input.description)
    .bind(&input.trigger_words)
    .bind(&input.callback_urls)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("Webhook not found".to_string()))?;

    Ok(Json(map_outgoing_hook(hook)))
}

async fn delete_outgoing_hook(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_outgoing_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot delete this webhook".to_string()));
    }

    sqlx::query("DELETE FROM outgoing_webhooks WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn regen_outgoing_hook_token(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(hook_id): Path<String>,
) -> ApiResult<Json<mm::OutgoingWebhook>> {
    let id = parse_mm_or_uuid(&hook_id)
        .ok_or_else(|| AppError::Validation("Invalid hook_id".to_string()))?;

    // Check ownership/permission
    if !can_manage_outgoing_hook(&state, id, &auth).await? {
        return Err(AppError::Forbidden("Cannot modify this webhook".to_string()));
    }

    let new_token = Uuid::new_v4().to_string().replace("-", "");

    let hook: OutgoingWebhook = sqlx::query_as(
        "UPDATE outgoing_webhooks SET token = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&new_token)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("Webhook not found".to_string()))?;

    Ok(Json(map_outgoing_hook(hook)))
}

/// Public endpoint for executing incoming webhooks (no auth required)
async fn execute_incoming_hook(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<crate::models::WebhookPayload>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::services::webhooks::execute_incoming_webhook(&state, &token, payload).await?;
    Ok(Json(serde_json::json!({"status": "OK"})))
}
