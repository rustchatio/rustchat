use axum::{
    body::Bytes,
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::user_channels::resolve_team_id;
use super::utils::{UsersByIdsRequest, UsersByUsernamesRequest};
use super::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::User;

#[derive(Deserialize)]
pub struct AutocompleteQuery {
    pub in_team: Option<String>,
    pub in_channel: Option<String>,
    pub name: Option<String>,
    pub limit: Option<i64>,
}

pub async fn autocomplete_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<AutocompleteQuery>,
) -> ApiResult<Json<Vec<mm::User>>> {
    let limit = query.limit.unwrap_or(25).clamp(1, 200);
    let name = query.name.unwrap_or_default();
    let name_like = format!("%{}%", name);

    let mut users: Vec<User> = if let Some(channel_id) = query.in_channel {
        let channel_id = parse_mm_or_uuid(&channel_id)
            .ok_or_else(|| AppError::BadRequest("Invalid in_channel".to_string()))?;

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

        sqlx::query_as(
            r#"
            SELECT u.*
            FROM users u
            JOIN channel_members cm ON u.id = cm.user_id
            WHERE cm.channel_id = $1
              AND (u.username ILIKE $2 OR u.email ILIKE $2)
              AND u.is_active = true
            ORDER BY u.username ASC
            LIMIT $3
            "#,
        )
        .bind(channel_id)
        .bind(&name_like)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else if let Some(team_id) = query.in_team {
        let team_id = resolve_team_id(&state, &team_id).await?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(team_id)
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;

        if !is_member {
            return Err(AppError::Forbidden("Not a member of this team".to_string()));
        }

        sqlx::query_as(
            r#"
            SELECT u.*
            FROM users u
            JOIN team_members tm ON u.id = tm.user_id
            WHERE tm.team_id = $1
              AND (u.username ILIKE $2 OR u.email ILIKE $2)
              AND u.is_active = true
            ORDER BY u.username ASC
            LIMIT $3
            "#,
        )
        .bind(team_id)
        .bind(&name_like)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM users WHERE (username ILIKE $1 OR email ILIKE $1) AND is_active = true ORDER BY username ASC LIMIT $2",
        )
        .bind(&name_like)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    users.truncate(limit as usize);
    let mm_users: Vec<mm::User> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(mm_users))
}

#[derive(Deserialize)]
pub struct UserSearchRequest {
    pub term: Option<String>,
    pub team_id: Option<String>,
    #[serde(rename = "not_in_channel_id")]
    pub _not_in_channel_id: Option<String>,
    pub in_channel_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn search_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::User>>> {
    let input: UserSearchRequest = super::parse_body(&headers, &body, "Invalid search body")?;
    let term = input.term.unwrap_or_default();
    let like = format!("%{}%", term);
    let limit = input.limit.unwrap_or(100).clamp(1, 200) as i64;

    let users: Vec<User> = if let Some(channel_id) = input.in_channel_id {
        let channel_id = parse_mm_or_uuid(&channel_id)
            .ok_or_else(|| AppError::BadRequest("Invalid in_channel_id".to_string()))?;

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

        sqlx::query_as(
            r#"
            SELECT u.*
            FROM users u
            JOIN channel_members cm ON u.id = cm.user_id
            WHERE cm.channel_id = $1
              AND (u.username ILIKE $2 OR u.email ILIKE $2)
              AND u.is_active = true
            ORDER BY u.username ASC
            LIMIT $3
            "#,
        )
        .bind(channel_id)
        .bind(&like)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else if let Some(team_id) = input.team_id {
        let team_id = resolve_team_id(&state, &team_id).await?;

        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(team_id)
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;

        if !is_member {
            return Err(AppError::Forbidden("Not a member of this team".to_string()));
        }

        sqlx::query_as(
            r#"
            SELECT u.*
            FROM users u
            JOIN team_members tm ON u.id = tm.user_id
            WHERE tm.team_id = $1
              AND (u.username ILIKE $2 OR u.email ILIKE $2)
              AND u.is_active = true
            ORDER BY u.username ASC
            LIMIT $3
            "#,
        )
        .bind(team_id)
        .bind(&like)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM users WHERE (username ILIKE $1 OR email ILIKE $1) AND is_active = true ORDER BY username ASC LIMIT $2",
        )
        .bind(&like)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    let mm_users: Vec<mm::User> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(mm_users))
}

pub async fn get_users_by_ids(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    headers: HeaderMap,
    Query(_query): Query<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::User>>> {
    let ids = super::parse_body::<UsersByIdsRequest>(&headers, &body, "Invalid users/ids body")
        .map(|parsed| match parsed {
            UsersByIdsRequest::Ids(ids) => ids,
            UsersByIdsRequest::Wrapped { user_ids } => user_ids,
        })?;

    let uuids: Vec<Uuid> = ids.iter().filter_map(|id| parse_mm_or_uuid(id)).collect();

    if uuids.is_empty() {
        return Ok(Json(vec![]));
    }

    let users: Vec<User> = sqlx::query_as(
        "SELECT * FROM users WHERE id = ANY($1) AND is_active = true AND deleted_at IS NULL",
    )
    .bind(&uuids)
    .fetch_all(&state.db)
    .await?;

    let _ = super::status::clear_expired_custom_statuses_for_users(&state, &uuids).await?;

    let mm_users: Vec<mm::User> = users
        .into_iter()
        .map(|mut u| {
            u.clear_custom_status_if_expired();
            u.into()
        })
        .collect();
    Ok(Json(mm_users))
}

pub async fn get_users_by_usernames(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::User>>> {
    // Mattermost clients send a raw JSON array for this endpoint:
    // ["user1","user2"] (not an object wrapper). We also accept
    // {"usernames":[...]} for compatibility with custom clients.
    let usernames =
        super::parse_body::<UsersByUsernamesRequest>(&headers, &body, "Invalid usernames body")
            .map(|parsed| match parsed {
                UsersByUsernamesRequest::Usernames(usernames) => usernames,
                UsersByUsernamesRequest::Wrapped { usernames } => usernames,
            })?;
    if usernames.is_empty() {
        return Ok(Json(vec![]));
    }

    let users: Vec<User> = sqlx::query_as("SELECT * FROM users WHERE username = ANY($1)")
        .bind(&usernames)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(users.into_iter().map(|u| u.into()).collect()))
}

pub async fn get_user_by_email(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(email): axum::extract::Path<String>,
) -> ApiResult<Json<mm::User>> {
    use crate::auth::policy::permissions;
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "Missing permission to lookup users by email".to_string(),
        ));
    }
    let user: User = sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

#[derive(Deserialize)]
pub struct UsersQuery {
    pub in_channel: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<UsersQuery>,
) -> ApiResult<Json<Vec<mm::User>>> {
    let channel_id = match query.in_channel.as_deref() {
        Some(id) => parse_mm_or_uuid(id)
            .ok_or_else(|| AppError::BadRequest("Invalid in_channel".to_string()))?,
        None => return Ok(Json(vec![])),
    };

    let page = query.page.unwrap_or(0).max(0);
    let per_page = query.per_page.unwrap_or(60).clamp(1, 200);
    let offset = page * per_page;

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

    let users: Vec<User> = sqlx::query_as(
        r#"
        SELECT u.*
        FROM users u
        JOIN channel_members cm ON u.id = cm.user_id
        WHERE cm.channel_id = $1 AND u.is_active = true
        ORDER BY u.username ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(channel_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let mm_users: Vec<mm::User> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(mm_users))
}

pub async fn get_known_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<String>>> {
    let user_ids: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT cm2.user_id
        FROM channel_members cm
        JOIN channel_members cm2 ON cm.channel_id = cm2.channel_id
        WHERE cm.user_id = $1 AND cm2.user_id != $1
        "#,
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let ids = user_ids.into_iter().map(encode_mm_id).collect();
    Ok(Json(ids))
}
