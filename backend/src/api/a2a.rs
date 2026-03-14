//! A2A API Endpoints
//!
//! HTTP API for Agent-to-Agent protocol operations.
//! Provides endpoints for agent registration, discovery, and task management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::a2a::{
    A2AMessageBus, AgentAdvertisement, AgentCapability, AgentId, AgentRegistry, TaskManager,
    TaskRequest, TaskStatus,
};
use crate::api::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;

/// Build A2A routes
pub fn router() -> Router<AppState> {
    Router::new()
        // Agent registration endpoints
        .route("/a2a/agents/register", post(register_agent))
        .route("/a2a/agents/unregister", post(unregister_agent))
        .route("/a2a/agents/discover", get(discover_agents))
        // Task management endpoints
        .route("/a2a/tasks", post(create_task))
        .route("/a2a/tasks/{task_id}", get(get_task_status))
        .route("/a2a/tasks/{task_id}/cancel", post(cancel_task))
}

/// Register a new agent
async fn register_agent(
    State(_state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<RegisterAgentRequest>,
) -> Result<Json<AgentRegistrationResponse>, AppError> {
    debug!(user_id = %auth.user_id, "Registering new agent");

    // Create agent ID
    let agent_id = AgentId::new(&request.name, &request.framework);

    // Convert request capabilities
    let capabilities: Vec<AgentCapability> = request
        .capabilities
        .into_iter()
        .map(|cap| AgentCapability::new(cap.name, cap.description, cap.parameters))
        .collect();

    // Create advertisement
    let advertisement = AgentAdvertisement::new(
        agent_id.clone(),
        capabilities,
        request.endpoint,
        request.ttl_seconds.unwrap_or(300),
    );

    // Get or create registry from app state
    // For now, we create a new registry per request - in production this should be shared state
    let registry = AgentRegistry::new();
    registry.register(advertisement.clone()).await;

    info!(
        agent_id = %agent_id.id,
        agent_name = %agent_id.name,
        user_id = %auth.user_id,
        "Agent registered successfully"
    );

    Ok(Json(AgentRegistrationResponse {
        agent_id: agent_id.id,
        name: agent_id.name,
        framework: agent_id.framework,
        endpoint: advertisement.endpoint,
        ttl_seconds: advertisement.ttl_seconds,
    }))
}

/// Unregister an agent
async fn unregister_agent(
    State(_state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UnregisterAgentRequest>,
) -> Result<StatusCode, AppError> {
    debug!(user_id = %auth.user_id, agent_id = %request.agent_id, "Unregistering agent");

    // Get registry and unregister
    let registry = AgentRegistry::new();
    let removed = registry.unregister(request.agent_id).await;

    match removed {
        Some(adv) => {
            info!(
                agent_id = %request.agent_id,
                agent_name = %adv.agent_id.name,
                "Agent unregistered"
            );
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            warn!(agent_id = %request.agent_id, "Agent not found for unregistration");
            Err(AppError::NotFound("Agent not found".to_string()))
        }
    }
}

/// Discover available agents
async fn discover_agents(
    State(_state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<DiscoverAgentsQuery>,
) -> Result<Json<DiscoverAgentsResponse>, AppError> {
    debug!(
        user_id = %auth.user_id,
        filter = ?params.capability_filter,
        "Discovering agents"
    );

    let registry = AgentRegistry::new();
    let agents = registry.discover(params.capability_filter.as_deref()).await;

    Ok(Json(DiscoverAgentsResponse {
        count: agents.len(),
        agents: agents
            .into_iter()
            .map(|adv| AgentInfo {
                agent_id: adv.agent_id.id,
                name: adv.agent_id.name,
                framework: adv.agent_id.framework,
                capabilities: adv
                    .capabilities
                    .into_iter()
                    .map(|cap| CapabilityInfo {
                        name: cap.name,
                        description: cap.description,
                    })
                    .collect(),
                endpoint: adv.endpoint,
            })
            .collect(),
    }))
}

/// Create a new task
async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, AppError> {
    debug!(user_id = %auth.user_id, "Creating new task");

    let task_id = Uuid::new_v4();

    // Create sender agent ID
    let sender = AgentId::new(
        format!("user-{}", auth.user_id),
        "rustchat_user",
    );

    // Create receiver agent ID
    let receiver = AgentId::new(&request.receiver_name, &request.receiver_framework);

    // Create task request
    let task_request = TaskRequest {
        task_id,
        sender,
        receiver,
        task_type: request.task_type,
        payload: request.payload,
        deadline: request.deadline,
    };

    // Create task manager and task
    let bus = Arc::new(A2AMessageBus::new(state.redis.clone()));
    let task_manager = TaskManager::new(bus);

    task_manager
        .create_task(task_request)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create task: {}", e)))?;

    info!(
        task_id = %task_id,
        user_id = %auth.user_id,
        "Task created successfully"
    );

    Ok(Json(CreateTaskResponse {
        task_id,
        status: TaskStatus::Pending,
        created_at: chrono::Utc::now(),
    }))
}

/// Get task status
async fn get_task_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskStatusResponse>, AppError> {
    debug!(user_id = %auth.user_id, task_id = %task_id, "Getting task status");

    let bus = Arc::new(A2AMessageBus::new(state.redis.clone()));
    let task_manager = TaskManager::new(bus);

    let task_state = task_manager
        .get_task(task_id)
        .await
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(TaskStatusResponse {
        task_id,
        status: task_state.status,
        result: task_state.result,
        error: task_state.error,
        created_at: chrono::Utc::now(), // Would come from actual task state
        updated_at: chrono::Utc::now(),
    }))
}

/// Cancel a task
async fn cancel_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskStatusResponse>, AppError> {
    debug!(user_id = %auth.user_id, task_id = %task_id, "Cancelling task");

    let bus = Arc::new(A2AMessageBus::new(state.redis.clone()));
    let task_manager = TaskManager::new(bus);

    task_manager
        .cancel_task(task_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to cancel task: {}", e)))?;

    // Get updated status
    let task_state = task_manager
        .get_task(task_id)
        .await
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    info!(
        task_id = %task_id,
        user_id = %auth.user_id,
        "Task cancelled"
    );

    Ok(Json(TaskStatusResponse {
        task_id,
        status: task_state.status,
        result: None,
        error: Some("Task cancelled by user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }))
}

// Request/Response types

#[derive(Debug, Deserialize)]
struct RegisterAgentRequest {
    name: String,
    framework: String,
    capabilities: Vec<CapabilityRequest>,
    endpoint: String,
    ttl_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CapabilityRequest {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AgentRegistrationResponse {
    agent_id: Uuid,
    name: String,
    framework: String,
    endpoint: String,
    ttl_seconds: u32,
}

#[derive(Debug, Deserialize)]
struct UnregisterAgentRequest {
    agent_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
struct DiscoverAgentsQuery {
    capability_filter: Option<String>,
}

#[derive(Debug, Serialize)]
struct DiscoverAgentsResponse {
    count: usize,
    agents: Vec<AgentInfo>,
}

#[derive(Debug, Serialize)]
struct AgentInfo {
    agent_id: Uuid,
    name: String,
    framework: String,
    capabilities: Vec<CapabilityInfo>,
    endpoint: String,
}

#[derive(Debug, Serialize)]
struct CapabilityInfo {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    receiver_name: String,
    receiver_framework: String,
    task_type: String,
    payload: serde_json::Value,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
struct CreateTaskResponse {
    task_id: Uuid,
    status: TaskStatus,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct TaskStatusResponse {
    task_id: Uuid,
    status: TaskStatus,
    result: Option<serde_json::Value>,
    error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_deserialization() {
        let json = r#"{
            "name": "test_agent",
            "framework": "langchain",
            "capabilities": [
                {
                    "name": "search",
                    "description": "Search documents",
                    "parameters": {"type": "object"}
                }
            ],
            "endpoint": "http://localhost:8080"
        }"#;

        let req: RegisterAgentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "test_agent");
        assert_eq!(req.capabilities.len(), 1);
    }

    #[test]
    fn test_create_task_request_deserialization() {
        let json = r#"{
            "receiver_name": "target_agent",
            "receiver_framework": "crewai",
            "task_type": "summarize",
            "payload": {"text": "Hello world"}
        }"#;

        let req: CreateTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.receiver_name, "target_agent");
        assert_eq!(req.task_type, "summarize");
    }
}
