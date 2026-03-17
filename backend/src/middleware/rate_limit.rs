//! Rate limiting middleware

use crate::error::AppError;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_redis::Pool as RedisPool;

/// Rate limit configuration
pub struct RateLimitConfig {
    pub window_secs: u64,
    pub max_requests: u64,
}

impl RateLimitConfig {
    /// Create a config for auth endpoints (per minute)
    pub fn auth_per_minute(max_requests: u32) -> Self {
        Self {
            window_secs: 60,
            max_requests: max_requests as u64,
        }
    }
}

/// Rate limit check result
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u64,
    pub reset_at: u64,
}

/// Rate limiting middleware layer
/// Extracts auth context and checks rate limit
pub async fn rate_limit_middleware(
    State(_redis): State<RedisPool>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract entity_id and tier from request
    // This is a simplified version - in production, you'd extract from auth headers
    // For now, we'll skip rate limiting in middleware and do it in individual handlers

    // TODO: Extract auth from request headers manually
    // For Phase 1, rate limiting is checked in individual handlers

    Ok(next.run(request).await)
}

/// Check rate limit for a key
pub async fn check_rate_limit(
    _redis: &RedisPool,
    config: &RateLimitConfig,
    _key: &str,
) -> Result<RateLimitResult, AppError> {
    // Stub implementation - will be completed in Phase 2
    // Always allow for now
    Ok(RateLimitResult {
        allowed: true,
        remaining: config.max_requests,
        reset_at: 0,
    })
}

/// Rate limit middleware for registration endpoints
pub async fn register_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Stub - will be implemented in Phase 2
    Ok(next.run(request).await)
}

/// Rate limit middleware for auth endpoints
pub async fn auth_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Stub - will be implemented in Phase 2
    Ok(next.run(request).await)
}

/// Rate limit middleware for password reset endpoints
pub async fn password_reset_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Stub - will be implemented in Phase 2
    Ok(next.run(request).await)
}

/// Rate limit middleware for WebSocket endpoints
pub async fn websocket_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Stub - will be implemented in Phase 2
    Ok(next.run(request).await)
}
