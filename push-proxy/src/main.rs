mod fcm;

use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use fcm::{FcmClient, FcmError, PushPayload};
use std::sync::Arc;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    fcm_client: FcmClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "push_proxy=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let project_id = std::env::var("FIREBASE_PROJECT_ID")
        .expect("FIREBASE_PROJECT_ID must be set");
    let key_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .expect("GOOGLE_APPLICATION_CREDENTIALS must be set");

    info!("Starting Push Proxy for project: {}", project_id);

    // Initialize FCM Client
    let fcm_client = FcmClient::new(project_id, PathBuf::from(key_path)).await?;
    let state = Arc::new(AppState { fcm_client });

    // Build routes
    let app = Router::new()
        .route("/send", post(send_notification))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn send_notification(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PushPayload>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.fcm_client.send(payload).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!("Failed to send notification: {}", e);
            let (status, msg) = match e {
                FcmError::Api(ref s) if s.contains("UNREGISTERED") => (StatusCode::GONE, "Token unregistered"),
                FcmError::Api(ref s) if s.contains("INVALID_ARGUMENT") => (StatusCode::BAD_REQUEST, "Invalid argument"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to relay notification"),
            };
            Err((status, Json(serde_json::json!({ "error": msg, "details": e.to_string() }))))
        }
    }
}
