use axum::{extract::State, Json};
use serde::Deserialize;
use uuid::Uuid;

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{id::parse_mm_or_uuid, models as mm};
use crate::models::channel::Channel;

/// Resolves a team identifier to a UUID.
/// First tries to parse as UUID/mm-id, then falls back to looking up by team name.
pub async fn resolve_team_id(state: &AppState, team_id_str: &str) -> ApiResult<Uuid> {
    // First try to parse as UUID or Mattermost ID
    if let Some(team_id) = parse_mm_or_uuid(team_id_str) {
        return Ok(team_id);
    }

    // Fall back to looking up by team name
    let team: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM teams WHERE name = $1")
        .bind(team_id_str)
        .fetch_optional(&state.db)
        .await?;

    match team {
        Some((id,)) => Ok(id),
        None => Err(AppError::NotFound("Team not found".to_string())),
    }
}

pub async fn hydrate_direct_channel_display_name(
    state: &AppState,
    viewer_id: Uuid,
    channel: &mut Channel,
) -> ApiResult<()> {
    // For Direct channels, ALWAYS compute display_name from the other participant
    // This ensures each user sees the other person's name, not their own
    if channel.channel_type != crate::models::channel::ChannelType::Direct {
        return Ok(());
    }

    let display_name: Option<String> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(NULLIF(u.display_name, ''), u.username)
        FROM channel_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.channel_id = $1
          AND cm.user_id <> $2
        ORDER BY u.username ASC
        LIMIT 1
        "#,
    )
    .bind(channel.id)
    .bind(viewer_id)
    .fetch_optional(&state.db)
    .await?;

    channel.display_name = display_name.or_else(|| Some("Direct Message".to_string()));
    Ok(())
}

#[derive(Deserialize)]
pub struct MyTeamChannelsQuery {
    #[serde(default)]
    pub last_delete_at: i64,
    #[serde(default)]
    pub include_deleted: bool,
}

pub async fn my_team_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(team_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<MyTeamChannelsQuery>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = resolve_team_id(&state, &team_id).await?;

    tracing::debug!(
        user_id = %auth.user_id,
        team_id = %team_id,
        "Fetching channels for user"
    );

    // Determine the filter condition for deleted channels based on query parameters:
    // 1. If include_deleted is true: return all channels (including deleted ones)
    // 2. If last_delete_at > 0: return non-deleted channels AND channels deleted since last_delete_at
    //    This allows mobile clients to sync deleted channels for cache invalidation
    // 3. Default: only return non-deleted channels
    let mut channels: Vec<Channel> = if query.include_deleted {
        // Return all channels including deleted ones
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.team_id = $1 AND cm.user_id = $2
            "#,
        )
        .bind(team_id)
        .bind(auth.user_id)
        .fetch_all(&state.db)
        .await?
    } else if query.last_delete_at > 0 {
        let ts = chrono::DateTime::from_timestamp_millis(query.last_delete_at).unwrap_or_default();
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.team_id = $1 AND cm.user_id = $2
              AND (c.deleted_at IS NULL OR c.deleted_at >= $3)
            "#,
        )
        .bind(team_id)
        .bind(auth.user_id)
        .bind(ts)
        .fetch_all(&state.db)
        .await?
    } else {
        // Default: only return non-deleted channels
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.team_id = $1 AND cm.user_id = $2 AND c.deleted_at IS NULL
            "#,
        )
        .bind(team_id)
        .bind(auth.user_id)
        .fetch_all(&state.db)
        .await?
    };

    tracing::debug!(
        user_id = %auth.user_id,
        team_id = %team_id,
        channel_count = channels.len(),
        "Found channels for user"
    );

    for channel in &mut channels {
        hydrate_direct_channel_display_name(&state, auth.user_id, channel).await?;
    }

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

pub async fn get_team_channels_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path((user_id, team_id)): axum::extract::Path<(String, String)>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let user_id = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    let team_id = resolve_team_id(&state, &team_id).await?;
    let mut channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.* FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE c.team_id = $1 AND cm.user_id = $2
        "#,
    )
    .bind(team_id)
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    for channel in &mut channels {
        hydrate_direct_channel_display_name(&state, user_id, channel).await?;
    }

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

#[derive(Deserialize)]
pub struct MyChannelsQuery {
    #[serde(default)]
    pub since: i64,
}

pub async fn my_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Query(query): axum::extract::Query<MyChannelsQuery>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let mut channels: Vec<Channel> = if query.since > 0 {
        let ts = chrono::DateTime::from_timestamp_millis(query.since).unwrap_or_default();
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm.user_id = $1 AND c.updated_at >= $2
            "#,
        )
        .bind(auth.user_id)
        .bind(ts)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm.user_id = $1
            "#,
        )
        .bind(auth.user_id)
        .fetch_all(&state.db)
        .await?
    };

    for channel in &mut channels {
        hydrate_direct_channel_display_name(&state, auth.user_id, channel).await?;
    }

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

pub async fn get_channels_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let user_id = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    let mut channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.* FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE cm.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    for channel in &mut channels {
        hydrate_direct_channel_display_name(&state, user_id, channel).await?;
    }

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

#[derive(Deserialize)]
pub struct NotMembersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn my_team_channels_not_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(team_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<NotMembersQuery>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = resolve_team_id(&state, &team_id).await?;

    let page = query.page.unwrap_or(0).max(0);
    let per_page = query.per_page.unwrap_or(60).clamp(1, 200);
    let offset = page * per_page;

    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.*
        FROM channels c
        WHERE c.team_id = $1
          AND c.is_archived = false
          AND c.type IN ('public', 'private')
          AND NOT EXISTS (
              SELECT 1 FROM channel_members cm
              WHERE cm.channel_id = c.id AND cm.user_id = $2
          )
        ORDER BY COALESCE(c.display_name, c.name) ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}
