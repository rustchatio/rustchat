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

#[tokio::test]
#[ignore] // Requires database
async fn test_api_key_auth_uses_prefix_lookup() {
    // This test will be implemented with integration test setup
    // For now, mark as placeholder for when database is available
    println!("Test placeholder - implement when test database available");
}

#[tokio::test]
#[ignore] // Requires database - run with: cargo test --test test_api_key_auth -- --ignored
async fn test_api_key_auth_with_prefix_lookup() {
    // This test verifies the O(1) prefix lookup works end-to-end
    // Note: Requires test database setup

    // TODO: Implement with spawn_app() when database available
    // Should test:
    // 1. Register entity with new prefixed key
    // 2. Authenticate with the key
    // 3. Verify only 1 database query was made (not N queries)

    println!("Test placeholder - implement when test database available");
}

#[tokio::test]
#[ignore]
async fn test_api_key_auth_nonexistent_prefix() {
    // Test that invalid prefix returns 401 quickly (no table scan)

    println!("Test placeholder - implement when test database available");
}

#[tokio::test]
#[ignore]
async fn test_api_key_auth_legacy_key_rejected() {
    // Test that 64-char legacy keys (no prefix) are rejected

    let _legacy_key = "abc123def456890abc123def456890abc123def456890abc123def456890abcd";

    // Should fail with 401 - Invalid API key format
    println!("Test placeholder - verify legacy key rejection");
}

#[tokio::test]
#[ignore] // Performance test - run manually
async fn test_api_key_auth_performance_with_1000_entities() {
    // This test verifies O(1) performance at scale
    // Goal: Auth latency < 50ms avg with 1000 entities

    // TODO: Implement when test database available
    // 1. Create 1000 test entities
    // 2. Measure auth latency for 100 random requests
    // 3. Assert avg latency < 50ms
    // 4. Assert P95 latency < 100ms

    println!("Performance test placeholder - implement when test database available");
}
