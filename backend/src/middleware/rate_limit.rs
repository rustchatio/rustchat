//! Rate limiting middleware
//!
//! ## Architecture
//!
//! Rate limiting in RustChat operates at two levels:
//!
//! ### 1. Entity-Level Rate Limiting (Implemented)
//! Authenticated requests (API keys) are rate limited per entity using Redis-backed
//! atomic counters. This is handled by:
//! - `services/rate_limit.rs` - RateLimitService with Lua scripts
//! - `auth/extractors.rs` - ApiKeyAuth and PolymorphicAuth call RateLimitService
//!
//! Tiers:
//! - HumanStandard: 1k req/hr
//! - AgentHigh: 10k req/hr
//! - ServiceUnlimited: no limit
//! - CIStandard: 5k req/hr
//!
//! ### 2. IP-Based Rate Limiting (Delegated to Reverse Proxy)
//! Unauthenticated endpoints (login, registration, password reset) are rate limited
//! by IP address. This is **delegated to the reverse proxy layer** (nginx, Cloudflare, etc.)
//! for better performance and DDoS protection.
//!
//! The middleware functions below are stubs that pass through requests, relying on
//! upstream infrastructure for IP rate limiting. In production:
//! - nginx limit_req module
//! - Cloudflare rate limiting rules
//! - AWS WAF rate-based rules
//!
//! This approach is preferred because:
//! - Reverse proxies handle rate limiting before requests reach the application
//! - Better protection against layer 7 DDoS attacks
//! - Lower latency (no Redis roundtrip per request)
//! - Centralized rate limit configuration

use crate::error::AppError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_redis::redis::AsyncCommands;

/// Rate limit middleware for registration endpoints
///
/// **Note:** IP-based rate limiting is delegated to reverse proxy (nginx/Cloudflare).
/// This middleware passes requests through without additional rate limiting.
///
/// Recommended reverse proxy configuration:
/// - nginx: `limit_req zone=registration burst=5 nodelay;`
/// - Cloudflare: 5 requests per minute per IP
pub async fn register_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // IP rate limiting delegated to reverse proxy
    Ok(next.run(request).await)
}

/// Rate limit middleware for auth endpoints
///
/// **Note:** IP-based rate limiting is delegated to reverse proxy (nginx/Cloudflare).
/// This middleware passes requests through without additional rate limiting.
///
/// Recommended reverse proxy configuration:
/// - nginx: `limit_req zone=auth burst=10 nodelay;`
/// - Cloudflare: 10 requests per minute per IP
pub async fn auth_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // IP rate limiting delegated to reverse proxy
    Ok(next.run(request).await)
}

/// Rate limit middleware for password reset endpoints
///
/// **Note:** IP-based rate limiting is delegated to reverse proxy (nginx/Cloudflare).
/// This middleware passes requests through without additional rate limiting.
///
/// Recommended reverse proxy configuration:
/// - nginx: `limit_req zone=password_reset burst=3 nodelay;`
/// - Cloudflare: 3 requests per minute per IP
pub async fn password_reset_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // IP rate limiting delegated to reverse proxy
    Ok(next.run(request).await)
}

/// Rate limit middleware for WebSocket endpoints
///
/// **Note:** IP-based rate limiting is delegated to reverse proxy (nginx/Cloudflare).
/// This middleware passes requests through without additional rate limiting.
///
/// Recommended reverse proxy configuration:
/// - nginx: `limit_req zone=websocket burst=20 nodelay;`
/// - Cloudflare: 20 requests per minute per IP
pub async fn websocket_ip_rate_limit(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // IP rate limiting delegated to reverse proxy
    Ok(next.run(request).await)
}

// ============================================================================
// Legacy API - Kept for backward compatibility
// ============================================================================
// These types and functions are used by existing code (src/api/auth.rs, src/api/v4/users.rs)
// They are stubs that always allow requests. Real rate limiting is handled by:
// 1. Entity-level: RateLimitService (services/rate_limit.rs)
// 2. IP-level: Reverse proxy (nginx/Cloudflare)

/// Legacy rate limit configuration (stub)
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub window_secs: u64,
    pub max_requests: u64,
}

impl RateLimitConfig {
    /// Create config for auth endpoints (stub)
    pub fn auth_per_minute(max_requests: u32) -> Self {
        Self {
            window_secs: 60,
            max_requests: max_requests as u64,
        }
    }
}

/// Legacy rate limit check result (stub)
#[derive(Debug, Clone, Copy)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u64,
    pub reset_at: u64,
}

/// Per-account rate limit check using a Redis sliding window.
///
/// Used by login handlers to throttle individual accounts independently of
/// IP-based limits enforced at the reverse proxy layer.
pub async fn check_rate_limit(
    redis: &deadpool_redis::Pool,
    config: &RateLimitConfig,
    key: &str,
) -> Result<RateLimitResult, AppError> {
    let now = chrono::Utc::now().timestamp();
    let window_start = now - config.window_secs as i64;
    let redis_key = format!("ratelimit:{}", key);

    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;

    // Remove entries outside the sliding window
    let _: () = conn
        .zrembyscore(&redis_key, 0i64, window_start)
        .await
        .map_err(AppError::Redis)?;

    // Count remaining entries in the window
    let current_count: u64 = conn.zcard(&redis_key).await.map_err(AppError::Redis)?;

    if current_count >= config.max_requests {
        let reset_at = (now as u64) + config.window_secs;
        return Ok(RateLimitResult {
            allowed: false,
            remaining: 0,
            reset_at,
        });
    }

    // Record this attempt and set TTL
    let _: () = conn
        .zadd(&redis_key, now, now)
        .await
        .map_err(AppError::Redis)?;
    let _: () = conn
        .expire(&redis_key, config.window_secs as i64)
        .await
        .map_err(AppError::Redis)?;

    Ok(RateLimitResult {
        allowed: true,
        remaining: config.max_requests - current_count - 1,
        reset_at: (now as u64) + config.window_secs,
    })
}
