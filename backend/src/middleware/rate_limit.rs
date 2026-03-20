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
//! ### 2. IP-Based Rate Limiting (Application Layer)
//! Unauthenticated endpoints (login, registration, password reset, WebSocket) are
//! rate limited by IP address using the RateLimitService (Redis-backed sliding window).
//!
//! When `TRUST_PROXY=true`, the client IP is read from the first value in the
//! `X-Forwarded-For` header (set by nginx/Cloudflare). Otherwise the socket address
//! is used directly.

use crate::error::AppError;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Extract the originating client IP.
/// When the `TRUST_PROXY` environment variable is `"true"`, reads the first
/// value from `X-Forwarded-For` (the client IP). Otherwise uses the socket address.
fn extract_client_ip(addr: &SocketAddr, headers: &HeaderMap) -> String {
    let trust_proxy = std::env::var("TRUST_PROXY")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if trust_proxy {
        if let Some(xff) = headers.get("x-forwarded-for") {
            if let Ok(s) = xff.to_str() {
                if let Some(ip) = s.split(',').next() {
                    return ip.trim().to_string();
                }
            }
        }
    }

    addr.ip().to_string()
}

/// Rate limit middleware for registration endpoints
pub async fn register_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_register_ip(&ip).await?;
    Ok(next.run(request).await)
}

/// Rate limit middleware for auth endpoints
pub async fn auth_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_auth_ip(&ip).await?;
    Ok(next.run(request).await)
}

/// Rate limit middleware for password reset endpoints
pub async fn password_reset_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_password_reset_ip(&ip).await?;
    Ok(next.run(request).await)
}

/// Rate limit middleware for WebSocket endpoints
pub async fn websocket_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_websocket_ip(&ip).await?;
    Ok(next.run(request).await)
}

