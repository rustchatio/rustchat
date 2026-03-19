//! Error types for rustchat
//!
//! Provides structured error handling with HTTP status code mapping.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String, retry_after_secs: i64 },
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl AppError {
    /// Get the error code string
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Redis(_) => "REDIS_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Config(_) => "CONFIG_ERROR",
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::TooManyRequests(_) => "TOO_MANY_REQUESTS",
            AppError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
        }
    }

    /// Get the HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Special-case RateLimitExceeded to include Retry-After / X-RateLimit-Reset headers.
        if let AppError::RateLimitExceeded { ref message, retry_after_secs } = self {
            use std::time::{SystemTime, UNIX_EPOCH};

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let reset_at = now + retry_after_secs.max(0) as u64;

            let body = serde_json::json!({
                "error": {
                    "code": "RATE_LIMIT_EXCEEDED",
                    "message": message,
                }
            });

            tracing::error!(
                error = %message,
                code = "RATE_LIMIT_EXCEEDED",
                status = %StatusCode::TOO_MANY_REQUESTS,
                retry_after_secs,
                "API error"
            );

            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(body),
            )
                .into_response();

            let headers = response.headers_mut();
            if let Ok(v) = retry_after_secs.max(0).to_string().parse() {
                headers.insert("Retry-After", v);
            }
            if let Ok(v) = reset_at.to_string().parse() {
                headers.insert("X-RateLimit-Reset", v);
            }
            return response;
        }

        let status = self.status_code();
        let message = self.to_string();

        // Log the error for debugging
        tracing::error!(error = %message, code = %self.code(), status = %status, "API error");

        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code().to_string(),
                message,
                details: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, AppError>;
