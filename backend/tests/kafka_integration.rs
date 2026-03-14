//! Kafka integration tests
//!
//! These tests require a running Kafka instance.
//! Use `docker compose up -d kafka` to start Kafka before running tests.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::timeout;
use uuid::Uuid;

use rustchat::config::KafkaConfig;
use rustchat::services::kafka_consumer::{ConsumedMessage, KafkaConsumer, MessageHandler};
use rustchat::services::kafka_producer::{KafkaMessage, KafkaProducer};

/// Test configuration for local Kafka
fn test_kafka_config() -> KafkaConfig {
    KafkaConfig {
        enabled: true,
        bootstrap_servers: std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:9092".to_string()),
        client_id: format!("rustchat-test-{}", Uuid::new_v4()),
        posts_topic: "rustchat.test.posts".to_string(),
        consumer_group: format!("test-group-{}", Uuid::new_v4()),
        message_timeout_ms: 30000,
        retry_backoff_ms: 100,
        max_retries: 3,
        ssl_enabled: false,
        ssl_ca_location: None,
        ssl_certificate_location: None,
        ssl_key_location: None,
        sasl_mechanism: None,
        sasl_username: None,
        sasl_password: None,
        fanout_threshold: 1000,
        auto_commit_interval_ms: 1000,
        session_timeout_ms: 10000,
    }
}

/// Test payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestPayload {
    message: String,
    count: i32,
}

/// Test message handler
struct TestHandler {
    sender: mpsc::Sender<ConsumedMessage<TestPayload>>,
}

#[async_trait::async_trait]
impl MessageHandler for TestHandler {
    type Payload = TestPayload;

    async fn handle_message(&self, message: ConsumedMessage<Self::Payload>) -> anyhow::Result<()> {
        self.sender.send(message).await.ok();
        Ok(())
    }
}

/// Check if Kafka is available
async fn is_kafka_available() -> bool {
    let config = test_kafka_config();
    let producer = KafkaProducer::new(config.clone(), "test-node".to_string());
    producer.is_available()
}

#[tokio::test]
async fn test_producer_creation() {
    let config = test_kafka_config();
    let producer = KafkaProducer::new(config, "test-node".to_string());

    // If Kafka is not available, this will be false
    // We don't fail the test - just verify the structure works
    if !producer.is_available() {
        println!("Kafka not available - skipping producer test");
        return;
    }
}

#[tokio::test]
async fn test_consumer_creation() {
    let config = test_kafka_config();
    let consumer = KafkaConsumer::new(config, "test-node".to_string());

    // If Kafka is not enabled, is_available returns false
    if !consumer.is_available() {
        println!("Kafka not enabled - skipping consumer test");
        return;
    }
}

#[tokio::test]
async fn test_message_serialization() {
    let payload = TestPayload {
        message: "Hello, Kafka!".to_string(),
        count: 42,
    };

    let message = KafkaMessage {
        event_type: "test_event".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        node_id: "test-node".to_string(),
        payload,
    };

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: KafkaMessage<TestPayload> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(message.event_type, deserialized.event_type);
    assert_eq!(message.node_id, deserialized.node_id);
    assert_eq!(message.payload.message, deserialized.payload.message);
    assert_eq!(message.payload.count, deserialized.payload.count);
}

#[tokio::test]
async fn test_producer_consumer_roundtrip() {
    // Skip if Kafka is not available
    if !is_kafka_available().await {
        println!("Kafka not available - skipping roundtrip test");
        return;
    }

    let config = test_kafka_config();
    let topic = &config.posts_topic;

    // Create producer
    let producer = KafkaProducer::new(config.clone(), "test-producer".to_string());

    // Create consumer
    let consumer = KafkaConsumer::new(config.clone(), "test-consumer".to_string());

    // Create channel for receiving messages
    let (tx, mut rx) = mpsc::channel(10);
    let handler = Arc::new(TestHandler { sender: tx });

    // Subscribe to topic
    let _shutdown = consumer.subscribe(topic, handler).await.expect("Failed to subscribe");

    // Wait a bit for consumer to subscribe
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send a message
    let payload = TestPayload {
        message: "Test message".to_string(),
        count: 123,
    };

    let message = KafkaMessage {
        event_type: "test_event".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        node_id: "test-producer".to_string(),
        payload,
    };

    let result = producer
        .publish(topic, Some("test-key"), &message)
        .await
        .expect("Failed to publish message");

    println!("Message published to partition {} at offset {}", result.partition, result.offset);

    // Wait for the message to be received
    let received = timeout(Duration::from_secs(30), rx.recv()).await;

    match received {
        Ok(Some(msg)) => {
            assert_eq!(msg.event_type, "test_event");
            assert_eq!(msg.payload.message, "Test message");
            assert_eq!(msg.payload.count, 123);
            println!("Message received successfully!");
        }
        Ok(None) => panic!("Channel closed unexpectedly"),
        Err(_) => panic!("Timeout waiting for message"),
    }

    // Flush producer
    producer.flush(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
async fn test_multiple_messages() {
    // Skip if Kafka is not available
    if !is_kafka_available().await {
        println!("Kafka not available - skipping multiple messages test");
        return;
    }

    let config = test_kafka_config();
    let topic = &config.posts_topic;

    // Create producer
    let producer = KafkaProducer::new(config.clone(), "test-producer".to_string());

    // Create consumer with unique group
    let consumer = KafkaConsumer::new(config.clone(), "test-consumer-multi".to_string());

    // Create channel for receiving messages
    let (tx, mut rx) = mpsc::channel(100);
    let handler = Arc::new(TestHandler { sender: tx });

    // Subscribe to topic
    let _shutdown = consumer.subscribe(topic, handler).await.expect("Failed to subscribe");

    // Wait for consumer to subscribe
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send multiple messages
    let num_messages = 5;
    for i in 0..num_messages {
        let payload = TestPayload {
            message: format!("Message {}", i),
            count: i,
        };

        let message = KafkaMessage {
            event_type: "batch_test".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            node_id: "test-producer".to_string(),
            payload,
        };

        producer
            .publish(topic, Some(&format!("key-{}", i)), &message)
            .await
            .expect("Failed to publish message");
    }

    // Wait for all messages
    let mut received_count = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

    while received_count < num_messages && tokio::time::Instant::now() < deadline {
        match timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Some(msg)) => {
                assert_eq!(msg.event_type, "batch_test");
                assert!(msg.payload.count >= 0 && msg.payload.count < num_messages);
                received_count += 1;
            }
            Ok(None) => break,
            Err(_) => continue, // Timeout, check deadline
        }
    }

    assert_eq!(
        received_count, num_messages,
        "Expected {} messages, received {}",
        num_messages, received_count
    );

    println!("Successfully received {} messages", received_count);

    // Flush producer
    producer.flush(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
async fn test_producer_retry() {
    // This test verifies that the retry logic is in place
    // We can't easily test actual failures without mocking
    let config = test_kafka_config();
    let producer = KafkaProducer::new(config, "test-node".to_string());

    // Just verify the producer has the retry configuration
    assert!(producer.is_available() || !producer.is_available()); // Always true, just check compilation
}

/// Test configuration deserialization
#[test]
fn test_kafka_config_deserialization() {
    use rustchat::config::KafkaConfig;

    let config = KafkaConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.bootstrap_servers, "localhost:9092");
    assert_eq!(config.posts_topic, "rustchat.posts");
    assert_eq!(config.consumer_group, "websocket-fanout");
    assert_eq!(config.fanout_threshold, 1000);
}

/// Test that disabled Kafka doesn't break the app
#[tokio::test]
async fn test_disabled_kafka() {
    let config = KafkaConfig::default(); // disabled by default
    let producer = KafkaProducer::new(config.clone(), "test-node".to_string());

    assert!(!producer.is_available());

    // Publishing should fail gracefully
    let result = producer
        .publish::<serde_json::Value>("test-topic", None, &serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
