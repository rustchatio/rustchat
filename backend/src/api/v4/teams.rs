use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::extractors::MmAuthUser;
use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::channel::ChannelType;
use crate::models::{Channel, Team, TeamMember};
use crate::services::team_membership::apply_default_channel_membership_for_team_join;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/teams", get(get_teams))
        .route("/teams/{team_id}", get(get_team))
        .route("/teams/{team_id}/patch", put(patch_team))
        .route("/teams/{team_id}/privacy", put(update_team_privacy))
        .route("/teams/{team_id}/restore", post(restore_team))
        .route("/teams/name/{name}", get(get_team_by_name))
        .route("/teams/name/{name}/exists", get(team_name_exists))
        .route(
            "/teams/{team_id}/members",
            get(get_team_members).post(add_team_member),
        )
        .route("/teams/members/invite", post(add_team_member_by_invite))
        .route(
            "/teams/{team_id}/members/batch",
            post(add_team_members_batch),
        )
        .route(
            "/teams/{team_id}/members/{user_id}",
            get(get_team_member).delete(remove_team_member),
        )
        .route("/teams/{team_id}/members/ids", get(get_team_member_ids))
        .route("/teams/{team_id}/stats", get(get_team_stats))
        .route(
            "/teams/{team_id}/regenerate_invite_id",
            post(regenerate_team_invite_id),
        )
        .route(
            "/teams/{team_id}/members/{user_id}/roles",
            put(update_team_member_roles),
        )
        .route(
            "/teams/{team_id}/members/{user_id}/schemeRoles",
            put(update_team_member_scheme_roles),
        )
        .route("/teams/{team_id}/image", get(get_team_image))
        .route("/teams/{team_id}/members/me", get(get_team_member_me))
        .route("/teams/{team_id}/invite/email", post(invite_users_to_team))
        .route(
            "/teams/{team_id}/invite-guests/email",
            post(invite_guests_to_team),
        )
        .route("/teams/invites/email", post(invite_users_to_team_by_email))
        .route("/teams/{team_id}/import", post(import_team))
        .route("/teams/invite/{invite_id}", get(get_team_by_invite))
        .route(
            "/teams/{team_id}/scheme",
            get(get_team_scheme).put(update_team_scheme),
        )
        .route(
            "/teams/{team_id}/members_minus_group_members",
            get(get_team_members_minus_group_members),
        )
        .route("/teams/{team_id}/channels", get(get_team_channels))
        .route("/teams/{team_id}/channels/ids", get(get_team_channel_ids))
        .route(
            "/teams/{team_id}/channels/private",
            get(get_team_private_channels),
        )
        .route(
            "/teams/{team_id}/channels/deleted",
            get(get_team_deleted_channels),
        )
        .route(
            "/teams/{team_id}/channels/autocomplete",
            get(autocomplete_team_channels),
        )
        .route(
            "/teams/{team_id}/channels/search_autocomplete",
            get(search_autocomplete_team_channels),
        )
        .route(
            "/teams/{team_id}/channels/name/{channel_name}",
            get(get_team_channel_by_name),
        )
        .route(
            "/teams/name/{team_name}/channels/name/{channel_name}",
            get(get_team_channel_by_name_for_team_name),
        )
        .route("/teams/{team_id}/channels/search", post(search_channels))
        .route("/teams/search", post(search_teams))
        .route(
            "/teams/{team_id}/commands/autocomplete",
            get(autocomplete_team_commands),
        )
        .route("/teams/{team_id}/groups", get(get_team_groups))
        .route(
            "/teams/{team_id}/groups_by_channels",
            get(get_team_groups_by_channels),
        )
}

async fn get_teams(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Team>>> {
    // Return teams the user is member of
    let teams: Vec<Team> = sqlx::query_as(
        r#"
        SELECT t.* FROM teams t
        JOIN team_members tm ON t.id = tm.team_id
        WHERE tm.user_id = $1
        "#,
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let mm_teams: Vec<mm::Team> = teams.into_iter().map(|t| t.into()).collect();
    Ok(Json(mm_teams))
}

async fn get_team(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<mm::Team>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let team: Team = sqlx::query_as("SELECT * FROM teams WHERE id = $1")
        .bind(team_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(team.into()))
}

async fn patch_team(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Team>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid patch body")?;
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let team: Team = sqlx::query_as("SELECT * FROM teams WHERE id = $1")
        .bind(team_id)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(team.into()))
}

async fn update_team_privacy(
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid privacy body")?;
    Ok(status_ok())
}

async fn restore_team(
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    Ok(status_ok())
}

async fn get_team_by_name(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(name): Path<String>,
) -> ApiResult<Json<mm::Team>> {
    let team: Team = sqlx::query_as("SELECT * FROM teams WHERE name = $1")
        .bind(&name)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Team not found".to_string()))?;
    Ok(Json(team.into()))
}

async fn team_name_exists(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM teams WHERE name = $1)")
        .bind(&name)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"exists": exists})))
}

async fn get_team_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::TeamMember>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;

    // Join with users table to get user information including presence
    let rows: Vec<(Uuid, Uuid, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT
            tm.team_id,
            tm.user_id,
            tm.role,
            u.username,
            u.display_name,
            u.presence
        FROM team_members tm
        JOIN users u ON tm.user_id = u.id
        WHERE tm.team_id = $1
        ORDER BY u.username
        "#,
    )
    .bind(team_id)
    .fetch_all(&state.db)
    .await?;

    let members: Vec<mm::TeamMember> = rows
        .into_iter()
        .map(
            |(team_id, user_id, role, _username, _display_name, presence)| {
                map_team_member_with_presence(
                    TeamMember {
                        team_id,
                        user_id,
                        role,
                        created_at: chrono::Utc::now(),
                    },
                    presence,
                )
            },
        )
        .collect();

    Ok(Json(members))
}

#[derive(Deserialize)]
struct AddTeamMemberRequest {
    user_id: String,
    #[allow(dead_code)]
    roles: Option<String>,
}

async fn add_team_member(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::TeamMember>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let input: AddTeamMemberRequest = parse_body(&headers, &body, "Invalid member body")?;
    let user_id = parse_mm_or_uuid(&input.user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query(
        r#"
        INSERT INTO team_members (team_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (team_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(team_id)
    .bind(user_id)
    .bind("member")
    .execute(&state.db)
    .await?;
    if let Err(err) = apply_default_channel_membership_for_team_join(&state, team_id, user_id).await
    {
        tracing::warn!(
            team_id = %team_id,
            user_id = %user_id,
            error = %err,
            "Default channel auto-join failed after v4 add_team_member"
        );
    }

    Ok(Json(map_team_member(TeamMember {
        team_id,
        user_id,
        role: "member".to_string(),
        created_at: chrono::Utc::now(),
    })))
}

async fn add_team_member_by_invite(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<AddTeamMemberByInviteQuery>,
) -> ApiResult<(axum::http::StatusCode, Json<mm::TeamMember>)> {
    let token = query
        .token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let invite_id = query
        .invite_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let team_id = if let Some(token_value) = token {
        let mut tx = state.db.begin().await?;
        let token_row: TeamInviteTokenRow = sqlx::query_as(
            r#"
            SELECT team_id, expires_at, used_at
            FROM team_invite_tokens
            WHERE token = $1
            FOR UPDATE
            "#,
        )
        .bind(token_value)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("Invalid invitation token".to_string())
        })?;

        if token_row.used_at.is_some() || token_row.expires_at <= chrono::Utc::now() {
            return Err(crate::error::AppError::BadRequest(
                "Invalid or expired invitation token".to_string(),
            ));
        }

        let team: Team = sqlx::query_as("SELECT * FROM teams WHERE id = $1")
            .bind(token_row.team_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Team not found".to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO team_members (team_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (team_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            "#,
        )
        .bind(team.id)
        .bind(auth.user_id)
        .bind("member")
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE team_invite_tokens SET used_at = NOW() WHERE token = $1")
            .bind(token_value)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        team.id
    } else if let Some(invite_value) = invite_id {
        if auth.has_role("guest") {
            return Err(crate::error::AppError::Forbidden(
                "Guests cannot join teams through invite_id".to_string(),
            ));
        }

        let team: Team = sqlx::query_as("SELECT * FROM teams WHERE invite_id = $1")
            .bind(invite_value)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Team not found".to_string()))?;

        if !team.allow_open_invite {
            return Err(crate::error::AppError::Forbidden(
                "This team does not allow open joining".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO team_members (team_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (team_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            "#,
        )
        .bind(team.id)
        .bind(auth.user_id)
        .bind("member")
        .execute(&state.db)
        .await?;

        team.id
    } else {
        return Err(crate::error::AppError::BadRequest(
            "Missing invitation token or invite_id".to_string(),
        ));
    };

    if let Err(err) =
        apply_default_channel_membership_for_team_join(&state, team_id, auth.user_id).await
    {
        tracing::warn!(
            team_id = %team_id,
            user_id = %auth.user_id,
            error = %err,
            "Default channel auto-join failed after v4 add_team_member_by_invite"
        );
    }

    Ok((
        axum::http::StatusCode::CREATED,
        Json(map_team_member(TeamMember {
            team_id,
            user_id: auth.user_id,
            role: "member".to_string(),
            created_at: chrono::Utc::now(),
        })),
    ))
}

#[derive(Deserialize)]
struct AddTeamMemberByInviteQuery {
    token: Option<String>,
    invite_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TeamInviteTokenRow {
    team_id: Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
    used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
struct AddTeamMembersBatchRequest {
    user_ids: Vec<String>,
}

async fn add_team_members_batch(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::TeamMember>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let input: AddTeamMembersBatchRequest = parse_body(&headers, &body, "Invalid batch body")?;
    let mut members = Vec::new();
    for user_id in input.user_ids {
        let user_id = match parse_mm_or_uuid(&user_id) {
            Some(id) => id,
            None => continue,
        };
        sqlx::query(
            r#"
            INSERT INTO team_members (team_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (team_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind("member")
        .execute(&state.db)
        .await?;
        if let Err(err) =
            apply_default_channel_membership_for_team_join(&state, team_id, user_id).await
        {
            tracing::warn!(
                team_id = %team_id,
                user_id = %user_id,
                error = %err,
                "Default channel auto-join failed after v4 add_team_members_batch"
            );
        }
        members.push(map_team_member(TeamMember {
            team_id,
            user_id,
            role: "member".to_string(),
            created_at: chrono::Utc::now(),
        }));
    }
    Ok(Json(members))
}

async fn get_team_member(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((team_id, user_id)): Path<(String, String)>,
) -> ApiResult<Json<mm::TeamMember>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    let member: TeamMember =
        sqlx::query_as("SELECT * FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Team member not found".to_string()))?;

    Ok(Json(map_team_member(member)))
}

async fn remove_team_member(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path((team_id, user_id)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
        .bind(team_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(status_ok())
}

async fn get_team_member_ids(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    let ids: Vec<uuid::Uuid> =
        sqlx::query_scalar("SELECT user_id FROM team_members WHERE team_id = $1")
            .bind(team_id)
            .fetch_all(&state.db)
            .await?;
    Ok(Json(ids.into_iter().map(encode_mm_id).collect()))
}

async fn get_team_stats(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    let total_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM team_members WHERE team_id = $1")
            .bind(team_id)
            .fetch_one(&state.db)
            .await?;
    let active_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM team_members tm
        JOIN users u ON u.id = tm.user_id
        WHERE tm.team_id = $1 AND u.is_active = true
        "#,
    )
    .bind(team_id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(serde_json::json!({
        "team_id": encode_mm_id(team_id),
        "total_member_count": total_count,
        "active_member_count": active_count,
    })))
}

async fn regenerate_team_invite_id(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    let invite_id: String = sqlx::query_scalar(
        r#"
        UPDATE teams
        SET invite_id = replace(gen_random_uuid()::text, '-', '')
        WHERE id = $1
        RETURNING invite_id
        "#,
    )
    .bind(team_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Team not found".to_string()))?;

    Ok(Json(serde_json::json!({"invite_id": invite_id})))
}

#[derive(Deserialize)]
struct TeamMemberRolesRequest {
    roles: String,
}

async fn update_team_member_roles(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path((team_id, user_id)): Path<(String, String)>,
    Json(input): Json<TeamMemberRolesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;
    let role = if input.roles.contains("team_admin") {
        "admin"
    } else {
        "member"
    };
    sqlx::query("UPDATE team_members SET role = $1 WHERE team_id = $2 AND user_id = $3")
        .bind(role)
        .bind(team_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(status_ok())
}

#[derive(Deserialize)]
struct TeamMemberSchemeRolesRequest {
    #[allow(dead_code)]
    scheme_admin: Option<bool>,
    #[allow(dead_code)]
    scheme_user: Option<bool>,
    #[allow(dead_code)]
    scheme_guest: Option<bool>,
}

async fn update_team_member_scheme_roles(
    _auth: MmAuthUser,
    Path((team_id, user_id)): Path<(String, String)>,
    Json(_input): Json<TeamMemberSchemeRolesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let _user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;
    Ok(status_ok())
}

async fn get_team_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
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

async fn get_team_channel_ids(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
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

async fn get_team_private_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
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

async fn get_team_deleted_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(&state, team_id, auth.user_id).await?;
    let channels: Vec<Channel> =
        sqlx::query_as("SELECT * FROM channels WHERE team_id = $1 AND is_archived = true")
            .bind(team_id)
            .fetch_all(&state.db)
            .await?;
    Ok(Json(channels.into_iter().map(|c| c.into()).collect()))
}

#[derive(Deserialize)]
struct ChannelAutocompleteQuery {
    name: Option<String>,
    term: Option<String>,
}

#[derive(Serialize)]
struct ChannelAutocompleteResponse {
    channels: Vec<mm::Channel>,
    users: Vec<mm::User>,
}

async fn autocomplete_team_channels(
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

async fn search_autocomplete_team_channels(
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

async fn get_team_channel_by_name(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((team_id, channel_name)): Path<(String, String)>,
) -> ApiResult<Json<mm::Channel>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let channel: Channel =
        sqlx::query_as("SELECT * FROM channels WHERE team_id = $1 AND name = $2")
            .bind(team_id)
            .bind(&channel_name)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Channel not found".to_string()))?;

    if channel.channel_type == ChannelType::Private {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
        )
        .bind(channel.id)
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;
        if !is_member {
            return Err(crate::error::AppError::Forbidden(
                "Not a member of this channel".to_string(),
            ));
        }
    }

    Ok(Json(channel.into()))
}

async fn get_team_channel_by_name_for_team_name(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((team_name, channel_name)): Path<(String, String)>,
) -> ApiResult<Json<mm::Channel>> {
    let team: Team = sqlx::query_as("SELECT * FROM teams WHERE name = $1")
        .bind(&team_name)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Team not found".to_string()))?;
    get_team_channel_by_name(
        State(state),
        auth,
        Path((encode_mm_id(team.id), channel_name)),
    )
    .await
}

async fn get_team_member_me(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<mm::TeamMember>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let member: crate::models::TeamMember =
        sqlx::query_as("SELECT * FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this team".to_string())
            })?;

    Ok(Json(mm::TeamMember {
        team_id: encode_mm_id(member.team_id),
        user_id: encode_mm_id(member.user_id),
        roles: crate::mattermost_compat::mappers::map_team_role(&member.role),
        delete_at: 0,
        scheme_guest: false,
        scheme_user: true,
        scheme_admin: member.role == "admin" || member.role == "team_admin",
        presence: None,
    }))
}

async fn invite_users_to_team(
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid invite body")?;
    Ok(status_ok())
}

async fn invite_guests_to_team(
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid invite body")?;
    Ok(status_ok())
}

async fn invite_users_to_team_by_email(
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid invite body")?;
    Ok(status_ok())
}

async fn import_team(
    Path(team_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid import body")?;
    Ok(status_ok())
}

async fn get_team_by_invite(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(invite_id): Path<String>,
) -> ApiResult<Json<mm::Team>> {
    let team: Team = sqlx::query_as("SELECT * FROM teams WHERE invite_id = $1")
        .bind(invite_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Team not found".to_string()))?;
    Ok(Json(team.into()))
}

async fn get_team_scheme(Path(team_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    Ok(Json(serde_json::json!({
        "team_id": encode_mm_id(team_id),
        "scheme_id": "",
    })))
}

async fn update_team_scheme(
    Path(team_id): Path<String>,
    Json(_input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    Ok(status_ok())
}

async fn get_team_members_minus_group_members(
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    Ok(Json(vec![]))
}

async fn get_team_image(
    State(_state): State<AppState>,
    Path(team_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let _team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    const PNG_1X1: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0, 5, 0, 1,
        13, 10, 45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];

    Ok(([(axum::http::header::CONTENT_TYPE, "image/png")], PNG_1X1))
}

/// POST /teams/{team_id}/channels/search - Search channels in a team
#[derive(Deserialize)]
struct SearchChannelsRequest {
    term: String,
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
        serde_json::from_slice(body)
            .map_err(|_| crate::error::AppError::BadRequest(message.to_string()))
    } else {
        serde_json::from_slice(body)
            .or_else(|_| serde_urlencoded::from_bytes(body))
            .map_err(|_| crate::error::AppError::BadRequest(message.to_string()))
    }
}

fn status_ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "OK"}))
}

async fn ensure_team_member(
    state: &AppState,
    team_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> ApiResult<()> {
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
    )
    .bind(team_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    if !is_member {
        return Err(crate::error::AppError::Forbidden(
            "Not a member of this team".to_string(),
        ));
    }
    Ok(())
}

fn map_team_member(member: TeamMember) -> mm::TeamMember {
    mm::TeamMember {
        team_id: encode_mm_id(member.team_id),
        user_id: encode_mm_id(member.user_id),
        roles: crate::mattermost_compat::mappers::map_team_role(&member.role),
        delete_at: 0,
        scheme_guest: false,
        scheme_user: true,
        scheme_admin: member.role == "admin" || member.role == "team_admin",
        presence: None,
    }
}

fn map_team_member_with_presence(member: TeamMember, presence: Option<String>) -> mm::TeamMember {
    mm::TeamMember {
        team_id: encode_mm_id(member.team_id),
        user_id: encode_mm_id(member.user_id),
        roles: crate::mattermost_compat::mappers::map_team_role(&member.role),
        delete_at: 0,
        scheme_guest: false,
        scheme_user: true,
        scheme_admin: member.role == "admin" || member.role == "team_admin",
        presence: presence.filter(|p| !p.is_empty()),
    }
}

async fn search_team_channels(
    state: &AppState,
    user_id: uuid::Uuid,
    team_id: &str,
    term: &str,
    limit: i64,
) -> ApiResult<Vec<mm::Channel>> {
    let team_id = parse_mm_or_uuid(team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;
    ensure_team_member(state, team_id, user_id).await?;
    let search_term = format!("%{}%", term.to_lowercase());
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT DISTINCT c.* FROM channels c
        LEFT JOIN channel_members cm ON c.id = cm.channel_id AND cm.user_id = $2
        WHERE c.team_id = $1
          AND (LOWER(c.name) LIKE $3 OR LOWER(c.display_name) LIKE $3)
          AND (c.type = 'public' OR cm.user_id IS NOT NULL)
        ORDER BY COALESCE(c.display_name, c.name) ASC
        LIMIT $4
        "#,
    )
    .bind(team_id)
    .bind(user_id)
    .bind(&search_term)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(channels.into_iter().map(|c| c.into()).collect())
}

async fn search_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    let input: SearchChannelsRequest = parse_body(&headers, &body, "Invalid search request")?;
    let search_term = format!("%{}%", input.term.to_lowercase());

    // Search public channels and private channels the user is a member of
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT DISTINCT c.* FROM channels c
        LEFT JOIN channel_members cm ON c.id = cm.channel_id AND cm.user_id = $2
        WHERE c.team_id = $1
          AND c.deleted_at IS NULL
          AND (LOWER(c.name) LIKE $3 OR LOWER(c.display_name) LIKE $3)
          AND (c.type = 'public' OR cm.user_id IS NOT NULL)
        ORDER BY c.display_name ASC
        LIMIT 50
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .bind(&search_term)
    .fetch_all(&state.db)
    .await?;

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

async fn search_teams(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Json(input): Json<HashMap<String, String>>,
) -> ApiResult<Json<Vec<mm::Team>>> {
    let term = input.get("term").map(|s| s.as_str()).unwrap_or_default();

    let teams: Vec<Team> =
        sqlx::query_as("SELECT * FROM teams WHERE name ILIKE $1 OR display_name ILIKE $1")
            .bind(format!("%{}%", term))
            .fetch_all(&state.db)
            .await?;

    Ok(Json(teams.into_iter().map(|t| t.into()).collect()))
}

async fn autocomplete_team_commands(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_team_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn get_team_groups(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
    Query(query): Query<GroupAssociationQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;

    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        ensure_team_member(&state, team_id, auth.user_id).await?;
    }

    let search_term = query.q.clone().unwrap_or_default().to_ascii_lowercase();
    let filter_allow_reference = should_filter_allow_reference(&auth, &query);
    let (paginate, offset, per_page) = pagination_from_group_query(&query);

    let rows: Vec<TeamGroupRow> = sqlx::query_as(
        r#"
        SELECT
            g.id,
            g.name,
            g.display_name,
            g.description,
            g.source,
            g.remote_id,
            g.allow_reference,
            g.created_at,
            g.updated_at,
            g.deleted_at,
            gs.scheme_admin,
            EXISTS(
                SELECT 1
                FROM group_syncables gs2
                WHERE gs2.group_id = g.id
                  AND gs2.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = g.id
            ) AS member_count
        FROM groups g
        JOIN group_syncables gs
          ON gs.group_id = g.id
         AND gs.syncable_type = 'team'
         AND gs.syncable_id = $1
         AND gs.delete_at IS NULL
        WHERE g.deleted_at IS NULL
          AND ($2 = FALSE OR g.allow_reference = TRUE)
          AND (
                $3 = ''
                OR LOWER(COALESCE(g.name, '')) LIKE $4
                OR LOWER(g.display_name) LIKE $4
          )
        ORDER BY g.display_name ASC
        "#,
    )
    .bind(team_id)
    .bind(filter_allow_reference)
    .bind(&search_term)
    .bind(format!("%{}%", search_term))
    .fetch_all(&state.db)
    .await?;

    let total_group_count = rows.len();
    let paged_rows = if paginate {
        rows.into_iter()
            .skip(offset)
            .take(per_page)
            .collect::<Vec<_>>()
    } else {
        rows
    };

    Ok(Json(serde_json::json!({
        "groups": paged_rows.iter().map(team_group_json).collect::<Vec<_>>(),
        "total_group_count": total_group_count
    })))
}

async fn get_team_groups_by_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
    Query(query): Query<GroupAssociationQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;

    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        ensure_team_member(&state, team_id, auth.user_id).await?;
    }

    let search_term = query.q.clone().unwrap_or_default().to_ascii_lowercase();
    let filter_allow_reference = should_filter_allow_reference(&auth, &query);
    let (paginate, offset, per_page) = pagination_from_group_query(&query);

    let rows: Vec<ChannelGroupByTeamRow> = sqlx::query_as(
        r#"
        SELECT
            c.id AS channel_id,
            g.id AS group_id,
            g.name,
            g.display_name,
            g.description,
            g.source,
            g.remote_id,
            g.allow_reference,
            g.created_at,
            g.updated_at,
            g.deleted_at,
            gs.scheme_admin,
            EXISTS(
                SELECT 1
                FROM group_syncables gs2
                WHERE gs2.group_id = g.id
                  AND gs2.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = g.id
            ) AS member_count
        FROM channels c
        JOIN group_syncables gs
          ON gs.syncable_type = 'channel'
         AND gs.syncable_id = c.id
         AND gs.delete_at IS NULL
        JOIN groups g ON g.id = gs.group_id
        WHERE c.team_id = $1
          AND g.deleted_at IS NULL
          AND ($2 = FALSE OR g.allow_reference = TRUE)
          AND (
                $3 = ''
                OR LOWER(COALESCE(g.name, '')) LIKE $4
                OR LOWER(g.display_name) LIKE $4
          )
        ORDER BY c.id, g.display_name ASC
        "#,
    )
    .bind(team_id)
    .bind(filter_allow_reference)
    .bind(&search_term)
    .bind(format!("%{}%", search_term))
    .fetch_all(&state.db)
    .await?;

    let mut grouped: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for row in rows {
        grouped
            .entry(encode_mm_id(row.channel_id))
            .or_default()
            .push(team_group_json_from_parts(
                row.group_id,
                row.name,
                row.display_name,
                row.description,
                row.source,
                row.remote_id,
                row.allow_reference,
                row.created_at,
                row.updated_at,
                row.deleted_at,
                row.has_syncables,
                row.member_count,
                row.scheme_admin,
            ));
    }

    if paginate {
        for groups in grouped.values_mut() {
            let paged = groups
                .iter()
                .skip(offset)
                .take(per_page)
                .cloned()
                .collect::<Vec<_>>();
            *groups = paged;
        }
    }

    Ok(Json(serde_json::json!({
        "groups": grouped
    })))
}

#[derive(Debug, Deserialize, Default)]
struct GroupAssociationQuery {
    q: Option<String>,
    include_member_count: Option<bool>,
    filter_allow_reference: Option<bool>,
    page: Option<i64>,
    per_page: Option<i64>,
    paginate: Option<bool>,
}

fn should_filter_allow_reference(auth: &MmAuthUser, query: &GroupAssociationQuery) -> bool {
    let has_system_group_read = auth.has_permission(&permissions::SYSTEM_MANAGE)
        || auth.has_permission(&permissions::ADMIN_FULL);

    query.filter_allow_reference.unwrap_or(false) || !has_system_group_read
}

fn pagination_from_group_query(query: &GroupAssociationQuery) -> (bool, usize, usize) {
    let _ = query.include_member_count.unwrap_or(false);
    let paginate = query.paginate.unwrap_or(true);
    let page = query.page.unwrap_or(0).max(0) as usize;
    let per_page = query.per_page.unwrap_or(60).clamp(1, 200) as usize;
    let offset = page.saturating_mul(per_page);
    (paginate, offset, per_page)
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct TeamGroupRow {
    id: Uuid,
    name: Option<String>,
    display_name: String,
    description: String,
    source: String,
    remote_id: Option<String>,
    allow_reference: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    scheme_admin: bool,
    has_syncables: bool,
    member_count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ChannelGroupByTeamRow {
    channel_id: Uuid,
    group_id: Uuid,
    name: Option<String>,
    display_name: String,
    description: String,
    source: String,
    remote_id: Option<String>,
    allow_reference: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    scheme_admin: bool,
    has_syncables: bool,
    member_count: i64,
}

fn team_group_json(row: &TeamGroupRow) -> serde_json::Value {
    team_group_json_from_parts(
        row.id,
        row.name.clone(),
        row.display_name.clone(),
        row.description.clone(),
        row.source.clone(),
        row.remote_id.clone(),
        row.allow_reference,
        row.created_at,
        row.updated_at,
        row.deleted_at,
        row.has_syncables,
        row.member_count,
        row.scheme_admin,
    )
}

#[allow(clippy::too_many_arguments)]
fn team_group_json_from_parts(
    id: Uuid,
    name: Option<String>,
    display_name: String,
    description: String,
    source: String,
    remote_id: Option<String>,
    allow_reference: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    has_syncables: bool,
    member_count: i64,
    scheme_admin: bool,
) -> serde_json::Value {
    serde_json::json!({
        "id": encode_mm_id(id),
        "name": name,
        "display_name": display_name,
        "description": description,
        "source": source,
        "remote_id": remote_id,
        "allow_reference": allow_reference,
        "create_at": created_at.timestamp_millis(),
        "update_at": updated_at.timestamp_millis(),
        "delete_at": deleted_at.map(|t| t.timestamp_millis()).unwrap_or(0),
        "has_syncables": has_syncables,
        "member_count": member_count,
        "scheme_admin": scheme_admin,
    })
}
