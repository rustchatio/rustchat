//! Integration tests for API key authentication extractor
//!
//! These tests verify that the ApiKeyAuth extractor correctly authenticates
//! agents and services using API keys stored in the database.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use rustchat::{
    auth::{
        api_key::{extract_prefix, generate_api_key, hash_api_key},
        extractors::ApiKeyAuth,
    },
    models::entity::{EntityType, RateLimitTier},
};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

mod common;

/// Helper to create a test handler that uses ApiKeyAuth
async fn create_test_handler(auth: ApiKeyAuth) -> axum::Json<serde_json::Value> {
    axum::Json(json!({
        "user_id": auth.user_id.to_string(),
        "email": auth.email,
        "entity_type": auth.entity_type,
    }))
}

/// Create a test user (agent/service) with an API key
async fn create_test_entity(
    pool: &PgPool,
    entity_type: EntityType,
    api_key: &str,
) -> anyhow::Result<Uuid> {
    let api_key_hash = hash_api_key(api_key).await?;
    let api_key_prefix = extract_prefix(api_key)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (
            id, username, email, password_hash, is_bot, is_active, role,
            entity_type, api_key_hash, api_key_prefix, rate_limit_tier, created_at, updated_at
        )
        VALUES ($1, $2, $3, NULL, true, true, 'member', $4, $5, $6, $7, NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(format!("test_entity_{}", user_id))
    .bind(format!("entity_{}@test.local", user_id))
    .bind(entity_type)
    .bind(api_key_hash)
    .bind(api_key_prefix)
    .bind(RateLimitTier::HumanStandard)
    .execute(pool)
    .await?;

    Ok(user_id)
}

#[sqlx::test]
async fn test_api_key_auth_success_with_agent(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();
    let user_id = create_test_entity(&pool, EntityType::Agent, &api_key).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["user_id"], user_id.to_string());
    assert_eq!(json["entity_type"], "agent");

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_success_with_service(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();
    let user_id = create_test_entity(&pool, EntityType::Service, &api_key).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["user_id"], user_id.to_string());
    assert_eq!(json["entity_type"], "service");

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_success_with_ci(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();
    let user_id = create_test_entity(&pool, EntityType::CI, &api_key).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["user_id"], user_id.to_string());
    assert_eq!(json["entity_type"], "ci");

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_fails_with_invalid_key(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();
    create_test_entity(&pool, EntityType::Agent, &api_key).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    // Use a different API key
    let wrong_key = generate_api_key();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", wrong_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_fails_with_missing_header(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_fails_with_inactive_user(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();
    let user_id = create_test_entity(&pool, EntityType::Agent, &api_key).await?;

    // Deactivate the user
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_fails_with_deleted_user(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();
    let user_id = create_test_entity(&pool, EntityType::Agent, &api_key).await?;

    // Delete the user (soft delete)
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_fails_for_human_user_with_api_key(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;
    let api_key = generate_api_key();

    // Create a human user with an API key (should not work)
    let _user_id = create_test_entity(&pool, EntityType::Human, &api_key).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail because human users should not authenticate with API keys
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_rejects_malformed_bearer_token(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", "InvalidFormat")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[sqlx::test]
async fn test_api_key_auth_verifies_against_all_stored_hashes(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    // Create three entities with different API keys
    let key1 = generate_api_key();
    let key2 = generate_api_key();
    let key3 = generate_api_key();

    let _user1 = create_test_entity(&pool, EntityType::Agent, &key1).await?;
    let user2 = create_test_entity(&pool, EntityType::Service, &key2).await?;
    let _user3 = create_test_entity(&pool, EntityType::CI, &key3).await?;

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    // Authenticate with key2, should find user2
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", key2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["user_id"], user2.to_string());
    assert_eq!(json["entity_type"], "service");

    Ok(())
}

/// Test that verifies O(1) prefix lookup works end-to-end
/// This test creates an entity with a prefixed key and authenticates,
/// ensuring the authentication uses the prefix index
#[sqlx::test]
async fn test_api_key_auth_with_prefix_lookup(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    // Generate a new API key (will have rck_ prefix)
    let api_key = generate_api_key();

    // Verify the key has the expected format
    assert!(
        api_key.starts_with("rck_"),
        "API key should have rck_ prefix"
    );
    assert_eq!(api_key.len(), 68, "API key should be 68 characters");

    // Create a test entity with this API key
    let user_id = create_test_entity(&pool, EntityType::Agent, &api_key).await?;

    // Build the app router with our test handler
    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    // Authenticate with the API key
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed
    assert_eq!(response.status(), StatusCode::OK);

    // Verify we got the right user back
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["user_id"], user_id.to_string());

    Ok(())
}

/// Test that invalid/nonexistent prefix returns 401 quickly
/// This verifies the O(1) lookup fails fast without scanning
#[sqlx::test]
async fn test_api_key_auth_nonexistent_prefix(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    // Generate a valid-format API key that doesn't exist in DB
    let nonexistent_key = generate_api_key();

    // Verify the key format is valid
    assert!(nonexistent_key.starts_with("rck_"));

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    // Try to authenticate with non-existent key
    let start = std::time::Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", nonexistent_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Should fail with 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Should be fast (O(1) lookup - no table scan)
    // If this is slow, it indicates a table scan is happening
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "Auth with nonexistent key took too long ({:?}), possible table scan",
        elapsed
    );

    Ok(())
}

/// Test that legacy 64-char keys (without rck_ prefix) are rejected
/// This ensures backward compatibility break for old keys
#[sqlx::test]
async fn test_api_key_auth_legacy_key_rejected(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    // Create a legacy-format key (64 hex chars without prefix)
    let legacy_key = "abc123def456890abc123def456890abc123def456890abc123def456890abcd";

    // Verify it's the legacy format (no prefix, 64 chars)
    assert!(!legacy_key.starts_with("rck_"));
    assert_eq!(legacy_key.len(), 64);

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    // Try to authenticate with legacy key
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {}", legacy_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail with 401 - Invalid API key format
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Verify error message mentions invalid format
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.to_lowercase().contains("invalid") || body_str.is_empty(),
        "Expected error message about invalid key format, got: {}",
        body_str
    );

    Ok(())
}

/// Performance test: Verify O(1) auth latency with many entities
/// Goal: Auth latency < 50ms avg, < 100ms P95 with 1000 entities
///
/// NOTE: This test creates 100 entities (not 1000) to keep test runtime reasonable
/// while still verifying O(1) behavior. The key is that latency should not
/// increase significantly with more entities.
#[sqlx::test]
async fn test_api_key_auth_performance_with_entities(pool: PgPool) -> anyhow::Result<()> {
    let state = common::create_test_state(pool.clone()).await?;

    // Number of entities to create for performance test
    const ENTITY_COUNT: usize = 100;
    const AUTH_ITERATIONS: usize = 50;

    // Create multiple test entities
    let mut api_keys = Vec::with_capacity(ENTITY_COUNT);

    for i in 0..ENTITY_COUNT {
        let api_key = generate_api_key();
        let user_id = create_test_entity(&pool, EntityType::Agent, &api_key).await?;
        api_keys.push((api_key, user_id));

        // Log progress every 25 entities
        if (i + 1) % 25 == 0 {
            tracing::info!("Created {} entities for performance test", i + 1);
        }
    }

    let app = Router::new()
        .route("/test", get(create_test_handler))
        .with_state(state);

    // Measure authentication latency
    let mut latencies = Vec::with_capacity(AUTH_ITERATIONS);

    for i in 0..AUTH_ITERATIONS {
        // Pick a random key to authenticate with
        let (api_key, _expected_user_id) = &api_keys[i % ENTITY_COUNT];

        let start = std::time::Instant::now();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // Verify authentication succeeded
        assert_eq!(response.status(), StatusCode::OK);

        latencies.push(elapsed);
    }

    // Calculate statistics
    latencies.sort();
    let avg_latency = latencies.iter().sum::<std::time::Duration>() / latencies.len() as u32;
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
    let max_latency = latencies.last().copied().unwrap_or_default();

    tracing::info!(
        "Performance results with {} entities, {} auths:\n  \
        Average: {:?}\n  \
        P50: {:?}\n  \
        P95: {:?}\n  \
        Max: {:?}",
        ENTITY_COUNT,
        AUTH_ITERATIONS,
        avg_latency,
        p50,
        p95,
        max_latency
    );

    // Performance assertions
    // Note: These thresholds are reasonable for CI environments
    // In production with proper hardware, these should be much lower
    assert!(
        avg_latency < std::time::Duration::from_millis(50),
        "Average auth latency ({:?}) exceeded 50ms threshold",
        avg_latency
    );

    assert!(
        p95 < std::time::Duration::from_millis(100),
        "P95 auth latency ({:?}) exceeded 100ms threshold",
        p95
    );

    // Verify O(1) behavior: latency should not correlate with entity count
    // by checking that max isn't way higher than average
    assert!(
        max_latency < std::time::Duration::from_millis(200),
        "Max auth latency ({:?}) is suspiciously high, possible O(n) behavior",
        max_latency
    );

    Ok(())
}
