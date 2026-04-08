use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::utils::ensure_channel_admin_or_system_manage;
use super::utils::resolve_direct_channel_display_name;
use super::{
    mm, parse_mm_or_uuid, permissions, ApiResult, AppError, AppState, Channel, MmAuthUser,
};
use crate::api::v4::channels::utils::map_channel_with_team_data_row;
use crate::realtime::events::{EventType, WsBroadcast, WsEnvelope};
use serde_json::json;

#[derive(Debug, Deserialize, Default)]
pub struct GetAllChannelsQuery {
    #[serde(default)]
    pub not_associated_to_group: Option<String>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub per_page: Option<u64>,
    #[serde(default)]
    pub exclude_default_channels: bool,
    #[serde(default)]
    pub include_deleted: bool,
    #[serde(default)]
    pub include_total_count: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ChannelWithTeamDataRow {
    pub id: Uuid,
    pub team_id: Uuid,
    #[sqlx(rename = "type")]
    pub channel_type: crate::models::channel::ChannelType,
    pub name: String,
    pub display_name: Option<String>,
    pub purpose: Option<String>,
    pub header: Option<String>,
    pub is_archived: bool,
    pub creator_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub team_display_name: Option<String>,
    pub team_name: String,
    pub team_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct ChannelWithTeamDataResponse {
    #[serde(flatten)]
    pub channel: mm::Channel,
    pub team_display_name: String,
    pub team_name: String,
    pub team_update_at: i64,
}

pub async fn get_all_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<GetAllChannelsQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE)
        && !auth.has_permission(&permissions::ADMIN_FULL)
    {
        return Err(AppError::Forbidden(
            "Insufficient permissions to list all channels".to_string(),
        ));
    }

    let not_associated_group_id = query
        .not_associated_to_group
        .as_deref()
        .filter(|raw| !raw.trim().is_empty())
        .map(|raw| {
            parse_mm_or_uuid(raw)
                .ok_or_else(|| AppError::BadRequest("Invalid not_associated_to_group".to_string()))
        })
        .transpose()?;

    let page = query.page.unwrap_or(0);
    let mut per_page = query.per_page.unwrap_or(60);
    if per_page == 0 {
        per_page = 60;
    }
    per_page = per_page.min(10_000);
    let offset = page.saturating_mul(per_page) as i64;

    let rows: Vec<ChannelWithTeamDataRow> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.team_id,
            c.type,
            c.name,
            c.display_name,
            c.purpose,
            c.header,
            c.is_archived,
            c.creator_id,
            c.created_at,
            c.updated_at,
            t.display_name AS team_display_name,
            t.name AS team_name,
            t.updated_at AS team_updated_at
        FROM channels c
        JOIN teams t ON t.id = c.team_id
        WHERE
            ($1::bool OR c.is_archived = false)
            AND (NOT $2::bool OR c.name NOT IN ('town-square', 'off-topic'))
            AND (
                $3::uuid IS NULL
                OR NOT EXISTS (
                    SELECT 1
                    FROM group_syncables gs
                    WHERE gs.syncable_type = 'channel'
                      AND gs.group_id = $3
                      AND gs.syncable_id = c.id
                )
            )
        ORDER BY c.created_at ASC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(query.include_deleted)
    .bind(query.exclude_default_channels)
    .bind(not_associated_group_id)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let channels: Vec<ChannelWithTeamDataResponse> = rows
        .into_iter()
        .map(map_channel_with_team_data_row)
        .collect();

    if query.include_total_count {
        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM channels c
            WHERE
                ($1::bool OR c.is_archived = false)
                AND (NOT $2::bool OR c.name NOT IN ('town-square', 'off-topic'))
                AND (
                    $3::uuid IS NULL
                    OR NOT EXISTS (
                        SELECT 1
                        FROM group_syncables gs
                        WHERE gs.syncable_type = 'channel'
                          AND gs.group_id = $3
                          AND gs.syncable_id = c.id
                    )
                )
            "#,
        )
        .bind(query.include_deleted)
        .bind(query.exclude_default_channels)
        .bind(not_associated_group_id)
        .fetch_one(&state.db)
        .await?;

        return Ok(Json(json!({
            "channels": channels,
            "total_count": total_count
        })));
    }

    Ok(Json(json!(channels)))
}

pub async fn get_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
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

    let mut channel: crate::models::Channel =
        sqlx::query_as("SELECT * FROM channels WHERE id = $1 AND deleted_at IS NULL")
            .bind(channel_id)
            .fetch_one(&state.db)
            .await?;

    // For Direct channels, ALWAYS compute display_name from the other participant
    // This ensures each user sees the other person's name, not their own
    if channel.channel_type == crate::models::channel::ChannelType::Direct {
        channel.display_name =
            resolve_direct_channel_display_name(&state, channel.id, auth.user_id)
                .await?
                .or_else(|| Some("Direct Message".to_string()));
    }

    Ok(Json(channel.into()))
}

/// POST /channels - Create a new channel
#[derive(serde::Deserialize)]
pub struct CreateChannelRequest {
    pub team_id: String,
    pub name: String,
    pub display_name: String,
    #[serde(rename = "type", default = "default_channel_type")]
    pub channel_type: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub header: String,
}

fn default_channel_type() -> String {
    "O".to_string()
}

pub async fn create_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    let input: CreateChannelRequest =
        super::utils::parse_body(&headers, &body, "Invalid channel body")?;

    let team_id = parse_mm_or_uuid(&input.team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    // Verify team membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(crate::error::AppError::Forbidden(
            "Not a member of this team".to_string(),
        ));
    }

    // Map MM channel type to RustChat type
    let channel_type = match input.channel_type.as_str() {
        "O" => "public",
        "P" => "private",
        _ => "public",
    };

    let channel: Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (team_id, type, name, display_name, purpose, header, creator_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(channel_type)
    .bind(&input.name)
    .bind(&input.display_name)
    .bind(&input.purpose)
    .bind(&input.header)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    // Add creator as member
    sqlx::query(
        "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'admin') ON CONFLICT DO NOTHING",
    )
    .bind(channel.id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// PUT /channels/{channel_id} - Update channel
#[derive(serde::Deserialize)]
pub struct UpdateChannelRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub header: Option<String>,
}

pub async fn update_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let input: UpdateChannelRequest =
        super::utils::parse_body(&headers, &body, "Invalid channel update")?;

    // Build update query dynamically
    let channel: Channel = sqlx::query_as(
        r#"
        UPDATE channels SET
            display_name = COALESCE($2, display_name),
            name = COALESCE($3, name),
            purpose = COALESCE($4, purpose),
            header = COALESCE($5, header),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(channel_id)
    .bind(&input.display_name)
    .bind(&input.name)
    .bind(&input.purpose)
    .bind(&input.header)
    .fetch_one(&state.db)
    .await?;

    // Broadcast ChannelUpdated event
    let broadcast = WsBroadcast {
        channel_id: Some(channel_id),
        team_id: Some(channel.team_id),
        user_id: None,
        exclude_user_id: Some(auth.user_id),
    };
    let event = WsEnvelope::event(EventType::ChannelUpdated, &channel, Some(channel_id))
        .with_broadcast(broadcast);
    state.ws_hub.broadcast(event).await;

    Ok(Json(channel.into()))
}

/// DELETE /channels/{channel_id} - Delete/archive channel
pub async fn delete_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Only channel admins or system admins may delete a channel
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    // Get channel info for the broadcast
    let channel: Channel =
        sqlx::query_as("SELECT * FROM channels WHERE id = $1 AND deleted_at IS NULL")
            .bind(channel_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    // Soft delete the channel
    sqlx::query("UPDATE channels SET deleted_at = NOW() WHERE id = $1")
        .bind(channel_id)
        .execute(&state.db)
        .await?;

    // Broadcast ChannelDeleted event
    let broadcast = WsBroadcast {
        channel_id: Some(channel_id),
        team_id: Some(channel.team_id),
        user_id: None,
        exclude_user_id: Some(auth.user_id),
    };
    let event = WsEnvelope::event(
        EventType::ChannelDeleted,
        serde_json::json!({
            "channel_id": channel_id.to_string(),
            "team_id": channel.team_id.to_string(),
        }),
        Some(channel_id),
    )
    .with_broadcast(broadcast);
    state.ws_hub.broadcast(event).await;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// PUT /channels/{channel_id}/patch - Patch channel (partial update)
#[derive(serde::Deserialize)]
pub struct PatchChannelRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub header: Option<String>,
}

pub async fn patch_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(input): Json<PatchChannelRequest>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let channel: Channel = sqlx::query_as(
        r#"
        UPDATE channels SET
            display_name = COALESCE($2, display_name),
            name = COALESCE($3, name),
            purpose = COALESCE($4, purpose),
            header = COALESCE($5, header),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(channel_id)
    .bind(&input.display_name)
    .bind(&input.name)
    .bind(&input.purpose)
    .bind(&input.header)
    .fetch_one(&state.db)
    .await?;

    // Broadcast ChannelUpdated event
    let broadcast = WsBroadcast {
        channel_id: Some(channel_id),
        team_id: Some(channel.team_id),
        user_id: None,
        exclude_user_id: Some(auth.user_id),
    };
    let event = WsEnvelope::event(EventType::ChannelUpdated, &channel, Some(channel_id))
        .with_broadcast(broadcast);
    state.ws_hub.broadcast(event).await;

    Ok(Json(channel.into()))
}

/// PUT /channels/{channel_id}/privacy - Update channel privacy
#[derive(serde::Deserialize)]
pub struct UpdatePrivacyRequest {
    pub privacy: String, // "O" for public, "P" for private
}

pub async fn update_channel_privacy(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(input): Json<UpdatePrivacyRequest>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Only channel admins or system admins may change channel privacy
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let channel_type = match input.privacy.as_str() {
        "O" => "public",
        "P" => "private",
        _ => {
            return Err(crate::error::AppError::BadRequest(
                "Invalid privacy value".to_string(),
            ))
        }
    };

    let channel: Channel = sqlx::query_as(
        r#"UPDATE channels SET type = $2, updated_at = NOW() WHERE id = $1 RETURNING *"#,
    )
    .bind(channel_id)
    .bind(channel_type)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// POST /channels/{channel_id}/restore - Restore a deleted channel
pub async fn restore_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Only channel admins or system admins may restore a deleted channel
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let channel: Channel = sqlx::query_as(
        r#"UPDATE channels SET deleted_at = NULL, updated_at = NOW() WHERE id = $1 RETURNING *"#,
    )
    .bind(channel_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// POST /channels/{channel_id}/move - Move channel to another team
#[derive(serde::Deserialize)]
pub struct MoveChannelRequest {
    pub team_id: String,
    #[serde(rename = "force", default)]
    pub _force: bool,
}

pub async fn move_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(input): Json<MoveChannelRequest>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let new_team_id = parse_mm_or_uuid(&input.team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    // Only system admins may move a channel between teams
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "System admin privileges required to move channels between teams".to_string(),
        ));
    }

    // Verify membership in new team
    let is_team_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
    )
    .bind(new_team_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_team_member {
        return Err(crate::error::AppError::Forbidden(
            "Not a member of the target team".to_string(),
        ));
    }

    let channel: Channel = sqlx::query_as(
        r#"UPDATE channels SET team_id = $2, updated_at = NOW() WHERE id = $1 RETURNING *"#,
    )
    .bind(channel_id)
    .bind(new_team_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}
