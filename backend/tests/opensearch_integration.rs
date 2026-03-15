//! OpenSearch integration tests
//!
//! These tests require a running OpenSearch instance.
//! Use `docker compose up -d opensearch` to start OpenSearch before running tests.

use chrono::Utc;
use uuid::Uuid;

use rustchat::config::OpenSearchConfig;
use rustchat::search::{IndexManager, OpenSearchClient, PostDocument, SearchIndexer};

/// Test configuration for local OpenSearch
fn test_opensearch_config() -> OpenSearchConfig {
    OpenSearchConfig {
        enabled: true,
        url: std::env::var("OPENSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string()),
        username: std::env::var("OPENSEARCH_USERNAME").ok(),
        password: std::env::var("OPENSEARCH_PASSWORD").ok(),
        aws_region: None,
        aws_access_key: None,
        aws_secret_key: None,
        ssl_enabled: false,
        skip_ssl_verify: false,
        connection_timeout_secs: 10,
        request_timeout_secs: 30,
        index_prefix: format!("rustchat-test-{}", Uuid::new_v4()),
        number_of_shards: 1,
        number_of_replicas: 0,
    }
}

/// Check if OpenSearch is available
async fn is_opensearch_available() -> bool {
    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);

    if !client.is_available() {
        return false;
    }

    matches!(client.ping().await, Ok(true))
}

#[tokio::test]
async fn test_client_creation() {
    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);

    // If OpenSearch is not available, this will not be available
    if !client.is_available() {
        println!("OpenSearch not available - skipping client test");
        return;
    }
}

#[tokio::test]
async fn test_health_check() {
    // Skip if OpenSearch is not available
    if !is_opensearch_available().await {
        println!("OpenSearch not available - skipping health check test");
        return;
    }

    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);

    let health = client.health_check().await.expect("Health check failed");

    println!(
        "Cluster health: {:?}, nodes: {}, active_shards: {}",
        health.status, health.number_of_nodes, health.active_shards
    );

    // Health should be green or yellow in a healthy cluster
    assert!(
        matches!(
            health.status,
            rustchat::search::client::HealthStatus::Green
                | rustchat::search::client::HealthStatus::Yellow
        ),
        "Cluster health should be green or yellow"
    );
}

#[tokio::test]
async fn test_index_creation() {
    // Skip if OpenSearch is not available
    if !is_opensearch_available().await {
        println!("OpenSearch not available - skipping index creation test");
        return;
    }

    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);
    let manager = IndexManager::new(client);

    let index_name = manager
        .create_posts_index()
        .await
        .expect("Failed to create index");

    println!("Created index: {}", index_name);

    // Verify index exists
    let exists = manager
        .index_exists(&index_name)
        .await
        .expect("Failed to check index");
    assert!(exists, "Index should exist after creation");

    // Clean up
    manager.delete_index(&index_name).await.ok();
}

#[tokio::test]
async fn test_document_indexing() {
    // Skip if OpenSearch is not available
    if !is_opensearch_available().await {
        println!("OpenSearch not available - skipping document indexing test");
        return;
    }

    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);
    let manager = IndexManager::new(client.clone());
    let indexer = SearchIndexer::new(client);

    // Create index
    let index_name = manager
        .create_posts_index()
        .await
        .expect("Failed to create index");

    // Create a test document
    let doc = PostDocument::from_post(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        "Hello, OpenSearch! This is a #test message.".to_string(),
        false,
        Utc::now(),
        None,
        0,
        None,
        &[],
    );

    // Index the document
    let result = indexer
        .index_post(&doc)
        .await
        .expect("Failed to index document");

    println!(
        "Indexed document: index={}, id={}, version={}, result={}",
        result.index, result.id, result.version, result.result
    );

    assert_eq!(result.result, "created", "Document should be created");

    // Clean up
    manager.delete_index(&index_name).await.ok();
}

#[tokio::test]
async fn test_bulk_indexing() {
    // Skip if OpenSearch is not available
    if !is_opensearch_available().await {
        println!("OpenSearch not available - skipping bulk indexing test");
        return;
    }

    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);
    let manager = IndexManager::new(client.clone());
    let indexer = SearchIndexer::new(client);

    // Create index
    let index_name = manager
        .create_posts_index()
        .await
        .expect("Failed to create index");

    // Create multiple test documents
    let mut docs = Vec::new();
    for i in 0..10 {
        let doc = PostDocument::from_post(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            format!("Bulk test message #{}", i),
            false,
            Utc::now(),
            None,
            0,
            None,
            &[],
        );
        docs.push(doc);
    }

    // Bulk index
    let result = indexer
        .bulk_index(&docs)
        .await
        .expect("Failed to bulk index");

    println!(
        "Bulk indexed: took={}ms, items={}, errors={}",
        result.took, result.items_processed, result.errors
    );

    assert_eq!(result.items_processed, 10, "Should process 10 documents");

    // Clean up
    manager.delete_index(&index_name).await.ok();
}

#[tokio::test]
async fn test_document_update() {
    // Skip if OpenSearch is not available
    if !is_opensearch_available().await {
        println!("OpenSearch not available - skipping document update test");
        return;
    }

    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);
    let manager = IndexManager::new(client.clone());
    let indexer = SearchIndexer::new(client);

    // Create index
    let index_name = manager
        .create_posts_index()
        .await
        .expect("Failed to create index");

    // Create and index a document
    let post_id = Uuid::new_v4();
    let doc = PostDocument::from_post(
        post_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        "Original message".to_string(),
        false,
        Utc::now(),
        None,
        0,
        None,
        &[],
    );

    indexer
        .index_post(&doc)
        .await
        .expect("Failed to index document");

    // Update the document
    let result = indexer
        .update_post(
            &post_id.to_string(),
            Some("Updated message".to_string()),
            Some(true),
        )
        .await
        .expect("Failed to update document");

    println!("Updated document: result={}", result.result);

    // Clean up
    manager.delete_index(&index_name).await.ok();
}

#[tokio::test]
async fn test_document_deletion() {
    // Skip if OpenSearch is not available
    if !is_opensearch_available().await {
        println!("OpenSearch not available - skipping document deletion test");
        return;
    }

    let config = test_opensearch_config();
    let client = OpenSearchClient::new(config);
    let manager = IndexManager::new(client.clone());
    let indexer = SearchIndexer::new(client);

    // Create index
    let index_name = manager
        .create_posts_index()
        .await
        .expect("Failed to create index");

    // Create and index a document
    let post_id = Uuid::new_v4();
    let doc = PostDocument::from_post(
        post_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        "Message to delete".to_string(),
        false,
        Utc::now(),
        None,
        0,
        None,
        &[],
    );

    indexer
        .index_post(&doc)
        .await
        .expect("Failed to index document");

    // Delete the document
    indexer
        .delete_post(&post_id.to_string())
        .await
        .expect("Failed to delete document");

    println!("Deleted document: {}", post_id);

    // Clean up
    manager.delete_index(&index_name).await.ok();
}

#[tokio::test]
async fn test_disabled_opensearch() {
    let config = OpenSearchConfig::default(); // disabled by default
    let client = OpenSearchClient::new(config);

    assert!(!client.is_available());

    // Health check should fail
    let result = client.health_check().await;
    assert!(result.is_err());
}

#[test]
fn test_post_document_creation() {
    use rustchat::search::indexer::PostDocument;

    let doc = PostDocument::from_post(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        "Hello #world! Check out @rustchat".to_string(),
        false,
        Utc::now(),
        None,
        0,
        None,
        &[],
    );

    assert_eq!(doc.hashtags, Some(vec!["world".to_string()]));
    assert_eq!(doc.mentions, Some(vec!["rustchat".to_string()]));
    assert!(!doc.has_attachments);
    assert!(!doc.is_reply);
}

#[test]
fn test_index_name_generation() {
    use rustchat::config::OpenSearchConfig;

    let config = OpenSearchConfig {
        index_prefix: "test".to_string(),
        ..OpenSearchConfig::default()
    };

    let client = OpenSearchClient::new(config);
    assert_eq!(client.posts_index(), "test-posts");
    assert_eq!(client.index_name("users"), "test-users");
}

#[test]
fn test_time_based_index_name() {
    use chrono::TimeZone;
    use rustchat::config::OpenSearchConfig;

    let config = OpenSearchConfig::default();
    let client = OpenSearchClient::new(config);
    let manager = IndexManager::new(client);

    let date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    let name = manager.time_based_index_name("rustchat-posts", date);

    assert_eq!(name, "rustchat-posts-2024.01.15");
}
