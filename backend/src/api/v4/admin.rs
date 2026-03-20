use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use crate::services::rate_limit::{IpRateLimitConfig, RateLimitLimits};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LimitEntry {
    pub limit: u32,
    pub window_secs: u32,
    pub enabled: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct RateLimitsResponse {
    pub auth_ip: LimitEntry,
    pub auth_user: LimitEntry,
    pub register_ip: LimitEntry,
    pub password_reset_ip: LimitEntry,
    pub websocket_ip: LimitEntry,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateRateLimitsRequest {
    pub auth_ip: Option<LimitEntry>,
    pub auth_user: Option<LimitEntry>,
    pub register_ip: Option<LimitEntry>,
    pub password_reset_ip: Option<LimitEntry>,
    pub websocket_ip: Option<LimitEntry>,
}

fn limits_to_response(limits: &RateLimitLimits) -> RateLimitsResponse {
    let e = |c: IpRateLimitConfig| LimitEntry {
        limit: c.limit as u32,
        window_secs: c.window_secs as u32,
        enabled: c.enabled,
    };
    RateLimitsResponse {
        auth_ip: e(limits.auth_ip),
        auth_user: e(limits.auth_user),
        register_ip: e(limits.register_ip),
        password_reset_ip: e(limits.password_reset_ip),
        websocket_ip: e(limits.websocket_ip),
    }
}

fn entry_to_config(e: &LimitEntry) -> crate::services::rate_limit::IpRateLimitConfig {
    crate::services::rate_limit::IpRateLimitConfig {
        limit: e.limit as u64,
        window_secs: e.window_secs as u64,
        enabled: e.enabled,
    }
}
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/audits", get(get_audits))
        .route("/admin/email/test", post(test_email_config))
        .route("/admin/keycloak/sync", post(trigger_keycloak_sync))
        .route(
            "/admin/keycloak/sync/users/{user_id}",
            post(trigger_keycloak_user_sync),
        )
        .route("/admin/rate-limits", get(get_rate_limits).put(update_rate_limits))
}
use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::ApiResult;
use crate::error::AppError;
use crate::mattermost_compat::id::parse_mm_or_uuid;
use crate::mattermost_compat::models as mm;
use crate::models::email::MailProviderSettings;
use crate::services::email_provider::{EmailAddress, EmailContent, MailProvider, SmtpProvider};
use crate::services::keycloak_sync;

pub async fn get_audits(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Audit>>> {
    // Audit logs contain sensitive PII (IP addresses) - restrict to system admins only
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "Missing permission to view audit logs".to_string(),
        ));
    }
    let audits: Vec<mm::Audit> = sqlx::query_as(
        r#"
        SELECT id::text, 
               (extract(epoch from created_at)*1000)::int8 as create_at,
               actor_user_id::text as user_id,
               action,
               metadata::text as extra_info,
               actor_ip as ip_address,
               '' as session_id
        FROM audit_logs
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(audits))
}

#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    pub email: Option<String>,
    #[serde(rename = "to")]
    pub to_email: Option<String>,
}

pub async fn test_email_config(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(payload): Json<TestEmailRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "Missing permission to test email configuration".to_string(),
        ));
    }

    // Get default provider from the new provider system
    let provider_settings: Option<MailProviderSettings> = sqlx::query_as(
        r#"
        SELECT * FROM mail_provider_settings
        WHERE enabled = true AND is_default = true
        ORDER BY tenant_id NULLS LAST
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let provider_settings = match provider_settings {
        Some(p) => p,
        None => {
            return Ok(Json(serde_json::json!({
                "status": "error",
                "error": "No default email provider configured. Configure one in the admin console."
            })));
        }
    };

    // Check if SMTP is configured
    if provider_settings.host.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "error": "SMTP server not configured for the default provider."
        })));
    }

    if provider_settings.from_address.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "error": "From address not configured for the default provider."
        })));
    }

    // Determine test recipient (use 'to' field or 'email' field, fallback to user's email)
    let test_email = payload
        .to_email
        .or(payload.email)
        .unwrap_or_else(|| auth.email.clone());

    // Create provider and test
    let provider =
        match SmtpProvider::new(provider_settings.clone(), &state.config.encryption_key).await {
            Ok(p) => p,
            Err(e) => {
                return Ok(Json(serde_json::json!({
                    "status": "error",
                    "error": format!("Failed to initialize SMTP provider: {}", e)
                })));
            }
        };

    // Test connection first
    if let Err(e) = provider.test_connection().await {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "error": format!("SMTP connection test failed: {}", e)
        })));
    }

    // Send test email
    let from = EmailAddress::with_name(
        &provider_settings.from_address,
        &provider_settings.from_name,
    );
    let to = EmailAddress::new(&test_email);
    let content = EmailContent {
        subject: "RustChat Test Email".to_string(),
        body_text: format!(
            "This is a test email from RustChat.\n\nIf you received this, your email configuration is working correctly!\n\nConfiguration used:\n- SMTP Server: {}:{}\n- TLS: {}\n- From: {}\n",
            provider_settings.host,
            provider_settings.port,
            provider_settings.tls_mode.as_str(),
            provider_settings.from_address
        ),
        body_html: None,
        headers: vec![],
    };

    match provider.send_email(&from, &to, &content).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": format!("Test email sent successfully to {}", test_email),
            "delivery": {
                "accepted": true,
                "message_id": result.message_id,
                "server_response": result.server_response,
            },
            "config": {
                "smtp_host": provider_settings.host,
                "smtp_port": provider_settings.port,
                "smtp_security": provider_settings.tls_mode.as_str(),
                "from_address": provider_settings.from_address,
                "from_name": provider_settings.from_name,
                "reply_to": provider_settings.reply_to.as_deref().unwrap_or(""),
            }
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "error",
            "error": format!("Failed to send test email: {}", e),
            "details": "Check your SMTP configuration and ensure the server allows sending."
        }))),
    }
}

async fn trigger_keycloak_sync(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "Missing permission to run Keycloak sync".to_string(),
        ));
    }

    let report = keycloak_sync::run_full_sync(&state).await?;
    Ok(Json(serde_json::json!({
        "status": "OK",
        "report": report
    })))
}

async fn trigger_keycloak_user_sync(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "Missing permission to run Keycloak user sync".to_string(),
        ));
    }

    let user_uuid: Uuid = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
    let report = keycloak_sync::resync_user(&state, user_uuid).await?;
    Ok(Json(serde_json::json!({
        "status": "OK",
        "report": report
    })))
}

async fn upsert_rate_limit(
    db: &sqlx::PgPool,
    limit_key: &str,
    flag_key: &str,
    entry: &LimitEntry,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO rate_limits (key, limit_value, window_secs, updated_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (key) DO UPDATE
         SET limit_value = EXCLUDED.limit_value,
             window_secs = EXCLUDED.window_secs,
             updated_at = NOW()",
    )
    .bind(limit_key)
    .bind(entry.limit as i32)
    .bind(entry.window_secs as i32)
    .execute(db)
    .await?;

    // window_secs = 0 marks this as an enabled-flag row (see rate_limits table comment)
    sqlx::query(
        "INSERT INTO rate_limits (key, limit_value, window_secs, updated_at)
         VALUES ($1, $2, 0, NOW())
         ON CONFLICT (key) DO UPDATE
         SET limit_value = EXCLUDED.limit_value,
             updated_at = NOW()",
    )
    .bind(flag_key)
    .bind(if entry.enabled { 1i32 } else { 0i32 })
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_rate_limits(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<RateLimitsResponse>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }
    let limits = state.rate_limit.ip_limits().await;
    Ok(Json(limits_to_response(&limits)))
}

pub async fn update_rate_limits(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(payload): Json<UpdateRateLimitsRequest>,
) -> ApiResult<Json<RateLimitsResponse>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    // Validate that limit values fit safely into the DB column (INTEGER = i32)
    let entries = [
        payload.auth_ip.as_ref(),
        payload.auth_user.as_ref(),
        payload.register_ip.as_ref(),
        payload.password_reset_ip.as_ref(),
        payload.websocket_ip.as_ref(),
    ];
    for entry in entries.into_iter().flatten() {
        if entry.limit > i32::MAX as u32 {
            return Err(AppError::BadRequest(format!(
                "limit value {} exceeds maximum allowed ({})",
                entry.limit,
                i32::MAX
            )));
        }
    }

    if let Some(ref e) = payload.auth_ip {
        upsert_rate_limit(&state.db, "auth_ip_per_minute", "auth_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.auth_user {
        upsert_rate_limit(&state.db, "auth_user_per_minute", "auth_user_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.register_ip {
        upsert_rate_limit(&state.db, "register_ip_per_minute", "register_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.password_reset_ip {
        upsert_rate_limit(&state.db, "password_reset_ip_per_minute", "password_reset_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.websocket_ip {
        upsert_rate_limit(&state.db, "websocket_ip_per_minute", "websocket_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Hot-reload — new limits take effect immediately for subsequent requests
    let reloaded = state.rate_limit.reload().await;
    if let Err(ref e) = reloaded {
        tracing::warn!(error = %e, "Rate limit hot-reload failed after admin update");
    }

    // If reload failed, build the response from the current cache merged with the
    // payload so the caller sees what was written to DB, not stale in-memory state.
    let limits = if reloaded.is_ok() {
        state.rate_limit.ip_limits().await
    } else {
        let mut current = state.rate_limit.ip_limits().await;
        if let Some(ref e) = payload.auth_ip {
            current.auth_ip = entry_to_config(e);
        }
        if let Some(ref e) = payload.auth_user {
            current.auth_user = entry_to_config(e);
        }
        if let Some(ref e) = payload.register_ip {
            current.register_ip = entry_to_config(e);
        }
        if let Some(ref e) = payload.password_reset_ip {
            current.password_reset_ip = entry_to_config(e);
        }
        if let Some(ref e) = payload.websocket_ip {
            current.websocket_ip = entry_to_config(e);
        }
        current
    };
    Ok(Json(limits_to_response(&limits)))
}
