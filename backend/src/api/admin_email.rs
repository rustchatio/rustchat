//! Admin Email API Endpoints
//!
//! Provides administrative endpoints for managing the email subsystem:
//! - Mail provider settings
//! - Notification workflows
//! - Email templates
//! - Outbox monitoring
//! - Email events/audit

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::api::{admin::require_admin, AppState};
use crate::error::{ApiResult, AppError};
use crate::models::email::*;
use crate::services::email_provider::{EmailAddress, EmailContent, MailProvider, SmtpProvider};
use crate::services::email_service::{EmailService, EnqueueOptions, OutboxFilters};
use crate::services::template_renderer::TemplateRenderer;

/// Build admin email routes
pub fn router() -> Router<AppState> {
    Router::new()
        // Provider Settings
        .route(
            "/admin/email/providers",
            get(list_providers).post(create_provider),
        )
        .route(
            "/admin/email/providers/{id}",
            get(get_provider)
                .put(update_provider)
                .delete(delete_provider),
        )
        .route("/admin/email/providers/{id}/test", post(test_provider))
        .route(
            "/admin/email/providers/{id}/default",
            post(set_default_provider),
        )
        // Workflows
        .route("/admin/email/workflows", get(list_workflows))
        .route(
            "/admin/email/workflows/{id}",
            get(get_workflow).patch(update_workflow),
        )
        // Template Families
        .route(
            "/admin/email/template-families",
            get(list_template_families).post(create_template_family),
        )
        .route(
            "/admin/email/template-families/{id}",
            get(get_template_family)
                .patch(update_template_family)
                .delete(delete_template_family),
        )
        // Template Versions
        .route(
            "/admin/email/template-families/{id}/versions",
            get(list_template_versions).post(create_template_version),
        )
        .route(
            "/admin/email/template-versions/{version_id}",
            get(get_template_version).patch(update_template_version),
        )
        .route(
            "/admin/email/template-versions/{version_id}/publish",
            post(publish_template_version),
        )
        .route(
            "/admin/email/template-versions/{version_id}/preview",
            post(preview_template),
        )
        .route(
            "/admin/email/template-versions/{version_id}/send-preview",
            post(send_preview_email),
        )
        // Outbox
        .route("/admin/email/outbox", get(list_outbox))
        .route("/admin/email/outbox/{id}", get(get_outbox_entry))
        .route("/admin/email/outbox/{id}/cancel", post(cancel_outbox_entry))
        .route("/admin/email/outbox/{id}/retry", post(retry_outbox_entry))
        // Events
        .route("/admin/email/events", get(list_email_events))
        // Send test email
        .route("/admin/email/send-test", post(send_test_email))
        // User preferences (admin view)
        .route(
            "/admin/email/users/{user_id}/prefs",
            get(get_user_prefs).put(update_user_prefs),
        )
}

// ============================================
// Provider Settings
// ============================================

#[derive(Debug, Deserialize)]
struct ListProvidersQuery {
    tenant_id: Option<Uuid>,
}

async fn list_providers(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Query(query): Query<ListProvidersQuery>,
) -> ApiResult<Json<Vec<MailProviderResponse>>> {
    require_admin(&auth)?;

    // Admin users can see all providers:
    // - If tenant_id is specified in query, filter by that tenant
    // - Otherwise, return ALL providers (both global with NULL tenant and org-specific)
    let providers: Vec<MailProviderSettings> = if let Some(tenant_id) = query.tenant_id {
        sqlx::query_as(
            r#"
            SELECT * FROM mail_provider_settings
            WHERE tenant_id = $1
            ORDER BY is_default DESC, created_at ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&state.db)
        .await?
    } else {
        // Return ALL providers for admin (both global NULL tenant and org-specific)
        sqlx::query_as(
            r#"
            SELECT * FROM mail_provider_settings
            ORDER BY is_default DESC, created_at ASC
            "#,
        )
        .fetch_all(&state.db)
        .await?
    };

    let responses: Vec<MailProviderResponse> = providers.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn get_provider(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MailProviderResponse>> {
    require_admin(&auth)?;

    let provider: MailProviderSettings =
        sqlx::query_as("SELECT * FROM mail_provider_settings WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Provider not found".to_string()))?;

    Ok(Json(provider.into()))
}

async fn create_provider(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Json(body): Json<CreateMailProviderRequest>,
) -> ApiResult<Json<MailProviderResponse>> {
    require_admin(&auth)?;

    let provider_type = MailProviderType::from_str(&body.provider_type).ok_or_else(|| {
        AppError::Validation(format!("Invalid provider type: {}", body.provider_type))
    })?;

    let tls_mode = TlsMode::from_str(&body.tls_mode)
        .ok_or_else(|| AppError::Validation(format!("Invalid TLS mode: {}", body.tls_mode)))?;

    // Encrypt password
    let password_encrypted = if body.password.is_empty() {
        String::new()
    } else {
        crate::crypto::encrypt(&body.password, &state.config.encryption_key)?
    };

    // If this is set as default, clear other defaults
    if body.is_default {
        sqlx::query(
            "UPDATE mail_provider_settings SET is_default = false WHERE is_default = true AND tenant_id IS NULL"
        )
        .execute(&state.db)
        .await?;
    }

    let provider: MailProviderSettings = sqlx::query_as(
        r#"
        INSERT INTO mail_provider_settings (
            tenant_id, provider_type, host, port, username, password_encrypted,
            tls_mode, skip_cert_verify, from_address, from_name, reply_to,
            max_emails_per_minute, max_emails_per_hour, enabled, is_default, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING *
        "#,
    )
    .bind(auth.org_id) // Use admin's org as tenant
    .bind(provider_type)
    .bind(&body.host)
    .bind(body.port)
    .bind(&body.username)
    .bind(password_encrypted)
    .bind(tls_mode)
    .bind(body.skip_cert_verify)
    .bind(&body.from_address)
    .bind(&body.from_name)
    .bind(body.reply_to)
    .bind(body.max_emails_per_minute)
    .bind(body.max_emails_per_hour)
    .bind(body.enabled)
    .bind(body.is_default)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    info!(
        "Created mail provider: id={}, host={}",
        provider.id, provider.host
    );
    Ok(Json(provider.into()))
}

async fn update_provider(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMailProviderRequest>,
) -> ApiResult<Json<MailProviderResponse>> {
    require_admin(&auth)?;

    // Get existing provider
    let existing: MailProviderSettings =
        sqlx::query_as("SELECT * FROM mail_provider_settings WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Provider not found".to_string()))?;

    // Process password if provided
    let password_encrypted = if let Some(ref password) = body.password {
        if password.is_empty() {
            None
        } else {
            Some(crate::crypto::encrypt(
                password,
                &state.config.encryption_key,
            )?)
        }
    } else {
        None
    };

    // If setting as default, clear others
    if body.is_default == Some(true) && !existing.is_default {
        sqlx::query(
            "UPDATE mail_provider_settings SET is_default = false WHERE is_default = true AND tenant_id = $1"
        )
        .bind(existing.tenant_id)
        .execute(&state.db)
        .await?;
    }

    let provider: MailProviderSettings = sqlx::query_as(
        r#"
        UPDATE mail_provider_settings SET
            provider_type = COALESCE($2, provider_type),
            host = COALESCE($3, host),
            port = COALESCE($4, port),
            username = COALESCE($5, username),
            password_encrypted = COALESCE($6, password_encrypted),
            tls_mode = COALESCE($7, tls_mode),
            skip_cert_verify = COALESCE($8, skip_cert_verify),
            from_address = COALESCE($9, from_address),
            from_name = COALESCE($10, from_name),
            reply_to = COALESCE($11, reply_to),
            max_emails_per_minute = COALESCE($12, max_emails_per_minute),
            max_emails_per_hour = COALESCE($13, max_emails_per_hour),
            enabled = COALESCE($14, enabled),
            is_default = COALESCE($15, is_default),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(
        body.provider_type
            .as_deref()
            .and_then(MailProviderType::from_str),
    )
    .bind(&body.host)
    .bind(body.port)
    .bind(&body.username)
    .bind(password_encrypted)
    .bind(body.tls_mode.as_deref().and_then(TlsMode::from_str))
    .bind(body.skip_cert_verify)
    .bind(&body.from_address)
    .bind(&body.from_name)
    .bind(body.reply_to)
    .bind(body.max_emails_per_minute)
    .bind(body.max_emails_per_hour)
    .bind(body.enabled)
    .bind(body.is_default)
    .fetch_one(&state.db)
    .await?;

    info!("Updated mail provider: id={}", id);
    Ok(Json(provider.into()))
}

async fn delete_provider(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    let result = sqlx::query("DELETE FROM mail_provider_settings WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Provider not found".to_string()));
    }

    info!("Deleted mail provider: id={}", id);
    Ok(Json(serde_json::json!({"status": "deleted"})))
}

async fn set_default_provider(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MailProviderResponse>> {
    require_admin(&auth)?;

    // Get the provider
    let provider: MailProviderSettings =
        sqlx::query_as("SELECT * FROM mail_provider_settings WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Provider not found".to_string()))?;

    // Clear other defaults for this tenant
    sqlx::query(
        "UPDATE mail_provider_settings SET is_default = false WHERE is_default = true AND tenant_id IS NOT DISTINCT FROM $1"
    )
    .bind(provider.tenant_id)
    .execute(&state.db)
    .await?;

    // Set this one as default
    let provider: MailProviderSettings = sqlx::query_as(
        "UPDATE mail_provider_settings SET is_default = true WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(provider.into()))
}

#[derive(Debug, Deserialize)]
struct TestProviderRequest {
    to_email: String,
}

async fn test_provider(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<TestProviderRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    // Get provider settings
    let settings: MailProviderSettings =
        sqlx::query_as("SELECT * FROM mail_provider_settings WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Provider not found".to_string()))?;

    // Create provider and test connection
    let provider = SmtpProvider::new(settings.clone(), &state.config.encryption_key)
        .await
        .map_err(|e| AppError::ExternalService(format!("Failed to create provider: {}", e)))?;

    // Test connection first
    if let Err(e) = provider.test_connection().await {
        return Ok(Json(serde_json::json!({
            "success": false,
            "stage": "connection",
            "error": e.to_string()
        })));
    }

    // Send test email
    let from = EmailAddress::with_name(&settings.from_address, &settings.from_name);
    let to = EmailAddress::new(&body.to_email);
    let content = EmailContent {
        subject: "RustChat Email Test".to_string(),
        body_text: format!(
            "This is a test email from RustChat.\n\nProvider: {}:{}\nTLS: {}\nSent at: {}",
            settings.host,
            settings.port,
            settings.tls_mode.as_str(),
            Utc::now()
        ),
        body_html: None,
        headers: vec![],
    };

    match provider.send_email(&from, &to, &content).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "success": true,
            "stage": "sent",
            "message": format!("Test email sent to {}", body.to_email),
            "server_response": result.server_response
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "stage": "sending",
            "error": e.to_string()
        }))),
    }
}

// ============================================
// Workflows
// ============================================

async fn list_workflows(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
) -> ApiResult<Json<Vec<WorkflowResponse>>> {
    require_admin(&auth)?;

    let workflows: Vec<NotificationWorkflow> = sqlx::query_as(
        r#"
        SELECT * FROM notification_workflows
        WHERE tenant_id IS NULL OR tenant_id = $1
        ORDER BY category, workflow_key
        "#,
    )
    .bind(auth.org_id)
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<WorkflowResponse> = workflows.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn get_workflow(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<WorkflowResponse>> {
    require_admin(&auth)?;

    let workflow: NotificationWorkflow =
        sqlx::query_as("SELECT * FROM notification_workflows WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Workflow not found".to_string()))?;

    Ok(Json(workflow.into()))
}

async fn update_workflow(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateWorkflowRequest>,
) -> ApiResult<Json<WorkflowResponse>> {
    require_admin(&auth)?;

    // Get existing to check if system required
    let existing: NotificationWorkflow =
        sqlx::query_as("SELECT * FROM notification_workflows WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Workflow not found".to_string()))?;

    // Don't allow disabling system required workflows
    if let Some(false) = body.enabled {
        if existing.system_required {
            return Err(AppError::Forbidden(
                "Cannot disable system-required workflow".to_string(),
            ));
        }
    }

    let policy_json = body.policy.map(sqlx::types::Json);

    let workflow: NotificationWorkflow = sqlx::query_as(
        r#"
        UPDATE notification_workflows SET
            enabled = COALESCE($2, enabled),
            default_locale = COALESCE($3, default_locale),
            selected_template_family_id = COALESCE($4, selected_template_family_id),
            policy_json = COALESCE($5, policy_json),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(body.enabled)
    .bind(&body.default_locale)
    .bind(body.selected_template_family_id)
    .bind(policy_json)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(workflow.into()))
}

// ============================================
// Template Families
// ============================================

async fn list_template_families(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
) -> ApiResult<Json<Vec<EmailTemplateFamily>>> {
    require_admin(&auth)?;

    let families: Vec<EmailTemplateFamily> = sqlx::query_as(
        r#"
        SELECT * FROM email_template_families
        WHERE tenant_id IS NULL OR tenant_id = $1
        ORDER BY key
        "#,
    )
    .bind(auth.org_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(families))
}

async fn get_template_family(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailTemplateFamily>> {
    require_admin(&auth)?;

    let family: EmailTemplateFamily =
        sqlx::query_as("SELECT * FROM email_template_families WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Template family not found".to_string()))?;

    Ok(Json(family))
}

async fn create_template_family(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Json(body): Json<CreateTemplateFamilyRequest>,
) -> ApiResult<Json<EmailTemplateFamily>> {
    require_admin(&auth)?;

    let family: EmailTemplateFamily = sqlx::query_as(
        r#"
        INSERT INTO email_template_families (tenant_id, key, name, description, workflow_key, is_system, created_by)
        VALUES ($1, $2, $3, $4, $5, false, $6)
        RETURNING *
        "#,
    )
    .bind(auth.org_id)
    .bind(&body.key)
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.workflow_key)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    info!(
        "Created template family: id={}, key={}",
        family.id, family.key
    );
    Ok(Json(family))
}

async fn update_template_family(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTemplateFamilyRequest>,
) -> ApiResult<Json<EmailTemplateFamily>> {
    require_admin(&auth)?;

    let family: EmailTemplateFamily = sqlx::query_as(
        r#"
        UPDATE email_template_families SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            updated_at = NOW()
        WHERE id = $1 AND is_system = false
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&body.name)
    .bind(&body.description)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Template family not found or is system".to_string()))?;

    Ok(Json(family))
}

async fn delete_template_family(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    let result =
        sqlx::query("DELETE FROM email_template_families WHERE id = $1 AND is_system = false")
            .bind(id)
            .execute(&state.db)
            .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Template family not found or is system".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({"status": "deleted"})))
}

// ============================================
// Template Versions
// ============================================

async fn list_template_versions(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(family_id): Path<Uuid>,
) -> ApiResult<Json<Vec<TemplateVersionResponse>>> {
    require_admin(&auth)?;

    let versions: Vec<EmailTemplateVersion> = sqlx::query_as(
        r#"
        SELECT * FROM email_template_versions
        WHERE family_id = $1
        ORDER BY locale, version DESC
        "#,
    )
    .bind(family_id)
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<TemplateVersionResponse> = versions.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn get_template_version(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(version_id): Path<Uuid>,
) -> ApiResult<Json<TemplateVersionResponse>> {
    require_admin(&auth)?;

    let version: EmailTemplateVersion =
        sqlx::query_as("SELECT * FROM email_template_versions WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Template version not found".to_string()))?;

    Ok(Json(version.into()))
}

async fn create_template_version(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(family_id): Path<Uuid>,
    Json(body): Json<CreateTemplateVersionRequest>,
) -> ApiResult<Json<TemplateVersionResponse>> {
    require_admin(&auth)?;

    // Get next version number for this locale
    let max_version: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(version) FROM email_template_versions WHERE family_id = $1 AND locale = $2",
    )
    .bind(family_id)
    .bind(&body.locale)
    .fetch_one(&state.db)
    .await?;

    let version = max_version.unwrap_or(0) + 1;

    let new_version: EmailTemplateVersion = sqlx::query_as(
        r#"
        INSERT INTO email_template_versions (
            family_id, version, status, locale, subject, body_text, body_html,
            variables_schema_json, is_compiled_from_mjml, mjml_source, created_by
        )
        VALUES ($1, $2, 'draft', $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(family_id)
    .bind(version)
    .bind(&body.locale)
    .bind(&body.subject)
    .bind(&body.body_text)
    .bind(&body.body_html)
    .bind(sqlx::types::Json(body.variables))
    .bind(body.is_compiled_from_mjml)
    .bind(body.mjml_source)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(new_version.into()))
}

async fn update_template_version(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(version_id): Path<Uuid>,
    Json(body): Json<UpdateTemplateVersionRequest>,
) -> ApiResult<Json<TemplateVersionResponse>> {
    require_admin(&auth)?;

    // Can only update draft versions
    let existing: EmailTemplateVersion =
        sqlx::query_as("SELECT * FROM email_template_versions WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Template version not found".to_string()))?;

    if existing.status != TemplateStatus::Draft {
        return Err(AppError::Forbidden(
            "Cannot edit published or archived versions".to_string(),
        ));
    }

    let variables_json = body.variables.map(sqlx::types::Json);

    let version: EmailTemplateVersion = sqlx::query_as(
        r#"
        UPDATE email_template_versions SET
            subject = COALESCE($2, subject),
            body_text = COALESCE($3, body_text),
            body_html = COALESCE($4, body_html),
            variables_schema_json = COALESCE($5, variables_schema_json),
            mjml_source = COALESCE($6, mjml_source)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(version_id)
    .bind(&body.subject)
    .bind(&body.body_text)
    .bind(&body.body_html)
    .bind(variables_json)
    .bind(body.mjml_source)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(version.into()))
}

async fn publish_template_version(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(version_id): Path<Uuid>,
) -> ApiResult<Json<TemplateVersionResponse>> {
    require_admin(&auth)?;

    let version: EmailTemplateVersion = sqlx::query_as(
        r#"
        UPDATE email_template_versions SET
            status = 'published',
            published_at = NOW(),
            published_by = $2
        WHERE id = $1 AND status = 'draft'
        RETURNING *
        "#,
    )
    .bind(version_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        AppError::NotFound("Template version not found or not in draft status".to_string())
    })?;

    info!(
        "Published template version: id={}, family_id={}, version={}",
        version_id, version.family_id, version.version
    );

    Ok(Json(version.into()))
}

#[derive(Debug, Deserialize)]
struct PreviewTemplateRequest {
    sample_data: Option<serde_json::Value>,
}

async fn preview_template(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(version_id): Path<Uuid>,
    Json(body): Json<PreviewTemplateRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    let version: EmailTemplateVersion =
        sqlx::query_as("SELECT * FROM email_template_versions WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Template version not found".to_string()))?;

    let renderer = TemplateRenderer::new();

    // Use provided sample data or build from schema
    let sample_data = body.sample_data.unwrap_or_else(|| {
        TemplateRenderer::build_sample_payload(&version.variables_schema_json.0)
    });

    match renderer.preview_template(&version, &sample_data) {
        Ok(rendered) => Ok(Json(serde_json::json!({
            "subject": rendered.subject,
            "body_text": rendered.body_text,
            "body_html": rendered.body_html,
            "sample_data_used": sample_data,
        }))),
        Err(e) => Err(AppError::BadRequest(format!(
            "Template render error: {}",
            e
        ))),
    }
}

#[derive(Debug, Deserialize)]
struct SendPreviewRequest {
    to_email: String,
    sample_data: Option<serde_json::Value>,
}

async fn send_preview_email(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(version_id): Path<Uuid>,
    Json(body): Json<SendPreviewRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    // Get default provider
    let provider_settings: Option<MailProviderSettings> = sqlx::query_as(
        r#"
        SELECT * FROM mail_provider_settings
        WHERE enabled = true AND is_default = true
        ORDER BY tenant_id NULLS LAST
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    let settings = provider_settings
        .ok_or_else(|| AppError::Config("No default mail provider configured".to_string()))?;

    // Get template
    let version: EmailTemplateVersion =
        sqlx::query_as("SELECT * FROM email_template_versions WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Template version not found".to_string()))?;

    // Render
    let renderer = TemplateRenderer::new();
    let sample_data = body.sample_data.unwrap_or_else(|| {
        TemplateRenderer::build_sample_payload(&version.variables_schema_json.0)
    });

    let rendered = renderer
        .preview_template(&version, &sample_data)
        .map_err(|e| AppError::BadRequest(format!("Template render error: {}", e)))?;

    // Send via provider
    let provider = SmtpProvider::new(settings.clone(), &state.config.encryption_key)
        .await
        .map_err(|e| AppError::ExternalService(format!("Provider error: {}", e)))?;

    let from = EmailAddress::with_name(&settings.from_address, &settings.from_name);
    let to = EmailAddress::new(&body.to_email);
    let content = EmailContent {
        subject: format!("[PREVIEW] {}", rendered.subject),
        body_text: rendered.body_text,
        body_html: rendered.body_html,
        headers: vec![("X-RustChat-Preview".to_string(), "true".to_string())],
    };

    match provider.send_email(&from, &to, &content).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Preview email sent to {}", body.to_email)
        }))),
        Err(e) => Err(AppError::ExternalService(format!("Failed to send: {}", e))),
    }
}

// ============================================
// Outbox
// ============================================

#[derive(Debug, Deserialize)]
struct ListOutboxQuery {
    status: Option<EmailStatus>,
    workflow_key: Option<String>,
    recipient_email: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}

async fn list_outbox(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Query(query): Query<ListOutboxQuery>,
) -> ApiResult<Json<Vec<EmailOutboxResponse>>> {
    require_admin(&auth)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let service = EmailService::new(state.db.clone());
    let filters = OutboxFilters {
        status: query.status,
        workflow_key: query.workflow_key,
        recipient_email: query.recipient_email,
        recipient_user_id: None,
    };

    let entries = service
        .list_outbox(filters, per_page, offset)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let responses: Vec<EmailOutboxResponse> = entries.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

async fn get_outbox_entry(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<EmailOutbox>> {
    require_admin(&auth)?;

    let service = EmailService::new(state.db.clone());
    let entry = service
        .get_outbox_entry(id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Outbox entry not found".to_string()))?;

    Ok(Json(entry))
}

async fn cancel_outbox_entry(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    let result = sqlx::query(
        "UPDATE email_outbox SET status = 'cancelled' WHERE id = $1 AND status = 'queued'",
    )
    .bind(id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "Email cannot be cancelled (may already be sent or failed)".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({"status": "cancelled"})))
}

async fn retry_outbox_entry(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    let result = sqlx::query(
        r#"
        UPDATE email_outbox SET 
            status = 'queued',
            attempt_count = 0,
            next_attempt_at = NULL,
            last_error_category = NULL,
            last_error_message = NULL
        WHERE id = $1 AND status = 'failed'
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "Email cannot be retried (may not be in failed status)".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({"status": "queued_for_retry"})))
}

// ============================================
// Email Events
// ============================================

#[derive(Debug, Deserialize)]
struct ListEventsQuery {
    outbox_id: Option<Uuid>,
    workflow_key: Option<String>,
    event_type: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}

async fn list_email_events(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Query(query): Query<ListEventsQuery>,
) -> ApiResult<Json<Vec<EmailEventResponse>>> {
    require_admin(&auth)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let events: Vec<EmailEvent> = sqlx::query_as(
        r#"
        SELECT * FROM email_events
        WHERE ($1::uuid IS NULL OR outbox_id = $1)
          AND ($2::varchar IS NULL OR workflow_key = $2)
          AND ($3::varchar IS NULL OR event_type = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(query.outbox_id)
    .bind(query.workflow_key)
    .bind(query.event_type)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<EmailEventResponse> = events.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

// ============================================
// Send Test Email
// ============================================

#[derive(Debug, Deserialize)]
struct SendTestEmailRequest {
    provider_id: Option<Uuid>,
    to_email: String,
    workflow_key: Option<String>,
    #[allow(dead_code)]
    template_family_id: Option<Uuid>,
    locale: Option<String>,
    subject: Option<String>,
    body_text: Option<String>,
}

async fn send_test_email(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Json(body): Json<SendTestEmailRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    let service = EmailService::new(state.db.clone());

    // If workflow and template specified, use template rendering
    if let Some(workflow_key) = body.workflow_key {
        let options = EnqueueOptions {
            locale: body.locale,
            priority: EmailPriority::High,
            created_by: Some(auth.user_id),
            ..Default::default()
        };

        let payload = serde_json::json!({
            "user_name": "Test User",
            "email": body.to_email,
            "site_name": "RustChat",
            "verification_link": "https://example.com/verify?token=test",
            "reset_link": "https://example.com/reset?token=test",
            "channel_name": "general",
            "message_count": 5,
        });

        let outbox_id = service
            .enqueue_email(
                &workflow_key,
                &body.to_email,
                None, // No user_id for test
                payload,
                options,
            )
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        return Ok(Json(serde_json::json!({
            "success": true,
            "outbox_id": outbox_id,
            "message": format!("Test email enqueued: {}", outbox_id)
        })));
    }

    // Otherwise, send simple test via provider
    let provider_settings: Option<MailProviderSettings> = if let Some(id) = body.provider_id {
        sqlx::query_as("SELECT * FROM mail_provider_settings WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM mail_provider_settings WHERE enabled = true AND is_default = true LIMIT 1"
        )
        .fetch_optional(&state.db)
        .await?
    };

    let settings =
        provider_settings.ok_or_else(|| AppError::Config("No mail provider found".to_string()))?;

    let provider = SmtpProvider::new(settings.clone(), &state.config.encryption_key)
        .await
        .map_err(|e| AppError::ExternalService(format!("Provider error: {}", e)))?;

    let from = EmailAddress::with_name(&settings.from_address, &settings.from_name);
    let to = EmailAddress::new(&body.to_email);
    let content = EmailContent {
        subject: body
            .subject
            .unwrap_or_else(|| "RustChat Test Email".to_string()),
        body_text: body
            .body_text
            .unwrap_or_else(|| "This is a test email from RustChat.".to_string()),
        body_html: None,
        headers: vec![],
    };

    match provider.send_email(&from, &to, &content).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "success": true,
            "server_response": result.server_response,
            "message": format!("Test email sent to {}", body.to_email)
        }))),
        Err(e) => Err(AppError::ExternalService(format!("Failed to send: {}", e))),
    }
}

// ============================================
// User Preferences (Admin)
// ============================================

async fn get_user_prefs(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Json<UserNotificationPrefsResponse>> {
    require_admin(&auth)?;

    let service = EmailService::new(state.db.clone());
    let prefs = service
        .get_user_prefs(user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(prefs.into()))
}

async fn update_user_prefs(
    State(state): State<AppState>,
    auth: crate::auth::AuthUser,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateNotificationPrefsRequest>,
) -> ApiResult<Json<UserNotificationPrefsResponse>> {
    require_admin(&auth)?;

    let service = EmailService::new(state.db.clone());
    let prefs = service
        .update_user_prefs(user_id, body)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(prefs.into()))
}
