//! Call state management for Mattermost Calls plugin
//!
//! Manages active calls, participants, and call metadata in memory.
//! For multi-node deployments, this should be backed by Redis.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents an active call
#[derive(Debug, Clone)]
pub struct CallState {
    pub call_id: Uuid,
    pub channel_id: Uuid,
    pub owner_id: Uuid,
    pub started_at: i64,
    pub participants: HashMap<Uuid, Participant>,
    pub screen_sharer: Option<Uuid>,
    pub thread_id: Option<Uuid>,
}

/// Represents a call participant
#[derive(Debug, Clone)]
pub struct Participant {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub joined_at: i64,
    pub muted: bool,
    pub screen_sharing: bool,
    pub hand_raised: bool,
}

/// Manages all active call state
pub struct CallStateManager {
    /// Active calls by call_id
    calls: RwLock<HashMap<Uuid, CallState>>,
    /// Index: channel_id -> call_id for quick lookup
    channel_index: RwLock<HashMap<Uuid, Uuid>>,
}

impl CallStateManager {
    /// Create a new call state manager
    pub fn new() -> Self {
        Self {
            calls: RwLock::new(HashMap::new()),
            channel_index: RwLock::new(HashMap::new()),
        }
    }

    /// Add a new call
    pub async fn add_call(&self, call: CallState) {
        let mut calls = self.calls.write().await;
        let mut index = self.channel_index.write().await;

        index.insert(call.channel_id, call.call_id);
        calls.insert(call.call_id, call);
    }

    /// Remove a call
    pub async fn remove_call(&self, call_id: Uuid) {
        let mut calls = self.calls.write().await;
        let mut index = self.channel_index.write().await;

        if let Some(call) = calls.remove(&call_id) {
            index.remove(&call.channel_id);
        }
    }

    /// Get call by ID
    pub async fn get_call(&self, call_id: Uuid) -> Option<CallState> {
        let calls = self.calls.read().await;
        calls.get(&call_id).cloned()
    }

    /// Get call by channel ID
    pub async fn get_call_by_channel(&self, channel_id: &Uuid) -> Option<CallState> {
        let index = self.channel_index.read().await;
        let call_id = index.get(channel_id)?;

        let calls = self.calls.read().await;
        calls.get(call_id).cloned()
    }

    /// Add a participant to a call
    pub async fn add_participant(&self, call_id: Uuid, participant: Participant) {
        let mut calls = self.calls.write().await;

        if let Some(call) = calls.get_mut(&call_id) {
            call.participants.insert(participant.user_id, participant);
        }
    }

    /// Remove a participant from a call
    pub async fn remove_participant(&self, call_id: Uuid, user_id: Uuid) {
        let mut calls = self.calls.write().await;

        if let Some(call) = calls.get_mut(&call_id) {
            call.participants.remove(&user_id);

            // If screen sharer left, clear screen sharer
            if call.screen_sharer == Some(user_id) {
                call.screen_sharer = None;
            }
        }
    }

    /// Get a participant
    pub async fn get_participant(&self, call_id: Uuid, user_id: Uuid) -> Option<Participant> {
        let calls = self.calls.read().await;

        calls
            .get(&call_id)
            .and_then(|call| call.participants.get(&user_id).cloned())
    }

    /// Get all participants in a call
    pub async fn get_participants(&self, call_id: Uuid) -> Vec<Participant> {
        let calls = self.calls.read().await;

        calls
            .get(&call_id)
            .map(|call| call.participants.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Set participant muted status
    pub async fn set_muted(&self, call_id: Uuid, user_id: Uuid, muted: bool) {
        let mut calls = self.calls.write().await;

        if let Some(call) = calls.get_mut(&call_id) {
            if let Some(participant) = call.participants.get_mut(&user_id) {
                participant.muted = muted;
            }
        }
    }

    /// Set participant screen sharing status
    pub async fn set_screen_sharing(&self, call_id: Uuid, user_id: Uuid, sharing: bool) {
        let mut calls = self.calls.write().await;

        if let Some(call) = calls.get_mut(&call_id) {
            if let Some(participant) = call.participants.get_mut(&user_id) {
                participant.screen_sharing = sharing;
            }
        }
    }

    /// Set call screen sharer
    pub async fn set_screen_sharer(&self, call_id: Uuid, user_id: Option<Uuid>) {
        let mut calls = self.calls.write().await;

        if let Some(call) = calls.get_mut(&call_id) {
            call.screen_sharer = user_id;
        }
    }

    /// Set participant hand raised status
    pub async fn set_hand_raised(&self, call_id: Uuid, user_id: Uuid, raised: bool) {
        let mut calls = self.calls.write().await;

        if let Some(call) = calls.get_mut(&call_id) {
            if let Some(participant) = call.participants.get_mut(&user_id) {
                participant.hand_raised = raised;
            }
        }
    }

    /// Get all active calls (for cleanup/debugging)
    pub async fn get_all_calls(&self) -> Vec<CallState> {
        let calls = self.calls.read().await;
        calls.values().cloned().collect()
    }

    /// Get participant count for a call
    pub async fn get_participant_count(&self, call_id: Uuid) -> usize {
        let calls = self.calls.read().await;
        calls
            .get(&call_id)
            .map(|call| call.participants.len())
            .unwrap_or(0)
    }

    /// End all calls (for shutdown)
    pub async fn end_all_calls(&self) {
        let mut calls = self.calls.write().await;
        let mut index = self.channel_index.write().await;

        calls.clear();
        index.clear();
    }
}

impl Default for CallStateManager {
    fn default() -> Self {
        Self::new()
    }
}
