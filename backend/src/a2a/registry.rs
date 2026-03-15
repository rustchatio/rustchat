//! Agent Registry
//!
//! Manages agent discovery and capability advertisement.
//! Uses in-memory storage with RwLock for concurrent access.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::protocol::AgentAdvertisement;

/// Registry for managing agent advertisements
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<Uuid, AgentAdvertisement>>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    /// Create a new empty agent registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register or update an agent advertisement
    pub async fn register(&self, advertisement: AgentAdvertisement) {
        let mut agents = self.agents.write().await;
        agents.insert(advertisement.agent_id.id, advertisement);
    }

    /// Unregister an agent by its ID
    pub async fn unregister(&self, agent_id: Uuid) -> Option<AgentAdvertisement> {
        let mut agents = self.agents.write().await;
        agents.remove(&agent_id)
    }

    /// Discover agents matching a capability filter
    ///
    /// If `capability_filter` is `None`, returns all registered agents.
    /// Otherwise, returns agents that have a capability whose name contains the filter string.
    pub async fn discover(&self, capability_filter: Option<&str>) -> Vec<AgentAdvertisement> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|adv| {
                if let Some(filter) = capability_filter {
                    adv.capabilities.iter().any(|c| c.name.contains(filter))
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    /// Get a specific agent by ID
    pub async fn get_agent(&self, agent_id: Uuid) -> Option<AgentAdvertisement> {
        let agents = self.agents.read().await;
        agents.get(&agent_id).cloned()
    }

    /// Get all registered agents
    pub async fn list_all(&self) -> Vec<AgentAdvertisement> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Count registered agents
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// Clear all registrations
    pub async fn clear(&self) {
        let mut agents = self.agents.write().await;
        agents.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a2a::protocol::{AgentCapability, AgentId};

    fn create_test_advertisement(name: &str, capabilities: Vec<&str>) -> AgentAdvertisement {
        let agent_id = AgentId::new(name, "test");
        let caps = capabilities
            .into_iter()
            .map(|cap| {
                AgentCapability::new(
                    cap,
                    format!("Can {}", cap),
                    serde_json::json!({"type": "object"}),
                )
            })
            .collect();

        AgentAdvertisement::new(agent_id, caps, "http://localhost:8080", 300)
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let registry = AgentRegistry::new();
        let adv = create_test_advertisement("agent1", vec!["search", "summarize"]);
        let id = adv.agent_id.id;

        registry.register(adv.clone()).await;

        let retrieved = registry.get_agent(id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_id.name, "agent1");
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = AgentRegistry::new();
        let adv = create_test_advertisement("agent1", vec!["search"]);
        let id = adv.agent_id.id;

        registry.register(adv).await;
        assert_eq!(registry.count().await, 1);

        let removed = registry.unregister(id).await;
        assert!(removed.is_some());
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_discover_with_filter() {
        let registry = AgentRegistry::new();

        let adv1 = create_test_advertisement("agent1", vec!["search", "index"]);
        let adv2 = create_test_advertisement("agent2", vec!["summarize", "translate"]);
        let adv3 = create_test_advertisement("agent3", vec!["search", "translate"]);

        registry.register(adv1).await;
        registry.register(adv2).await;
        registry.register(adv3).await;

        // Filter by "search" capability
        let search_agents = registry.discover(Some("search")).await;
        assert_eq!(search_agents.len(), 2);

        // Filter by "translate" capability
        let translate_agents = registry.discover(Some("translate")).await;
        assert_eq!(translate_agents.len(), 2);

        // No filter - all agents
        let all_agents = registry.discover(None).await;
        assert_eq!(all_agents.len(), 3);

        // Non-matching filter
        let none = registry.discover(Some("nonexistent")).await;
        assert!(none.is_empty());
    }

    #[tokio::test]
    async fn test_discover_partial_match() {
        let registry = AgentRegistry::new();

        let adv1 = create_test_advertisement("agent1", vec!["text_search", "image_search"]);
        let adv2 = create_test_advertisement("agent2", vec!["summarize"]);

        registry.register(adv1).await;
        registry.register(adv2).await;

        // Partial match should find both search agents
        let search_agents = registry.discover(Some("search")).await;
        assert_eq!(search_agents.len(), 1); // Only agent1 has capabilities containing "search"
    }

    #[tokio::test]
    async fn test_clear() {
        let registry = AgentRegistry::new();

        for i in 0..5 {
            let adv = create_test_advertisement(&format!("agent{}", i), vec!["test"]);
            registry.register(adv).await;
        }

        assert_eq!(registry.count().await, 5);

        registry.clear().await;

        assert_eq!(registry.count().await, 0);
    }
}
