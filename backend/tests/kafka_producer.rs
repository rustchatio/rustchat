//! Kafka Producer Integration Tests
//!
//! These tests require a running Kafka instance on localhost:9092.
//! Run with: `cargo test kafka_producer -- --nocapture`

use std::time::Duration;
use uuid::Uuid;

/// Test Kafka producer initialization
#[test]
fn test_kafka_producer_initialization() {
    use rustchat::config::KafkaConfig;
    use rustchat::services::kafka_producer::KafkaProducer;

    // Test with disabled config
    let config = KafkaConfig::default();
    let producer = KafkaProducer::new(config, "test-node".to_string());
    
    assert!(!producer.is_available(), "Producer should not be available when disabled");
}

/// Test Kafka producer with actual broker
/// This test requires a running Kafka on localhost:9092
#[tokio::test]
async fn test_kafka_producer_send_event() {
    use rustchat::config::KafkaConfig;
    use rustchat::services::kafka_producer::KafkaProducer;

    let config = KafkaConfig {
        enabled: true,
        bootstrap_servers: "localhost:9092".to_string(),
        posts_topic: "rustchat.posts.test".to_string(),
        ..Default::default()
    };

    let producer = KafkaProducer::new(config, "test-node".to_string());
    
    // Skip test if Kafka is not available
    if !producer.is_available() {
        println!("Skipping test: Kafka not available at localhost:9092");
        return;
    }

    // Test sending a post event
    let post_id = Uuid::new_v4();
    let channel_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = producer.send_post_event(post_id, channel_id, user_id).await;
    
    assert!(
        result.is_ok(),
        "Failed to send post event: {:?}",
        result.err()
    );

    let delivery = result.unwrap();
    println!(
        "Message delivered to partition {} at offset {}",
        delivery.partition, delivery.offset
    );

    // Flush to ensure message is sent
    let _ = producer.flush(Duration::from_secs(5)).await;
}

/// Test publish with custom payload
#[tokio::test]
async fn test_kafka_producer_publish_custom() {
    use rustchat::config::KafkaConfig;
    use rustchat::services::kafka_producer::KafkaProducer;
    use serde_json::json;

    let config = KafkaConfig {
        enabled: true,
        bootstrap_servers: "localhost:9092".to_string(),
        posts_topic: "rustchat.posts.test".to_string(),
        ..Default::default()
    };

    let producer = KafkaProducer::new(config, "test-node".to_string());
    
    // Skip test if Kafka is not available
    if !producer.is_available() {
        println!("Skipping test: Kafka not available at localhost:9092");
        return;
    }

    let payload = json!({
        "event": "test_event",
        "data": {
            "key": "value",
            "number": 42
        }
    });

    let result = producer
        .publish("rustchat.posts.test", Some("test-key"), &payload)
        .await;

    assert!(
        result.is_ok(),
        "Failed to publish custom message: {:?}",
        result.err()
    );

    let _ = producer.flush(Duration::from_secs(5)).await;
}
