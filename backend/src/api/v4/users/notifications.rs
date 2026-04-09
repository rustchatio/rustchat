use axum::Json;

use crate::error::ApiResult;

pub async fn get_notifications() -> ApiResult<Json<serde_json::Value>> {
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

pub async fn update_notifications(
    Json(_input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}
