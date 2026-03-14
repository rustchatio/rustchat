//! A2A Protocol Types
//!
//! Core message types for Agent-to-Agent communication.
//! Supports discovery, task delegation, and status tracking.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentId {
    pub id: Uuid,
    pub name: String,
    pub framework: String, // "langchain", "crewai", "custom", etc.
}

impl AgentId {
    /// Create a new agent identifier
    pub fn new(name: impl Into<String>, framework: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            framework: framework.into(),
        }
    }
}

/// A capability that an agent can advertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

impl AgentCapability {
    /// Create a new capability
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// Advertisement published by an agent to announce its presence and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAdvertisement {
    pub agent_id: AgentId,
    pub capabilities: Vec<AgentCapability>,
    pub endpoint: String, // Where to send messages
    pub ttl_seconds: u32,
}

impl AgentAdvertisement {
    /// Create a new agent advertisement
    pub fn new(
        agent_id: AgentId,
        capabilities: Vec<AgentCapability>,
        endpoint: impl Into<String>,
        ttl_seconds: u32,
    ) -> Self {
        Self {
            agent_id,
            capabilities,
            endpoint: endpoint.into(),
            ttl_seconds,
        }
    }
}

/// A2A Message types for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum A2AMessage {
    /// Request to discover available agents
    DiscoveryRequest {
        requester: AgentId,
        capability_filter: Option<String>,
    },
    /// Response containing matching agents
    DiscoveryResponse {
        agents: Vec<AgentAdvertisement>,
    },
    /// Request to execute a task
    TaskRequest {
        task_id: Uuid,
        sender: AgentId,
        receiver: AgentId,
        task_type: String,
        payload: serde_json::Value,
        deadline: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// Response indicating task completion
    TaskResponse {
        task_id: Uuid,
        sender: AgentId,
        status: TaskStatus,
        result: Option<serde_json::Value>,
        error: Option<String>,
    },
    /// Status update for a running task
    TaskStatusUpdate {
        task_id: Uuid,
        sender: AgentId,
        status: TaskStatus,
        progress_percent: Option<u8>,
        message: Option<String>,
    },
}

/// Current status of a task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Request to execute a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: Uuid,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response to a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: Uuid,
    pub sender: AgentId,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Status update for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusUpdate {
    pub task_id: Uuid,
    pub sender: AgentId,
    pub status: TaskStatus,
    pub progress_percent: Option<u8>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let agent = AgentId::new("test_agent", "custom");
        assert_eq!(agent.name, "test_agent");
        assert_eq!(agent.framework, "custom");
    }

    #[test]
    fn test_agent_capability_creation() {
        let params = serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            }
        });
        let cap = AgentCapability::new("search", "Search documents", params.clone());
        assert_eq!(cap.name, "search");
        assert_eq!(cap.parameters, params);
    }

    #[test]
    fn test_a2a_message_serialization() {
        let agent = AgentId::new("sender", "custom");
        let msg = A2AMessage::DiscoveryRequest {
            requester: agent,
            capability_filter: Some("search".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("DiscoveryRequest"));
        assert!(json.contains("search"));

        let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            A2AMessage::DiscoveryRequest {
                capability_filter, ..
            } => {
                assert_eq!(capability_filter, Some("search".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_task_status_serialization() {
        let statuses = vec![
            TaskStatus::Pending,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Cancelled,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}
