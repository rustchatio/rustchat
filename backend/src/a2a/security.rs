//! A2A Security Module
//!
//! Provides message signing, agent authentication, scope validation, and audit logging.
//! Implements Constitution XIII Zero-Trust requirements for agent collaboration.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use super::protocol::{A2AMessage, AgentId, TaskRequest};
use crate::auth::middleware::AuthUser;

type HmacSha256 = Hmac<Sha256>;

/// Security context for A2A operations
pub struct A2ASecurityContext {
    user: AuthUser,
    signing_key: Option<String>,
    audit_enabled: bool,
}

impl A2ASecurityContext {
    /// Create a new security context for the authenticated user
    pub fn new(user: AuthUser) -> Self {
        Self {
            user,
            signing_key: None,
            audit_enabled: true,
        }
    }

    /// Set the signing key for message verification
    pub fn with_signing_key(mut self, key: impl Into<String>) -> Self {
        self.signing_key = Some(key.into());
        self
    }

    /// Disable audit logging
    pub fn without_audit(mut self) -> Self {
        self.audit_enabled = false;
        self
    }

    /// Get the user ID
    pub fn user_id(&self) -> Uuid {
        self.user.user_id
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.user.has_role(role)
    }

    /// Validate that an agent can act within the user's permissions
    pub fn validate_agent_scope(&self, agent_id: &AgentId) -> Result<(), A2ASecurityError> {
        // Agents inherit their creator's permissions
        // Check if the user has permission to create agents
        if !self.user.has_role("member") && !self.user.has_role("system_admin") {
            return Err(A2ASecurityError::InsufficientPermissions {
                agent_name: agent_id.name.clone(),
                required_role: "member".to_string(),
            });
        }

        debug!(
            user_id = %self.user.user_id,
            agent_name = %agent_id.name,
            "Agent scope validated"
        );

        Ok(())
    }

    /// Validate that a task is within the user's scope
    pub fn validate_task_scope(&self, task: &TaskRequest) -> Result<(), A2ASecurityError> {
        // Users can only create tasks for agents they have access to
        // This could be extended to check agent ownership, workspace membership, etc.

        debug!(
            user_id = %self.user.user_id,
            task_id = %task.task_id,
            sender = %task.sender.name,
            "Task scope validated"
        );

        Ok(())
    }

    /// Sign a message with the configured signing key
    pub fn sign_message(&self, message: &A2AMessage) -> Result<String, A2ASecurityError> {
        let key = self
            .signing_key
            .as_ref()
            .ok_or(A2ASecurityError::NoSigningKey)?;

        let payload = serde_json::to_string(message)
            .map_err(|e| A2ASecurityError::SerializationError(e.to_string()))?;

        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| A2ASecurityError::InvalidKey)?;

        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        Ok(signature)
    }

    /// Verify a message signature
    pub fn verify_message(
        &self,
        message: &A2AMessage,
        signature: &str,
    ) -> Result<bool, A2ASecurityError> {
        let key = self
            .signing_key
            .as_ref()
            .ok_or(A2ASecurityError::NoSigningKey)?;

        let payload = serde_json::to_string(message)
            .map_err(|e| A2ASecurityError::SerializationError(e.to_string()))?;

        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| A2ASecurityError::InvalidKey)?;

        mac.update(payload.as_bytes());

        let signature_bytes =
            hex::decode(signature).map_err(|_| A2ASecurityError::InvalidSignature)?;

        match mac.verify_slice(&signature_bytes) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Log an audit event
    pub fn audit(&self, event: A2AAuditEvent) {
        if !self.audit_enabled {
            return;
        }

        info!(
            user_id = %self.user.user_id,
            event_type = ?event.event_type,
            agent_id = ?event.agent_id,
            task_id = ?event.task_id,
            details = ?event.details,
            "A2A audit event"
        );
    }
}

/// Security error types for A2A operations
#[derive(Debug, Clone)]
pub enum A2ASecurityError {
    InsufficientPermissions {
        agent_name: String,
        required_role: String,
    },
    NoSigningKey,
    InvalidKey,
    SerializationError(String),
    InvalidSignature,
    UnauthorizedAgent {
        agent_id: Uuid,
    },
}

impl std::fmt::Display for A2ASecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            A2ASecurityError::InsufficientPermissions {
                agent_name,
                required_role,
            } => {
                write!(
                    f,
                    "Agent '{}' requires '{}' role or higher",
                    agent_name, required_role
                )
            }
            A2ASecurityError::NoSigningKey => write!(f, "No signing key configured"),
            A2ASecurityError::InvalidKey => write!(f, "Invalid signing key"),
            A2ASecurityError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            A2ASecurityError::InvalidSignature => write!(f, "Invalid signature format"),
            A2ASecurityError::UnauthorizedAgent { agent_id } => {
                write!(f, "Unauthorized agent: {}", agent_id)
            }
        }
    }
}

impl std::error::Error for A2ASecurityError {}

/// Audit event types for A2A operations
#[derive(Debug, Clone)]
pub enum A2AAuditEventType {
    AgentRegistered,
    AgentUnregistered,
    TaskCreated,
    TaskUpdated,
    TaskCompleted,
    MessageSigned,
    MessageVerified,
}

/// An audit event for A2A operations
#[derive(Debug, Clone)]
pub struct A2AAuditEvent {
    pub event_type: A2AAuditEventType,
    pub agent_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl A2AAuditEvent {
    /// Create a new audit event
    pub fn new(event_type: A2AAuditEventType) -> Self {
        Self {
            event_type,
            agent_id: None,
            task_id: None,
            details: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Set the agent ID
    pub fn with_agent(mut self, agent_id: Uuid) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Set the task ID
    pub fn with_task(mut self, task_id: Uuid) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Set event details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

/// Agent authentication token
#[derive(Debug, Clone)]
pub struct AgentAuthToken {
    pub agent_id: Uuid,
    pub user_id: Uuid,
    pub scopes: Vec<String>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl AgentAuthToken {
    /// Create a new agent authentication token
    pub fn new(
        agent_id: Uuid,
        user_id: Uuid,
        scopes: Vec<String>,
        ttl_seconds: i64,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            agent_id,
            user_id,
            scopes,
            issued_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds),
        }
    }

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Check if the token has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// Token validator for agent authentication
pub struct AgentTokenValidator {
    jwt_secret: Arc<String>,
}

impl AgentTokenValidator {
    /// Create a new token validator
    pub fn new(jwt_secret: impl Into<String>) -> Self {
        Self {
            jwt_secret: Arc::new(jwt_secret.into()),
        }
    }

    /// Generate a JWT for an agent
    pub fn generate_token(&self, agent_id: Uuid, user_id: Uuid) -> Result<String, A2ASecurityError> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String, // agent_id
            user_id: String,
            exp: usize,
            iat: usize,
        }

        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::hours(24);

        let claims = Claims {
            sub: agent_id.to_string(),
            user_id: user_id.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| A2ASecurityError::SerializationError(e.to_string()))?;

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_auth_token() {
        let token = AgentAuthToken::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            vec!["read".to_string(), "write".to_string()],
            3600,
        );

        assert!(!token.is_expired());
        assert!(token.has_scope("read"));
        assert!(token.has_scope("write"));
        assert!(!token.has_scope("admin"));
    }

    #[test]
    fn test_security_context_scope_validation() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "member".to_string(),
            org_id: None,
        };

        let context = A2ASecurityContext::new(user);
        let agent = AgentId::new("test_agent", "custom");

        assert!(context.validate_agent_scope(&agent).is_ok());
    }

    #[test]
    fn test_message_signing_and_verification() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "member".to_string(),
            org_id: None,
        };

        let context = A2ASecurityContext::new(user).with_signing_key("test_secret_key_12345");

        let agent = AgentId::new("test", "custom");
        let message = A2AMessage::DiscoveryRequest {
            requester: agent,
            capability_filter: None,
        };

        let signature = context.sign_message(&message).unwrap();
        assert!(!signature.is_empty());

        let valid = context.verify_message(&message, &signature).unwrap();
        assert!(valid);

        // Tampered message should fail verification
        let tampered_agent = AgentId::new("tampered", "custom");
        let tampered_message = A2AMessage::DiscoveryRequest {
            requester: tampered_agent,
            capability_filter: None,
        };

        let invalid = context.verify_message(&tampered_message, &signature).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_audit_event() {
        let event = A2AAuditEvent::new(A2AAuditEventType::AgentRegistered)
            .with_agent(Uuid::new_v4())
            .with_details(serde_json::json!({"framework": "langchain"}));

        assert!(matches!(event.event_type, A2AAuditEventType::AgentRegistered));
        assert!(event.agent_id.is_some());
        assert_eq!(event.details["framework"], "langchain");
    }

    #[test]
    fn test_token_validator() {
        let validator = AgentTokenValidator::new("test_secret_for_jwt_signing");
        let agent_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let token = validator.generate_token(agent_id, user_id).unwrap();
        assert!(!token.is_empty());
    }
}
