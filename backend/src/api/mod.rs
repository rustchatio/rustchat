//! API module for rustchat
//!
//! Provides HTTP routes and handlers.

mod admin;
mod admin_email;
mod auth;
mod calls;
mod channels;
mod files;
mod health;
mod integrations;
mod oauth;
mod playbooks;
mod posts;
mod preferences;
mod search;
mod site;
mod teams;
mod unreads;
mod users;
mod v4;
mod video;
mod websocket_core;
mod ws;

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::{
    extract::DefaultBodyLimit,
    extract::MatchedPath,
    http::Request,
    http::{HeaderValue, Method},
    Router,
};
use sqlx::PgPool;
use tower_http::{
    catch_panic::CatchPanicLayer,
    classify::ServerErrorsFailureClass,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::Level;

/// Handle panics by converting them to 500 responses
fn handle_panic(
    err: Box<dyn std::any::Any + Send + 'static>,
) -> axum::http::Response<axum::body::Body> {
    let panic_message = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic".to_string()
    };

    tracing::error!("PANIC: {}", panic_message);

    axum::http::Response::builder()
        .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(format!(
            r#"{{"error":{{"code":"PANIC","message":"Internal server error"}}}}"#
        )))
        .unwrap()
}

use crate::api::v4::calls_plugin::sfu::SFUManager;
use crate::api::v4::calls_plugin::start_voice_event_listener;
use crate::api::v4::calls_plugin::state::{CallStateBackend, CallStateManager};
use crate::config::Config;
use crate::realtime::{ConnectionStore, WsHub};
use crate::storage::S3Client;
use tokio::sync::mpsc;

fn parse_cors_allowed_origins(raw: &str) -> Vec<HeaderValue> {
    raw.split(',')
        .filter_map(|origin| {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                return None;
            }
            HeaderValue::from_str(trimmed).ok()
        })
        .collect()
}

fn build_cors_layer(config: &Config) -> CorsLayer {
    let cors = CorsLayer::new().allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
    ]);

    if let Some(raw_origins) = config.cors_allowed_origins.as_deref() {
        let origins = parse_cors_allowed_origins(raw_origins);
        if !origins.is_empty() {
            return cors.allow_origin(origins).allow_headers(Any);
        }

        if config.is_production() {
            tracing::warn!(
                "RUSTCHAT_CORS_ALLOWED_ORIGINS is set but no valid origins were parsed; CORS is restricted"
            );
            return cors;
        }
    }

    if config.is_production() {
        tracing::warn!(
            "No CORS allowlist configured in production mode; cross-origin browser requests are blocked"
        );
        return cors;
    }

    cors.allow_origin(Any).allow_headers(Any)
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: deadpool_redis::Pool,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub ws_hub: Arc<WsHub>,
    pub connection_store: Arc<ConnectionStore>,
    pub s3_client: S3Client,
    pub http_client: reqwest::Client,
    pub start_time: std::time::Instant,
    pub config: Config,
    pub sfu_manager: Arc<SFUManager>,
    pub call_state_manager: Arc<CallStateManager>,
}

/// Build the main application router
pub fn router(
    db: PgPool,
    redis: deadpool_redis::Pool,
    jwt_secret: String,
    jwt_expiry_hours: u64,
    ws_hub: Arc<WsHub>,
    s3_client: S3Client,
    config: Config,
) -> Router {
    let (voice_event_tx, voice_event_rx) = mpsc::unbounded_channel();
    let sfu_manager = SFUManager::new(config.calls.clone(), voice_event_tx);
    let call_state_manager = Arc::new(CallStateManager::with_backend(
        Some(redis.clone()),
        CallStateBackend::parse(&config.calls.state_backend),
    ));
    let connection_store = ConnectionStore::new();

    let state = AppState {
        db,
        redis,
        jwt_secret,
        jwt_expiry_hours,
        ws_hub,
        connection_store,
        s3_client,
        http_client: reqwest::Client::new(),
        start_time: std::time::Instant::now(),
        config,
        sfu_manager,
        call_state_manager,
    };

    // Start Calls voice event listener
    tokio::spawn(start_voice_event_listener(state.clone(), voice_event_rx));

    // CORS configuration
    let cors = build_cors_layer(&state.config);

    // API v1 routes
    let api_v1 = Router::new()
        .nest("/health", health::router())
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/teams", teams::router())
        .nest("/channels", channels::router())
        .nest("/unreads", unreads::router())
        .merge(posts::router())
        .merge(files::router())
        .merge(search::router())
        .merge(integrations::router())
        .merge(admin::router())
        .merge(preferences::router())
        .merge(playbooks::router())
        .merge(calls::router())
        .merge(oauth::router())
        .merge(site::router())
        .nest("/video", video::router())
        .merge(ws::router());

    let api_v4 = v4::router().layer(DefaultBodyLimit::max(50 * 1024 * 1024));

    Router::new()
        .nest("/api/v1", api_v1)
        .nest("/api/v4", api_v4)
        .layer(CatchPanicLayer::custom(handle_panic))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or("<unknown>");
                    tracing::span!(
                        Level::INFO,
                        "http.request",
                        method = %request.method(),
                        uri = %request.uri(),
                        matched_path = matched_path
                    )
                })
                .on_request(|_request: &Request<Body>, _span: &tracing::Span| {
                    tracing::debug!("request started");
                })
                .on_response(
                    |response: &axum::http::Response<Body>,
                     latency: Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status(),
                            latency_ms = latency.as_millis(),
                            "request completed"
                        );
                    },
                )
                .on_failure(
                    |failure: ServerErrorsFailureClass,
                     latency: Duration,
                     _span: &tracing::Span| {
                        tracing::error!(
                            classification = %failure,
                            latency_ms = latency.as_millis(),
                            "request failed"
                        );
                    },
                ),
        )
        .layer(cors)
        .with_state(state)
}
