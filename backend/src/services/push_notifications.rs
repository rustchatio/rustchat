//! Push Notification Service
//!
//! Handles sending push notifications to mobile devices via:
//! - FCM (Firebase Cloud Messaging) for Android
//! - APNS (Apple Push Notification Service) for iOS
//!
//! This service is essential for mattermost-mobile to receive:
//! - Call ringing notifications when app is in background
//! - Message notifications for mentions and direct messages

use std::collections::HashMap;
use serde::Serialize;
use uuid::Uuid;
use tracing::{debug, error, info, warn};

use crate::api::AppState;

/// Push notification payload
#[derive(Debug, Clone, Serialize)]
pub struct PushNotification {
    /// Target device token
    pub device_token: String,
    /// Platform (ios/android)
    pub platform: String,
    /// Notification title
    pub title: String,
    /// Notification body
    pub body: String,
    /// Custom data payload
    pub data: serde_json::Value,
    /// Priority (high/normal)
    pub priority: PushPriority,
    /// Sound to play
    pub sound: Option<String>,
    /// Badge count
    pub badge: Option<i32>,
    /// Category for iOS
    pub category: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum PushPriority {
    High,
    Normal,
}

impl PushPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            PushPriority::High => "high",
            PushPriority::Normal => "normal",
        }
    }
}

/// FCM message structure (HTTP v1 API)
#[derive(Debug, Serialize)]
struct FcmMessage {
    message: FcmMessageBody,
}

#[derive(Debug, Serialize)]
struct FcmMessageBody {
    token: String,
    notification: Option<FcmNotification>,
    data: Option<HashMap<String, String>>,
    android: Option<FcmAndroidConfig>,
    apns: Option<FcmApnsConfig>,
}

#[derive(Debug, Serialize)]
struct FcmNotification {
    title: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct FcmAndroidConfig {
    priority: String,
    notification: FcmAndroidNotification,
}

#[derive(Debug, Serialize)]
struct FcmAndroidNotification {
    channel_id: String,
    sound: String,
    priority: String,
}

#[derive(Debug, Serialize)]
struct FcmApnsConfig {
    headers: HashMap<String, String>,
    payload: FcmApnsPayload,
}

#[derive(Debug, Serialize)]
struct FcmApnsPayload {
    aps: FcmAps,
}

#[derive(Debug, Serialize)]
struct FcmAps {
    alert: FcmAlert,
    badge: i32,
    sound: String,
    #[serde(rename = "content-available")]
    content_available: i32,
    #[serde(rename = "mutable-content")]
    mutable_content: i32,
}

#[derive(Debug, Serialize)]
struct FcmAlert {
    title: String,
    body: String,
}

/// Send push notification to a specific device
pub async fn send_push_notification(
    state: &AppState,
    notification: PushNotification,
) -> Result<(), PushNotificationError> {
    debug!(
        platform = %notification.platform,
        priority = ?notification.priority,
        "Sending push notification"
    );

    // Check if FCM is configured
    let fcm_config = match get_fcm_config(state).await {
        Some(config) => config,
        None => {
            warn!("FCM not configured, skipping push notification");
            return Err(PushNotificationError::NotConfigured);
        }
    };

    // Build FCM message
    let fcm_message = build_fcm_message(&notification);

    // Send via FCM HTTP v1 API
    let result = send_fcm_message(&fcm_config, &fcm_message).await;

    match &result {
        Ok(_) => {
            debug!("Push notification sent successfully");
        }
        Err(e) => {
            error!(error = %e, "Failed to send push notification");
        }
    }

    result
}

/// Send push notification to multiple devices for a user
pub async fn send_push_to_user(
    state: &AppState,
    user_id: Uuid,
    title: String,
    body: String,
    data: serde_json::Value,
    priority: PushPriority,
) -> Result<usize, PushNotificationError> {
    // Get user's devices
    let devices = get_user_devices(state, user_id).await?;

    if devices.is_empty() {
        debug!(user_id = %user_id, "No devices found for user, skipping push notification");
        return Ok(0);
    }

    let mut sent_count = 0;

    for device in &devices {
        let notification = PushNotification {
            device_token: device.token.clone(),
            platform: device.platform.clone(),
            title: title.clone(),
            body: body.clone(),
            data: data.clone(),
            priority: priority,
            sound: Some("default".to_string()),
            badge: None,
            category: None,
        };

        match send_push_notification(state, notification).await {
            Ok(_) => sent_count += 1,
            Err(PushNotificationError::NotConfigured) => {
                // FCM not configured, skip silently
                return Ok(0);
            }
            Err(e) => {
                error!(user_id = %user_id, error = %e, "Failed to send push to device");
            }
        }
    }

    info!(
        user_id = %user_id,
        device_count = devices.len(),
        sent_count = sent_count,
        "Sent push notifications to user devices"
    );

    Ok(sent_count)
}

/// Send call ringing notification to a user
pub async fn send_call_ringing_notification(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
    call_id: Uuid,
    caller_name: String,
) -> Result<usize, PushNotificationError> {
    let title = format!("Incoming call from {}", caller_name);
    let body = "Tap to answer".to_string();

    let data = serde_json::json!({
        "type": "call_ringing",
        "channel_id": channel_id.to_string(),
        "call_id": call_id.to_string(),
        "caller_name": caller_name,
    });

    send_push_to_user(
        state,
        user_id,
        title,
        body,
        data,
        PushPriority::High, // High priority for calls
    )
    .await
}

/// Send message notification to a user
pub async fn send_message_notification(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
    channel_name: String,
    sender_name: String,
    message: String,
    is_dm: bool,
) -> Result<usize, PushNotificationError> {
    let (title, body) = if is_dm {
        (sender_name.clone(), message)
    } else {
        (format!("{} in {}", sender_name, channel_name), message)
    };

    let data = serde_json::json!({
        "type": "message",
        "channel_id": channel_id.to_string(),
        "sender_name": sender_name,
    });

    send_push_to_user(
        state,
        user_id,
        title,
        body,
        data,
        PushPriority::Normal,
    )
    .await
}

/// Device info from database
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub token: String,
    pub platform: String,
}

/// Get all devices for a user
async fn get_user_devices(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<DeviceInfo>, PushNotificationError> {
    let devices: Vec<(String, String)> = sqlx::query_as(
        "SELECT token, platform FROM user_devices WHERE user_id = $1 AND token IS NOT NULL"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| PushNotificationError::DatabaseError(e.to_string()))?;

    Ok(devices
        .into_iter()
        .map(|(token, platform)| DeviceInfo { token, platform })
        .collect())
}

/// FCM configuration
#[derive(Debug, Clone)]
struct FcmConfig {
    project_id: String,
    access_token: String,
}

/// Get FCM configuration from database or environment
async fn get_fcm_config(state: &AppState) -> Option<FcmConfig> {
    // Try to get from database first
    let config: Option<(String, String)> = sqlx::query_as(
        "SELECT fcm_project_id, fcm_access_token FROM server_config WHERE id = 'default'"
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some((project_id, access_token)) = config {
        if !project_id.is_empty() && !access_token.is_empty() {
            return Some(FcmConfig {
                project_id,
                access_token,
            });
        }
    }

    // Fall back to environment variables
    let project_id = std::env::var("FCM_PROJECT_ID").ok()?;
    let access_token = std::env::var("FCM_ACCESS_TOKEN").ok()?;

    if project_id.is_empty() || access_token.is_empty() {
        return None;
    }

    Some(FcmConfig {
        project_id,
        access_token,
    })
}

/// Build FCM message from notification
fn build_fcm_message(notification: &PushNotification) -> FcmMessage {
    let is_high_priority = matches!(notification.priority, PushPriority::High);

    let mut data_map = HashMap::new();
    if let serde_json::Value::Object(map) = &notification.data {
        for (key, value) in map {
            if let Some(str_val) = value.as_str() {
                data_map.insert(key.clone(), str_val.to_string());
            } else {
                data_map.insert(key.clone(), value.to_string());
            }
        }
    }

    let android_config = if is_high_priority {
        Some(FcmAndroidConfig {
            priority: "high".to_string(),
            notification: FcmAndroidNotification {
                channel_id: "calls".to_string(),
                sound: notification.sound.clone().unwrap_or_else(|| "default".to_string()),
                priority: "high".to_string(),
            },
        })
    } else {
        Some(FcmAndroidConfig {
            priority: "normal".to_string(),
            notification: FcmAndroidNotification {
                channel_id: "messages".to_string(),
                sound: notification.sound.clone().unwrap_or_else(|| "default".to_string()),
                priority: "default".to_string(),
            },
        })
    };

    let mut apns_headers = HashMap::new();
    apns_headers.insert("apns-priority".to_string(), if is_high_priority { "10".to_string() } else { "5".to_string() });

    let apns_config = FcmApnsConfig {
        headers: apns_headers,
        payload: FcmApnsPayload {
            aps: FcmAps {
                alert: FcmAlert {
                    title: notification.title.clone(),
                    body: notification.body.clone(),
                },
                badge: notification.badge.unwrap_or(1),
                sound: notification.sound.clone().unwrap_or_else(|| "default".to_string()),
                content_available: 1,
                mutable_content: 1,
            },
        },
    };

    FcmMessage {
        message: FcmMessageBody {
            token: notification.device_token.clone(),
            notification: Some(FcmNotification {
                title: notification.title.clone(),
                body: notification.body.clone(),
            }),
            data: if data_map.is_empty() { None } else { Some(data_map) },
            android: android_config,
            apns: Some(apns_config),
        },
    }
}

/// Send message via FCM HTTP v1 API
async fn send_fcm_message(
    config: &FcmConfig,
    message: &FcmMessage,
) -> Result<(), PushNotificationError> {
    let url = format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send",
        config.project_id
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.access_token))
        .header("Content-Type", "application/json")
        .json(message)
        .send()
        .await
        .map_err(|e| PushNotificationError::NetworkError(e.to_string()))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());

    if status.is_success() {
        debug!(response = %response_text, "FCM message sent successfully");
        Ok(())
    } else {
        error!(
            status = %status,
            response = %response_text,
            "FCM API error"
        );
        Err(PushNotificationError::FcmError(format!(
            "HTTP {}: {}",
            status, response_text
        )))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PushNotificationError {
    #[error("Push notifications not configured")]
    NotConfigured,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("FCM API error: {0}")]
    FcmError(String),

    #[error("Invalid device token")]
    InvalidToken,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_fcm_message() {
        let notification = PushNotification {
            device_token: "test_token".to_string(),
            platform: "android".to_string(),
            title: "Test Title".to_string(),
            body: "Test Body".to_string(),
            data: serde_json::json!({"key": "value"}),
            priority: PushPriority::High,
            sound: Some("default".to_string()),
            badge: Some(1),
            category: None,
        };

        let fcm_message = build_fcm_message(&notification);
        assert_eq!(fcm_message.message.token, "test_token");
        assert!(fcm_message.message.android.is_some());
    }
}
