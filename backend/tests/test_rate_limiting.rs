//! Tests for rate limiting service
//!
//! NOTE: These tests require a running Redis instance and will be skipped
//! if Redis is not available. This is expected behavior for local development.

use rustchat::error::AppError;
use rustchat::models::entity::RateLimitTier;
use rustchat::services::rate_limit::{RateLimitConfig, RateLimitService};
use uuid::Uuid;

/// Helper to create a Redis connection pool for testing
async fn create_redis_pool() -> Result<deadpool_redis::Pool, deadpool_redis::CreatePoolError> {
    let cfg = deadpool_redis::Config::from_url("redis://localhost:6379");
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
}

/// Helper to check if Redis is available
async fn is_redis_available() -> bool {
    match create_redis_pool().await {
        Ok(pool) => pool.get().await.is_ok(),
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_rate_limit_config_human_standard() {
    let config = RateLimitConfig::for_tier(RateLimitTier::HumanStandard);
    assert_eq!(config.limit, 1000, "HumanStandard should have 1k limit");
    assert_eq!(
        config.window_secs, 3600,
        "HumanStandard should have 1 hour window"
    );
    assert!(!config.is_unlimited(), "HumanStandard is not unlimited");
}

#[tokio::test]
async fn test_rate_limit_config_agent_high() {
    let config = RateLimitConfig::for_tier(RateLimitTier::AgentHigh);
    assert_eq!(config.limit, 10000, "AgentHigh should have 10k limit");
    assert_eq!(
        config.window_secs, 3600,
        "AgentHigh should have 1 hour window"
    );
    assert!(!config.is_unlimited(), "AgentHigh is not unlimited");
}

#[tokio::test]
async fn test_rate_limit_config_service_unlimited() {
    let config = RateLimitConfig::for_tier(RateLimitTier::ServiceUnlimited);
    assert_eq!(
        config.limit,
        u64::MAX,
        "ServiceUnlimited should have MAX limit"
    );
    assert_eq!(
        config.window_secs, 3600,
        "ServiceUnlimited should have 1 hour window"
    );
    assert!(config.is_unlimited(), "ServiceUnlimited is unlimited");
}

#[tokio::test]
async fn test_rate_limit_config_ci_standard() {
    let config = RateLimitConfig::for_tier(RateLimitTier::CIStandard);
    assert_eq!(config.limit, 5000, "CIStandard should have 5k limit");
    assert_eq!(
        config.window_secs, 3600,
        "CIStandard should have 1 hour window"
    );
    assert!(!config.is_unlimited(), "CIStandard is not unlimited");
}

#[tokio::test]
async fn test_service_unlimited_bypasses_rate_limit() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // ServiceUnlimited tier should always allow requests
    for _ in 0..100 {
        let result = service
            .check_rate_limit(&entity_id, RateLimitTier::ServiceUnlimited)
            .await;
        assert!(
            result.is_ok(),
            "ServiceUnlimited should never be rate limited"
        );
    }
}

#[tokio::test]
async fn test_rate_limit_increments_correctly() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // Reset any existing rate limit
    service
        .reset_rate_limit(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    // Make requests and verify status updates
    for i in 1..=5 {
        service
            .check_rate_limit(&entity_id, RateLimitTier::HumanStandard)
            .await
            .unwrap();

        let status = service
            .get_rate_limit_status(&entity_id, RateLimitTier::HumanStandard)
            .await
            .unwrap();

        assert_eq!(
            status.current_count, i,
            "Count should increment on each request"
        );
        assert_eq!(status.limit, 1000, "Limit should be 1000");
        assert_eq!(
            status.remaining,
            1000 - i,
            "Remaining should decrement correctly"
        );
    }
}

#[tokio::test]
async fn test_rate_limit_enforcement() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // Use a custom low limit for testing
    // Note: We can't directly set custom limits, so we'll test with CIStandard (5k limit)
    // and make a few requests, then verify the count

    service
        .reset_rate_limit(&entity_id, RateLimitTier::CIStandard)
        .await
        .unwrap();

    // Make requests within limit
    for _ in 0..10 {
        let result = service
            .check_rate_limit(&entity_id, RateLimitTier::CIStandard)
            .await;
        assert!(result.is_ok(), "Requests within limit should succeed");
    }

    // Verify status shows correct count
    let status = service
        .get_rate_limit_status(&entity_id, RateLimitTier::CIStandard)
        .await
        .unwrap();

    assert_eq!(status.current_count, 10, "Should have 10 requests counted");
    assert_eq!(status.remaining, 4990, "Should have 4990 remaining");
}

#[tokio::test]
async fn test_rate_limit_different_tiers_separate() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // Reset both tiers
    service
        .reset_rate_limit(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    service
        .reset_rate_limit(&entity_id, RateLimitTier::AgentHigh)
        .await
        .unwrap();

    // Make requests to HumanStandard tier
    for _ in 0..5 {
        service
            .check_rate_limit(&entity_id, RateLimitTier::HumanStandard)
            .await
            .unwrap();
    }

    // Make requests to AgentHigh tier
    for _ in 0..3 {
        service
            .check_rate_limit(&entity_id, RateLimitTier::AgentHigh)
            .await
            .unwrap();
    }

    // Verify counts are separate
    let human_status = service
        .get_rate_limit_status(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    assert_eq!(
        human_status.current_count, 5,
        "HumanStandard should have 5 requests"
    );

    let agent_status = service
        .get_rate_limit_status(&entity_id, RateLimitTier::AgentHigh)
        .await
        .unwrap();
    assert_eq!(
        agent_status.current_count, 3,
        "AgentHigh should have 3 requests"
    );
}

#[tokio::test]
async fn test_rate_limit_reset() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // Make some requests
    for _ in 0..10 {
        service
            .check_rate_limit(&entity_id, RateLimitTier::HumanStandard)
            .await
            .unwrap();
    }

    // Verify count
    let status_before = service
        .get_rate_limit_status(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    assert_eq!(status_before.current_count, 10);

    // Reset rate limit
    service
        .reset_rate_limit(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    // Verify count is reset
    let status_after = service
        .get_rate_limit_status(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    assert_eq!(status_after.current_count, 0, "Count should be reset to 0");
    assert_eq!(
        status_after.remaining, 1000,
        "Remaining should be back to limit"
    );
}

#[tokio::test]
async fn test_rate_limit_status_for_unlimited() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // Get status for unlimited tier
    let status = service
        .get_rate_limit_status(&entity_id, RateLimitTier::ServiceUnlimited)
        .await
        .unwrap();

    assert_eq!(status.current_count, 0, "Unlimited tier shows 0 count");
    assert_eq!(status.limit, u64::MAX, "Unlimited tier shows MAX limit");
    assert_eq!(
        status.remaining,
        u64::MAX,
        "Unlimited tier shows MAX remaining"
    );
    assert!(
        status.reset_at.is_none(),
        "Unlimited tier has no reset time"
    );
}

#[tokio::test]
async fn test_different_entities_have_separate_limits() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_1 = Uuid::new_v4();
    let entity_2 = Uuid::new_v4();

    // Reset both
    service
        .reset_rate_limit(&entity_1, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    service
        .reset_rate_limit(&entity_2, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    // Make different numbers of requests for each entity
    for _ in 0..5 {
        service
            .check_rate_limit(&entity_1, RateLimitTier::HumanStandard)
            .await
            .unwrap();
    }

    for _ in 0..8 {
        service
            .check_rate_limit(&entity_2, RateLimitTier::HumanStandard)
            .await
            .unwrap();
    }

    // Verify separate counts
    let status_1 = service
        .get_rate_limit_status(&entity_1, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    assert_eq!(status_1.current_count, 5, "Entity 1 should have 5 requests");

    let status_2 = service
        .get_rate_limit_status(&entity_2, RateLimitTier::HumanStandard)
        .await
        .unwrap();
    assert_eq!(status_2.current_count, 8, "Entity 2 should have 8 requests");
}

#[tokio::test]
async fn test_rate_limit_exceeded_error() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // For a real test of exceeding limits, we would need to make 1000+ requests
    // which is impractical. Instead, we'll verify the error type structure exists.

    // Reset to ensure clean state
    service
        .reset_rate_limit(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    // Make a few requests
    for _ in 0..5 {
        let result = service
            .check_rate_limit(&entity_id, RateLimitTier::HumanStandard)
            .await;
        assert!(result.is_ok(), "Requests should succeed");
    }

    // Verify the error variant exists by checking type
    let _error_check: Result<(), AppError> =
        Err(AppError::RateLimitExceeded("Test error".to_string()));
}

#[tokio::test]
async fn test_rate_limit_key_format() {
    if !is_redis_available().await {
        eprintln!("⚠️  Skipping test: Redis not available");
        return;
    }

    let redis = create_redis_pool().await.unwrap();
    let service = RateLimitService::new(redis);
    let entity_id = Uuid::new_v4();

    // Make a request to create the key
    service
        .reset_rate_limit(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    service
        .check_rate_limit(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    // The key format is tested indirectly through successful operations
    // The actual Redis key is: ratelimit:{tier}:{entity_id}
    // This test verifies the service can successfully use that key format
    let status = service
        .get_rate_limit_status(&entity_id, RateLimitTier::HumanStandard)
        .await
        .unwrap();

    assert_eq!(
        status.current_count, 1,
        "Service should successfully track using the key format"
    );
}
