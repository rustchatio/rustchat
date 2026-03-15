//! Kafka producer service for rustchat
//!
//! Provides async message publishing with retry logic and delivery confirmation.

use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::config::KafkaConfig;

/// Kafka message key type
pub type MessageKey = String;

/// Kafka producer wrapper with retry logic
pub struct KafkaProducer {
    config: KafkaConfig,
    producer: Option<FutureProducer>,
    node_id: String,
}

/// Message delivery result
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub partition: i32,
    pub offset: i64,
}

/// Message to be published to Kafka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaMessage<T> {
    /// Event type
    pub event_type: String,
    /// Timestamp in milliseconds
    pub timestamp: i64,
    /// Node that produced the message
    pub node_id: String,
    /// Message payload
    pub payload: T,
}

impl KafkaProducer {
    /// Create a new Kafka producer
    pub fn new(config: KafkaConfig, node_id: String) -> Arc<Self> {
        let producer = if config.enabled {
            match Self::create_producer(&config, &node_id) {
                Ok(producer) => {
                    info!(
                        bootstrap_servers = %config.bootstrap_servers,
                        client_id = %config.client_id,
                        "Kafka producer initialized"
                    );
                    Some(producer)
                }
                Err(e) => {
                    error!(error = %e, "Failed to create Kafka producer");
                    None
                }
            }
        } else {
            info!("Kafka producer disabled");
            None
        };

        Arc::new(Self {
            config,
            producer,
            node_id,
        })
    }

    /// Check if the producer is available
    pub fn is_available(&self) -> bool {
        self.producer.is_some()
    }

    /// Create the underlying Kafka producer
    fn create_producer(config: &KafkaConfig, node_id: &str) -> anyhow::Result<FutureProducer> {
        let mut client_config = ClientConfig::new();

        client_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("client.id", format!("{}-{}", config.client_id, node_id))
            .set("message.timeout.ms", config.message_timeout_ms.to_string())
            .set("retry.backoff.ms", config.retry_backoff_ms.to_string())
            .set("message.send.max.retries", config.max_retries.to_string())
            .set("queue.buffering.max.messages", "100000")
            .set("queue.buffering.max.ms", "100")
            .set("batch.num.messages", "1000")
            .set("compression.type", "lz4")
            .set("acks", "all");

        // SSL configuration
        if config.ssl_enabled {
            client_config.set("security.protocol", "ssl");

            if let Some(ref ca_location) = config.ssl_ca_location {
                client_config.set("ssl.ca.location", ca_location);
            }
            if let Some(ref cert_location) = config.ssl_certificate_location {
                client_config.set("ssl.certificate.location", cert_location);
            }
            if let Some(ref key_location) = config.ssl_key_location {
                client_config.set("ssl.key.location", key_location);
            }
        }

        // SASL configuration
        if let Some(ref mechanism) = config.sasl_mechanism {
            let protocol = if config.ssl_enabled {
                "SASL_SSL"
            } else {
                "SASL_PLAINTEXT"
            };
            client_config.set("security.protocol", protocol);
            client_config.set("sasl.mechanism", mechanism);

            if let Some(ref username) = config.sasl_username {
                client_config.set("sasl.username", username);
            }
            if let Some(ref password) = config.sasl_password {
                client_config.set("sasl.password", password);
            }
        }

        let producer: FutureProducer = client_config.create()?;
        Ok(producer)
    }

    /// Publish a message to Kafka with retry logic
    pub async fn publish<T: Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &T,
    ) -> anyhow::Result<DeliveryResult> {
        if !self.is_available() {
            anyhow::bail!("Kafka producer not available");
        }

        let producer = self.producer.as_ref().unwrap();

        // Serialize payload
        let payload_bytes = serde_json::to_vec(payload)?;
        let key_bytes = key.map(|k| k.as_bytes().to_vec());

        // Publish with retry logic
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(self.config.retry_backoff_ms * attempt as u64);
                warn!(attempt, ?backoff, "Retrying Kafka publish after backoff");
                tokio::time::sleep(backoff).await;
            }

            // Build a fresh record for each attempt (FutureRecord doesn't implement Clone)
            let headers = OwnedHeaders::new().insert(Header {
                key: "node_id",
                value: Some(&self.node_id),
            });

            let mut record = FutureRecord::to(topic)
                .payload(&payload_bytes)
                .headers(headers);

            if let Some(ref k) = key_bytes {
                record = record.key(k.as_slice());
            }

            match producer.send(record, Duration::from_secs(30)).await {
                Ok((partition, offset)) => {
                    debug!(
                        topic = %topic,
                        partition = partition,
                        offset = offset,
                        "Message published to Kafka"
                    );
                    return Ok(DeliveryResult { partition, offset });
                }
                Err((e, _)) => {
                    error!(attempt, error = %e, "Failed to publish message to Kafka");
                    last_error = Some(e);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to publish message after {} retries: {:?}",
            self.config.max_retries,
            last_error
        ))
    }

    /// Publish a post event to Kafka
    pub async fn publish_post_event<T: Serialize>(
        &self,
        event_type: &str,
        channel_id: &str,
        payload: &T,
    ) -> anyhow::Result<DeliveryResult> {
        let message = KafkaMessage {
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            node_id: self.node_id.clone(),
            payload,
        };

        self.publish(&self.config.posts_topic, Some(channel_id), &message)
            .await
    }

    /// Simplified method to send a post created event
    /// This is a convenience wrapper for the common use case
    pub async fn send_post_event(
        &self,
        post_id: uuid::Uuid,
        channel_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> anyhow::Result<DeliveryResult> {
        use serde_json::json;

        let payload = json!({
            "post_id": post_id,
            "user_id": user_id,
            "channel_id": channel_id,
        });

        self.publish_post_event("post_created", &channel_id.to_string(), &payload)
            .await
    }

    /// Flush pending messages
    pub async fn flush(&self, timeout: Duration) -> anyhow::Result<()> {
        if let Some(ref producer) = self.producer {
            producer.flush(timeout)?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_message_serialization() {
        let message = KafkaMessage {
            event_type: "post_created".to_string(),
            timestamp: 1234567890,
            node_id: "test-node".to_string(),
            payload: serde_json::json!({"id": "test-id"}),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("post_created"));
        assert!(serialized.contains("test-node"));
    }

    #[test]
    fn test_producer_disabled() {
        let config = KafkaConfig::default();
        let producer = KafkaProducer::new(config, "test-node".to_string());
        assert!(!producer.is_available());
    }
}
