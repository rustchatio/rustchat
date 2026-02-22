//! Email service using lettre
//!
//! Handles sending emails via SMTP based on server configuration.
//! Supports TLS, STARTTLS, and plain connections.

use lettre::{
    transport::smtp::authentication::Credentials,
    transport::smtp::client::{Tls, TlsParameters},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::{error, info, warn};

use crate::models::server_config::EmailConfig;

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmailSendResult {
    pub accepted: bool,
    pub server_response: Option<String>,
    pub message_id: Option<String>,
}

/// Send an email using the provided configuration
pub async fn send_email(
    config: &EmailConfig,
    to_address: &str,
    subject: &str,
    body: &str,
) -> Result<EmailSendResult, String> {
    // Check if email notifications are enabled
    if !config.send_email_notifications {
        return Err("Email notifications are disabled".to_string());
    }

    if config.smtp_host.is_empty() {
        return Err("SMTP host not configured".to_string());
    }

    let mut builder = Message::builder()
        .from(
            format!("{} <{}>", config.from_name, config.from_address)
                .parse()
                .map_err(|e| format!("Invalid from address: {}", e))?,
        )
        .to(to_address
            .parse()
            .map_err(|e| format!("Invalid to address: {}", e))?)
        .subject(subject);

    if !config.reply_to.trim().is_empty() {
        builder = builder.reply_to(
            config.reply_to
                .parse()
                .map_err(|e| format!("Invalid reply-to address: {}", e))?,
        );
    }

    let email = builder
        .body(body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(
        config.smtp_username.clone(),
        config.smtp_password_encrypted.clone(),
    );

    // Build SMTP transport based on security settings
    let mailer = build_smtp_transport(config, creds).await?;

    // Send the email
    match mailer.send(email).await {
        Ok(response) => {
            info!("Email sent successfully to {}", to_address);
            Ok(EmailSendResult {
                accepted: true,
                server_response: Some(format!("{:?}", response)),
                message_id: None,
            })
        }
        Err(e) => {
            error!("Failed to send email to {}: {}", to_address, e);
            Err(format!("Failed to send email: {}", e))
        }
    }
}

/// Test SMTP connection without sending an email
pub async fn test_smtp_connection(config: &EmailConfig) -> Result<(), String> {
    if config.smtp_host.is_empty() {
        return Err("SMTP host not configured".to_string());
    }

    let creds = Credentials::new(
        config.smtp_username.clone(),
        config.smtp_password_encrypted.clone(),
    );

    let mailer = build_smtp_transport(config, creds).await?;

    match mailer.test_connection().await {
        Ok(true) => {
            info!("SMTP connection test successful for {}", config.smtp_host);
            Ok(())
        }
        Ok(false) => {
            warn!(
                "SMTP connection test returned false for {}",
                config.smtp_host
            );
            Err("SMTP connection test failed".to_string())
        }
        Err(e) => {
            error!("SMTP connection test error for {}: {}", config.smtp_host, e);
            Err(format!("SMTP connection test error: {}", e))
        }
    }
}

/// Build SMTP transport based on configuration
async fn build_smtp_transport(
    config: &EmailConfig,
    creds: Credentials,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let host = &config.smtp_host;
    let port = config.smtp_port as u16;

    // Build TLS parameters
    let tls_params = if config.smtp_skip_cert_verify {
        TlsParameters::builder(host.clone())
            .dangerous_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to build TLS parameters: {}", e))?
    } else {
        TlsParameters::new(host.clone())
            .map_err(|e| format!("Failed to build TLS parameters: {}", e))?
    };

    let mailer: AsyncSmtpTransport<Tokio1Executor> = match config.smtp_security.as_str() {
        "tls" => {
            // Direct TLS connection (SMTPS/SSL)
            info!(
                "Building SMTP transport with TLS (SMTPS) for {}:{}",
                host, port
            );
            AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| format!("Failed to create TLS transport: {}", e))?
                .credentials(creds)
                .port(port)
                .tls(Tls::Required(tls_params))
                .build()
        }
        "none" => {
            // No encryption (plaintext)
            warn!(
                "Building SMTP transport without encryption for {}:{}",
                host, port
            );
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                .port(port)
                .credentials(creds)
                .build()
        }
        _ => {
            // Default: STARTTLS (upgrade to TLS after connection)
            info!(
                "Building SMTP transport with STARTTLS for {}:{}",
                host, port
            );
            AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| format!("Failed to create STARTTLS transport: {}", e))?
                .credentials(creds)
                .port(port)
                .tls(Tls::Required(tls_params))
                .build()
        }
    };

    Ok(mailer)
}

/// Get email notification settings for client config
pub fn get_email_settings(config: &EmailConfig) -> serde_json::Value {
    serde_json::json!({
        "send_email_notifications": config.send_email_notifications,
        "enable_email_batching": config.enable_email_batching,
        "email_batching_interval": config.email_batching_interval,
        "email_notification_content": config.email_notification_content,
        "smtp_host_configured": !config.smtp_host.is_empty(),
        "from_address_configured": !config.from_address.is_empty(),
    })
}
