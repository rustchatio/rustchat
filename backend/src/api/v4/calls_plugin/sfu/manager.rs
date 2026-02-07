//! SFU Manager
//!
//! Manages SFU instances for active calls. Each call has its own SFU
//! to isolate media traffic between different calls.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::SFU;
use crate::config::CallsConfig;

/// Manages SFU instances for all active calls
pub struct SFUManager {
    /// Active SFUs: call_id -> SFU instance
    sfus: Arc<RwLock<HashMap<Uuid, Arc<SFU>>>>,
    /// Calls configuration
    config: CallsConfig,
}

impl SFUManager {
    /// Create a new SFU manager
    pub fn new(config: CallsConfig) -> Arc<Self> {
        Arc::new(Self {
            sfus: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Create or get an SFU for a call
    pub async fn get_or_create_sfu(
        &self,
        call_id: Uuid,
    ) -> Result<Arc<SFU>, Box<dyn std::error::Error + Send + Sync>> {
        // Check if SFU already exists
        {
            let sfus = self.sfus.read().await;
            if let Some(sfu) = sfus.get(&call_id) {
                return Ok(sfu.clone());
            }
        }

        // Create new SFU
        let sfu = SFU::new(self.config.clone()).await?;

        // Store it
        self.sfus.write().await.insert(call_id, sfu.clone());

        Ok(sfu)
    }

    /// Get an existing SFU for a call
    pub async fn get_sfu(&self, call_id: Uuid) -> Option<Arc<SFU>> {
        self.sfus.read().await.get(&call_id).cloned()
    }

    /// Remove an SFU (when call ends)
    pub async fn remove_sfu(&self, call_id: Uuid) {
        self.sfus.write().await.remove(&call_id);
    }

    /// Check if an SFU exists for a call
    pub async fn has_sfu(&self, call_id: Uuid) -> bool {
        self.sfus.read().await.contains_key(&call_id)
    }

    /// Get count of active SFUs
    pub async fn active_sfu_count(&self) -> usize {
        self.sfus.read().await.len()
    }

    /// Get all active call IDs
    pub async fn active_call_ids(&self) -> Vec<Uuid> {
        self.sfus.read().await.keys().cloned().collect()
    }

    /// Clean up all SFUs (for shutdown)
    pub async fn cleanup_all(&self) {
        self.sfus.write().await.clear();
    }
}

impl Default for SFUManager {
    fn default() -> Self {
        // Create with empty config - this is mainly for testing
        Self {
            sfus: Arc::new(RwLock::new(HashMap::new())),
            config: CallsConfig::default(),
        }
    }
}
