//! JWT token handling

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User email
    pub email: String,
    /// User role
    pub role: String,
    /// Organization ID (optional)
    pub org_id: Option<Uuid>,
    /// JWT issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// JWT audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(
        user_id: Uuid,
        email: String,
        role: String,
        org_id: Option<Uuid>,
        issuer: Option<String>,
        audience: Option<String>,
        expiry_hours: u64,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiry_hours as i64);

        Self {
            sub: user_id,
            email,
            role,
            org_id,
            iss: issuer,
            aud: audience,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

/// Create a JWT token for a user
pub fn create_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    org_id: Option<Uuid>,
    secret: &str,
    expiry_hours: u64,
) -> Result<String, AppError> {
    create_token_with_policy(
        user_id,
        email,
        role,
        org_id,
        secret,
        None,
        None,
        expiry_hours,
    )
}

/// Create a JWT token for a user with optional issuer/audience policy.
#[allow(clippy::too_many_arguments)]
pub fn create_token_with_policy(
    user_id: Uuid,
    email: &str,
    role: &str,
    org_id: Option<Uuid>,
    secret: &str,
    issuer: Option<&str>,
    audience: Option<&str>,
    expiry_hours: u64,
) -> Result<String, AppError> {
    let claims = Claims::new(
        user_id,
        email.to_string(),
        role.to_string(),
        org_id,
        issuer.map(|s| s.to_string()),
        audience.map(|s| s.to_string()),
        expiry_hours,
    );

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str, secret: &str) -> Result<TokenData<Claims>, AppError> {
    validate_token_with_policy(token, secret, None, None)
}

/// Validate and decode a JWT token with optional issuer/audience requirements.
pub fn validate_token_with_policy(
    token: &str,
    secret: &str,
    issuer: Option<&str>,
    audience: Option<&str>,
) -> Result<TokenData<Claims>, AppError> {
    let mut validation = Validation::default();
    if let Some(aud) = audience {
        validation.set_audience(&[aud]);
    }
    if let Some(iss) = issuer {
        validation.set_issuer(&[iss]);
    }

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret-key";

        let token = create_token(user_id, "test@example.com", "member", None, secret, 24).unwrap();

        let decoded = validate_token(&token, secret).unwrap();
        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.email, "test@example.com");
    }

    #[test]
    fn test_invalid_token() {
        let result = validate_token("invalid-token", "secret");
        assert!(result.is_err());
    }
}
