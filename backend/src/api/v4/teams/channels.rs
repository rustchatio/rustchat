use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use super::{ensure_team_member, search_team_channels};
use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::channel::ChannelType;
use crate::models::{Channel, Team};

pub async fn get_team_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.* FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE c.team_id = $1 AND cm.user_id = $2
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

pub async fn get_team_channel_ids(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    let ids: Vec<uuid::Uuid> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT c.id
        FROM channels c
        LEFT JOIN channel_members cm ON c.id = cm.channel_id AND cm.user_id = $2
        WHERE c.team_id = $1 AND (c.type = 'public' OR cm.user_id IS NOT NULL)
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(ids.into_iter().map(encode_mm_id).collect()))
}

pub async fn get_team_private_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.*
        FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE c.team_id = $1 AND c.type = 'private' AND cm.user_id = $2
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(channels.into_iter().map(|c| c.into()).collect()))
}

pub async fn get_team_deleted_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    // Return public archived channels plus private archived channels the user belongs to
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.* FROM channels c
        WHERE c.team_id = $1 AND c.is_archived = true
          AND (
            c.type != 'private'
            OR EXISTS (
                SELECT 1 FROM channel_members cm
                WHERE cm.channel_id = c.id AND cm.user_id = $2
            )
          )
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(channels.into_iter().map(|c| c.into()).collect()))
}

#[derive(Deserialize)]
pub struct ChannelAutocompleteQuery {
    name: Option<String>,
    term: Option<String>,
}

#[derive(Serialize)]
pub struct ChannelAutocompleteResponse {
    pub channels: Vec<mm::Channel>,
    pub users: Vec<mm::User>,
}

pub async fn autocomplete_team_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
    Query(query): Query<ChannelAutocompleteQuery>,
) -> ApiResult<Json<ChannelAutocompleteResponse>> {
    let term = query.name.or(query.term).unwrap_or_default();
    let channels = search_team_channels(&state, auth.user_id, &team_id, &term, 20).await?;
    Ok(Json(ChannelAutocompleteResponse {
        channels,
        users: vec![],
    }))
}

pub async fn search_autocomplete_team_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
    Query(query): Query<ChannelAutocompleteQuery>,
) -> ApiResult<Json<ChannelAutocompleteResponse>> {
    let term = query.name.or(query.term).unwrap_or_default();
    let channels = search_team_channels(&state, auth.user_id, &team_id, &term, 20).await?;
    Ok(Json(ChannelAutocompleteResponse {
        channels,
        users: vec![],
    }))
}

pub async fn get_team_channel_by_name(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((team_id, channel_name)): Path<(String, String)>,
) -> ApiResult<Json<mm::Channel>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    let channel: Channel =
        sqlx::query_as("SELECT * FROM channels WHERE team_id = $1 AND name = $2")
            .bind(team_id)
            .bind(&channel_name)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    if channel.channel_type == ChannelType::Private {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
        )
        .bind(channel.id)
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;
        if !is_member {
            return Err(AppError::Forbidden(
                "Not a member of this channel".to_string(),
            ));
        }
    }

    Ok(Json(channel.into()))
}

pub async fn get_team_channel_by_name_for_team_name(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((team_name, channel_name)): Path<(String, String)>,
) -> ApiResult<Json<mm::Channel>> {
    let team: Team = sqlx::query_as("SELECT * FROM teams WHERE name = $1")
        .bind(&team_name)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;
    get_team_channel_by_name(
        State(state),
        auth,
        Path((encode_mm_id(team.id), channel_name)),
    )
    .await
}
