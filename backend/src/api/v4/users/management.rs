use axum::{body::Bytes, extract::State, http::HeaderMap, Json};
use serde::Deserialize;

use super::MmAuthUser;
use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::auth::{hash_password, verify_password};
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::parse_mm_or_uuid;
use crate::models::User;

#[derive(Deserialize)]
pub struct UserRolesRequest {
    pub roles: String,
}

pub async fn update_user_roles(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    Json(input): Json<UserRolesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Only system admins can grant system_admin role - prevents org_admin self-escalation
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
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

    Ok(super::status_ok())
}

#[derive(Deserialize)]
pub struct UserActiveRequest {
    pub active: bool,
}

pub async fn update_user_active(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    Json(input): Json<UserActiveRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    if !auth.can_access_owned(user_id, &permissions::USER_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    sqlx::query("UPDATE users SET is_active = $1 WHERE id = $2")
        .bind(input.active)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(super::status_ok())
}

pub async fn demote_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::USER_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("UPDATE users SET role = 'member' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(super::status_ok())
}

pub async fn promote_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Only system admins can promote to system_admin - prevents org_admin self-escalation
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("UPDATE users SET role = 'system_admin' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(super::status_ok())
}

pub async fn convert_user_to_bot(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::USER_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    sqlx::query("UPDATE users SET is_bot = true WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(super::status_ok())
}

#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}

pub async fn update_user_password(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    Json(input): Json<UpdatePasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    if user_id != auth.user_id {
        // Admins resetting another user's password do not need current_password
        if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
            return Err(AppError::Forbidden(
                "Cannot change another user's password".to_string(),
            ));
        }
    } else {
        // Self-service: current_password is mandatory
        let current = input
            .current_password
            .as_deref()
            .ok_or_else(|| AppError::BadRequest("current_password is required".to_string()))?;
        let password_hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| AppError::BadRequest("No existing password to change".to_string()))?;
        if !verify_password(current, password_hash)? {
            return Err(AppError::BadRequest("Invalid current password".to_string()));
        }
    }

    let new_hash = hash_password(&input.new_password)?;
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(super::status_ok())
}

pub async fn reset_password(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = super::parse_body(&headers, &body, "Invalid reset body")?;
    Ok(super::status_ok())
}

pub async fn send_password_reset(
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = super::parse_body(&headers, &body, "Invalid reset body")?;
    Ok(super::status_ok())
}

#[derive(Deserialize)]
pub struct CheckMfaRequest {
    #[serde(rename = "login_id")]
    pub _login_id: String,
}

pub async fn check_user_mfa(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _input: CheckMfaRequest = super::parse_body(&headers, &body, "Invalid mfa body")?;
    Ok(Json(serde_json::json!({"mfa_required": false})))
}

#[derive(Deserialize)]
pub struct UpdateMfaRequest {
    #[serde(rename = "activate")]
    pub _activate: bool,
    #[allow(dead_code)]
    pub code: Option<String>,
}

pub async fn update_user_mfa(
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    Json(_input): Json<UpdateMfaRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    let _ = user_id;
    Ok(super::status_ok())
}

pub async fn generate_mfa_secret(
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    Ok(Json(serde_json::json!({"secret": "", "qr_code": ""})))
}

pub async fn get_user_audits(
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

pub async fn verify_member_email() -> ApiResult<Json<serde_json::Value>> {
    Ok(super::status_ok())
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(input): Json<VerifyEmailRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::services::email_verification::verify_token(&state.db, &input.token, "registration")
        .await?;

    Ok(super::status_ok())
}

#[derive(Debug, Deserialize)]
pub struct SendVerificationRequest {
    pub email: String,
}

pub async fn send_email_verification(
    State(state): State<AppState>,
    Json(input): Json<SendVerificationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Find user by email
    let user: Option<User> = sqlx::query_as(
        "SELECT * FROM users WHERE email = $1 AND is_active = true AND deleted_at IS NULL",
    )
    .bind(&input.email)
    .fetch_optional(&state.db)
    .await?;

    if let Some(user) = user {
        if !user.email_verified {
            // Fetch site_url from server_config
            let site_url: Option<String> = sqlx::query_scalar(
                "SELECT site->>'site_url' FROM server_config WHERE id = 'default'",
            )
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .and_then(|url: String| if url.is_empty() { None } else { Some(url) });

            if let Some(site_url) = site_url {
                let verification_base_url = format!("{}/verify-email", site_url);
                // Send but ignore errors to prevent email enumeration
                let _ = crate::services::email_verification::send_verification_email(
                    &state.db,
                    user.id,
                    &user.username,
                    &user.email,
                    &verification_base_url,
                )
                .await;
            }
        }
    }

    // Always return success to prevent email enumeration
    Ok(super::status_ok())
}
