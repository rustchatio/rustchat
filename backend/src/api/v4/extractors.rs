use axum::{
    extract::FromRequestParts,
    http::{
        header::{HeaderName, AUTHORIZATION},
        request::Parts,
    },
};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::middleware::{ensure_user_access_active, FromRef};
use crate::auth::{validate_token, Claims};
use crate::error::AppError;

pub struct MmAuthUser {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub email: String,
    #[allow(dead_code)]
    pub role: String,
    #[allow(dead_code)]
    pub org_id: Option<Uuid>,
}

impl From<Claims> for MmAuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
            org_id: claims.org_id,
        }
    }
}

impl<S> FromRequestParts<S> for MmAuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let token = if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid authorization header".to_string()))?;
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("Token ") {
                stripped.trim()
            } else {
                auth_str.trim()
            }
        } else if let Some(token_header) = parts.headers.get(HeaderName::from_static("token")) {
            token_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid token header".to_string()))?
                .trim()
        } else if let Some(cookie_header) = parts.headers.get(HeaderName::from_static("cookie")) {
            // Parse cookies to find MMAUTHTOKEN - used by img/video tags that can't send headers
            let cookie_str = cookie_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid cookie header".to_string()))?;

            cookie_str
                .split(';')
                .map(|s| s.trim())
                .find_map(|cookie| cookie.strip_prefix("MMAUTHTOKEN="))
                .ok_or_else(|| AppError::Unauthorized("Missing MMAUTHTOKEN cookie".to_string()))?
        } else {
            return Err(AppError::Unauthorized(
                "Missing authorization header".to_string(),
            ));
        };

        let token_data = validate_token(token, &app_state.jwt_secret)?;
        ensure_user_access_active(&app_state, token_data.claims.sub).await?;

        Ok(MmAuthUser::from(token_data.claims))
    }
}
