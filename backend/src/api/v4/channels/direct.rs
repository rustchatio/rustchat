use axum::{body::Bytes, extract::State, Json};
use serde::Deserialize;
use uuid::Uuid;

use super::{mm, parse_mm_or_uuid, ApiResult, AppState, MmAuthUser};

const KEYCLOAK_GROUP_SOURCE: &str = "plugin_keycloak";

#[derive(Deserialize)]
pub struct DirectChannelRequest {
    pub user_ids: Vec<String>,
}

pub async fn create_direct_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    // Mattermost sends either a plain array ["id1", "id2"] or an object {"user_ids": ["id1", "id2"]}
    // Try parsing as plain array first, then fall back to object format
    let user_ids: Vec<String> = serde_json::from_slice::<Vec<String>>(&body).or_else(|_| {
        super::utils::parse_body::<DirectChannelRequest>(&headers, &body, "Invalid user_ids")
            .map(|req| req.user_ids)
    })?;

    if user_ids.len() != 2 {
        return Err(crate::error::AppError::BadRequest(
            "Request body must contain exactly 2 user IDs".to_string(),
        ));
    }

    let ids: Vec<Uuid> = user_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    if ids.len() != 2 {
        return Err(crate::error::AppError::BadRequest(
            "Invalid user IDs provided".to_string(),
        ));
    }

    if !ids.contains(&auth.user_id) {
        return Err(crate::error::AppError::Forbidden(
            "Must include your user id".to_string(),
        ));
    }

    let other_id = if ids[0] == auth.user_id {
        ids[1]
    } else {
        ids[0]
    };

    let channel = create_direct_channel_internal(&state, auth.user_id, other_id).await?;
    Ok(Json(channel.into()))
}

pub async fn enforce_dm_acl_for_users(state: &AppState, user_ids: &[Uuid]) -> ApiResult<()> {
    if !state.config.messaging.dm_acl_enabled {
        return Ok(());
    }

    let mut unique_users = user_ids.to_vec();
    unique_users.sort_unstable();
    unique_users.dedup();

    if unique_users.len() < 2 {
        return Ok(());
    }

    let allowed: bool = if unique_users.len() == 2 {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM groups g
                JOIN group_dm_acl_flags gf ON gf.group_id = g.id AND gf.enabled = TRUE
                JOIN group_members gm1 ON gm1.group_id = g.id
                JOIN group_members gm2 ON gm2.group_id = g.id
                WHERE g.deleted_at IS NULL
                  AND g.source = $3
                  AND gm1.user_id = $1
                  AND gm2.user_id = $2
            )
            "#,
        )
        .bind(unique_users[0])
        .bind(unique_users[1])
        .bind(KEYCLOAK_GROUP_SOURCE)
        .fetch_one(&state.db)
        .await?
    } else {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT gm.group_id
                FROM group_members gm
                JOIN groups g ON g.id = gm.group_id
                JOIN group_dm_acl_flags gf ON gf.group_id = g.id AND gf.enabled = TRUE
                WHERE g.deleted_at IS NULL
                  AND g.source = $1
                  AND gm.user_id = ANY($2)
                GROUP BY gm.group_id
                HAVING COUNT(DISTINCT gm.user_id) = $3
            )
            "#,
        )
        .bind(KEYCLOAK_GROUP_SOURCE)
        .bind(&unique_users)
        .bind(unique_users.len() as i64)
        .fetch_one(&state.db)
        .await?
    };

    if !allowed {
        return Err(crate::error::AppError::Forbidden(
            "Direct and group messaging is restricted by group policy".to_string(),
        ));
    }

    Ok(())
}

pub async fn create_direct_channel_internal(
    state: &AppState,
    creator_id: Uuid,
    other_id: Uuid,
) -> ApiResult<crate::models::channel::Channel> {
    enforce_dm_acl_for_users(state, &[creator_id, other_id]).await?;

    let canonical_name = crate::models::canonical_direct_channel_name(creator_id, other_id);
    let legacy_name = crate::models::legacy_direct_channel_name(creator_id, other_id);
    let mut ids = vec![creator_id, other_id];
    ids.sort();

    let team_id: Uuid = sqlx::query_scalar(
        "SELECT team_id FROM team_members WHERE user_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(creator_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::BadRequest("User has no team".to_string()))?;

    let display_name: String = sqlx::query_scalar(
        "SELECT COALESCE(NULLIF(display_name, ''), username) FROM users WHERE id = $1",
    )
    .bind(other_id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or_else(|| "Direct Message".to_string());

    if let Some(channel) = sqlx::query_as::<_, crate::models::Channel>(
        r#"
        SELECT *
        FROM channels
        WHERE team_id = $1
          AND type = 'direct'::channel_type
          AND (name = $2 OR name = $3)
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(team_id)
    .bind(&canonical_name)
    .bind(&legacy_name)
    .fetch_optional(&state.db)
    .await?
    {
        for user_id in ids {
            sqlx::query(
                "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
            )
            .bind(channel.id)
            .bind(user_id)
            .execute(&state.db)
            .await?;
        }

        return Ok(channel);
    }

    let channel: crate::models::Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (team_id, type, name, display_name, purpose, header, creator_id)
        VALUES ($1, 'direct', $2, $3, '', '', $4)
        ON CONFLICT (team_id, name) DO UPDATE SET
            name = EXCLUDED.name,
            display_name = CASE
                WHEN channels.display_name IS NULL OR channels.display_name = '' THEN EXCLUDED.display_name
                ELSE channels.display_name
            END
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(&canonical_name)
    .bind(&display_name)
    .bind(creator_id)
    .fetch_one(&state.db)
    .await?;

    for user_id in ids {
        sqlx::query(
            "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
        )
        .bind(channel.id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    }

    Ok(channel)
}

/// POST /channels/group - Create group DM (3+ users)
pub async fn create_group_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    // Group DMs also use array format
    let input: DirectChannelRequest =
        super::utils::parse_body(&headers, &body, "Invalid user_ids")?;

    if input.user_ids.len() < 2 {
        return Err(crate::error::AppError::BadRequest(
            "user_ids must contain at least 2 users".to_string(),
        ));
    }

    let uuids: Vec<Uuid> = input
        .user_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    let channel = create_group_channel_internal(&state, auth.user_id, uuids).await?;
    Ok(Json(channel.into()))
}

pub async fn create_group_channel_internal(
    state: &AppState,
    creator_id: Uuid,
    user_ids: Vec<Uuid>,
) -> ApiResult<crate::models::channel::Channel> {
    let mut ids = user_ids;
    if !ids.contains(&creator_id) {
        ids.push(creator_id);
    }

    ids.sort();
    ids.dedup();
    enforce_dm_acl_for_users(state, &ids).await?;

    let name = format!(
        "gm_{}",
        ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("_")
    );

    let team_id: Uuid = sqlx::query_scalar(
        "SELECT team_id FROM team_members WHERE user_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(creator_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::BadRequest("User has no team".to_string()))?;

    // Generate display name from usernames
    let usernames: Vec<String> =
        sqlx::query_scalar("SELECT username FROM users WHERE id = ANY($1)")
            .bind(&ids)
            .fetch_all(&state.db)
            .await?;
    let display_name = usernames.join(", ");

    let channel: crate::models::Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (team_id, type, name, display_name, purpose, header, creator_id)
        VALUES ($1, 'group', $2, $3, '', '', $4)
        ON CONFLICT (team_id, name) DO UPDATE SET name = EXCLUDED.name
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(&name)
    .bind(&display_name)
    .bind(creator_id)
    .fetch_one(&state.db)
    .await?;

    for user_id in ids {
        sqlx::query(
            "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
        )
        .bind(channel.id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    }

    Ok(channel)
}
