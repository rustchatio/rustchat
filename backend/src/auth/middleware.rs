//! Auth middleware and extractors

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use super::jwt::{validate_token, Claims};
use crate::api::AppState;
use crate::error::AppError;

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub org_id: Option<Uuid>,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
            org_id: claims.org_id,
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()))?;

        // Validate token
        let token_data = validate_token(token, &app_state.jwt_secret)?;
        ensure_user_access_active(&app_state, token_data.claims.sub).await?;

        Ok(AuthUser::from(token_data.claims))
    }
}

pub async fn ensure_user_access_active(app_state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let row: Option<(bool, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as("SELECT is_active, deleted_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&app_state.db)
            .await?;

    match row {
        Some((true, None)) => Ok(()),
        Some((false, _)) => Err(AppError::Unauthorized("Account is inactive".to_string())),
        Some((_, Some(_))) => Err(AppError::Unauthorized("Account has been deleted".to_string())),
        None => Err(AppError::Unauthorized("User not found".to_string())),
    }
}

/// Helper trait to extract AppState from state
pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<AppState> for AppState {
    fn from_ref(input: &AppState) -> Self {
        input.clone()
    }
}
