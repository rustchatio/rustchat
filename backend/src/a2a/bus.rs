//! A2A Message Bus
//!
//! Redis-backed pub/sub for distributed agent communication.
//! Supports message broadcasting and subscription-based delivery.

use futures_util::StreamExt;
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::protocol::A2AMessage;

// Redis connection URL for pub/sub
const REDIS_URL: &str = "redis://127.0.0.1:6379";

/// Message bus for A2A communication using Redis pub/sub
pub struct A2AMessageBus {
    redis: deadpool_redis::Pool,
    channel: String,
}

impl A2AMessageBus {
    /// Create a new message bus with the given Redis pool
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        Self {
            redis,
            channel: "rustchat:a2a:messages".to_string(),
        }
    }

    /// Create a new message bus with a custom channel name
    pub fn with_channel(redis: deadpool_redis::Pool, channel: impl Into<String>) -> Self {
        Self {
            redis,
            channel: channel.into(),
        }
    }

    /// Get the channel name
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Publish a message to the bus
    pub async fn publish(&self, message: &A2AMessage) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_string(message)?;
        let mut conn = self.redis.get().await?;
        conn.publish::<_, _, ()>(&self.channel, payload).await?;
        Ok(())
    }

    /// Publish a raw message payload
    pub async fn publish_raw(&self, payload: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.redis.get().await?;
        conn.publish::<_, _, ()>(&self.channel, payload).await?;
        Ok(())
    }

    /// Subscribe to messages and handle them with the provided handler
    ///
    /// The handler is called for each message received. This function runs indefinitely
    /// until the stream ends or an error occurs.
    pub async fn subscribe<F>(self: Arc<Self>, mut handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(A2AMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send,
    {
        // Create a new Redis client for pub/sub
        let client = redis::Client::open(REDIS_URL)?;
        let mut pubsub = client.get_async_pubsub().await?;
        pubsub.subscribe(&self.channel).await?;

        info!(
            channel = %self.channel,
            "A2A message bus subscriber started"
        );

        let mut msg_stream = pubsub.on_message();
        loop {
            match msg_stream.next().await {
                Some(msg) => {
                    match msg.get_payload::<String>() {
                        Ok(payload) => {
                            match serde_json::from_str::<A2AMessage>(&payload) {
                                Ok(message) => {
                                    handler(message).await;
                                }
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        payload = %payload,
                                        "Failed to deserialize A2A message"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to get message payload");
                        }
                    }
                }
                None => {
                    error!("A2A pub/sub stream ended");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Subscribe with typed error handling
    pub async fn subscribe_with_result<F>(
        self: Arc<Self>,
        mut handler: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(A2AMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
            + Send,
    {
        let client = redis::Client::open(REDIS_URL)?;
        let mut pubsub = client.get_async_pubsub().await?;
        pubsub.subscribe(&self.channel).await?;

        info!(
            channel = %self.channel,
            "A2A message bus subscriber started (with result handling)"
        );

        let mut msg_stream = pubsub.on_message();
        loop {
            match msg_stream.next().await {
                Some(msg) => {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Ok(message) = serde_json::from_str::<A2AMessage>(&payload) {
                            if let Err(e) = handler(message).await {
                                warn!(error = %e, "Handler returned error");
                            }
                        }
                    }
                }
                None => {
                    error!("A2A pub/sub stream ended");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a2a::protocol::{AgentId, AgentCapability, AgentAdvertisement};

    // Note: These tests would require a running Redis instance
    // For unit tests without Redis, we test serialization/deserialization

    #[test]
    fn test_message_serialization() {
        let agent = AgentId::new("test", "custom");
        let msg = A2AMessage::DiscoveryRequest {
            requester: agent,
            capability_filter: Some("search".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            A2AMessage::DiscoveryRequest { requester, .. } => {
                assert_eq!(requester.name, "test");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_discover_response_serialization() {
        let agent_id = AgentId::new("agent1", "langchain");
        let advertisement = AgentAdvertisement {
            agent_id,
            capabilities: vec![AgentCapability::new(
                "summarize",
                "Summarize text",
                serde_json::json!({"type": "object"}),
            )],
            endpoint: "http://localhost:8080".to_string(),
            ttl_seconds: 300,
        };

        let msg = A2AMessage::DiscoveryResponse {
            agents: vec![advertisement],
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("DiscoveryResponse"));
        assert!(json.contains("langchain"));
    }
}
