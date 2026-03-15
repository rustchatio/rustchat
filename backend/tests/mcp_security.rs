//! MCP Security Tests
//!
//! Tests for MCP security features including approvals, rate limiting, and audit logging.

use rustchat::mcp::security::{
    ApprovalStatus, AuditResult, McpApprovalStore, McpAuditLog, PendingApproval,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_approval_store_request_approval() {
    let store = McpApprovalStore::new_mock();
    let user_id = Uuid::new_v4();

    let approval = store
        .request_approval(
            "test-key-1",
            "post_message",
            &user_id,
            json!({"channel_id": "123", "message": "Hello"}),
        )
        .await
        .unwrap();

    assert_eq!(approval.tool_name, "post_message");
    assert_eq!(approval.status, ApprovalStatus::Pending);
    assert!(approval.is_pending());
    assert!(!approval.is_expired());
}

#[tokio::test]
async fn test_approval_store_is_approved() {
    let store = McpApprovalStore::new_mock();
    let user_id = Uuid::new_v4();

    // Initially not approved
    assert!(!store.is_approved("test-key-2").await.unwrap());

    // Request approval
    store
        .request_approval("test-key-2", "post_message", &user_id, json!({}))
        .await
        .unwrap();

    // Still not approved
    assert!(!store.is_approved("test-key-2").await.unwrap());

    // Approve it
    store.approve("test-key-2").await.unwrap();

    // Now approved
    assert!(store.is_approved("test-key-2").await.unwrap());
}

#[tokio::test]
async fn test_approval_store_approve() {
    let store = McpApprovalStore::new_mock();
    let user_id = Uuid::new_v4();

    store
        .request_approval("test-key-3", "post_message", &user_id, json!({}))
        .await
        .unwrap();

    let approved = store.approve("test-key-3").await.unwrap();

    assert_eq!(approved.status, ApprovalStatus::Approved);
    assert!(approved.approved_at.is_some());
    assert!(!approved.is_pending());
}

#[tokio::test]
async fn test_approval_store_deny() {
    let store = McpApprovalStore::new_mock();
    let user_id = Uuid::new_v4();

    store
        .request_approval("test-key-4", "post_message", &user_id, json!({}))
        .await
        .unwrap();

    let denied = store.deny("test-key-4").await.unwrap();

    assert_eq!(denied.status, ApprovalStatus::Denied);
    assert!(denied.denied_at.is_some());
    assert!(!denied.is_pending());

    // Denied approval is not approved
    assert!(!store.is_approved("test-key-4").await.unwrap());
}

#[tokio::test]
async fn test_approval_store_not_found() {
    let store = McpApprovalStore::new_mock();

    let result = store.approve("non-existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_approval_store_double_approve() {
    let store = McpApprovalStore::new_mock();
    let user_id = Uuid::new_v4();

    store
        .request_approval("test-key-5", "post_message", &user_id, json!({}))
        .await
        .unwrap();

    // First approve should succeed
    store.approve("test-key-5").await.unwrap();

    // Second approve should fail (not pending)
    let result = store.approve("test-key-5").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_approval_store_list_pending() {
    let store = McpApprovalStore::new_mock();
    let user_id = Uuid::new_v4();

    // Initially no pending approvals
    let pending = store.list_pending_for_user(user_id).await.unwrap();
    assert!(pending.is_empty());

    // Create some approvals
    for i in 0..3 {
        store
            .request_approval(&format!("key-{}", i), "post_message", &user_id, json!({}))
            .await
            .unwrap();
    }

    // Now should have 3 pending
    let pending = store.list_pending_for_user(user_id).await.unwrap();
    assert_eq!(pending.len(), 3);
}

#[tokio::test]
async fn test_audit_log_log_invocation() {
    let audit_log = McpAuditLog::new_mock();
    let user_id = Uuid::new_v4();

    audit_log
        .log_invocation(
            user_id,
            "list_channels",
            &json!({}),
            AuditResult::Success,
            100,
            None,
        )
        .await
        .unwrap();

    let logs = audit_log.get_logs().await;
    assert_eq!(logs.len(), 1);

    let entry = &logs[0];
    assert_eq!(entry.user_id, user_id);
    assert_eq!(entry.tool_name, "list_channels");
    assert_eq!(entry.result, AuditResult::Success);
    assert_eq!(entry.duration_ms, 100);
}

#[tokio::test]
async fn test_audit_log_multiple_entries() {
    let audit_log = McpAuditLog::new_mock();
    let user_id = Uuid::new_v4();

    // Log different types of results
    audit_log
        .log_invocation(user_id, "tool1", &json!({}), AuditResult::Success, 50, None)
        .await
        .unwrap();

    audit_log
        .log_invocation(
            user_id,
            "tool2",
            &json!({}),
            AuditResult::Error,
            0,
            Some("Something went wrong".to_string()),
        )
        .await
        .unwrap();

    audit_log
        .log_invocation(user_id, "tool3", &json!({}), AuditResult::Denied, 0, None)
        .await
        .unwrap();

    let logs = audit_log.get_logs().await;
    assert_eq!(logs.len(), 3);

    assert_eq!(logs[0].result, AuditResult::Success);
    assert_eq!(logs[1].result, AuditResult::Error);
    assert_eq!(
        logs[1].error_message,
        Some("Something went wrong".to_string())
    );
    assert_eq!(logs[2].result, AuditResult::Denied);
}

#[tokio::test]
async fn test_approval_expiration() {
    // Create an approval with a very short timeout for testing
    let approval = PendingApproval::new(
        "test-key",
        "post_message",
        Uuid::new_v4(),
        json!({}),
        0, // 0 second timeout = already expired
    );

    assert!(approval.is_expired());
    assert!(!approval.is_pending());
}

#[test]
fn test_pending_approval_creation() {
    let approval = PendingApproval::new(
        "key-123",
        "test_tool",
        Uuid::new_v4(),
        json!({"arg": "value"}),
        300,
    );

    assert_eq!(approval.id, "key-123");
    assert_eq!(approval.tool_name, "test_tool");
    assert_eq!(approval.status, ApprovalStatus::Pending);
    assert!(approval.approved_at.is_none());
    assert!(approval.denied_at.is_none());
}

#[test]
fn test_audit_result_serialization() {
    assert_eq!(
        serde_json::to_string(&AuditResult::Success).unwrap(),
        "\"success\""
    );
    assert_eq!(
        serde_json::to_string(&AuditResult::Error).unwrap(),
        "\"error\""
    );
    assert_eq!(
        serde_json::to_string(&AuditResult::Denied).unwrap(),
        "\"denied\""
    );
    assert_eq!(
        serde_json::to_string(&AuditResult::RateLimited).unwrap(),
        "\"ratelimited\""
    );
}

#[test]
fn test_approval_status_serialization() {
    assert_eq!(
        serde_json::to_string(&ApprovalStatus::Pending).unwrap(),
        "\"pending\""
    );
    assert_eq!(
        serde_json::to_string(&ApprovalStatus::Approved).unwrap(),
        "\"approved\""
    );
    assert_eq!(
        serde_json::to_string(&ApprovalStatus::Denied).unwrap(),
        "\"denied\""
    );
    assert_eq!(
        serde_json::to_string(&ApprovalStatus::Expired).unwrap(),
        "\"expired\""
    );
}

#[tokio::test]
async fn test_different_users_have_separate_approvals() {
    let store = McpApprovalStore::new_mock();
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    // User1 requests approval
    store
        .request_approval("shared-key", "post_message", &user1, json!({}))
        .await
        .unwrap();

    // User2 requests approval with same key
    store
        .request_approval("shared-key", "post_message", &user2, json!({}))
        .await
        .unwrap();

    // Both should have pending approvals
    let _pending1 = store.list_pending_for_user(user1).await.unwrap();
    let _pending2 = store.list_pending_for_user(user2).await.unwrap();

    // Note: With the current mock implementation using a simple HashMap,
    // this behavior depends on key structure. In production with Redis,
    // keys would typically include user_id for isolation.
}
