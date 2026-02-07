//! API module for rustchat
//!
//! Provides HTTP routes and handlers.

mod admin;
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
mod ws;

use std::sync::Arc;

use axum::{extract::DefaultBodyLimit, http::Method, Router};
use sqlx::PgPool;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::{TraceLayer, DefaultOnResponse, DefaultOnRequest, DefaultMakeSpan},
};
use tracing::Level;

/// Handle panics by converting them to 500 responses
fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> axum::http::Response<axum::body::Body> {
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

use crate::realtime::{ConnectionStore, WsHub};
use crate::storage::S3Client;
use crate::config::Config;
use crate::api::v4::calls_plugin::sfu::SFUManager;

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
    let sfu_manager = SFUManager::new(config.calls.clone());
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
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers(Any);

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
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        )
        .layer(cors)
        .with_state(state)
}
