//! MCP Security Module
//!
//! Handles user approval workflows, scope validation, and audit logging
//! for MCP tool invocations.

use chrono::{DateTime, Utc};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Security context for MCP operations
#[derive(Clone)]
pub struct McpSecurityContext {
    /// Authenticated user ID
    pub user_id: Uuid,
    /// Organization ID (if applicable)
    pub org_id: Option<Uuid>,
    /// Approval store for pending approvals
    pub approval_store: McpApprovalStore,
    /// Audit logger
    pub audit_log: McpAuditLog,
    /// Redis connection pool (optional)
    pub redis: Option<deadpool_redis::Pool>,
    /// Database pool (optional)
    pub db: Option<sqlx::PgPool>,
}

impl McpSecurityContext {
    pub fn new(user_id: Uuid, approval_store: McpApprovalStore, audit_log: McpAuditLog) -> Self {
        Self {
            user_id,
            org_id: None,
            approval_store,
            audit_log,
            redis: None,
            db: None,
        }
    }

    pub fn with_org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_redis(mut self, redis: deadpool_redis::Pool) -> Self {
        self.redis = Some(redis);
        self
    }

    pub fn with_db(mut self, db: sqlx::PgPool) -> Self {
        self.db = Some(db);
        self
    }
}

/// Pending approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub tool_name: String,
    pub user_id: Uuid,
    pub arguments: Value,
    pub status: ApprovalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub denied_at: Option<DateTime<Utc>>,
}

impl PendingApproval {
    pub fn new(
        id: impl Into<String>,
        tool_name: impl Into<String>,
        user_id: Uuid,
        arguments: Value,
        timeout_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            tool_name: tool_name.into(),
            user_id,
            arguments,
            status: ApprovalStatus::Pending,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(timeout_seconds as i64),
            approved_at: None,
            denied_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_pending(&self) -> bool {
        self.status == ApprovalStatus::Pending && !self.is_expired()
    }
}

/// Approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// Approval store error
#[derive(Debug, Error)]
pub enum ApprovalStoreError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// MCP Approval Store
#[derive(Clone)]
pub struct McpApprovalStore {
    inner: Arc<RwLock<ApprovalStoreInner>>,
}

struct ApprovalStoreInner {
    redis: Option<deadpool_redis::Pool>,
    // In-memory fallback for testing
    memory_store: HashMap<String, PendingApproval>,
}

impl McpApprovalStore {
    /// Create a new approval store with Redis backend
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        let inner = ApprovalStoreInner {
            redis: Some(redis),
            memory_store: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Create a mock approval store for testing
    pub fn new_mock() -> Self {
        let inner = ApprovalStoreInner {
            redis: None,
            memory_store: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Request approval for a tool call
    pub async fn request_approval(
        &self,
        key: &str,
        tool_name: &str,
        user_id: &Uuid,
        arguments: Value,
    ) -> Result<PendingApproval, ApprovalStoreError> {
        let approval = PendingApproval::new(
            key.to_string(),
            tool_name,
            *user_id,
            arguments,
            300, // 5 minute default timeout
        );

        let mut inner = self.inner.write().await;

        if let Some(ref redis) = inner.redis {
            // Store in Redis
            let value = serde_json::to_string(&approval)
                .map_err(|e| ApprovalStoreError::Serialization(e.to_string()))?;
            let ttl = 300i64; // 5 minutes

            let mut conn = redis.get().await.map_err(|e| {
                ApprovalStoreError::Internal(format!("Redis connection error: {}", e))
            })?;

            let redis_key = format!("mcp:approval:{}", key);
            conn.set_ex::<_, _, ()>(redis_key, value, ttl as u64)
                .await?;
        } else {
            // Store in memory
            inner.memory_store.insert(key.to_string(), approval.clone());
        }

        Ok(approval)
    }

    /// Check if an approval key is approved
    pub async fn is_approved(&self, key: &str) -> Result<bool, ApprovalStoreError> {
        let approval = self.get_approval(key).await?;

        match approval {
            Some(a) => Ok(a.status == ApprovalStatus::Approved),
            None => Ok(false),
        }
    }

    /// Get pending approval by key
    pub async fn get_approval(
        &self,
        key: &str,
    ) -> Result<Option<PendingApproval>, ApprovalStoreError> {
        let inner = self.inner.write().await;

        if let Some(ref redis) = inner.redis {
            let mut conn = redis.get().await.map_err(|e| {
                ApprovalStoreError::Internal(format!("Redis connection error: {}", e))
            })?;

            let redis_key = format!("mcp:approval:{}", key);
            let value: Option<String> = conn.get::<_, Option<String>>(&redis_key).await?;

            match value {
                Some(v) => {
                    let approval: PendingApproval = serde_json::from_str(&v)
                        .map_err(|e| ApprovalStoreError::Serialization(e.to_string()))?;

                    // Check if expired
                    if approval.is_expired() && approval.status == ApprovalStatus::Pending {
                        let mut expired = approval.clone();
                        expired.status = ApprovalStatus::Expired;
                        return Ok(Some(expired));
                    }

                    Ok(Some(approval))
                }
                None => Ok(None),
            }
        } else {
            // Check memory store
            match inner.memory_store.get(key) {
                Some(approval) => {
                    let mut approval = approval.clone();
                    if approval.is_expired() && approval.status == ApprovalStatus::Pending {
                        approval.status = ApprovalStatus::Expired;
                    }
                    Ok(Some(approval))
                }
                None => Ok(None),
            }
        }
    }

    /// Approve a pending request
    pub async fn approve(&self, key: &str) -> Result<PendingApproval, ApprovalStoreError> {
        let mut inner = self.inner.write().await;

        if let Some(ref redis) = inner.redis {
            // Get current approval
            let mut conn = redis.get().await.map_err(|e| {
                ApprovalStoreError::Internal(format!("Redis connection error: {}", e))
            })?;

            let redis_key = format!("mcp:approval:{}", key);
            let value: Option<String> = conn.get::<_, Option<String>>(&redis_key).await?;

            match value {
                Some(v) => {
                    let mut approval: PendingApproval = serde_json::from_str(&v)
                        .map_err(|e| ApprovalStoreError::Serialization(e.to_string()))?;

                    if approval.status != ApprovalStatus::Pending {
                        return Err(ApprovalStoreError::Internal(
                            "Approval is not pending".to_string(),
                        ));
                    }

                    approval.status = ApprovalStatus::Approved;
                    approval.approved_at = Some(Utc::now());

                    let new_value = serde_json::to_string(&approval)
                        .map_err(|e| ApprovalStoreError::Serialization(e.to_string()))?;

                    let redis_key = format!("mcp:approval:{}", key);
                    conn.set::<_, _, ()>(redis_key, new_value).await?;

                    Ok(approval)
                }
                None => Err(ApprovalStoreError::NotFound(key.to_string())),
            }
        } else {
            // Memory store
            match inner.memory_store.get_mut(key) {
                Some(approval) => {
                    if approval.status != ApprovalStatus::Pending {
                        return Err(ApprovalStoreError::Internal(
                            "Approval is not pending".to_string(),
                        ));
                    }

                    approval.status = ApprovalStatus::Approved;
                    approval.approved_at = Some(Utc::now());
                    Ok(approval.clone())
                }
                None => Err(ApprovalStoreError::NotFound(key.to_string())),
            }
        }
    }

    /// Deny a pending request
    pub async fn deny(&self, key: &str) -> Result<PendingApproval, ApprovalStoreError> {
        let mut inner = self.inner.write().await;

        if let Some(ref redis) = inner.redis {
            // Get current approval
            let mut conn = redis.get().await.map_err(|e| {
                ApprovalStoreError::Internal(format!("Redis connection error: {}", e))
            })?;

            let redis_key = format!("mcp:approval:{}", key);
            let value: Option<String> = conn.get::<_, Option<String>>(&redis_key).await?;

            match value {
                Some(v) => {
                    let mut approval: PendingApproval = serde_json::from_str(&v)
                        .map_err(|e| ApprovalStoreError::Serialization(e.to_string()))?;

                    if approval.status != ApprovalStatus::Pending {
                        return Err(ApprovalStoreError::Internal(
                            "Approval is not pending".to_string(),
                        ));
                    }

                    approval.status = ApprovalStatus::Denied;
                    approval.denied_at = Some(Utc::now());

                    let new_value = serde_json::to_string(&approval)
                        .map_err(|e| ApprovalStoreError::Serialization(e.to_string()))?;

                    let redis_key = format!("mcp:approval:{}", key);
                    conn.set::<_, _, ()>(redis_key, new_value).await?;

                    Ok(approval)
                }
                None => Err(ApprovalStoreError::NotFound(key.to_string())),
            }
        } else {
            // Memory store
            match inner.memory_store.get_mut(key) {
                Some(approval) => {
                    if approval.status != ApprovalStatus::Pending {
                        return Err(ApprovalStoreError::Internal(
                            "Approval is not pending".to_string(),
                        ));
                    }

                    approval.status = ApprovalStatus::Denied;
                    approval.denied_at = Some(Utc::now());
                    Ok(approval.clone())
                }
                None => Err(ApprovalStoreError::NotFound(key.to_string())),
            }
        }
    }

    /// List pending approvals for a user
    pub async fn list_pending_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<PendingApproval>, ApprovalStoreError> {
        let inner = self.inner.read().await;

        // In a real implementation, we'd use Redis SCAN or a database query
        // For now, filter from memory store
        let pending: Vec<PendingApproval> = inner
            .memory_store
            .values()
            .filter(|a| a.user_id == user_id && a.is_pending())
            .cloned()
            .collect();

        Ok(pending)
    }
}

/// MCP Audit Log Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub tool_name: String,
    pub arguments: Value,
    pub result: AuditResult,
    pub duration_ms: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditResult {
    Success,
    Error,
    Denied,
    RateLimited,
}

/// MCP Audit Logger
#[derive(Clone)]
pub struct McpAuditLog {
    inner: Arc<RwLock<AuditLogInner>>,
}

struct AuditLogInner {
    #[allow(dead_code)]
    redis: Option<deadpool_redis::Pool>,
    memory_logs: Vec<AuditLogEntry>,
    enabled: bool,
}

impl McpAuditLog {
    /// Create a new audit logger
    pub fn new(redis: deadpool_redis::Pool, enabled: bool) -> Self {
        let inner = AuditLogInner {
            redis: Some(redis),
            memory_logs: Vec::new(),
            enabled,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Create a mock audit logger for testing
    pub fn new_mock() -> Self {
        let inner = AuditLogInner {
            redis: None,
            memory_logs: Vec::new(),
            enabled: true,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Log a tool invocation
    pub async fn log_invocation(
        &self,
        user_id: Uuid,
        tool_name: &str,
        arguments: &Value,
        result: AuditResult,
        duration_ms: i64,
        error_message: Option<String>,
    ) -> Result<(), ApprovalStoreError> {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id,
            tool_name: tool_name.to_string(),
            arguments: arguments.clone(),
            result,
            duration_ms,
            error_message,
        };

        let mut inner = self.inner.write().await;

        if !inner.enabled {
            return Ok(());
        }

        // Store in memory for now (Redis stream support can be added later)
        inner.memory_logs.push(entry);

        Ok(())
    }

    /// Get recent audit logs (for testing)
    pub async fn get_logs(&self) -> Vec<AuditLogEntry> {
        let inner = self.inner.read().await;
        inner.memory_logs.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_approval_store_mock() {
        let store = McpApprovalStore::new_mock();
        let user_id = Uuid::new_v4();

        // Request approval
        let approval = store
            .request_approval(
                "test-key",
                "post_message",
                &user_id,
                serde_json::json!({"channel_id": "123"}),
            )
            .await
            .unwrap();

        assert_eq!(approval.tool_name, "post_message");
        assert_eq!(approval.status, ApprovalStatus::Pending);

        // Check not approved yet
        assert!(!store.is_approved("test-key").await.unwrap());

        // Approve
        let approved = store.approve("test-key").await.unwrap();
        assert_eq!(approved.status, ApprovalStatus::Approved);

        // Now approved
        assert!(store.is_approved("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_approval_deny() {
        let store = McpApprovalStore::new_mock();
        let user_id = Uuid::new_v4();

        // Request approval
        store
            .request_approval("test-key", "post_message", &user_id, serde_json::json!({}))
            .await
            .unwrap();

        // Deny
        let denied = store.deny("test-key").await.unwrap();
        assert_eq!(denied.status, ApprovalStatus::Denied);

        // Not approved
        assert!(!store.is_approved("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_audit_log_mock() {
        let audit_log = McpAuditLog::new_mock();
        let user_id = Uuid::new_v4();

        audit_log
            .log_invocation(
                user_id,
                "list_channels",
                &serde_json::json!({}),
                AuditResult::Success,
                100,
                None,
            )
            .await
            .unwrap();

        let logs = audit_log.get_logs().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].tool_name, "list_channels");
        assert_eq!(logs[0].result, AuditResult::Success);
    }
}
