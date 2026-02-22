use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/audits", get(get_audits))
        .route("/admin/email/test", post(test_email_config))
}
use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::ApiResult;
use crate::mattermost_compat::models as mm;

pub async fn get_audits(
    State(state): State<AppState>,
    _auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Audit>>> {
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
    // Get email config
    let config =
        sqlx::query_as::<_, (sqlx::types::Json<crate::models::server_config::EmailConfig>,)>(
            "SELECT email FROM server_config WHERE id = 'default'",
        )
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|row| row.0 .0)
        .unwrap_or_default();

    // Check if email notifications are enabled
    if !config.send_email_notifications {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "error": "Email notifications are disabled. Enable them in System Console > Notifications > Email."
        })));
    }

    // Check if SMTP is configured
    if config.smtp_host.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "error": "SMTP server not configured. Configure it in System Console > Notifications > Email."
        })));
    }

    if config.from_address.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "error": "From address not configured. Set it in System Console > Notifications > Email."
        })));
    }

    // Determine test recipient (use 'to' field or 'email' field, fallback to user's email)
    let test_email = payload
        .to_email
        .or(payload.email)
        .unwrap_or_else(|| auth.email.clone());

    // Send test email using the email service
    match crate::services::email::send_email(
        &config,
        &test_email,
        "RustChat Test Email",
        &format!(
            "This is a test email from RustChat.\n\nIf you received this, your email configuration is working correctly!\n\nConfiguration used:\n- SMTP Server: {}:{}\n- Security: {}\n- From: {}\n",
            config.smtp_host,
            config.smtp_port,
            config.smtp_security,
            config.from_address
        ),
    ).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": format!("Test email sent successfully to {}", test_email),
            "delivery": {
                "accepted": result.accepted,
                "message_id": result.message_id,
                "server_response": result.server_response,
            },
            "config": {
                "smtp_host": config.smtp_host,
                "smtp_port": config.smtp_port,
                "smtp_security": config.smtp_security,
                "from_address": config.from_address,
                "from_name": config.from_name,
                "reply_to": config.reply_to,
            }
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "error",
            "error": format!("Failed to send test email: {}", e),
            "details": "Check your SMTP configuration and ensure the server allows sending."
        }))),
    }
}
