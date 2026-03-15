//! Auth API endpoints

use axum::{
    extract::{ConnectInfo, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;

use super::AppState;
use crate::auth::{create_token_with_policy, hash_password, verify_password, AuthUser};
use crate::error::{ApiResult, AppError};
use crate::middleware::rate_limit::{self, RateLimitConfig};
use crate::models::{AuthResponse, CreateUser, LoginRequest, User, UserResponse};
use crate::services::membership_policies::apply_auto_membership_for_new_user;
use crate::services::password_reset::{
    request_password_reset, reset_password, validate_token, PasswordResetError,
};
use crate::services::team_membership::{
    apply_default_channel_membership_for_team_join, ensure_default_channels_for_team,
};
use crate::services::turnstile;
use uuid::Uuid;

const DEFAULT_WORKSPACE_NAME: &str = "rustchat";
const DEFAULT_WORKSPACE_DISPLAY_NAME: &str = "RustChat";

#[derive(sqlx::FromRow)]
struct DefaultTeamRow {
    id: Uuid,
    org_id: Uuid,
}

async fn ensure_default_workspace_membership(state: &AppState, user: &User) -> ApiResult<User> {
    let mut user_org_id = user.org_id;
    let default_team = sqlx::query_as::<_, DefaultTeamRow>(
        "SELECT id, org_id FROM teams WHERE lower(name) = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(DEFAULT_WORKSPACE_NAME)
    .fetch_optional(&state.db)
    .await?;

    let team = if let Some(team) = default_team {
        team
    } else {
        let org_id = if let Some(existing_org_id) = user_org_id {
            existing_org_id
        } else {
            let new_org_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO organizations (id, name, display_name, description) VALUES ($1, $2, $3, $4)",
            )
            .bind(new_org_id)
            .bind(DEFAULT_WORKSPACE_NAME)
            .bind(DEFAULT_WORKSPACE_DISPLAY_NAME)
            .bind("Default RustChat organization")
            .execute(&state.db)
            .await?;

            sqlx::query("UPDATE users SET org_id = $1 WHERE id = $2")
                .bind(new_org_id)
                .bind(user.id)
                .execute(&state.db)
                .await?;
            user_org_id = Some(new_org_id);
            new_org_id
        };

        sqlx::query_as::<_, DefaultTeamRow>(
            r#"
            INSERT INTO teams (id, org_id, name, display_name, description, is_public, allow_open_invite)
            VALUES ($1, $2, $3, $4, $5, true, true)
            ON CONFLICT (org_id, name) DO UPDATE
                SET display_name = EXCLUDED.display_name
            RETURNING id, org_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(DEFAULT_WORKSPACE_NAME)
        .bind(DEFAULT_WORKSPACE_DISPLAY_NAME)
        .bind("Default RustChat team")
        .fetch_one(&state.db)
        .await?
    };

    if user_org_id.is_none() {
        sqlx::query("UPDATE users SET org_id = $1 WHERE id = $2")
            .bind(team.org_id)
            .bind(user.id)
            .execute(&state.db)
            .await?;
    }

    ensure_default_channels_for_team(state, team.id, user.id).await?;

    let membership_role = if user.role == "system_admin" || user.role == "org_admin" {
        "admin"
    } else {
        "member"
    };
    sqlx::query(
        "INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT (team_id, user_id) DO NOTHING",
    )
    .bind(team.id)
    .bind(user.id)
    .bind(membership_role)
    .execute(&state.db)
    .await?;

    if let Err(err) = apply_default_channel_membership_for_team_join(state, team.id, user.id).await {
        tracing::warn!(
            team_id = %team.id,
            user_id = %user.id,
            error = %err,
            "Default channel auto-join failed during auth workspace bootstrap"
        );
    }

    let refreshed_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
    Ok(refreshed_user)
}

/// Build auth routes
pub fn router(state: AppState) -> Router<AppState> {
    let registration_routes =
        Router::new()
            .route("/register", post(register))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::rate_limit::register_ip_rate_limit,
            ));
    let login_routes =
        Router::new()
            .route("/login", post(login))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::rate_limit::auth_ip_rate_limit,
            ));
    let verification_routes = Router::new()
        .route("/verify-email", post(verify_email))
        .route("/resend-verification", post(resend_verification))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit::auth_ip_rate_limit,
        ));
    let password_reset_routes = Router::new()
        .route("/password/forgot", post(forgot_password))
        .route("/password/reset", post(reset_password_handler))
        .route("/password/validate", post(validate_token_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit::password_reset_ip_rate_limit,
        ));

    Router::new()
        .merge(registration_routes)
        .merge(login_routes)
        .merge(verification_routes)
        .merge(password_reset_routes)
        .route("/me", get(me))
        .route("/policy", get(get_auth_policy))
        .route("/config", get(get_public_auth_config))
}

/// Get current authentication policy
async fn get_auth_policy(
    State(state): State<AppState>,
) -> ApiResult<Json<crate::models::AuthConfig>> {
    let config = crate::services::auth_config::get_password_rules(&state.db).await?;
    Ok(Json(config))
}

/// Get public auth configuration (safe to expose to frontend)
async fn get_public_auth_config(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "turnstile": {
            "enabled": state.config.turnstile.enabled,
            "site_key": state.config.turnstile.site_key,
        },
        "registration_enabled": true,
        "password_reset_enabled": true,
    })))
}

/// Register a new user
///
/// If password is provided, user is registered with that password.
/// If password is not provided, a password setup email is sent and user must set password via email link.
async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<CreateUser>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check honeypot - if filled, likely a bot
    if let Some(ref honeypot) = input.honeypot {
        if !honeypot.is_empty() {
            tracing::warn!(
                "Honeypot field filled, likely bot attempt from {}",
                addr.ip()
            );
            // Return generic error without revealing honeypot detection
            return Err(AppError::Validation("Invalid request".to_string()));
        }
    }

    // Verify Turnstile token if enabled
    if state.config.turnstile.enabled {
        let token = input
            .turnstile_token
            .as_deref()
            .ok_or_else(|| AppError::Validation("Verification required".to_string()))?;

        let remote_ip = Some(addr.ip().to_string());
        if let Err(e) = turnstile::verify_token(
            &state.config.turnstile.secret_key,
            token,
            remote_ip.as_deref(),
        )
        .await
        {
            tracing::warn!("Turnstile verification failed: {}", e);
            return Err(AppError::Validation(
                "Verification failed. Please try again.".to_string(),
            ));
        }
    }

    // Validate input
    if input.username.len() < 3 {
        return Err(AppError::Validation(
            "Username must be at least 3 characters".to_string(),
        ));
    }

    if !input.email.contains('@') {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }

    // Check if email already exists
    let existing: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(&input.email)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Check if username already exists
    let existing_username: Option<User> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(&input.username)
        .fetch_optional(&state.db)
        .await?;

    if existing_username.is_some() {
        return Err(AppError::Conflict("Username already taken".to_string()));
    }

    // Determine if this is passwordless registration
    let has_password = input.password.is_some() && !input.password.as_ref().unwrap().is_empty();

    // Validate password if provided
    let password_hash = if has_password {
        let config = crate::services::auth_config::get_password_rules(&state.db).await?;
        crate::services::auth_config::validate_password(input.password.as_ref().unwrap(), &config)?;
        Some(hash_password(input.password.as_ref().unwrap())?)
    } else {
        None
    };

    // Passwordless registrations stay inactive until password setup succeeds.
    let is_active = has_password;

    // Insert user (email_verified defaults to false, password_hash may be NULL for passwordless)
    let user: User = sqlx::query_as(
        r#"
        INSERT INTO users (username, email, password_hash, display_name, org_id, role, is_active, email_verified)
        VALUES ($1, $2, $3, $4, $5, 'member', $6, false)
        RETURNING *
        "#,
    )
    .bind(&input.username)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(&input.display_name)
    .bind(input.org_id)
    .bind(is_active)
    .fetch_one(&state.db)
    .await?;

    // Seed default preferences for the new user
    seed_default_preferences(&state.db, user.id).await?;

    // Apply auto-membership policies for the new user (global policies that add to teams/channels)
    match apply_auto_membership_for_new_user(&state, user.id).await {
        Ok(audit_entries) => {
            let success_count = audit_entries
                .iter()
                .filter(|e| e.status == "success" && e.action == "add")
                .count();
            if success_count > 0 {
                tracing::info!(
                    "Applied auto-membership policies for new user {}: {} memberships added",
                    user.id,
                    success_count
                );
            }
        }
        Err(e) => {
            // Don't fail registration if policy application fails, just log the error
            tracing::error!(
                "Failed to apply auto-membership policies for new user {}: {}",
                user.id,
                e
            );
        }
    }

    let user = ensure_default_workspace_membership(&state, &user).await?;

    // Fetch site_url from server_config
    let site_url: Option<String> =
        sqlx::query_scalar("SELECT site->>'site_url' FROM server_config WHERE id = 'default'")
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .and_then(|url: String| if url.is_empty() { None } else { Some(url) });

    if let Some(site_url) = site_url {
        if has_password {
            // Send verification email for users who provided password
            let verification_base_url = format!("{}/verify-email", site_url);
            match crate::services::email_verification::send_verification_email(
                &state.db,
                user.id,
                &user.username,
                &user.email,
                &verification_base_url,
            )
            .await
            {
                Ok(_) => {
                    tracing::info!("Verification email sent to {}", user.email);
                }
                Err(e) => {
                    tracing::warn!("Failed to send verification email: {}", e);
                }
            }

            // Generate token for immediate login
            let token = create_token_with_policy(
                user.id,
                &user.email,
                &user.role,
                user.org_id,
                &state.jwt_secret,
                state.jwt_issuer.as_deref(),
                state.jwt_audience.as_deref(),
                state.jwt_expiry_hours,
            )?;

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Registration successful. Please check your email to verify your account.",
                "requires_password_setup": false,
                "token": token,
                "user": UserResponse::from(user)
            })))
        } else {
            // Passwordless registration: send password setup email
            match crate::services::password_reset::send_password_setup_email(
                &state.db,
                user.id,
                &user.username,
                &user.email,
                &site_url,
            )
            .await
            {
                Ok(_) => {
                    tracing::info!("Password setup email sent to {}", user.email);
                }
                Err(e) => {
                    tracing::error!("Failed to send password setup email: {}", e);
                    // Don't fail registration, but inform user
                }
            }

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Registration successful. Please check your email to set your password.",
                "requires_password_setup": true,
                "email": user.email
            })))
        }
    } else {
        tracing::warn!("site_url not configured, skipping email sending");

        if has_password {
            // Generate token for immediate login
            let token = create_token_with_policy(
                user.id,
                &user.email,
                &user.role,
                user.org_id,
                &state.jwt_secret,
                state.jwt_issuer.as_deref(),
                state.jwt_audience.as_deref(),
                state.jwt_expiry_hours,
            )?;

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Registration successful.",
                "requires_password_setup": false,
                "token": token,
                "user": UserResponse::from(user)
            })))
        } else {
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Registration successful. Please contact administrator to set your password.",
                "requires_password_setup": true,
                "email": user.email
            })))
        }
    }
}

/// Login with email and password
async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Find user by email
    let user: User = sqlx::query_as(
        "SELECT * FROM users WHERE email = $1 AND is_active = true AND deleted_at IS NULL",
    )
    .bind(&input.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    enforce_password_login_allowed(&state, &user.email).await?;

    // Keep per-account throttle in addition to centralized per-IP middleware.
    if state.config.security.rate_limit_enabled {
        let config =
            RateLimitConfig::auth_per_minute(state.config.security.rate_limit_auth_per_minute);
        let user_key = format!("user:{}", user.id);
        let user_result = rate_limit::check_rate_limit(&state.redis, &config, &user_key).await?;

        if !user_result.allowed {
            tracing::warn!(user_id = %user.id, "Rate limit exceeded for user login");
            return Err(AppError::TooManyRequests(
                "Too many login attempts. Please try again later.".to_string(),
            ));
        }
    }

    // Verify password (OAuth users or users pending password setup cannot login with password)
    let password_hash = user.password_hash.as_deref().ok_or_else(|| {
        if user.email_verified {
            AppError::Unauthorized(
                "Please set your password using the link sent to your email.".to_string(),
            )
        } else {
            AppError::Unauthorized(
                "Please verify your email and set your password first.".to_string(),
            )
        }
    })?;

    if !verify_password(&input.password, password_hash)? {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    // Update last login
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    let user = ensure_default_workspace_membership(&state, &user).await?;

    // Generate token
    let token = create_token_with_policy(
        user.id,
        &user.email,
        &user.role,
        user.org_id,
        &state.jwt_secret,
        state.jwt_issuer.as_deref(),
        state.jwt_audience.as_deref(),
        state.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiry_hours * 3600,
        user: UserResponse::from(user),
    }))
}

async fn enforce_password_login_allowed(state: &AppState, user_email: &str) -> ApiResult<()> {
    let auth_config = crate::services::auth_config::get_password_rules(&state.db).await?;
    if !auth_config.require_sso {
        return Ok(());
    }

    let email_lc = user_email.trim().to_ascii_lowercase();
    let allowed = auth_config
        .sso_break_glass_emails
        .iter()
        .any(|email| email.trim().eq_ignore_ascii_case(&email_lc));
    if allowed {
        return Ok(());
    }

    Err(AppError::BadRequest(
        "Password login is disabled because SSO is required".to_string(),
    ))
}

/// Verify email with token
#[derive(Debug, serde::Deserialize)]
struct VerifyEmailRequest {
    token: String,
}

async fn verify_email(
    State(state): State<AppState>,
    Json(input): Json<VerifyEmailRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id =
        crate::services::email_verification::verify_token(&state.db, &input.token, "registration")
            .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email verified successfully",
        "user_id": user_id.to_string()
    })))
}

/// Resend verification email
#[derive(Debug, serde::Deserialize)]
struct ResendVerificationRequest {
    email: String,
}

async fn resend_verification(
    State(state): State<AppState>,
    Json(input): Json<ResendVerificationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Find user by email
    let user: Option<User> = sqlx::query_as(
        "SELECT * FROM users WHERE email = $1 AND is_active = true AND deleted_at IS NULL",
    )
    .bind(&input.email)
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(u) => u,
        None => {
            // Return success even if user not found to prevent email enumeration
            return Ok(Json(serde_json::json!({
                "success": true,
                "message": "If the email exists, a verification email has been sent"
            })));
        }
    };

    if user.email_verified {
        return Ok(Json(serde_json::json!({
            "success": true,
            "message": "Email is already verified"
        })));
    }

    // Send verification email
    // Fetch site_url from server_config
    let site_url: Option<String> =
        sqlx::query_scalar("SELECT site->>'site_url' FROM server_config WHERE id = 'default'")
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .and_then(|url: String| if url.is_empty() { None } else { Some(url) });

    let verification_result = if let Some(site_url) = site_url {
        let verification_base_url = format!("{}/verify-email", site_url);
        crate::services::email_verification::send_verification_email(
            &state.db,
            user.id,
            &user.username,
            &user.email,
            &verification_base_url,
        )
        .await
    } else {
        tracing::warn!("site_url not configured, cannot send verification email");
        Ok(())
    };

    match verification_result {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Verification email sent"
        }))),
        Err(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "If the email exists, a verification email has been sent"
        }))),
    }
}

/// Get current authenticated user
async fn me(State(state): State<AppState>, auth: AuthUser) -> ApiResult<Json<UserResponse>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?;
    let user = ensure_default_workspace_membership(&state, &user).await?;

    Ok(Json(UserResponse::from(user)))
}

// ============================================
// Password Reset Handlers
// ============================================

#[derive(Debug, serde::Deserialize)]
struct ForgotPasswordRequest {
    email: String,
    /// Cloudflare Turnstile token (bot protection)
    #[serde(rename = "cf-turnstile-response")]
    turnstile_token: Option<String>,
    /// Honeypot field - should be empty (bots usually fill this)
    #[serde(rename = "website")]
    honeypot: Option<String>,
}

/// Request password reset email
/// Returns same response regardless of email existence (anti-enumeration)
async fn forgot_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<ForgotPasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check honeypot - if filled, likely a bot
    if let Some(ref honeypot) = input.honeypot {
        if !honeypot.is_empty() {
            tracing::warn!(
                "Honeypot field filled, likely bot attempt from {}",
                addr.ip()
            );
            // Return success response to not reveal honeypot detection
            return Ok(Json(serde_json::json!({
                "success": true,
                "message": "If an account with that email exists, you will receive a password reset link"
            })));
        }
    }

    // Verify Turnstile token if enabled
    if state.config.turnstile.enabled {
        let token = match input.turnstile_token.as_deref() {
            Some(t) if !t.is_empty() => t,
            _ => {
                return Ok(Json(serde_json::json!({
                    "success": true,
                    "message": "If an account with that email exists, you will receive a password reset link"
                })));
            }
        };

        let remote_ip = Some(addr.ip().to_string());
        if let Err(e) = turnstile::verify_token(
            &state.config.turnstile.secret_key,
            token,
            remote_ip.as_deref(),
        )
        .await
        {
            tracing::warn!("Turnstile verification failed: {}", e);
            // Return success response to not reveal verification failure
            return Ok(Json(serde_json::json!({
                "success": true,
                "message": "If an account with that email exists, you will receive a password reset link"
            })));
        }
    }

    // Get IP address from connection
    let ip_address = Some(addr.ip());

    // Request password reset (always returns Ok for anti-enumeration)
    let result = request_password_reset(
        &state.db,
        &input.email,
        ip_address,
        None, // user_agent could be extracted from headers if needed
    )
    .await;

    // Log but don't expose errors
    if let Err(ref e) = result {
        tracing::debug!("Password reset request result: {:?}", e);
    }

    // Always return same response
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "If an account with that email exists, you will receive a password reset link"
    })))
}

#[derive(Debug, serde::Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

/// Reset password with token
async fn reset_password_handler(
    State(state): State<AppState>,
    Json(input): Json<ResetPasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    match reset_password(&state.db, &input.token, &input.new_password).await {
        Ok(user_id) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Password reset successful",
            "user_id": user_id.to_string()
        }))),
        Err(
            PasswordResetError::TokenNotFound
            | PasswordResetError::TokenExpired
            | PasswordResetError::TokenAlreadyUsed,
        ) => Err(AppError::BadRequest("Invalid or expired token".to_string())),
        Err(PasswordResetError::InvalidPassword(msg)) => Err(AppError::Validation(msg)),
        Err(PasswordResetError::RateLimitExceeded) => Err(AppError::TooManyRequests(
            "Too many attempts. Please try again later.".to_string(),
        )),
        Err(e) => {
            tracing::error!("Password reset error: {}", e);
            Err(AppError::Internal("Failed to reset password".to_string()))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct ValidateTokenRequest {
    token: String,
}

/// Validate token without consuming it (for UI)
async fn validate_token_handler(
    State(state): State<AppState>,
    Json(input): Json<ValidateTokenRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    match validate_token(&state.db, &input.token).await {
        Ok((user_id, email)) => Ok(Json(serde_json::json!({
            "valid": true,
            "user_id": user_id.to_string(),
            "email": email
        }))),
        Err(
            PasswordResetError::TokenNotFound
            | PasswordResetError::TokenExpired
            | PasswordResetError::TokenAlreadyUsed,
        ) => Ok(Json(serde_json::json!({
            "valid": false
        }))),
        Err(e) => {
            tracing::error!("Token validation error: {}", e);
            Err(AppError::Internal("Failed to validate token".to_string()))
        }
    }
}

/// Seed default preferences for a new user
async fn seed_default_preferences(db: &sqlx::PgPool, user_id: uuid::Uuid) -> ApiResult<()> {
    // Build theme JSON using serde_json to avoid raw string issues with #" sequences
    let default_theme = serde_json::json!({
        "type": "RustChat",
        "sidebarBg": "#1A1A18",
        "sidebarText": "#ffffff",
        "sidebarUnreadText": "#ffffff",
        "sidebarTextHoverBg": "#25262a",
        "sidebarTextActiveBorder": "#00FFC2",
        "sidebarTextActiveColor": "#ffffff",
        "sidebarHeaderBg": "#121213",
        "sidebarHeaderTextColor": "#ffffff",
        "sidebarTeamBarBg": "#121213",
        "onlineIndicator": "#00FFC2",
        "awayIndicator": "#ffbc1f",
        "dndIndicator": "#d24b4e",
        "mentionBg": "#ffffff",
        "mentionColor": "#1A1A18",
        "centerChannelBg": "#121213",
        "centerChannelColor": "#e3e4e8",
        "newMessageSeparator": "#00FFC2",
        "linkColor": "#00FFC2",
        "buttonBg": "#00FFC2",
        "buttonColor": "#121213",
        "errorTextColor": "#da6c6e",
        "mentionHighlightBg": "#0d6e6e",
        "mentionHighlightLink": "#a4f4f4",
        "codeTheme": "monokai"
    })
    .to_string();

    // Theme preference
    sqlx::query(
        r#"
        INSERT INTO mattermost_preferences (user_id, category, name, value)
        VALUES ($1, 'theme', '', $2)
        ON CONFLICT (user_id, category, name) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(default_theme)
    .execute(db)
    .await?;

    // Display settings
    let display_prefs = [
        ("use_military_time", "false"),
        ("timezone", "Auto"),
        ("collapsed_reply_threads", "on"),
    ];

    for (name, value) in display_prefs {
        sqlx::query(
            r#"
            INSERT INTO mattermost_preferences (user_id, category, name, value)
            VALUES ($1, 'display_settings', $2, $3)
            ON CONFLICT (user_id, category, name) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(value)
        .execute(db)
        .await?;
    }

    // Notification settings
    let notify_prefs = [
        ("desktop", "mention"),
        ("push", "mention"),
        ("email", "true"),
    ];

    for (name, value) in notify_prefs {
        sqlx::query(
            r#"
            INSERT INTO mattermost_preferences (user_id, category, name, value)
            VALUES ($1, 'notifications', $2, $3)
            ON CONFLICT (user_id, category, name) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(value)
        .execute(db)
        .await?;
    }

    // Sidebar settings
    sqlx::query(
        r#"
        INSERT INTO mattermost_preferences (user_id, category, name, value)
        VALUES ($1, 'sidebar_settings', 'show_unread_section', 'true')
        ON CONFLICT (user_id, category, name) DO NOTHING
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}
