use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use uuid::Uuid;

use super::extractors::MmAuthUser;
use crate::api::AppState;
use crate::auth::{create_token, hash_password, verify_password};
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::{channel::Channel, channel::ChannelMember, Team, TeamMember, User};

mod preferences;
mod sidebar_categories;

use preferences::{
    delete_preferences_for_user, get_preference_by_category_and_name, get_preferences,
    get_preferences_by_category, get_preferences_for_user, update_preferences,
    update_preferences_for_user,
};
use sidebar_categories::{
    create_category, get_categories, get_my_categories, update_categories, update_category_order,
};
pub(crate) use sidebar_categories::{
    create_category_internal, get_categories_internal, resolve_user_id, update_categories_internal,
    update_category_order_internal, CreateCategoryRequest, UpdateCategoriesRequest,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/login", post(login))
        .route("/users/login/type", post(login_type))
        .route("/users/login/cws", post(login_cws))
        .route(
            "/users/login/sso/code-exchange",
            post(login_sso_code_exchange),
        )
        .route("/users/login/switch", post(login_switch))
        .route("/users/me", get(me))
        .route("/users/me/teams", get(my_teams))
        .route("/users/me/teams/members", get(my_team_members))
        .route("/users/me/channels/categories", get(get_my_categories))
        .route("/users/me/teams/{team_id}/channels", get(my_team_channels))
        .route("/users/me/channels", get(my_channels))
        .route("/users/{user_id}/teams", get(get_teams_for_user))
        .route(
            "/users/{user_id}/teams/members",
            get(get_team_members_for_user),
        )
        .route(
            "/users/{user_id}/teams/{team_id}/channels",
            get(get_team_channels_for_user),
        )
        .route("/users/{user_id}/channels", get(get_channels_for_user))
        .route(
            "/users/me/teams/{team_id}/channels/not_members",
            get(my_team_channels_not_members),
        )
        .route("/users", get(list_users))
        .route("/users/{user_id}", get(get_user_by_id))
        .route("/users/username/{username}", get(get_user_by_username))
        .route(
            "/users/me/teams/{team_id}/channels/members",
            get(my_team_channel_members),
        )
        .route("/users/me/teams/unread", get(my_teams_unread))
        .route("/users/{user_id}/teams/unread", get(get_user_teams_unread))
        .route(
            "/users/{user_id}/teams/{team_id}/unread",
            get(get_user_team_unread),
        )
        .route(
            "/users/sessions/device",
            post(attach_device).put(attach_device).delete(detach_device),
        )
        .route(
            "/users/me/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route(
            "/users/{user_id}/preferences",
            get(get_preferences_for_user).put(update_preferences_for_user),
        )
        .route(
            "/users/{user_id}/preferences/delete",
            post(delete_preferences_for_user),
        )
        .route(
            "/users/{user_id}/preferences/{category}",
            get(get_preferences_by_category),
        )
        .route(
            "/users/{user_id}/preferences/{category}/name/{preference_name}",
            get(get_preference_by_category_and_name),
        )
        .route("/users/status/ids", post(get_statuses_by_ids))
        .route("/users/ids", post(get_users_by_ids))
        .route(
            "/users/{user_id}/status",
            get(get_status).put(update_status),
        )
        .route("/users/me/status", get(get_my_status).put(update_status))
        .route(
            "/users/{user_id}/channels/{channel_id}/typing",
            post(user_typing),
        )
        .route("/users/me/patch", put(patch_me))
        .route(
            "/users/{user_id}/image",
            get(get_user_image).post(upload_user_image),
        )
        .route(
            "/users/notifications",
            get(get_notifications).put(update_notifications),
        )
        .route("/users/me/sessions", get(get_sessions))
        .route("/users/logout", get(logout).post(logout))
        .route("/users/autocomplete", get(autocomplete_users))
        .route("/users/search", post(search_users))
        .route("/users/known", get(get_known_users))
        .route("/users/stats", get(get_user_stats))
        .route("/users/stats/filtered", post(get_user_stats_filtered))
        .route("/users/group_channels", get(get_user_group_channels).post(get_profiles_in_group_channels))
        .route(
            "/users/{user_id}/oauth/apps/authorized",
            get(get_authorized_oauth_apps),
        )
        .route("/users/usernames", post(get_users_by_usernames))
        .route("/users/email/{email}", get(get_user_by_email))
        .route("/users/{user_id}/patch", put(patch_user))
        .route("/users/{user_id}/roles", put(update_user_roles))
        .route("/users/{user_id}/active", put(update_user_active))
        .route(
            "/users/{user_id}/image/default",
            get(get_user_image_default),
        )
        .route("/users/password/reset", post(reset_password))
        .route("/users/password/reset/send", post(send_password_reset))
        .route("/users/mfa", post(check_user_mfa))
        .route("/users/{user_id}/mfa", put(update_user_mfa))
        .route("/users/{user_id}/mfa/generate", post(generate_mfa_secret))
        .route("/users/{user_id}/demote", post(demote_user))
        .route("/users/{user_id}/promote", post(promote_user))
        .route("/users/{user_id}/convert_to_bot", post(convert_user_to_bot))
        .route("/users/{user_id}/password", put(update_user_password))
        .route("/users/{user_id}/sessions", get(get_user_sessions))
        .route(
            "/users/{user_id}/sessions/revoke",
            post(revoke_user_session),
        )
        .route(
            "/users/{user_id}/sessions/revoke/all",
            post(revoke_user_sessions),
        )
        .route("/users/sessions/revoke/all", post(revoke_all_sessions))
        .route("/users/{user_id}/audits", get(get_user_audits))
        .route(
            "/users/{user_id}/email/verify/member",
            post(verify_member_email),
        )
        .route("/users/email/verify", post(verify_email))
        .route("/users/email/verify/send", post(send_email_verification))
        .route("/users/{user_id}/tokens", get(get_user_tokens))
        .route("/users/tokens", get(get_tokens))
        .route("/users/tokens/revoke", post(revoke_token))
        .route("/users/tokens/{token_id}", get(get_token))
        .route("/users/tokens/disable", post(disable_token))
        .route("/users/tokens/enable", post(enable_token))
        .route("/users/tokens/search", post(search_tokens))
        .route("/users/{user_id}/auth", put(update_user_auth))
        .route(
            "/users/{user_id}/terms_of_service",
            post(accept_terms_of_service),
        )
        .route("/users/{user_id}/typing", post(set_user_typing))
        .route("/users/{user_id}/uploads", get(get_user_uploads))
        .route(
            "/users/{user_id}/channel_members",
            get(get_user_channel_members),
        )
        .route("/users/migrate_auth/ldap", post(migrate_auth_ldap))
        .route("/users/migrate_auth/saml", post(migrate_auth_saml))
        .route("/users/invalid_emails", get(get_invalid_emails))
        .route(
            "/users/{user_id}/reset_failed_attempts",
            post(reset_failed_attempts),
        )
        .route(
            "/users/{user_id}/status/custom",
            put(update_custom_status).delete(clear_custom_status),
        )
        .route(
            "/users/{user_id}/status/custom/recent",
            get(get_recent_custom_status),
        )
        .route(
            "/users/{user_id}/status/custom/recent/delete",
            post(delete_recent_custom_status),
        )
        .route(
            "/users/{user_id}/sidebar/categories",
            get(get_categories)
                .post(create_category)
                .put(update_categories),
        )
        .route(
            "/users/{user_id}/sidebar/categories/order",
            put(update_category_order),
        )
        .route("/users/{user_id}/groups", get(get_user_groups))
}

#[derive(Deserialize)]
struct LoginRequest {
    login_id: Option<String>,
    #[serde(default)]
    email: Option<String>,
    password: String,
    #[allow(dead_code)]
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct LoginTypeRequest {
    #[allow(dead_code)]
    id: Option<String>,
    #[allow(dead_code)]
    login_id: Option<String>,
    #[allow(dead_code)]
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct LoginSwitchRequest {
    #[allow(dead_code)]
    current_service: Option<String>,
    #[allow(dead_code)]
    new_service: Option<String>,
    #[allow(dead_code)]
    email: Option<String>,
    #[allow(dead_code)]
    password: Option<String>,
    #[allow(dead_code)]
    mfa_code: Option<String>,
    #[allow(dead_code)]
    ldap_id: Option<String>,
}

#[derive(Deserialize)]
struct LoginCwsRequest {
    #[allow(dead_code)]
    login_id: Option<String>,
    #[allow(dead_code)]
    cws_token: Option<String>,
}

#[derive(Deserialize)]
struct LoginSsoCodeExchangeRequest {
    #[allow(dead_code)]
    login_code: Option<String>,
    #[allow(dead_code)]
    code_verifier: Option<String>,
    #[allow(dead_code)]
    state: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let input = parse_login_request(&headers, &body)?;
    let login_id = input
        .login_id
        .or(input.email)
        .ok_or_else(|| AppError::BadRequest("Missing login_id".to_string()))?;

    let user: Option<User> = sqlx::query_as(
        "SELECT * FROM users WHERE (email = $1 OR username = $1) AND is_active = true",
    )
    .bind(&login_id)
    .fetch_optional(&state.db)
    .await?;

    let user =
        user.ok_or_else(|| AppError::Unauthorized("Invalid login credentials".to_string()))?;

    if !verify_password(&input.password, &user.password_hash)? {
        return Err(AppError::Unauthorized(
            "Invalid login credentials".to_string(),
        ));
    }

    // Update last login
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    // Generate token
    let token = create_token(
        user.id,
        &user.email,
        &user.role,
        user.org_id,
        &state.jwt_secret,
        state.jwt_expiry_hours,
    )?;

    let mm_user: mm::User = user.into();

    let mut headers = HeaderMap::new();
    headers.insert("Token", HeaderValue::from_str(&token).unwrap());
    headers.insert("token", HeaderValue::from_str(&token).unwrap());
    headers.insert(
        axum::http::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {}", token)).unwrap(),
    );
    let max_age = state.jwt_expiry_hours.saturating_mul(3600);
    headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "MMAUTHTOKEN={}; Path=/; Max-Age={}; HttpOnly",
            token, max_age
        ))
        .unwrap(),
    );

    Ok((headers, Json(mm_user)))
}

async fn login_type(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _input: LoginTypeRequest = parse_request_body(&headers, &body)?;

    Ok(Json(serde_json::json!({
        "auth_service": ""
    })))
}

async fn login_cws(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _input: LoginCwsRequest = parse_request_body(&headers, &body)?;

    Err(AppError::BadRequest(
        "CWS login is not supported".to_string(),
    ))
}

async fn login_sso_code_exchange(
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _input: LoginSsoCodeExchangeRequest = parse_request_body(&headers, &body)?;

    Err(AppError::BadRequest(
        "SSO code exchange is not supported".to_string(),
    ))
}

async fn login_switch(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _input: LoginSwitchRequest = parse_request_body(&headers, &body)?;

    Err(AppError::BadRequest(
        "Login method switching is not supported".to_string(),
    ))
}

fn parse_login_request(headers: &HeaderMap, body: &Bytes) -> ApiResult<LoginRequest> {
    parse_request_body(headers, body)
}

fn parse_request_body<T: DeserializeOwned>(headers: &HeaderMap, body: &Bytes) -> ApiResult<T> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.starts_with("application/json") {
        serde_json::from_slice(body)
            .map_err(|_| AppError::BadRequest("Invalid JSON body".to_string()))
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        serde_urlencoded::from_bytes(body)
            .map_err(|_| AppError::BadRequest("Invalid form body".to_string()))
    } else {
        serde_json::from_slice(body)
            .or_else(|_| serde_urlencoded::from_bytes(body))
            .map_err(|_| AppError::BadRequest("Unsupported request body".to_string()))
    }
}

async fn me(State(state): State<AppState>, auth: MmAuthUser) -> ApiResult<Json<mm::User>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(user.into()))
}

/// GET /users/{user_id} - Get user by ID
async fn get_user_by_id(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<mm::User>> {
    // Handle "me" as a special case
    let user_uuid = if user_id == "me" {
        return Err(AppError::BadRequest("Use /users/me endpoint".to_string()));
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?
    };

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_uuid)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

/// GET /users/username/{username} - Get user by username
async fn get_user_by_username(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(username): Path<String>,
) -> ApiResult<Json<mm::User>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

async fn my_teams(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Team>>> {
    let teams = fetch_user_teams(&state, auth.user_id).await?;

    if teams.is_empty() {
        return Ok(Json(vec![default_team()]));
    }

    let mm_teams: Vec<mm::Team> = teams.into_iter().map(|t| t.into()).collect();
    Ok(Json(mm_teams))
}

async fn get_teams_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Team>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let teams = fetch_user_teams(&state, user_id).await?;

    if teams.is_empty() {
        return Ok(Json(vec![default_team()]));
    }

    let mm_teams: Vec<mm::Team> = teams.into_iter().map(|t| t.into()).collect();
    Ok(Json(mm_teams))
}

async fn fetch_user_teams(state: &AppState, user_id: Uuid) -> ApiResult<Vec<Team>> {
    sqlx::query_as(
        r#"
        SELECT t.* FROM teams t
        JOIN team_members tm ON t.id = tm.team_id
        WHERE tm.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(Into::into)
}

fn default_team() -> mm::Team {
    let id = Uuid::new_v4();
    mm::Team {
        id: encode_mm_id(id),
        create_at: 0,
        update_at: 0,
        delete_at: 0,
        display_name: "RustChat".to_string(),
        name: "rustchat".to_string(),
        description: "".to_string(),
        email: "".to_string(),
        team_type: "O".to_string(),
        company_name: "".to_string(),
        allowed_domains: "".to_string(),
        invite_id: "".to_string(),
        allow_open_invite: false,
    }
}

async fn my_team_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::TeamMember>>> {
    let members: Vec<TeamMember> = sqlx::query_as("SELECT * FROM team_members WHERE user_id = $1")
        .bind(auth.user_id)
        .fetch_all(&state.db)
        .await?;

    let mm_members = members
        .into_iter()
        .map(|m| mm::TeamMember {
            team_id: encode_mm_id(m.team_id),
            user_id: encode_mm_id(m.user_id),
            roles: crate::mattermost_compat::mappers::map_team_role(&m.role),
            delete_at: 0,
            scheme_guest: false,
            scheme_user: true,
            scheme_admin: m.role == "admin" || m.role == "team_admin",
        })
        .collect();

    Ok(Json(mm_members))
}

async fn get_team_members_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<mm::TeamMember>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let members: Vec<TeamMember> = sqlx::query_as("SELECT * FROM team_members WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&state.db)
        .await?;

    let mm_members = members
        .into_iter()
        .map(|m| mm::TeamMember {
            team_id: encode_mm_id(m.team_id),
            user_id: encode_mm_id(m.user_id),
            roles: crate::mattermost_compat::mappers::map_team_role(&m.role),
            delete_at: 0,
            scheme_guest: false,
            scheme_user: true,
            scheme_admin: m.role == "admin" || m.role == "team_admin",
        })
        .collect();

    Ok(Json(mm_members))
}



async fn hydrate_direct_channel_display_name(
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

async fn my_team_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    let mut channels: Vec<Channel> = sqlx::query_as(
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

    for channel in &mut channels {
        hydrate_direct_channel_display_name(&state, auth.user_id, channel).await?;
    }

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

async fn get_team_channels_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((user_id, team_id)): Path<(String, String)>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
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

async fn my_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let mut channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.* FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE cm.user_id = $1
        "#,
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    for channel in &mut channels {
        hydrate_direct_channel_display_name(&state, auth.user_id, channel).await?;
    }

    let mm_channels: Vec<mm::Channel> = channels.into_iter().map(|c| c.into()).collect();
    Ok(Json(mm_channels))
}

async fn get_channels_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
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

async fn my_team_channel_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
) -> ApiResult<Json<Vec<mm::ChannelMember>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    let members: Vec<ChannelMember> = sqlx::query_as(
        r#"
        SELECT cm.*, c.name as username, c.display_name, NULL as avatar_url, NULL as presence
        FROM channel_members cm
        JOIN channels c ON cm.channel_id = c.id
        WHERE c.team_id = $1 AND cm.user_id = $2
        "#,
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let mm_members = members
        .into_iter()
        .map(|m| mm::ChannelMember {
            channel_id: encode_mm_id(m.channel_id),
            user_id: encode_mm_id(m.user_id),
            roles: crate::mattermost_compat::mappers::map_channel_role(&m.role),
            last_viewed_at: m.last_viewed_at.map(|t| t.timestamp_millis()).unwrap_or(0),
            msg_count: 0,
            mention_count: 0,
            notify_props: normalize_notify_props(m.notify_props),
            last_update_at: 0,
            scheme_guest: false,
            scheme_user: true,
            scheme_admin: m.role == "admin" || m.role == "team_admin" || m.role == "channel_admin",
        })
        .collect();

    Ok(Json(mm_members))
}

#[derive(Deserialize)]
struct NotMembersQuery {
    page: Option<i64>,
    per_page: Option<i64>,
}

async fn my_team_channels_not_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team_id): Path<String>,
    Query(query): Query<NotMembersQuery>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;

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

async fn my_teams_unread(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn get_user_teams_unread(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn get_user_team_unread(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path((user_id, team_id)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let _user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    let team_id = parse_mm_or_uuid(&team_id)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    Ok(Json(serde_json::json!({
        "team_id": encode_mm_id(team_id),
        "msg_count": 0,
        "mention_count": 0,
    })))
}

fn normalize_notify_props(value: serde_json::Value) -> serde_json::Value {
    if value.is_null() {
        return serde_json::json!({"desktop": "default", "mark_unread": "all"});
    }

    if let Some(obj) = value.as_object() {
        if obj.is_empty() {
            return serde_json::json!({"desktop": "default", "mark_unread": "all"});
        }
    }

    value
}

#[derive(Deserialize)]
struct AttachDeviceRequest {
    device_id: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    platform: Option<String>,
}

async fn attach_device(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    // Try to parse body, but accept empty/malformed requests gracefully
    let input: AttachDeviceRequest = match parse_body(&headers, &body, "Invalid device body") {
        Ok(v) => v,
        Err(_) => {
            // Return OK for malformed requests - mobile sends various formats
            return Ok(Json(serde_json::json!({"status": "OK"})));
        }
    };

    // Only insert if we have device_id
    if let Some(device_id) = input.device_id {
        let _ = sqlx::query(
            r#"
            INSERT INTO user_devices (user_id, device_id, token, platform)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, device_id)
            DO UPDATE SET token = $3, platform = $4, last_seen_at = NOW()
            "#,
        )
        .bind(auth.user_id)
        .bind(&device_id)
        .bind(input.token.as_deref())
        .bind(input.platform.unwrap_or_else(|| "unknown".to_string()))
        .execute(&state.db)
        .await;
    }

    Ok(Json(serde_json::json!({"status": "OK"})))
}

#[derive(Deserialize)]
struct DetachDeviceRequest {
    device_id: String,
}

async fn detach_device(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<DetachDeviceRequest>,
) -> ApiResult<impl IntoResponse> {
    sqlx::query("DELETE FROM user_devices WHERE user_id = $1 AND device_id = $2")
        .bind(auth.user_id)
        .bind(input.device_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn get_notifications() -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "email": "true",
        "push": "mention",
        "desktop": "all",
        "desktop_sound": "Bing",
        "mention_keys": "",
        "channel": "true",
        "first_name": "false",
        "push_status": "online",
        "comments": "never",
        "milestones": "none",
        "auto_responder_active": "false",
        "auto_responder_message": ""
    })))
}

async fn update_notifications(
    Json(_input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn get_sessions() -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn logout() -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

#[derive(Deserialize)]
struct AutocompleteQuery {
    in_team: Option<String>,
    in_channel: Option<String>,
    name: Option<String>,
    limit: Option<i64>,
}

async fn autocomplete_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<AutocompleteQuery>,
) -> ApiResult<Json<Vec<mm::User>>> {
    let limit = query.limit.unwrap_or(25).clamp(1, 200) as i64;
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
        let team_id = parse_mm_or_uuid(&team_id)
            .ok_or_else(|| AppError::BadRequest("Invalid in_team".to_string()))?;

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
struct UserSearchRequest {
    term: Option<String>,
    team_id: Option<String>,
    #[serde(rename = "not_in_channel_id")]
    _not_in_channel_id: Option<String>,
    in_channel_id: Option<String>,
    limit: Option<i64>,
}

async fn search_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::User>>> {
    let input: UserSearchRequest = parse_body(&headers, &body, "Invalid search body")?;
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
        let team_id = parse_mm_or_uuid(&team_id)
            .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;

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

async fn get_statuses_by_ids(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::Status>>> {
    let ids: Vec<String> = parse_body(&headers, &body, "Invalid status ids body")?;
    let uuids: Vec<Uuid> = ids.iter().filter_map(|id| parse_mm_or_uuid(id)).collect();

    if uuids.is_empty() {
        return Ok(Json(vec![]));
    }

    let users: Vec<(Uuid, String, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT id, presence, last_login_at FROM users WHERE id = ANY($1)")
            .bind(&uuids)
            .fetch_all(&state.db)
            .await?;

    let statuses = users
        .into_iter()
        .map(|(id, presence, last_login)| mm::Status {
            user_id: encode_mm_id(id),
            status: if presence.is_empty() {
                "offline".to_string()
            } else {
                presence
            },
            manual: false,
            last_activity_at: last_login.map(|t| t.timestamp_millis()).unwrap_or(0),
        })
        .collect();

    Ok(Json(statuses))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UsersByIdsRequest {
    Ids(Vec<String>),
    Wrapped { user_ids: Vec<String> },
}

async fn get_users_by_ids(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(_query): Query<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::User>>> {
    let ids = parse_body::<UsersByIdsRequest>(&headers, &body, "Invalid users/ids body").map(
        |parsed| match parsed {
            UsersByIdsRequest::Ids(ids) => ids,
            UsersByIdsRequest::Wrapped { user_ids } => user_ids,
        },
    )?;

    let uuids: Vec<Uuid> = ids.iter().filter_map(|id| parse_mm_or_uuid(id)).collect();

    if uuids.is_empty() {
        return Ok(Json(vec![]));
    }

    let users: Vec<User> =
        sqlx::query_as("SELECT * FROM users WHERE id = ANY($1) AND is_active = true")
            .bind(&uuids)
            .fetch_all(&state.db)
            .await?;

    let mm_users: Vec<mm::User> = users.into_iter().map(|u| u.into()).collect();
    Ok(Json(mm_users))
}

fn parse_body<T: DeserializeOwned>(
    headers: &HeaderMap,
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

async fn get_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<mm::Status>> {
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;
    let (presence, last_login): (String, Option<DateTime<Utc>>) =
        sqlx::query_as("SELECT presence, last_login_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(mm::Status {
        user_id: encode_mm_id(user_id),
        status: if presence.is_empty() {
            "offline".to_string()
        } else {
            presence
        },
        manual: false,
        last_activity_at: last_login.map(|t| t.timestamp_millis()).unwrap_or(0),
    }))
}

async fn get_my_status(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<mm::Status>> {
    let (presence, last_login): (String, Option<DateTime<Utc>>) =
        sqlx::query_as("SELECT presence, last_login_at FROM users WHERE id = $1")
            .bind(auth.user_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(mm::Status {
        user_id: encode_mm_id(auth.user_id),
        status: if presence.is_empty() {
            "offline".to_string()
        } else {
            presence
        },
        manual: false,
        last_activity_at: last_login.map(|t| t.timestamp_millis()).unwrap_or(0),
    }))
}

#[derive(Deserialize)]
struct UpdateStatusRequest {
    user_id: String,
    status: String,
}

#[derive(Deserialize)]
struct PatchMeRequest {
    nickname: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    position: Option<String>,
}

async fn update_status(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Status>> {
    let input: UpdateStatusRequest = parse_body(&headers, &body, "Invalid status update request")?;

    let input_user_id = parse_mm_or_uuid(&input.user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;
    if input_user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot update other user's status".to_string(),
        ));
    }

    sqlx::query("UPDATE users SET presence = $1 WHERE id = $2")
        .bind(&input.status)
        .bind(auth.user_id)
        .execute(&state.db)
        .await?;

    let status = mm::Status {
        user_id: encode_mm_id(auth.user_id),
        status: input.status.clone(),
        manual: true,
        last_activity_at: Utc::now().timestamp_millis(),
    };

    // Broadcast status change
    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::UserUpdated, // Mapping to status_change in WS handler
        serde_json::json!({
             "user_id": auth.user_id,
             "status": input.status,
             "manual": true,
             "last_activity_at": status.last_activity_at
        }),
        None,
    );
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(status))
}

async fn patch_me(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::User>> {
    let input: PatchMeRequest = parse_body(&headers, &body, "Invalid patch body")?;
    
    // Update any provided fields
    sqlx::query(
        r#"
        UPDATE users 
        SET first_name = COALESCE($1, first_name),
            last_name = COALESCE($2, last_name),
            nickname = COALESCE($3, nickname),
            position = COALESCE($4, position),
            updated_at = NOW()
        WHERE id = $5
        "#,
    )
    .bind(&input.first_name)
    .bind(&input.last_name)
    .bind(&input.nickname)
    .bind(&input.position)
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    // Fetch updated user
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(user.into()))
}

#[derive(Deserialize)]
struct UsersQuery {
    in_channel: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}

async fn list_users(
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

/// GET /users/{user_id}/image - Get user profile image (requires auth)
async fn get_user_image(
    State(state): State<AppState>,
    _auth: MmAuthUser, // Require authentication - images are only accessible to logged-in users
    Path(user_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let user_uuid = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Try to fetch from S3
    let key = format!("avatars/{}.png", user_uuid);
    
    match state.s3_client.download(&key).await {
        Ok(data) => {
            // Detect content type from magic bytes
            let content_type = if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                "image/png"
            } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                "image/jpeg"
            } else if data.starts_with(b"GIF") {
                "image/gif"
            } else if data.len() > 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
                "image/webp"
            } else {
                "image/png"
            };
            
            Ok((
                [
                    (axum::http::header::CONTENT_TYPE, content_type),
                    (axum::http::header::CACHE_CONTROL, "private, max-age=86400"),
                ],
                data,
            ).into_response())
        }
        Err(_) => {
            // Return default 1x1 transparent PNG if no image uploaded
            const PNG_1X1: &[u8] = &[
                137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
                0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0, 5, 0, 1,
                13, 10, 45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
            ];
            
            Ok((
                [
                    (axum::http::header::CONTENT_TYPE, "image/png"),
                    (axum::http::header::CACHE_CONTROL, "private, max-age=300"),
                ],
                PNG_1X1.to_vec(),
            ).into_response())
        }
    }
}

/// POST /users/{user_id}/image - Upload user profile image
async fn upload_user_image(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    mut multipart: axum::extract::Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    let user_uuid = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;

    if user_uuid != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot update other user's image".to_string(),
        ));
    }

    // Process multipart upload
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        let filename = field.file_name().map(|s| s.to_string());
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        // Accept field named "image", "file", "picture", "avatar", or any field with:
        // - image content type
        // - a filename present (indicates it's a file upload)
        let is_image_field = name == "image"
            || name == "file"
            || name == "picture"
            || name == "avatar"
            || name.is_empty();
        let is_image_type = content_type.starts_with("image/");
        let has_filename = filename.is_some();

        if is_image_field && (is_image_type || has_filename) {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Read error: {}", e)))?
                .to_vec();

            if data.is_empty() {
                continue;
            }

            // Determine content type from data if not provided
            let final_content_type = if is_image_type {
                content_type.clone()
            } else {
                // Try to detect from magic bytes
                if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png".to_string()
                } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    "image/jpeg".to_string()
                } else if data.starts_with(b"GIF") {
                    "image/gif".to_string()
                } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
                    "image/webp".to_string()
                } else {
                    "image/png".to_string() // default to PNG
                }
            };

            // Upload to S3
            let key = format!("avatars/{}.png", user_uuid);
            state
                .s3_client
                .upload(&key, data, &final_content_type)
                .await?;

            // Update user avatar_url
            let avatar_url = format!("/api/v4/users/{}/image", encode_mm_id(user_uuid));
            sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
                .bind(&avatar_url)
                .bind(user_uuid)
                .execute(&state.db)
                .await?;

            return Ok(Json(serde_json::json!({"status": "OK"})));
        }
    }

    Err(AppError::BadRequest(
        "No image field found in upload".to_string(),
    ))
}

async fn user_typing(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((user_id, channel_id)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel ID".to_string()))?;
    if user_id != auth.user_id {
        return Err(AppError::Forbidden("Mismatch user_id".to_string()));
    }

    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::UserTyping,
        crate::realtime::TypingEvent {
            user_id: auth.user_id,
            display_name: "".to_string(), // Fetched by client usually
            thread_root_id: None,
        },
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: Some(channel_id),
        team_id: None,
        user_id: None,
        exclude_user_id: Some(auth.user_id),
    });

    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

fn status_ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "OK"}))
}

async fn get_known_users(
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

async fn get_user_stats(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"total_users_count": total})))
}

async fn get_user_stats_filtered(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    get_user_stats(State(state)).await
}

async fn get_user_group_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let channels: Vec<Channel> = sqlx::query_as(
        r#"
        SELECT c.* FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE cm.user_id = $1 AND c.type = 'group'
        "#,
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(channels.into_iter().map(|c| c.into()).collect()))
}

/// POST /users/group_channels - Get user profiles for multiple group/DM channels
/// Mobile client sends array of channel IDs and expects map of channel_id -> [user profiles]
async fn get_profiles_in_group_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    _headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<std::collections::HashMap<String, Vec<mm::User>>>> {
    // Parse channel IDs from body (sent as raw JSON array)
    let channel_ids: Vec<String> = serde_json::from_slice(&body)
        .map_err(|_| AppError::BadRequest("Expected array of channel IDs".to_string()))?;

    if channel_ids.is_empty() {
        return Ok(Json(std::collections::HashMap::new()));
    }

    // Convert to UUIDs
    let channel_uuids: Vec<Uuid> = channel_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    if channel_uuids.is_empty() {
        return Ok(Json(std::collections::HashMap::new()));
    }

    // Query users for each channel the requesting user is a member of
    let rows: Vec<(Uuid, Uuid, String, String, Option<String>, Option<String>, bool)> = sqlx::query_as(
        r#"
        SELECT 
            cm.channel_id,
            u.id,
            u.username,
            u.email,
            u.display_name,
            u.avatar_url,
            u.is_active
        FROM channel_members cm
        JOIN users u ON cm.user_id = u.id
        WHERE cm.channel_id = ANY($1)
          AND EXISTS (
              SELECT 1 FROM channel_members cm2 
              WHERE cm2.channel_id = cm.channel_id AND cm2.user_id = $2
          )
        ORDER BY cm.channel_id, u.username
        "#,
    )
    .bind(&channel_uuids)
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    // Group by channel_id
    let mut result: std::collections::HashMap<String, Vec<mm::User>> = std::collections::HashMap::new();
    for (channel_id, user_id, username, email, display_name, _avatar_url, _is_active) in rows {
        let mm_user = mm::User {
            id: encode_mm_id(user_id),
            username,
            email,
            nickname: display_name.clone().unwrap_or_default(),
            first_name: display_name.unwrap_or_default(),
            last_name: "".to_string(),
            email_verified: true,
            auth_service: "".to_string(),
            roles: "system_user".to_string(),
            locale: "en".to_string(),
            timezone: serde_json::json!({}),
            create_at: 0,
            update_at: 0,
            delete_at: 0,
            props: serde_json::json!({}),
            notify_props: serde_json::json!({}),
            last_password_update: 0,
            last_picture_update: 0,
            failed_attempts: 0,
            mfa_active: false,
        };
        result
            .entry(encode_mm_id(channel_id))
            .or_default()
            .push(mm_user);
    }


    Ok(Json(result))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UsersByUsernamesRequest {
    Usernames(Vec<String>),
    Wrapped { usernames: Vec<String> },
}

async fn get_users_by_usernames(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::User>>> {
    // Mattermost clients send a raw JSON array for this endpoint:
    // ["user1","user2"] (not an object wrapper). We also accept
    // {"usernames":[...]} for compatibility with custom clients.
    let usernames = parse_body::<UsersByUsernamesRequest>(&headers, &body, "Invalid usernames body")
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

async fn get_user_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> ApiResult<Json<mm::User>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

async fn patch_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::User>> {
    let _input: PatchMeRequest = parse_body(&headers, &body, "Invalid patch body")?;
    let user_id = resolve_user_id(&user_id, &auth)?;
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(user.into()))
}

#[derive(Deserialize)]
struct UserRolesRequest {
    roles: String,
}

async fn update_user_roles(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    Json(input): Json<UserRolesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if auth.role != "system_admin" && auth.role != "org_admin" {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    let role = if input.roles.contains("system_admin") {
        "system_admin"
    } else {
        "member"
    };

    sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(role)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(status_ok())
}

#[derive(Deserialize)]
struct UserActiveRequest {
    active: bool,
}

async fn update_user_active(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    Json(input): Json<UserActiveRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    if user_id != auth.user_id && auth.role != "system_admin" && auth.role != "org_admin" {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    sqlx::query("UPDATE users SET is_active = $1 WHERE id = $2")
        .bind(input.active)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(status_ok())
}

async fn get_user_image_default(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    get_user_image(State(state), auth, Path(user_id)).await
}

async fn reset_password(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid reset body")?;
    Ok(status_ok())
}

async fn send_password_reset(
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid reset body")?;
    Ok(status_ok())
}

#[derive(Deserialize)]
struct CheckMfaRequest {
    #[serde(rename = "login_id")]
    _login_id: String,
}

async fn check_user_mfa(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _input: CheckMfaRequest = parse_body(&headers, &body, "Invalid mfa body")?;
    Ok(Json(serde_json::json!({"mfa_required": false})))
}

#[derive(Deserialize)]
struct UpdateMfaRequest {
    #[serde(rename = "activate")]
    _activate: bool,
    #[allow(dead_code)]
    code: Option<String>,
}

async fn update_user_mfa(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    Json(_input): Json<UpdateMfaRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let _ = user_id;
    Ok(status_ok())
}

async fn generate_mfa_secret(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(Json(serde_json::json!({"secret": "", "qr_code": ""})))
}

async fn demote_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if auth.role != "system_admin" && auth.role != "org_admin" {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("UPDATE users SET role = 'member' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(status_ok())
}

async fn promote_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if auth.role != "system_admin" && auth.role != "org_admin" {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("UPDATE users SET role = 'system_admin' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(status_ok())
}

async fn convert_user_to_bot(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if auth.role != "system_admin" && auth.role != "org_admin" {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("UPDATE users SET is_bot = true WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(status_ok())
}

#[derive(Deserialize)]
struct UpdatePasswordRequest {
    current_password: Option<String>,
    new_password: String,
}

async fn update_user_password(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    Json(input): Json<UpdatePasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    if user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Cannot change another user's password".to_string(),
        ));
    }

    if let Some(current) = input.current_password.as_deref() {
        if !verify_password(current, &user.password_hash)? {
            return Err(AppError::BadRequest("Invalid current password".to_string()));
        }
    }

    let new_hash = hash_password(&input.new_password)?;
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(status_ok())
}

async fn get_user_sessions(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

async fn revoke_user_session(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid session body")?;
    Ok(status_ok())
}

async fn revoke_user_sessions(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(status_ok())
}

async fn revoke_all_sessions() -> ApiResult<Json<serde_json::Value>> {
    Ok(status_ok())
}

async fn get_user_audits(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

async fn verify_member_email() -> ApiResult<Json<serde_json::Value>> {
    Ok(status_ok())
}

async fn verify_email() -> ApiResult<Json<serde_json::Value>> {
    Ok(status_ok())
}

async fn send_email_verification() -> ApiResult<Json<serde_json::Value>> {
    Ok(status_ok())
}

async fn get_user_tokens(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

async fn get_tokens() -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn revoke_token(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid revoke body")?;
    Ok(status_ok())
}

async fn get_token(Path(_token_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({})))
}

async fn disable_token(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid disable body")?;
    Ok(status_ok())
}

async fn enable_token(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid enable body")?;
    Ok(status_ok())
}

async fn search_tokens(headers: HeaderMap, body: Bytes) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid search body")?;
    Ok(Json(vec![]))
}

async fn update_user_auth(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid auth body")?;
    Ok(status_ok())
}

async fn accept_terms_of_service(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid terms body")?;
    Ok(status_ok())
}

async fn set_user_typing(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(status_ok())
}

async fn get_user_uploads(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

async fn get_user_channel_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<mm::ChannelMember>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let members: Vec<ChannelMember> =
        sqlx::query_as("SELECT * FROM channel_members WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.db)
            .await?;

    let mm_members = members
        .into_iter()
        .map(|m| mm::ChannelMember {
            channel_id: encode_mm_id(m.channel_id),
            user_id: encode_mm_id(m.user_id),
            roles: crate::mattermost_compat::mappers::map_channel_role(&m.role),
            last_viewed_at: m.last_viewed_at.map(|t| t.timestamp_millis()).unwrap_or(0),
            msg_count: 0,
            mention_count: 0,
            notify_props: normalize_notify_props(m.notify_props),
            last_update_at: 0,
            scheme_guest: false,
            scheme_user: true,
            scheme_admin: m.role == "admin" || m.role == "team_admin" || m.role == "channel_admin",
        })
        .collect();

    Ok(Json(mm_members))
}

async fn migrate_auth_ldap(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid migrate body")?;
    Ok(status_ok())
}

async fn migrate_auth_saml(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid migrate body")?;
    Ok(status_ok())
}

async fn get_invalid_emails() -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![]))
}

async fn reset_failed_attempts(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(status_ok())
}

async fn update_custom_status(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid custom status")?;
    Ok(status_ok())
}

async fn clear_custom_status(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(status_ok())
}

async fn get_recent_custom_status(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

async fn delete_recent_custom_status(
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = resolve_user_id(&user_id, &auth)?;
    let _value: serde_json::Value = parse_body(&headers, &body, "Invalid custom status")?;
    Ok(status_ok())
}

/// GET /api/v4/users/{user_id}/oauth/apps/authorized
async fn get_authorized_oauth_apps(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn get_user_groups(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _user_uuid = if user_id == "me" {
        uuid::Uuid::new_v4()
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?
    };
    Ok(Json(vec![]))
}
