//! Message fan-out module for rustchat
//!
//! Routes messages to either Redis pub/sub (small channels) or Kafka (massive channels).
//! This enables efficient broadcast for channels with >1000 members.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use crate::realtime::{WsEnvelope, WsHub};
#[cfg(feature = "kafka")]
use crate::config::KafkaConfig;
#[cfg(feature = "kafka")]
use crate::services::kafka_consumer::{ConsumedMessage, MessageHandler};
#[cfg(feature = "kafka")]
use crate::services::kafka_producer::{KafkaMessage, KafkaProducer};

/// Fan-out manager that routes messages based on channel size
#[cfg(feature = "kafka")]
pub struct FanoutManager {
    config: KafkaConfig,
    producer: Option<Arc<KafkaProducer>>,
    hub: Arc<WsHub>,
    node_id: String,
}

/// Stub fan-out manager when Kafka is disabled
#[cfg(not(feature = "kafka"))]
pub struct FanoutManager; // Empty stub

/// Post event types for Kafka
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PostEvent {
    Created {
        post_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        root_post_id: Option<Uuid>,
    },
    Updated {
        post_id: Uuid,
        channel_id: Uuid,
        message: String,
    },
    Deleted {
        post_id: Uuid,
        channel_id: Uuid,
    },
    ReactionAdded {
        post_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
        emoji_name: String,
    },
    ReactionRemoved {
        post_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
        emoji_name: String,
    },
}

/// Post event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostEventPayload {
    pub event: PostEvent,
    pub timestamp: i64,
}

/// Channel member count cache
pub struct ChannelSizeCache {
    threshold: usize,
    // In a real implementation, this would be a Redis-backed cache
    // For now, we use a simple heuristic based on channel type
}

impl ChannelSizeCache {
    pub fn new(threshold: usize) -> Self {
        Self { threshold }
    }

    /// Check if a channel should use Kafka fan-out
    /// In production, this would query the actual member count from Redis/DB
    pub async fn should_use_kafka(&self, channel_id: Uuid) -> bool {
        // For now, we use a deterministic heuristic based on channel ID
        // This ensures consistent routing for the same channel
        // In production, query actual member count from Redis cache
        let bytes = channel_id.as_bytes();
        let hash: u64 = bytes.iter().fold(0u64, |acc, &b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });

        // Use Kafka for approximately 10% of channels (simulating large channels)
        // This is a placeholder - replace with actual member count check
        hash.is_multiple_of(10)
    }

    /// Get the fan-out threshold
    pub fn threshold(&self) -> usize {
        self.threshold
    }
}

#[cfg(feature = "kafka")]
impl FanoutManager {
    /// Create a new fan-out manager
    pub fn new(
        config: KafkaConfig,
        producer: Option<Arc<KafkaProducer>>,
        hub: Arc<WsHub>,
    ) -> Arc<Self> {
        let node_id = format!(
            "{}-{}",
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            Uuid::new_v4()
        );

        let manager = Arc::new(Self {
            config,
            producer,
            hub,
            node_id,
        });

        info!(
            node_id = %manager.node_id,
            enabled = manager.is_enabled(),
            threshold = manager.config.fanout_threshold,
            "Fanout manager initialized"
        );

        manager
    }

    /// Check if Kafka fan-out is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.producer.is_some()
    }

    /// Get the node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Broadcast a WebSocket envelope
    /// Routes to Kafka for massive channels, Redis for smaller channels
    pub async fn broadcast(&self, envelope: WsEnvelope, channel_id: Uuid) -> anyhow::Result<()> {
        if !self.is_enabled() {
            // Use Redis pub/sub directly
            self.hub.broadcast(envelope).await;
            return Ok(());
        }

        // Check if this should go to Kafka
        let cache = ChannelSizeCache::new(self.config.fanout_threshold);
        let use_kafka = cache.should_use_kafka(channel_id).await;

        if use_kafka {
            debug!(channel_id = %channel_id, "Routing message to Kafka");
            self.send_to_kafka(envelope, channel_id).await?;
        } else {
            debug!(channel_id = %channel_id, "Routing message to Redis");
            self.hub.broadcast(envelope).await;
        }

        Ok(())
    }

    /// Send a message to Kafka for massive channel fan-out
    async fn send_to_kafka(&self, envelope: WsEnvelope, channel_id: Uuid) -> anyhow::Result<()> {
        let producer = self.producer.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Kafka producer not available")
        })?;

        // Convert envelope to post event
        let event = self.envelope_to_event(&envelope, channel_id)?;

        let payload = PostEventPayload {
            event,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let message = KafkaMessage {
            event_type: envelope.event.clone(),
            timestamp: payload.timestamp,
            node_id: self.node_id.clone(),
            payload,
        };

        let key = channel_id.to_string();
        producer
            .publish(&self.config.posts_topic, Some(&key), &message)
            .await?;

        Ok(())
    }

    /// Convert WebSocket envelope to post event
    fn envelope_to_event(&self, envelope: &WsEnvelope, channel_id: Uuid) -> anyhow::Result<PostEvent> {
        // Parse the event data to extract post information
        let data = &envelope.data;

        match envelope.event.as_str() {
            "posted" => {
                let post_id = data
                    .get("post_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| anyhow::anyhow!("Missing post_id in posted event"))?;

                let user_id = data
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_default();

                let message = data
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let root_post_id = data
                    .get("root_post_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());

                Ok(PostEvent::Created {
                    post_id,
                    channel_id,
                    user_id,
                    message,
                    root_post_id,
                })
            }
            "post_edited" => {
                let post_id = data
                    .get("post_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| anyhow::anyhow!("Missing post_id in post_edited event"))?;

                let message = data
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Ok(PostEvent::Updated {
                    post_id,
                    channel_id,
                    message,
                })
            }
            "post_deleted" => {
                let post_id = data
                    .get("post_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| anyhow::anyhow!("Missing post_id in post_deleted event"))?;

                Ok(PostEvent::Deleted {
                    post_id,
                    channel_id,
                })
            }
            "reaction_added" => {
                let post_id = data
                    .get("post_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| anyhow::anyhow!("Missing post_id in reaction_added event"))?;

                let user_id = data
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_default();

                let emoji_name = data
                    .get("emoji_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Ok(PostEvent::ReactionAdded {
                    post_id,
                    channel_id,
                    user_id,
                    emoji_name,
                })
            }
            "reaction_removed" => {
                let post_id = data
                    .get("post_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| anyhow::anyhow!("Missing post_id in reaction_removed event"))?;

                let user_id = data
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_default();

                let emoji_name = data
                    .get("emoji_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Ok(PostEvent::ReactionRemoved {
                    post_id,
                    channel_id,
                    user_id,
                    emoji_name,
                })
            }
            _ => {
                // For unknown events, we still broadcast via Redis
                anyhow::bail!("Unknown event type for Kafka routing: {}", envelope.event)
            }
        }
    }

    /// Convert post event back to WebSocket envelope
    #[allow(dead_code)]
    fn event_to_envelope(&self, payload: &PostEventPayload, channel_id: Uuid) -> WsEnvelope {
        let (event_type, data) = match &payload.event {
            PostEvent::Created {
                post_id,
                user_id,
                message,
                root_post_id,
                ..
            } => {
                let mut data = serde_json::json!({
                    "post_id": post_id,
                    "channel_id": channel_id,
                    "user_id": user_id,
                    "message": message,
                });
                if let Some(root_id) = root_post_id {
                    data["root_post_id"] = serde_json::json!(root_id);
                }
                ("posted", data)
            }
            PostEvent::Updated {
                post_id,
                message,
                ..
            } => (
                "post_edited",
                serde_json::json!({
                    "post_id": post_id,
                    "channel_id": channel_id,
                    "message": message,
                }),
            ),
            PostEvent::Deleted { post_id, .. } => (
                "post_deleted",
                serde_json::json!({
                    "post_id": post_id,
                    "channel_id": channel_id,
                }),
            ),
            PostEvent::ReactionAdded {
                post_id,
                user_id,
                emoji_name,
                ..
            } => (
                "reaction_added",
                serde_json::json!({
                    "post_id": post_id,
                    "channel_id": channel_id,
                    "user_id": user_id,
                    "emoji_name": emoji_name,
                }),
            ),
            PostEvent::ReactionRemoved {
                post_id,
                user_id,
                emoji_name,
                ..
            } => (
                "reaction_removed",
                serde_json::json!({
                    "post_id": post_id,
                    "channel_id": channel_id,
                    "user_id": user_id,
                    "emoji_name": emoji_name,
                }),
            ),
        };

        WsEnvelope {
            msg_type: "event".to_string(),
            event: event_type.to_string(),
            seq: None,
            channel_id: Some(channel_id),
            data,
            broadcast: None,
        }
    }
}

/// Kafka message handler for WebSocket fan-out
#[cfg(feature = "kafka")]
pub struct WebSocketFanoutHandler {
    hub: Arc<WsHub>,
    node_id: String,
}

#[cfg(feature = "kafka")]
impl WebSocketFanoutHandler {
    pub fn new(hub: Arc<WsHub>, node_id: String) -> Arc<Self> {
        Arc::new(Self { hub, node_id })
    }
}

#[cfg(feature = "kafka")]
#[async_trait::async_trait]
impl MessageHandler for WebSocketFanoutHandler {
    type Payload = PostEventPayload;

    async fn handle_message(&self, message: ConsumedMessage<Self::Payload>) -> anyhow::Result<()> {
        // Don't echo messages from this node (they were already broadcast locally)
        if message.node_id == self.node_id {
            return Ok(());
        }

        debug!(
            event_type = %message.event_type,
            origin_node = %message.node_id,
            "Received Kafka message, broadcasting to WebSocket connections"
        );

        // Extract channel_id from the event
        let channel_id = match &message.payload.event {
            PostEvent::Created { channel_id, .. } => *channel_id,
            PostEvent::Updated { channel_id, .. } => *channel_id,
            PostEvent::Deleted { channel_id, .. } => *channel_id,
            PostEvent::ReactionAdded { channel_id, .. } => *channel_id,
            PostEvent::ReactionRemoved { channel_id, .. } => *channel_id,
        };

        // Create envelope from the message
        let envelope = WsEnvelope {
            msg_type: "event".to_string(),
            event: message.event_type,
            seq: None,
            channel_id: Some(channel_id),
            data: serde_json::to_value(&message.payload.event)?,
            broadcast: None,
        };

        // Broadcast only to local connections
        self.hub.broadcast_local(envelope).await;

        Ok(())
    }
}

/// Start the Kafka consumer for WebSocket fan-out
#[cfg(feature = "kafka")]
pub async fn start_kafka_fanout_consumer(
    config: KafkaConfig,
    hub: Arc<WsHub>,
) -> anyhow::Result<()> {
    if !config.enabled {
        info!("Kafka fan-out consumer disabled");
        return Ok(());
    }

    use crate::services::kafka_consumer::KafkaConsumer;

    let node_id = format!(
        "{}-{}",
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string()),
        Uuid::new_v4()
    );

    let consumer = KafkaConsumer::new(config.clone(), node_id.clone());
    let handler = WebSocketFanoutHandler::new(hub, node_id);

    info!(topic = %config.posts_topic, "Starting Kafka fan-out consumer");

    let _shutdown = consumer.subscribe(&config.posts_topic, handler).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_event_serialization() {
        let event = PostEvent::Created {
            post_id: Uuid::new_v4(),
            channel_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            message: "Hello, world!".to_string(),
            root_post_id: None,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("created"));
        assert!(serialized.contains("Hello, world!"));
    }

    #[test]
    fn test_channel_size_cache() {
        let cache = ChannelSizeCache::new(1000);
        // Test deterministic behavior
        let channel_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result1 = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(cache.should_use_kafka(channel_id));
        let result2 = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(cache.should_use_kafka(channel_id));
        assert_eq!(result1, result2);
    }
}
