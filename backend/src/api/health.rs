//! Health check endpoints
//!
//! Provides liveness and readiness probes for Kubernetes/Docker,
//! plus metrics endpoint for Prometheus.

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::collections::HashMap;

use super::AppState;
use crate::db;
use crate::telemetry::metrics;

#[derive(Serialize)]
pub struct LivenessResponse {
    status: &'static str,
    version: &'static str,
    uptime_seconds: u64,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    status: &'static str,
    database: &'static str,
    redis: &'static str,
    checks: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct MetricsResponse {
    websocket_connections: i64,
    active_users: i64,
    db_pool_saturation: f64,
}

/// Build health check routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/live", get(liveness))
        .route("/ready", get(readiness))
        .route("/metrics", get(metrics_endpoint))
        .route("/stats", get(stats))
}

/// Liveness probe - checks if the application is running
async fn liveness(State(state): State<AppState>) -> Json<LivenessResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    
    Json(LivenessResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: uptime,
    })
}

/// Readiness probe - checks if dependencies are available
async fn readiness(State(state): State<AppState>) -> Result<Json<ReadinessResponse>, StatusCode> {
    let mut checks = HashMap::new();
    
    // Check database
    let db_healthy = db::health_check(&state.db).await;
    checks.insert("database".to_string(), if db_healthy { "ok".to_string() } else { "error".to_string() });
    
    // Check Redis
    let redis_healthy = check_redis(&state.redis).await;
    checks.insert("redis".to_string(), if redis_healthy { "ok".to_string() } else { "error".to_string() });
    
    // WebSocket hub process-local health detail (informational)
    let ws_connections = state.ws_hub.count_connections().await;
    checks.insert(
        "websocket_hub".to_string(),
        format!("ok(connections={})", ws_connections),
    );
    
    let all_healthy = db_healthy && redis_healthy;
    
    let response = ReadinessResponse {
        status: if all_healthy { "ok" } else { "degraded" },
        database: if db_healthy { "connected" } else { "disconnected" },
        redis: if redis_healthy { "connected" } else { "disconnected" },
        checks,
    };
    
    if all_healthy {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Prometheus metrics endpoint
async fn metrics_endpoint() -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics".to_string())
        }
    }
}

/// Internal stats endpoint (JSON format)
async fn stats(State(state): State<AppState>) -> Json<MetricsResponse> {
    let ws_connections = state.ws_hub.count_connections().await as i64;
    let active_users = state.ws_hub.online_users().await.len() as i64;
    
    let max_connections = state.config.db_pool.max_connections.max(1) as f64;
    let current_size = state.db.size() as f64;
    let idle = state.db.num_idle() as f64;
    let in_use = (current_size - idle).max(0.0);
    let db_pool_saturation = (in_use / max_connections).clamp(0.0, 1.0);
    
    // Update gauges
    metrics::WS_ACTIVE_CONNECTIONS.set(ws_connections);
    metrics::ACTIVE_USERS.set(active_users);
    
    Json(MetricsResponse {
        websocket_connections: ws_connections,
        active_users,
        db_pool_saturation,
    })
}

/// Check Redis connectivity
async fn check_redis(redis: &deadpool_redis::Pool) -> bool {
    match redis.get().await {
        Ok(mut conn) => {
            deadpool_redis::redis::cmd("PING")
                .query_async::<Option<String>>(&mut conn)
                .await
                .is_ok()
        }
        Err(_) => false,
    }
}
