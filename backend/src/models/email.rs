//! Email subsystem models
//!
//! Provides comprehensive data structures for the email notification system
//! including provider settings, templates, workflows, outbox, and events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Provider Settings
// ============================================

/// Mail provider types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MailProviderType {
    Smtp,
    Ses,
    Sendgrid,
}

impl MailProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MailProviderType::Smtp => "smtp",
            MailProviderType::Ses => "ses",
            MailProviderType::Sendgrid => "sendgrid",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "smtp" => Some(MailProviderType::Smtp),
            "ses" => Some(MailProviderType::Ses),
            "sendgrid" => Some(MailProviderType::Sendgrid),
            _ => None,
        }
    }
}

/// TLS mode for SMTP connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TlsMode {
    Starttls,
    #[serde(rename = "implicit_tls")]
    ImplicitTls,
    None,
}

impl TlsMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TlsMode::Starttls => "starttls",
            TlsMode::ImplicitTls => "implicit_tls",
            TlsMode::None => "none",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "starttls" => Some(TlsMode::Starttls),
            "implicit_tls" | "tls" => Some(TlsMode::ImplicitTls),
            "none" => Some(TlsMode::None),
            _ => None,
        }
    }
}

/// Mail provider settings entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailProviderSettings {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub provider_type: MailProviderType,

    // SMTP Configuration
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password_encrypted: String,
    pub tls_mode: TlsMode,
    pub skip_cert_verify: bool,

    // Sender Configuration
    pub from_address: String,
    pub from_name: String,
    pub reply_to: Option<String>,

    // Rate Limiting
    pub max_emails_per_minute: i32,
    pub max_emails_per_hour: i32,

    // Status
    pub enabled: bool,
    pub is_default: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// DTO for creating/updating mail provider settings
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMailProviderRequest {
    pub provider_type: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String, // Will be encrypted
    pub tls_mode: String,
    #[serde(default)]
    pub skip_cert_verify: bool,
    pub from_address: String,
    pub from_name: String,
    pub reply_to: Option<String>,
    #[serde(default = "default_rate_limit_minute")]
    pub max_emails_per_minute: i32,
    #[serde(default = "default_rate_limit_hour")]
    pub max_emails_per_hour: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMailProviderRequest {
    pub provider_type: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>, // Will be encrypted if provided
    pub tls_mode: Option<String>,
    pub skip_cert_verify: Option<bool>,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
    pub max_emails_per_minute: Option<i32>,
    pub max_emails_per_hour: Option<i32>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

fn default_rate_limit_minute() -> i32 {
    60
}
fn default_rate_limit_hour() -> i32 {
    1000
}
fn default_enabled() -> bool {
    true
}

/// Response DTO (without sensitive fields)
#[derive(Debug, Clone, Serialize)]
pub struct MailProviderResponse {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub provider_type: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub has_password: bool,
    pub tls_mode: String,
    pub skip_cert_verify: bool,
    pub from_address: String,
    pub from_name: String,
    pub reply_to: Option<String>,
    pub max_emails_per_minute: i32,
    pub max_emails_per_hour: i32,
    pub enabled: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MailProviderSettings> for MailProviderResponse {
    fn from(settings: MailProviderSettings) -> Self {
        Self {
            id: settings.id,
            tenant_id: settings.tenant_id,
            provider_type: settings.provider_type.as_str().to_string(),
            host: settings.host,
            port: settings.port,
            username: settings.username,
            has_password: !settings.password_encrypted.is_empty(),
            tls_mode: settings.tls_mode.as_str().to_string(),
            skip_cert_verify: settings.skip_cert_verify,
            from_address: settings.from_address,
            from_name: settings.from_name,
            reply_to: settings.reply_to,
            max_emails_per_minute: settings.max_emails_per_minute,
            max_emails_per_hour: settings.max_emails_per_hour,
            enabled: settings.enabled,
            is_default: settings.is_default,
            created_at: settings.created_at,
            updated_at: settings.updated_at,
        }
    }
}

// ============================================
// Notification Workflows
// ============================================

/// Fixed workflow keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKey {
    UserRegistration,
    EmailVerification,
    PasswordReset,
    PasswordChanged,
    SecurityAlert,
    Announcements,
    OfflineMessages,
    MentionNotifications,
    AdminInvite,
    WeeklyDigest,
}

impl WorkflowKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowKey::UserRegistration => "user_registration",
            WorkflowKey::EmailVerification => "email_verification",
            WorkflowKey::PasswordReset => "password_reset",
            WorkflowKey::PasswordChanged => "password_changed",
            WorkflowKey::SecurityAlert => "security_alert",
            WorkflowKey::Announcements => "announcements",
            WorkflowKey::OfflineMessages => "offline_messages",
            WorkflowKey::MentionNotifications => "mention_notifications",
            WorkflowKey::AdminInvite => "admin_invite",
            WorkflowKey::WeeklyDigest => "weekly_digest",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user_registration" => Some(WorkflowKey::UserRegistration),
            "email_verification" => Some(WorkflowKey::EmailVerification),
            "password_reset" => Some(WorkflowKey::PasswordReset),
            "password_changed" => Some(WorkflowKey::PasswordChanged),
            "security_alert" => Some(WorkflowKey::SecurityAlert),
            "announcements" => Some(WorkflowKey::Announcements),
            "offline_messages" => Some(WorkflowKey::OfflineMessages),
            "mention_notifications" => Some(WorkflowKey::MentionNotifications),
            "admin_invite" => Some(WorkflowKey::AdminInvite),
            "weekly_digest" => Some(WorkflowKey::WeeklyDigest),
            _ => None,
        }
    }

    /// Whether this workflow is required (cannot be disabled)
    pub fn is_required(&self) -> bool {
        matches!(
            self,
            WorkflowKey::UserRegistration
                | WorkflowKey::EmailVerification
                | WorkflowKey::PasswordReset
                | WorkflowKey::SecurityAlert
        )
    }

    /// Default category for the workflow
    pub fn category(&self) -> &'static str {
        match self {
            WorkflowKey::UserRegistration
            | WorkflowKey::EmailVerification
            | WorkflowKey::PasswordReset
            | WorkflowKey::PasswordChanged
            | WorkflowKey::SecurityAlert => "system",
            WorkflowKey::Announcements => "marketing",
            _ => "notification",
        }
    }
}

/// Workflow policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowPolicy {
    /// For password reset: token expiry in hours
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expiry_hours: Option<i32>,
    /// For announcements: require opt-in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_opt_in: Option<bool>,
    /// For announcements: include list-unsubscribe header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_unsubscribe: Option<bool>,
    /// For offline messages: throttle interval in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle_minutes: Option<i32>,
    /// For offline messages: max per hour per user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_hour: Option<i32>,
    /// For offline messages: include message excerpt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_excerpt: Option<bool>,
    /// For offline messages: respect quiet hours
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respect_quiet_hours: Option<bool>,
    /// For digest: day of week (0=Sunday)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_of_week: Option<i32>,
    /// For digest: hour to send
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hour: Option<i32>,
}

/// Notification workflow entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationWorkflow {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub workflow_key: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub enabled: bool,
    pub system_required: bool,
    pub default_locale: String,
    pub selected_template_family_id: Option<Uuid>,
    pub policy_json: sqlx::types::Json<WorkflowPolicy>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkflowRequest {
    pub workflow_key: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub enabled: Option<bool>,
    pub system_required: Option<bool>,
    pub default_locale: Option<String>,
    pub selected_template_family_id: Option<Uuid>,
    pub policy: Option<WorkflowPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub enabled: Option<bool>,
    pub default_locale: Option<String>,
    pub selected_template_family_id: Option<Uuid>,
    pub policy: Option<WorkflowPolicy>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub workflow_key: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub enabled: bool,
    pub system_required: bool,
    pub can_disable: bool,
    pub default_locale: String,
    pub selected_template_family_id: Option<Uuid>,
    pub policy: WorkflowPolicy,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NotificationWorkflow> for WorkflowResponse {
    fn from(wf: NotificationWorkflow) -> Self {
        let can_disable = !wf.system_required;
        Self {
            id: wf.id,
            tenant_id: wf.tenant_id,
            workflow_key: wf.workflow_key.clone(),
            name: wf.name,
            description: wf.description,
            category: wf.category,
            enabled: wf.enabled,
            system_required: wf.system_required,
            can_disable,
            default_locale: wf.default_locale,
            selected_template_family_id: wf.selected_template_family_id,
            policy: wf.policy_json.0,
            created_at: wf.created_at,
            updated_at: wf.updated_at,
        }
    }
}

// ============================================
// Email Templates
// ============================================

/// Template status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TemplateStatus {
    Draft,
    Published,
    Archived,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Email template family entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailTemplateFamily {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub workflow_key: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateFamilyRequest {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub workflow_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateFamilyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Email template version entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailTemplateVersion {
    pub id: Uuid,
    pub family_id: Uuid,
    pub version: i32,
    pub status: TemplateStatus,
    pub locale: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: String,
    pub variables_schema_json: sqlx::types::Json<Vec<TemplateVariable>>,
    pub is_compiled_from_mjml: bool,
    pub mjml_source: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateVersionRequest {
    pub locale: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: String,
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
    #[serde(default)]
    pub is_compiled_from_mjml: bool,
    pub mjml_source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateVersionRequest {
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub variables: Option<Vec<TemplateVariable>>,
    pub mjml_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateVersionResponse {
    pub id: Uuid,
    pub family_id: Uuid,
    pub version: i32,
    pub status: String,
    pub locale: String,
    pub subject: String,
    pub body_text: Option<String>, // May be omitted in listings
    pub body_html: Option<String>,
    pub variables: Vec<TemplateVariable>,
    pub is_compiled_from_mjml: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<Uuid>,
}

impl From<EmailTemplateVersion> for TemplateVersionResponse {
    fn from(tv: EmailTemplateVersion) -> Self {
        Self {
            id: tv.id,
            family_id: tv.family_id,
            version: tv.version,
            status: match tv.status {
                TemplateStatus::Draft => "draft".to_string(),
                TemplateStatus::Published => "published".to_string(),
                TemplateStatus::Archived => "archived".to_string(),
            },
            locale: tv.locale,
            subject: tv.subject,
            body_text: Some(tv.body_text),
            body_html: Some(tv.body_html),
            variables: tv.variables_schema_json.0,
            is_compiled_from_mjml: tv.is_compiled_from_mjml,
            created_at: tv.created_at,
            created_by: tv.created_by,
            published_at: tv.published_at,
            published_by: tv.published_by,
        }
    }
}

// ============================================
// Email Outbox
// ============================================

/// Email status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "email_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EmailStatus {
    Queued,
    Sending,
    Sent,
    Failed,
    Cancelled,
}

/// Email priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "email_priority", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EmailPriority {
    High,
    #[default]
    Normal,
    Low,
}

/// Email outbox entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailOutbox {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub workflow_key: Option<String>,
    pub template_family_id: Option<Uuid>,
    pub template_version: Option<i32>,
    pub locale: Option<String>,
    pub recipient_email: String,
    pub recipient_user_id: Option<Uuid>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub payload_json: sqlx::types::Json<serde_json::Value>,
    pub headers_json: Option<sqlx::types::Json<serde_json::Value>>,
    pub status: EmailStatus,
    pub priority: EmailPriority,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub send_after: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_error_category: Option<String>,
    pub last_error_message: Option<String>,
    pub provider_id: Option<Uuid>,
    pub provider_message_id: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub throttle_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub source_ip: Option<String>,
    pub source_service: Option<String>,
}

/// Request to enqueue an email
#[derive(Debug, Clone, Deserialize)]
pub struct EnqueueEmailRequest {
    pub workflow_key: String,
    pub recipient_email: String,
    pub recipient_user_id: Option<Uuid>,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub priority: EmailPriority,
    #[serde(default)]
    pub scheduled_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub headers: Option<serde_json::Value>,
    #[serde(default)]
    pub throttle_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailOutboxResponse {
    pub id: Uuid,
    pub workflow_key: Option<String>,
    pub recipient_email: String,
    pub recipient_user_id: Option<Uuid>,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<EmailOutbox> for EmailOutboxResponse {
    fn from(e: EmailOutbox) -> Self {
        Self {
            id: e.id,
            workflow_key: e.workflow_key,
            recipient_email: e.recipient_email,
            recipient_user_id: e.recipient_user_id,
            subject: e.subject,
            status: match e.status {
                EmailStatus::Queued => "queued".to_string(),
                EmailStatus::Sending => "sending".to_string(),
                EmailStatus::Sent => "sent".to_string(),
                EmailStatus::Failed => "failed".to_string(),
                EmailStatus::Cancelled => "cancelled".to_string(),
            },
            priority: match e.priority {
                EmailPriority::High => "high".to_string(),
                EmailPriority::Normal => "normal".to_string(),
                EmailPriority::Low => "low".to_string(),
            },
            attempt_count: e.attempt_count,
            max_attempts: e.max_attempts,
            sent_at: e.sent_at,
            created_at: e.created_at,
        }
    }
}

// ============================================
// Email Events (Audit)
// ============================================

/// Email event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailEventType {
    Queued,
    Sent,
    Delivered,
    Bounced,
    Failed,
    Opened,
    Clicked,
    Unsubscribed,
    Complained,
}

impl EmailEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailEventType::Queued => "queued",
            EmailEventType::Sent => "sent",
            EmailEventType::Delivered => "delivered",
            EmailEventType::Bounced => "bounced",
            EmailEventType::Failed => "failed",
            EmailEventType::Opened => "opened",
            EmailEventType::Clicked => "clicked",
            EmailEventType::Unsubscribed => "unsubscribed",
            EmailEventType::Complained => "complained",
        }
    }
}

/// Email event entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailEvent {
    pub id: Uuid,
    pub outbox_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub workflow_key: Option<String>,
    pub event_type: String,
    pub recipient_email: String,
    pub recipient_user_id: Option<Uuid>,
    pub template_family_id: Option<Uuid>,
    pub template_version: Option<i32>,
    pub locale: Option<String>,
    pub provider_id: Option<Uuid>,
    pub provider_message_id: Option<String>,
    pub status_code: Option<i32>,
    pub error_category: Option<String>,
    pub error_message: Option<String>,
    pub provider_response: Option<sqlx::types::Json<serde_json::Value>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailEventResponse {
    pub id: Uuid,
    pub outbox_id: Option<Uuid>,
    pub workflow_key: Option<String>,
    pub event_type: String,
    pub recipient_email: String,
    pub template_version: Option<i32>,
    pub locale: Option<String>,
    pub status_code: Option<i32>,
    pub error_category: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<EmailEvent> for EmailEventResponse {
    fn from(e: EmailEvent) -> Self {
        Self {
            id: e.id,
            outbox_id: e.outbox_id,
            workflow_key: e.workflow_key,
            event_type: e.event_type,
            recipient_email: e.recipient_email,
            template_version: e.template_version,
            locale: e.locale,
            status_code: e.status_code,
            error_category: e.error_category,
            error_message: e.error_message,
            created_at: e.created_at,
        }
    }
}

// ============================================
// User Notification Preferences
// ============================================

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuietHoursConfig {
    pub enabled: bool,
    pub start: String, // "HH:MM" format
    pub end: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// User notification preferences entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserNotificationPrefs {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub announcements_opt_in: Option<bool>,
    pub offline_notifications_opt_in: Option<bool>,
    pub mention_notifications_opt_in: Option<bool>,
    pub digest_opt_in: Option<bool>,
    pub quiet_hours_json: Option<sqlx::types::Json<QuietHoursConfig>>,
    pub locale: Option<String>,
    pub offline_throttle_minutes: Option<i32>,
    pub include_message_content: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNotificationPrefsRequest {
    pub email_enabled: Option<bool>,
    pub announcements_opt_in: Option<bool>,
    pub offline_notifications_opt_in: Option<bool>,
    pub mention_notifications_opt_in: Option<bool>,
    pub digest_opt_in: Option<bool>,
    pub quiet_hours: Option<QuietHoursConfig>,
    pub locale: Option<String>,
    pub offline_throttle_minutes: Option<i32>,
    pub include_message_content: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserNotificationPrefsResponse {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub announcements_opt_in: Option<bool>,
    pub offline_notifications_opt_in: Option<bool>,
    pub mention_notifications_opt_in: Option<bool>,
    pub digest_opt_in: Option<bool>,
    pub quiet_hours: Option<QuietHoursConfig>,
    pub locale: Option<String>,
    pub offline_throttle_minutes: Option<i32>,
    pub include_message_content: bool,
}

impl From<UserNotificationPrefs> for UserNotificationPrefsResponse {
    fn from(p: UserNotificationPrefs) -> Self {
        Self {
            user_id: p.user_id,
            email_enabled: p.email_enabled,
            announcements_opt_in: p.announcements_opt_in,
            offline_notifications_opt_in: p.offline_notifications_opt_in,
            mention_notifications_opt_in: p.mention_notifications_opt_in,
            digest_opt_in: p.digest_opt_in,
            quiet_hours: p.quiet_hours_json.map(|j| j.0),
            locale: p.locale,
            offline_throttle_minutes: p.offline_throttle_minutes,
            include_message_content: p.include_message_content,
        }
    }
}

// ============================================
// Test Email
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct TestEmailProviderRequest {
    pub provider_id: Option<Uuid>, // Use default if not specified
    pub to_email: String,
    pub subject: Option<String>,
    pub use_template: Option<bool>, // If true, uses a template; otherwise simple test
    pub template_family_id: Option<Uuid>,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestEmailResult {
    pub success: bool,
    pub message: String,
    pub outbox_id: Option<Uuid>,
    pub provider_id: Uuid,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub error_category: Option<String>,
}
