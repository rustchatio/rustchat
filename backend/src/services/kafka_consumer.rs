//! Kafka consumer service for rustchat
//!
//! Provides async message consumption with auto-commit and reconnection logic.

use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::{BorrowedMessage, Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::KafkaConfig;

/// Kafka consumer wrapper with automatic reconnection
pub struct KafkaConsumer {
    config: KafkaConfig,
    #[allow(dead_code)]
    consumer: Option<Arc<StreamConsumer>>,
    node_id: String,
}

/// Parsed Kafka message
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumedMessage<T> {
    /// Event type
    pub event_type: String,
    /// Timestamp in milliseconds
    pub timestamp: i64,
    /// Node that produced the message
    pub node_id: String,
    /// Message payload
    pub payload: T,
    /// Kafka partition
    pub partition: i32,
    /// Kafka offset
    pub offset: i64,
}

/// Message handler trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    type Payload: for<'de> Deserialize<'de> + Send;

    /// Handle a consumed message
    async fn handle_message(&self, message: ConsumedMessage<Self::Payload>) -> anyhow::Result<()>;

    /// Handle deserialization errors
    async fn handle_deserialization_error(&self, error: serde_json::Error, raw_payload: &[u8]) {
        error!(
            error = %error,
            payload = %String::from_utf8_lossy(raw_payload),
            "Failed to deserialize Kafka message"
        );
    }
}

/// Shutdown signal for the consumer
#[derive(Debug, Clone)]
pub struct ShutdownSignal {
    sender: mpsc::Sender<()>,
}

impl ShutdownSignal {
    /// Send shutdown signal
    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.sender.send(()).await.ok();
        Ok(())
    }
}

impl KafkaConsumer {
    /// Create a new Kafka consumer
    pub fn new(config: KafkaConfig, node_id: String) -> Arc<Self> {
        info!(
            version = %format!("{}.{}", get_rdkafka_version().0, get_rdkafka_version().1),
            "Initializing Kafka consumer with librdkafka"
        );

        Arc::new(Self {
            config,
            consumer: None,
            node_id,
        })
    }

    /// Check if the consumer is available
    pub fn is_available(&self) -> bool {
        self.config.enabled
    }

    /// Create the underlying Kafka consumer
    fn create_consumer(&self, group_id_suffix: Option<&str>) -> anyhow::Result<StreamConsumer> {
        let group_id = if let Some(suffix) = group_id_suffix {
            format!("{}-{}", self.config.consumer_group, suffix)
        } else {
            self.config.consumer_group.clone()
        };

        let mut client_config = ClientConfig::new();

        client_config
            .set("group.id", &group_id)
            .set("bootstrap.servers", &self.config.bootstrap_servers)
            .set(
                "client.id",
                format!("{}-consumer-{}", self.config.client_id, self.node_id),
            )
            .set("enable.partition.eof", "false")
            .set(
                "session.timeout.ms",
                self.config.session_timeout_ms.to_string(),
            )
            .set("enable.auto.commit", "true")
            .set(
                "auto.commit.interval.ms",
                self.config.auto_commit_interval_ms.to_string(),
            )
            .set("auto.offset.reset", "latest")
            .set("max.poll.interval.ms", "300000") // 5 minutes
            .set("heartbeat.interval.ms", "10000"); // 10 seconds

        // SSL configuration
        if self.config.ssl_enabled {
            client_config.set("security.protocol", "ssl");

            if let Some(ref ca_location) = self.config.ssl_ca_location {
                client_config.set("ssl.ca.location", ca_location);
            }
            if let Some(ref cert_location) = self.config.ssl_certificate_location {
                client_config.set("ssl.certificate.location", cert_location);
            }
            if let Some(ref key_location) = self.config.ssl_key_location {
                client_config.set("ssl.key.location", key_location);
            }
        }

        // SASL configuration
        if let Some(ref mechanism) = self.config.sasl_mechanism {
            let protocol = if self.config.ssl_enabled {
                "SASL_SSL"
            } else {
                "SASL_PLAINTEXT"
            };
            client_config.set("security.protocol", protocol);
            client_config.set("sasl.mechanism", mechanism);

            if let Some(ref username) = self.config.sasl_username {
                client_config.set("sasl.username", username);
            }
            if let Some(ref password) = self.config.sasl_password {
                client_config.set("sasl.password", password);
            }
        }

        let consumer: StreamConsumer = client_config.create()?;
        Ok(consumer)
    }

    /// Subscribe to a topic and start consuming messages
    pub async fn subscribe<H: MessageHandler + 'static>(
        self: &Arc<Self>,
        topic: &str,
        handler: Arc<H>,
    ) -> anyhow::Result<ShutdownSignal> {
        if !self.config.enabled {
            anyhow::bail!("Kafka consumer is disabled");
        }

        let consumer = Arc::new(self.create_consumer(None)?);

        // Subscribe to the topic
        let mut topics = TopicPartitionList::new();
        topics.add_partition(topic, 0);
        consumer.subscribe(&[topic])?;

        info!(topic = %topic, group = %self.config.consumer_group, "Subscribed to Kafka topic");

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        let consumer_clone = consumer.clone();
        let _node_id = self.node_id.clone();

        // Spawn consumer task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Kafka consumer shutting down");
                        break;
                    }
                    result = consumer_clone.recv() => {
                        match result {
                            Ok(msg) => {
                                if let Err(e) = Self::process_message(&msg, handler.as_ref()).await {
                                    error!(error = %e, "Error processing Kafka message");
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Kafka consumer error");
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }

            // Unsubscribe on shutdown
            consumer_clone.unsubscribe();
        });

        Ok(ShutdownSignal {
            sender: shutdown_tx,
        })
    }

    /// Process a single message
    async fn process_message<H: MessageHandler>(
        msg: &BorrowedMessage<'_>,
        handler: &H,
    ) -> anyhow::Result<()> {
        let payload = match msg.payload() {
            Some(p) => p,
            None => {
                warn!("Empty message payload received");
                return Ok(());
            }
        };

        // Extract headers
        let _node_id = String::from("unknown");
        if let Some(headers) = msg.headers() {
            for i in 0..headers.count() {
                match headers.get_as::<str>(i) {
                    Ok(header) if header.key == "node_id" => {
                        let _ = header.value.unwrap_or("unknown");
                    }
                    _ => {}
                }
            }
        }

        // Try to deserialize as generic JSON first to get metadata
        #[derive(Deserialize)]
        struct MessageEnvelope {
            event_type: String,
            #[allow(dead_code)]
            timestamp: i64,
            #[serde(flatten)]
            _rest: serde_json::Value,
        }

        let envelope: MessageEnvelope = match serde_json::from_slice(payload) {
            Ok(e) => e,
            Err(e) => {
                handler.handle_deserialization_error(e, payload).await;
                return Ok(());
            }
        };

        // Deserialize full payload
        let full_message: ConsumedMessage<H::Payload> = match serde_json::from_slice(payload) {
            Ok(m) => m,
            Err(e) => {
                handler.handle_deserialization_error(e, payload).await;
                return Ok(());
            }
        };

        debug!(
            event_type = %envelope.event_type,
            partition = msg.partition(),
            offset = msg.offset(),
            "Processing Kafka message"
        );

        // Handle the message
        if let Err(e) = handler.handle_message(full_message).await {
            error!(error = %e, "Message handler failed");
        }

        Ok(())
    }

    /// Get consumer lag information (for monitoring)
    pub async fn get_lag(&self, topic: &str) -> anyhow::Result<Vec<(i32, i64, i64)>> {
        if !self.is_available() {
            anyhow::bail!("Kafka consumer not available");
        }

        let consumer = self.create_consumer(Some("lag-check"))?;

        // Get current assignment
        let metadata = consumer.fetch_metadata(Some(topic), Duration::from_secs(5))?;
        let topic_metadata = metadata
            .topics()
            .iter()
            .find(|t| t.name() == topic)
            .ok_or_else(|| anyhow::anyhow!("Topic not found"))?;

        let mut result = Vec::new();

        for partition in topic_metadata.partitions() {
            let partition_id = partition.id();

            // Get committed offset
            let mut tpl = TopicPartitionList::new();
            tpl.add_partition_offset(topic, partition_id, rdkafka::Offset::Offset(-1))?;

            let committed = consumer.committed(Duration::from_secs(5))?;
            let committed_offset = committed
                .find_partition(topic, partition_id)
                .and_then(|p| p.offset().to_raw())
                .unwrap_or(0);

            // Get high watermark (end offset)
            let (_, high_watermark) =
                consumer.fetch_watermarks(topic, partition_id, Duration::from_secs(5))?;

            let lag = high_watermark - committed_offset;
            result.push((partition_id, committed_offset, lag));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumer_disabled() {
        let config = KafkaConfig::default();
        let consumer = KafkaConsumer::new(config, "test-node".to_string());
        assert!(!consumer.is_available());
    }
}
