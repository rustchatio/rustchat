//! Rate limiting middleware for API endpoints
//!
//! Provides IP-based and user-based rate limiting with Redis backend.

use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use deadpool_redis::redis::AsyncCommands;

use crate::api::AppState;
use crate::error::AppError;
use crate::telemetry::metrics;

/// Rate limit configuration for different endpoint categories
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window size in seconds
    pub window_seconds: u64,
    /// Key prefix for Redis
    pub key_prefix: String,
}

impl RateLimitConfig {
    /// Configurable auth endpoint limits (per minute)
    pub fn auth_per_minute(max_requests: u32) -> Self {
        Self {
            max_requests,
            window_seconds: 60,
            key_prefix: "ratelimit:auth".to_string(),
        }
    }

    /// Configurable WebSocket connection attempt limits (per minute)
    pub fn websocket_per_minute(max_requests: u32) -> Self {
        Self {
            max_requests,
            window_seconds: 60,
            key_prefix: "ratelimit:ws".to_string(),
        }
    }

    /// Registration endpoint limits (per hour)
    pub fn registration_default() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 3600,
            key_prefix: "ratelimit:register".to_string(),
        }
    }

    /// Default configuration for password reset
    pub fn password_reset_default() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 3600, // 1 hour
            key_prefix: "ratelimit:pwreset".to_string(),
        }
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_at: i64,
    pub total_limit: u32,
}

/// Check if a request should be rate limited
///
/// Uses Redis for distributed rate limiting across multiple nodes.
pub async fn check_rate_limit(
    redis: &deadpool_redis::Pool,
    config: &RateLimitConfig,
    key: &str,
) -> Result<RateLimitResult, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))?;

    let redis_key = format!("{}:{}", config.key_prefix, key);
    let window = config.window_seconds as i64;
    let now = chrono::Utc::now().timestamp();
    let window_start = now - window;

    // Use Redis sorted set for sliding window rate limiting
    // Remove entries outside the current window
    let _: () = conn
        .zrembyscore(&redis_key, 0, window_start)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    // Count current requests in window
    let current_count: u32 = conn
        .zcard(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    if current_count >= config.max_requests {
        // Get the oldest timestamp to calculate reset time
        let oldest: Vec<i64> = conn
            .zrange(&redis_key, 0, 0)
            .await
            .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

        let reset_at = oldest.first().copied().unwrap_or(now) + window;

        return Ok(RateLimitResult {
            allowed: false,
            remaining: 0,
            reset_at,
            total_limit: config.max_requests,
        });
    }

    // Add current request to the window
    let _: () = conn
        .zadd(&redis_key, now, now)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    // Set expiry on the key to auto-cleanup
    let _: () = conn
        .expire(&redis_key, window)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    Ok(RateLimitResult {
        allowed: true,
        remaining: config.max_requests - current_count - 1,
        reset_at: now + window,
        total_limit: config.max_requests,
    })
}

/// Extract client IP from forwarding headers if present.
pub fn extract_client_ip_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    // Check for X-Forwarded-For header (when behind proxy)
    if let Some(forwarded) = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()) {
        // Take the first IP in the chain (original client)
        let first_ip = forwarded.split(',').next().map(|s| s.trim());
        if let Some(ip) = first_ip {
            if !ip.is_empty() {
                return Some(ip.to_string());
            }
        }
    }

    // Check for X-Real-IP header
    if let Some(real_ip) = headers.get("X-Real-IP").and_then(|v| v.to_str().ok()) {
        if !real_ip.is_empty() {
            return Some(real_ip.to_string());
        }
    }

    None
}

/// Extract client IP for rate limiting
pub fn extract_client_ip(addr: &SocketAddr, headers: &axum::http::HeaderMap) -> String {
    if let Some(ip) = extract_client_ip_from_headers(headers) {
        return ip;
    }

    // Fall back to direct connection IP
    addr.ip().to_string()
}

fn extract_client_ip_from_request(request: &Request) -> String {
    if let Some(ip) = extract_client_ip_from_headers(request.headers()) {
        return ip;
    }

    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

async fn enforce_rate_limit_for_request(
    state: &AppState,
    client_ip: String,
    config: RateLimitConfig,
    scope: &'static str,
    fail_open: bool,
) -> Option<Response> {
    if !state.config.security.rate_limit_enabled {
        return None;
    }

    match check_rate_limit(&state.redis, &config, &client_ip).await {
        Ok(result) => {
            if result.allowed {
                None
            } else {
                metrics::record_rate_limit_hit(scope, "ip");
                tracing::warn!(
                    scope = scope,
                    ip = %client_ip,
                    "Rate limit exceeded"
                );
                Some(rate_limit_response(&result))
            }
        }
        Err(err) => {
            if fail_open {
                tracing::error!(
                    scope = scope,
                    ip = %client_ip,
                    error = %err,
                    "Rate limit check failed; allowing request"
                );
                None
            } else {
                Some(
                    AppError::Internal("Rate limiting service unavailable".to_string())
                        .into_response(),
                )
            }
        }
    }
}

/// Centralized middleware for auth endpoint IP rate limiting.
pub async fn auth_ip_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip_from_request(&request);

    let config = RateLimitConfig::auth_per_minute(state.config.security.rate_limit_auth_per_minute);
    if let Some(response) =
        enforce_rate_limit_for_request(&state, client_ip, config, "auth", false).await
    {
        return response;
    }

    next.run(request).await
}

/// Stricter centralized middleware for registration endpoint IP limiting.
pub async fn register_ip_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip_from_request(&request);

    if let Some(response) = enforce_rate_limit_for_request(
        &state,
        client_ip,
        RateLimitConfig::registration_default(),
        "register",
        false,
    )
    .await
    {
        return response;
    }

    next.run(request).await
}

/// Centralized middleware for websocket upgrade attempt limiting.
pub async fn websocket_ip_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip_from_request(&request);

    let config =
        RateLimitConfig::websocket_per_minute(state.config.security.rate_limit_ws_per_minute);
    if let Some(response) =
        enforce_rate_limit_for_request(&state, client_ip, config, "ws", true).await
    {
        return response;
    }

    next.run(request).await
}

/// Centralized middleware for password reset flow endpoint limiting.
pub async fn password_reset_ip_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip_from_request(&request);

    if let Some(response) = enforce_rate_limit_for_request(
        &state,
        client_ip,
        RateLimitConfig::password_reset_default(),
        "password_reset",
        false,
    )
    .await
    {
        return response;
    }

    next.run(request).await
}

/// Rate limit error response
pub fn rate_limit_response(result: &RateLimitResult) -> Response {
    let retry_after = (result.reset_at - chrono::Utc::now().timestamp()).max(0);

    let body = serde_json::json!({
        "error": {
            "code": "RATE_LIMIT_EXCEEDED",
            "message": "Too many requests. Please try again later.",
            "retry_after": retry_after,
        }
    });

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Content-Type", "application/json")
        .header("X-RateLimit-Limit", result.total_limit.to_string())
        .header("X-RateLimit-Remaining", result.remaining.to_string())
        .header("X-RateLimit-Reset", result.reset_at.to_string())
        .header("Retry-After", retry_after.to_string())
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extract_client_ip_direct() {
        let addr = SocketAddr::from(([192, 168, 1, 1], 12345));
        let headers = HeaderMap::new();

        assert_eq!(extract_client_ip(&addr, &headers), "192.168.1.1");
    }

    #[test]
    fn test_extract_client_ip_forwarded() {
        let addr = SocketAddr::from(([192, 168, 1, 1], 12345));
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", "10.0.0.1, 10.0.0.2".parse().unwrap());

        assert_eq!(extract_client_ip(&addr, &headers), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let addr = SocketAddr::from(([192, 168, 1, 1], 12345));
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", "10.0.0.5".parse().unwrap());

        assert_eq!(extract_client_ip(&addr, &headers), "10.0.0.5");
    }

    #[test]
    fn test_extract_client_ip_from_headers_none() {
        let headers = HeaderMap::new();
        assert!(extract_client_ip_from_headers(&headers).is_none());
    }
}
