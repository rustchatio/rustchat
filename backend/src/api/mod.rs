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
use crate::middleware::reliability::ServiceCircuitBreakers;
use crate::middleware::security_headers::{SecurityHeadersLayer, cors_compatible_config};
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
    pub jwt_issuer: Option<String>,
    pub jwt_audience: Option<String>,
    pub jwt_expiry_hours: u64,
    pub ws_hub: Arc<WsHub>,
    pub connection_store: Arc<ConnectionStore>,
    pub s3_client: S3Client,
    pub http_client: reqwest::Client,
    pub start_time: std::time::Instant,
    pub config: Config,
    pub sfu_manager: Arc<SFUManager>,
    pub call_state_manager: Arc<CallStateManager>,
    pub circuit_breakers: Arc<ServiceCircuitBreakers>,
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
        jwt_issuer: config.jwt_issuer.clone(),
        jwt_audience: config.jwt_audience.clone(),
        jwt_expiry_hours,
        ws_hub,
        connection_store,
        s3_client,
        http_client: reqwest::Client::new(),
        start_time: std::time::Instant::now(),
        config,
        sfu_manager,
        call_state_manager,
        circuit_breakers: Arc::new(ServiceCircuitBreakers::new()),
    };

    // Start Calls voice event listener
    tokio::spawn(start_voice_event_listener(state.clone(), voice_event_rx));

    // CORS configuration
    let cors = build_cors_layer(&state.config);

    // Body size limits (in bytes)
    const SMALL_BODY_LIMIT: usize = 64 * 1024;      // 64KB - for most JSON APIs
    const MEDIUM_BODY_LIMIT: usize = 1024 * 1024;   // 1MB - for larger payloads
    const LARGE_BODY_LIMIT: usize = 50 * 1024 * 1024; // 50MB - for file uploads
    
    // API v1 routes with appropriate body limits
    // Routes that don't handle file uploads get smaller limits
    let api_v1 = Router::new()
        .nest("/health", health::router())
        .nest("/auth", auth::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .nest("/users", users::router().layer(DefaultBodyLimit::max(MEDIUM_BODY_LIMIT)))
        .nest("/teams", teams::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .nest("/channels", channels::router().layer(DefaultBodyLimit::max(MEDIUM_BODY_LIMIT)))
        .nest("/unreads", unreads::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .merge(posts::router().layer(DefaultBodyLimit::max(MEDIUM_BODY_LIMIT)))
        // Files router gets large limit for uploads
        .merge(files::router().layer(DefaultBodyLimit::max(LARGE_BODY_LIMIT)))
        .merge(search::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .merge(integrations::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .merge(admin::router().layer(DefaultBodyLimit::max(MEDIUM_BODY_LIMIT)))
        .merge(preferences::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .merge(playbooks::router().layer(DefaultBodyLimit::max(MEDIUM_BODY_LIMIT)))
        .merge(calls::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .merge(oauth::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .merge(site::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        .nest("/video", video::router().layer(DefaultBodyLimit::max(SMALL_BODY_LIMIT)))
        // WebSocket endpoint doesn't need body limit
        .merge(ws::router());

    // API v4 with route-specific limits
    let api_v4 = v4::router_with_body_limits(SMALL_BODY_LIMIT, MEDIUM_BODY_LIMIT, LARGE_BODY_LIMIT);

    // Configure security headers based on environment
    let security_config = if state.config.is_production() {
        cors_compatible_config()
    } else {
        crate::middleware::security_headers::SecurityHeadersConfig::development()
    };

    Router::new()
        .nest("/api/v1", api_v1)
        .nest("/api/v4", api_v4)
        .layer(CatchPanicLayer::custom(handle_panic))
        .layer(CompressionLayer::new())
        .layer(SecurityHeadersLayer::new(security_config))
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
