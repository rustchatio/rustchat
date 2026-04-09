use axum::{extract::State, Json};
use std::collections::HashMap;

use super::{mm, parse_mm_or_uuid, permissions, ApiResult, AppState, MmAuthUser};
use crate::models::Channel;

pub async fn search_channels_compat(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<HashMap<String, String>>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let term = input.get("term").cloned().unwrap_or_default();
    let team_id = input.get("team_id").and_then(|s| parse_mm_or_uuid(s));

    // Restrict results to channels the caller is a member of, scoped to team if provided.
    // System admins see all channels without the membership restriction.
    let channels: Vec<Channel> = if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        sqlx::query_as(
            r#"
            SELECT * FROM channels
            WHERE name ILIKE $1
              AND ($2::uuid IS NULL OR team_id = $2)
            ORDER BY name
            LIMIT 100
            "#,
        )
        .bind(format!("%{}%", term))
        .bind(team_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON cm.channel_id = c.id AND cm.user_id = $2
            WHERE c.name ILIKE $1
              AND ($3::uuid IS NULL OR c.team_id = $3)
            ORDER BY c.name
            LIMIT 100
            "#,
        )
        .bind(format!("%{}%", term))
        .bind(auth.user_id)
        .bind(team_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(channels.into_iter().map(|c| c.into()).collect()))
}
