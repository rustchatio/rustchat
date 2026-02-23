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
use crate::models::email::MailProviderSettings;
use crate::services::email_provider::{EmailAddress, EmailContent, MailProvider, SmtpProvider};

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
    // Get default provider from the new provider system
    let provider_settings: Option<MailProviderSettings> = sqlx::query_as(
        r#"
        SELECT * FROM mail_provider_settings
        WHERE enabled = true AND is_default = true
        ORDER BY tenant_id NULLS LAST
        LIMIT 1
        "#
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
    let provider = match SmtpProvider::new(provider_settings.clone()).await {
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
    let from = EmailAddress::with_name(&provider_settings.from_address, &provider_settings.from_name);
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
