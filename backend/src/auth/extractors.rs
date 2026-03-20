//! Authentication extractors for Axum handlers
//!
//! This module provides extractors for different authentication methods:
//! - `ApiKeyAuth` - Authenticates non-human entities (agents, services, CI) via API keys
//! - `McpAuth` - (Phase 2) Authenticates MCP (Model Context Protocol) clients
//! - `PolymorphicAuth` - Supports both JWT and API key authentication

use axum::{
    extract::FromRequestParts,
    http::{
        header::{HeaderName, AUTHORIZATION},
        request::Parts,
    },
};
use uuid::Uuid;

use super::api_key::{extract_prefix, validate_api_key};
use super::jwt::validate_token_with_policy;
use crate::api::AppState;
use crate::auth::middleware::{ensure_user_access_active, FromRef};
use crate::error::AppError;
use crate::models::entity::EntityType;
use crate::services::rate_limit::RateLimitService;

/// Authenticated non-human entity (agent, service, or CI) extracted from API key
///
/// This extractor validates API keys for non-human entities and ensures:
/// - The API key is valid and matches a stored hash in the database
/// - The entity is active and not deleted
/// - The entity is a non-human type (agent, service, or CI)
///
/// # Example
///
/// ```no_run
/// use axum::{Json, extract::State};
/// use rustchat::auth::extractors::ApiKeyAuth;
/// use rustchat::error::ApiResult;
///
/// async fn protected_handler(
///     auth: ApiKeyAuth,
/// ) -> ApiResult<Json<serde_json::Value>> {
///     // Handler logic - auth.user_id, auth.entity_type are available
///     Ok(Json(serde_json::json!({
///         "user_id": auth.user_id,
///         "entity_type": auth.entity_type
///     })))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub user_id: Uuid,
    pub email: String,
    pub entity_type: EntityType,
    pub org_id: Option<Uuid>,
    pub role: String,
}

impl<S> FromRequestParts<S> for ApiKeyAuth
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

        // Parse Bearer token (API key)
        let api_key = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()))?;

        // Extract the 16-character prefix for O(1) indexed lookup
        let prefix = extract_prefix(api_key)
            .map_err(|e| AppError::Unauthorized(format!("Invalid API key format: {}", e)))?;

        // Query by prefix (O(1) with unique index) instead of scanning all entities
        let entity: Option<(Uuid, String, EntityType, Option<Uuid>, String, String)> =
            sqlx::query_as(
                r#"
            SELECT id, email, entity_type, org_id, role, api_key_hash
            FROM users
            WHERE api_key_prefix = $1
                AND entity_type IN ('agent', 'service', 'ci')
                AND is_active = true
                AND deleted_at IS NULL
            "#,
            )
            .bind(&prefix)
            .fetch_optional(&app_state.db)
            .await?;

        // Validate the full API key against the hash
        if let Some((user_id, email, entity_type, org_id, role, hash)) = entity {
            match validate_api_key(api_key, &hash).await {
                Ok(true) => {
                    tracing::debug!(
                        user_id = %user_id,
                        entity_type = ?entity_type,
                        "API key authenticated successfully via prefix lookup"
                    );
                    // Enforce per-entity rate limiting based on tier
                    let tier = entity_type.default_rate_limit();
                    RateLimitService::new(app_state.redis.clone())
                        .check_rate_limit(&user_id, tier)
                        .await?;
                    return Ok(ApiKeyAuth {
                        user_id,
                        email,
                        entity_type,
                        org_id,
                        role,
                    });
                }
                Ok(false) => {
                    tracing::warn!(
                        user_id = %user_id,
                        "API key prefix matched but full key validation failed"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        user_id = %user_id,
                        error = %e,
                        "Failed to validate API key hash"
                    );
                }
            }
        }

        // No matching API key found or validation failed
        Err(AppError::Unauthorized("Invalid API key".to_string()))
    }
}

/// Polymorphic authentication that supports both JWT and API key
///
/// This extractor tries JWT authentication first, then falls back to API key.
/// It's useful for endpoints that should support both human users (JWT) and
/// non-human entities (API keys).
///
/// # Example
///
/// ```no_run
/// use axum::{Json, extract::State};
/// use rustchat::auth::extractors::PolymorphicAuth;
/// use rustchat::error::ApiResult;
///
/// async fn handler(
///     auth: PolymorphicAuth,
/// ) -> ApiResult<Json<serde_json::Value>> {
///     Ok(Json(serde_json::json!({
///         "user_id": auth.user_id,
///         "auth_type": auth.auth_type()
///     })))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PolymorphicAuth {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub org_id: Option<Uuid>,
    /// Indicates which authentication method was used
    pub is_api_key: bool,
}

impl PolymorphicAuth {
    pub fn auth_type(&self) -> &str {
        if self.is_api_key {
            "api_key"
        } else {
            "jwt"
        }
    }
}

impl<S> FromRequestParts<S> for PolymorphicAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract token using the same multi-format logic as MmAuthUser:
        // Bearer <token>, Token <token>, lowercase `token` header, MMAUTHTOKEN cookie.
        // Only the Bearer path also attempts API key auth.
        let (token, is_bearer) = if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid authorization header".to_string()))?;
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                (stripped.trim().to_string(), true)
            } else if let Some(stripped) = auth_str.strip_prefix("Token ") {
                (stripped.trim().to_string(), false)
            } else {
                (auth_str.trim().to_string(), false)
            }
        } else if let Some(token_header) = parts.headers.get(HeaderName::from_static("token")) {
            let t = token_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid token header".to_string()))?
                .trim()
                .to_string();
            (t, false)
        } else if let Some(cookie_header) = parts.headers.get(HeaderName::from_static("cookie")) {
            let cookie_str = cookie_header
                .to_str()
                .map_err(|_| AppError::Unauthorized("Invalid cookie header".to_string()))?;
            let t = cookie_str
                .split(';')
                .map(|s| s.trim())
                .find_map(|c| c.strip_prefix("MMAUTHTOKEN="))
                .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?
                .to_string();
            (t, false)
        } else {
            return Err(AppError::Unauthorized(
                "Missing authorization header".to_string(),
            ));
        };

        // Try JWT first (works for all token formats)
        if let Ok(token_data) = validate_token_with_policy(
            &token,
            &app_state.jwt_secret,
            app_state.jwt_issuer.as_deref(),
            app_state.jwt_audience.as_deref(),
        ) {
            if ensure_user_access_active(&app_state, token_data.claims.sub)
                .await
                .is_ok()
            {
                return Ok(PolymorphicAuth {
                    user_id: token_data.claims.sub,
                    email: token_data.claims.email,
                    role: token_data.claims.role,
                    org_id: token_data.claims.org_id,
                    is_api_key: false,
                });
            }
        }

        // API keys are only sent as Bearer tokens
        if is_bearer {
            if let Ok(prefix) = extract_prefix(&token) {
                let entity: Option<(Uuid, String, EntityType, Option<Uuid>, String, String)> =
                    sqlx::query_as(
                        r#"
                    SELECT id, email, entity_type, org_id, role, api_key_hash
                    FROM users
                    WHERE api_key_prefix = $1
                        AND entity_type IN ('agent', 'service', 'ci')
                        AND is_active = true
                        AND deleted_at IS NULL
                    "#,
                    )
                    .bind(&prefix)
                    .fetch_optional(&app_state.db)
                    .await?;

                if let Some((user_id, email, entity_type, org_id, role, hash)) = entity {
                    if let Ok(true) = validate_api_key(&token, &hash).await {
                        // Enforce per-entity rate limiting
                        let tier = entity_type.default_rate_limit();
                        RateLimitService::new(app_state.redis.clone())
                            .check_rate_limit(&user_id, tier)
                            .await?;
                        return Ok(PolymorphicAuth {
                            user_id,
                            email,
                            role,
                            org_id,
                            is_api_key: true,
                        });
                    }
                }
            }
        }

        Err(AppError::Unauthorized(
            "Invalid or missing authentication credentials".to_string(),
        ))
    }
}

/// MCP (Model Context Protocol) authentication extractor (Phase 2)
///
/// This is a stub for Phase 2 MCP authentication. It will be implemented
/// when MCP support is added to the system.
///
/// # Security Note
///
/// MCP authentication will likely use OAuth 2.0 or similar token-based auth.
/// The exact mechanism will be defined in Phase 2.
#[derive(Debug, Clone)]
pub struct McpAuth {
    pub client_id: String,
    pub scopes: Vec<String>,
}

impl<S> FromRequestParts<S> for McpAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Stub implementation for Phase 2
        Err(AppError::Unauthorized(
            "MCP authentication not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_auth_struct_creation() {
        let auth = ApiKeyAuth {
            user_id: Uuid::new_v4(),
            email: "agent@example.com".to_string(),
            entity_type: EntityType::Agent,
            org_id: Some(Uuid::new_v4()),
            role: "member".to_string(),
        };

        assert_eq!(auth.entity_type, EntityType::Agent);
    }

    #[test]
    fn test_mcp_auth_struct_creation() {
        let auth = McpAuth {
            client_id: "test-client".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };

        assert_eq!(auth.client_id, "test-client");
        assert_eq!(auth.scopes.len(), 2);
    }

    #[test]
    fn test_polymorphic_auth_type() {
        let jwt_auth = PolymorphicAuth {
            user_id: Uuid::new_v4(),
            email: "user@example.com".to_string(),
            role: "member".to_string(),
            org_id: None,
            is_api_key: false,
        };
        assert_eq!(jwt_auth.auth_type(), "jwt");

        let api_auth = PolymorphicAuth {
            user_id: Uuid::new_v4(),
            email: "agent@example.com".to_string(),
            role: "member".to_string(),
            org_id: None,
            is_api_key: true,
        };
        assert_eq!(api_auth.auth_type(), "api_key");
    }
}
