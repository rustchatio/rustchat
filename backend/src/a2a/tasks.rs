//! Task Management
//!
//! Manages task lifecycle: create, update status, complete/fail.
//! Coordinates with the message bus for distributed task delegation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::bus::A2AMessageBus;
use super::protocol::{A2AMessage, TaskRequest, TaskResponse, TaskStatus, TaskStatusUpdate};

/// Current state of a task
#[derive(Debug, Clone)]
pub struct TaskState {
    pub request: TaskRequest,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl TaskState {
    /// Create a new task state from a request
    pub fn new(request: TaskRequest) -> Self {
        Self {
            request,
            status: TaskStatus::Pending,
            result: None,
            error: None,
        }
    }

    /// Check if the task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
}

/// Manages task lifecycle and coordination
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<Uuid, TaskState>>>,
    bus: Arc<A2AMessageBus>,
}

impl TaskManager {
    /// Create a new task manager with the given message bus
    pub fn new(bus: Arc<A2AMessageBus>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            bus,
        }
    }

    /// Create a new task and broadcast the request
    pub async fn create_task(
        &self,
        request: TaskRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let task_id = request.task_id;
        let state = TaskState::new(request.clone());

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, state);
        }

        info!(
            task_id = %task_id,
            sender = %request.sender.name,
            receiver = %request.receiver.name,
            task_type = %request.task_type,
            "Task created"
        );

        // Broadcast task request
        let message = A2AMessage::TaskRequest {
            task_id: request.task_id,
            sender: request.sender,
            receiver: request.receiver,
            task_type: request.task_type,
            payload: request.payload,
            deadline: request.deadline,
        };
        self.bus.publish(&message).await?;

        debug!(task_id = %task_id, "Task request broadcasted");

        Ok(())
    }

    /// Update task status and broadcast the update
    pub async fn update_task_status(
        &self,
        update: TaskStatusUpdate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let task_id = update.task_id;

        {
            let mut tasks = self.tasks.write().await;
            if let Some(state) = tasks.get_mut(&task_id) {
                state.status = update.status.clone();
            } else {
                warn!(task_id = %task_id, "Status update for unknown task");
                return Ok(());
            }
        }

        debug!(
            task_id = %task_id,
            status = ?update.status,
            "Task status updated"
        );

        // Broadcast status update
        let message = A2AMessage::TaskStatusUpdate {
            task_id: update.task_id,
            sender: update.sender,
            status: update.status,
            progress_percent: update.progress_percent,
            message: update.message,
        };
        self.bus.publish(&message).await?;

        Ok(())
    }

    /// Complete a task with a response and broadcast
    pub async fn complete_task(
        &self,
        response: TaskResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let task_id = response.task_id;

        {
            let mut tasks = self.tasks.write().await;
            if let Some(state) = tasks.get_mut(&task_id) {
                state.status = response.status.clone();
                state.result = response.result.clone();
                state.error = response.error.clone();
            } else {
                warn!(task_id = %task_id, "Completion for unknown task");
                return Ok(());
            }
        }

        info!(
            task_id = %task_id,
            status = ?response.status,
            has_result = response.result.is_some(),
            has_error = response.error.is_some(),
            "Task completed"
        );

        // Broadcast task completion
        let message = A2AMessage::TaskResponse {
            task_id: response.task_id,
            sender: response.sender,
            status: response.status,
            result: response.result,
            error: response.error,
        };
        self.bus.publish(&message).await?;

        Ok(())
    }

    /// Get the current state of a task
    pub async fn get_task(&self, task_id: Uuid) -> Option<TaskState> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned()
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Vec<TaskState> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// List tasks by status
    pub async fn list_tasks_by_status(&self, status: TaskStatus) -> Vec<TaskState> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let update = TaskStatusUpdate {
            task_id,
            sender: AgentId::new("system", "rustchat"),
            status: TaskStatus::Cancelled,
            progress_percent: None,
            message: Some("Task cancelled by user".to_string()),
        };

        self.update_task_status(update).await
    }

    /// Clean up completed/cancelled/failed tasks older than a certain age
    pub async fn cleanup_terminal_tasks(
        &self,
        _max_age: chrono::Duration,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // This would require tracking task creation/update timestamps
        // For now, we just remove all terminal tasks
        let mut tasks = self.tasks.write().await;
        let initial_count = tasks.len();

        tasks.retain(|_id, state| !state.is_terminal());

        let removed = initial_count - tasks.len();
        info!(removed = removed, "Cleaned up terminal tasks");

        Ok(removed)
    }
}

// Re-export AgentId for use in cancel_task
use super::protocol::AgentId;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a2a::protocol::AgentId;

    // Mock message bus for testing (currently unused but reserved for future tests)
    #[allow(dead_code)]
    struct MockBus;

    #[tokio::test]
    async fn test_task_state() {
        let request = TaskRequest {
            task_id: Uuid::new_v4(),
            sender: AgentId::new("sender", "test"),
            receiver: AgentId::new("receiver", "test"),
            task_type: "test_task".to_string(),
            payload: serde_json::json!({"key": "value"}),
            deadline: None,
        };

        let state = TaskState::new(request);
        assert_eq!(state.status, TaskStatus::Pending);
        assert!(!state.is_terminal());
        assert!(state.result.is_none());
        assert!(state.error.is_none());
    }

    #[tokio::test]
    async fn test_task_state_terminal() {
        let request = TaskRequest {
            task_id: Uuid::new_v4(),
            sender: AgentId::new("sender", "test"),
            receiver: AgentId::new("receiver", "test"),
            task_type: "test_task".to_string(),
            payload: serde_json::json!({}),
            deadline: None,
        };

        let mut state = TaskState::new(request);

        state.status = TaskStatus::Completed;
        assert!(state.is_terminal());

        state.status = TaskStatus::Failed;
        assert!(state.is_terminal());

        state.status = TaskStatus::Cancelled;
        assert!(state.is_terminal());

        state.status = TaskStatus::InProgress;
        assert!(!state.is_terminal());
    }

    // Note: Tests requiring the actual message bus need a running Redis instance
    // Those would be integration tests
}
