//! Terms of Service Enforcement Middleware
//!
//! Blocks API access for users who haven't accepted the current Terms of Service.

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::api::AppState;
use crate::api::v4::extractors::MmAuthUser;

/// List of paths that are exempt from terms enforcement
const EXEMPT_PATHS: &[&str] = &[
    "/api/v4/terms_of_service",
    "/api/v4/terms_of_service/current",
    "/api/v4/terms_of_service/status",
    "/api/v4/terms_of_service/accept",
    "/api/v4/auth/login",
    "/api/v4/auth/logout",
    "/api/v4/auth/register",
    "/api/v4/auth/reset_password",
    "/api/v4/auth/send_password_reset",
    "/api/v4/config/client",
    "/api/v4/system/ping",
    "/api/v4/users/login",
    "/api/v4/users/logout",
];

/// Middleware that enforces terms of service acceptance
pub async fn terms_enforcement_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // Check if path is exempt
    if EXEMPT_PATHS.iter().any(|p| path.starts_with(p)) {
        return next.run(req).await;
    }

    // Check if user is authenticated
    let user = match extract_user(&state, &req).await {
        Some(u) => u,
        None => return next.run(req).await, // Not authenticated, let auth middleware handle it
    };

    // Check if terms acceptance is required
    match check_terms_acceptance(&state, user.id).await {
        Ok(true) => next.run(req).await, // Terms accepted, proceed
        Ok(false) => {
            // Terms not accepted, block with 403
            (
                StatusCode::FORBIDDEN,
                axum::Json(json!({
                    "id": "api.terms_of_service.not_accepted",
                    "message": "You must accept the Terms of Service to continue",
                    "detailed_error": "The user has not accepted the current Terms of Service version",
                    "status_code": 403
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Error checking terms acceptance: {}", e);
            // On error, allow the request through but log the error
            next.run(req).await
        }
    }
}

/// Extract user from request if authenticated
async fn extract_user(state: &AppState, req: &Request) -> Option<crate::models::user::User> {
    // Get the authorization header
    let auth_header = req.headers().get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    // Extract Bearer token
    if !auth_str.starts_with("Bearer ") {
        return None;
    }
    let token = &auth_str[7..];

    // Validate token and get user
    // This is a simplified version - in production you'd use the actual JWT validation
    let user_id = validate_token(state, token).await.ok()?;

    // Fetch user from database
    sqlx::query_as::<_, crate::models::user::User>(
        r#"SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
}

/// Validate JWT token and return user ID
async fn validate_token(state: &AppState, token: &str) -> Result<uuid::Uuid, crate::error::ApiError> {
    // Use the existing JWT validation logic from auth module
    // For now, return an error as placeholder
    // In production, this would integrate with the actual auth validation
    Err(crate::error::ApiError::unauthorized("Invalid token"))
}

/// Check if user has accepted the current terms
async fn check_terms_acceptance(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<bool, sqlx::Error> {
    // Get current active terms
    let terms_id: Option<uuid::Uuid> = sqlx::query_scalar(
        r#"
        SELECT id FROM terms_of_service 
        WHERE is_active = true 
        ORDER BY effective_date DESC 
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    // If no active terms, no enforcement needed
    let Some(terms_id) = terms_id else {
        return Ok(true);
    };

    // Check if user has accepted
    let accepted: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_terms_acceptance 
            WHERE user_id = $1 AND terms_id = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(terms_id)
    .fetch_one(&state.db)
    .await?;

    Ok(accepted)
}

/// Check if a user needs to accept terms (for use in login/auth flows)
pub async fn user_needs_terms_acceptance(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<Option<crate::models::terms::TermsOfService>, sqlx::Error> {
    // Get current active terms
    let terms = sqlx::query_as::<_, crate::models::terms::TermsOfService>(
        r#"
        SELECT * FROM terms_of_service 
        WHERE is_active = true 
        ORDER BY effective_date DESC 
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(terms) = terms else {
        return Ok(None);
    };

    // Check if user has accepted
    let accepted: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_terms_acceptance 
            WHERE user_id = $1 AND terms_id = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(terms.id)
    .fetch_one(&state.db)
    .await?;

    if accepted {
        Ok(None)
    } else {
        Ok(Some(terms))
    }
}
