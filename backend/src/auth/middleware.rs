//! Auth middleware and extractors

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use super::jwt::{validate_token_with_policy, Claims};
use super::policy::{AuthzResult, Permission, PolicyEngine};
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

impl AuthUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.role.split_whitespace().any(|r| r == role)
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        matches!(
            PolicyEngine::check_permission(&self.role, permission),
            AuthzResult::Allow
        )
    }

    pub fn can_access_owned(&self, owner_id: Uuid, permission: &Permission) -> bool {
        matches!(
            PolicyEngine::check_ownership(&self.role, permission, self.user_id, owner_id),
            AuthzResult::Allow
        )
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
        let token_data = validate_token_with_policy(
            token,
            &app_state.jwt_secret,
            app_state.jwt_issuer.as_deref(),
            app_state.jwt_audience.as_deref(),
        )?;
        ensure_user_access_active(&app_state, token_data.claims.sub).await?;

        Ok(AuthUser::from(token_data.claims))
    }
}

pub async fn ensure_user_access_active(
    app_state: &AppState,
    user_id: Uuid,
) -> Result<(), AppError> {
    let row: Option<(bool, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as("SELECT is_active, deleted_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&app_state.db)
            .await?;

    match row {
        Some((true, None)) => Ok(()),
        Some((false, _)) => Err(AppError::Unauthorized("Account is inactive".to_string())),
        Some((_, Some(_))) => Err(AppError::Unauthorized(
            "Account has been deleted".to_string(),
        )),
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

#[cfg(test)]
mod tests {
    use super::AuthUser;
    use uuid::Uuid;

    #[test]
    fn auth_user_role_helpers_support_multi_role_strings() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "system_admin team_admin".to_string(),
            org_id: None,
        };

        assert!(user.has_role("system_admin"));
        assert!(user.has_role("team_admin"));
        assert!(!user.has_role("org_admin"));
    }

    #[test]
    fn auth_user_permission_helpers_work() {
        use crate::auth::policy::permissions;

        let user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "member org_admin".to_string(),
            org_id: None,
        };

        // org_admin does NOT grant SYSTEM_MANAGE (removed to prevent privilege escalation)
        assert!(!user.has_permission(&permissions::SYSTEM_MANAGE));
        assert!(user.has_permission(&permissions::USER_MANAGE));
        assert!(!user.has_permission(&permissions::ADMIN_FULL));

        assert!(user.can_access_owned(user.user_id, &permissions::ADMIN_FULL));
    }
}
