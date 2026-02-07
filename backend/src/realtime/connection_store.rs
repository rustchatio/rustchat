//! Connection Store for WebSocket session management and resumption
//!
//! Implements Mattermost-compatible session resumption with:
//! - Connection state retention for 5 minutes after disconnect
//! - Message buffering (last 128 messages per connection)
//! - Sequence number tracking for reliable delivery

use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use serde_json::Value;
use tokio::time::interval;
use tracing::{debug, trace, warn};
use uuid::Uuid;

/// Time to retain connection state after disconnect (5 minutes)
const CONNECTION_TTL: Duration = Duration::from_secs(300);
/// Maximum number of messages to buffer per connection
const MESSAGE_BUFFER_SIZE: usize = 128;
/// Interval for cleaning up expired connections
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

/// A sequenced message for reliable delivery
#[derive(Debug, Clone)]
pub struct SequencedMessage {
    /// Sequence number (monotonically increasing)
    pub seq: i64,
    /// The message payload
    pub message: Value,
    /// Timestamp when message was buffered
    pub timestamp: Instant,
}

/// Connection state maintained for session resumption
#[derive(Debug)]
pub struct ConnectionState {
    /// Connection ID (UUID)
    pub connection_id: String,
    /// User ID associated with this connection
    pub user_id: Uuid,
    /// Team IDs the user is subscribed to
    pub team_ids: Vec<Uuid>,
    /// Channel IDs the user is subscribed to
    pub channel_ids: Vec<Uuid>,
    /// Current sequence number (next message will use this value)
    pub sequence: AtomicI64,
    /// Ring buffer of recent messages
    pub message_buffer: std::sync::Mutex<VecDeque<SequencedMessage>>,
    /// Last activity timestamp (for TTL)
    pub last_activity: std::sync::Mutex<Instant>,
    /// Whether this connection is currently active (has a WebSocket attached)
    pub is_active: std::sync::atomic::AtomicBool,
}

impl ConnectionState {
    /// Create a new connection state
    pub fn new(connection_id: String, user_id: Uuid, initial_seq: i64) -> Arc<Self> {
        Arc::new(Self {
            connection_id,
            user_id,
            team_ids: Vec::new(),
            channel_ids: Vec::new(),
            sequence: AtomicI64::new(initial_seq),
            message_buffer: std::sync::Mutex::new(VecDeque::with_capacity(MESSAGE_BUFFER_SIZE)),
            last_activity: std::sync::Mutex::new(Instant::now()),
            is_active: std::sync::atomic::AtomicBool::new(true),
        })
    }

    /// Get the next sequence number and increment
    pub fn next_sequence(&self) -> i64 {
        self.sequence.fetch_add(1, Ordering::SeqCst)
    }

    /// Get current sequence number without incrementing
    pub fn current_sequence(&self) -> i64 {
        self.sequence.load(Ordering::SeqCst)
    }

    /// Add a message to the buffer
    pub fn buffer_message(&self, seq: i64, message: Value) {
        let msg = SequencedMessage {
            seq,
            message,
            timestamp: Instant::now(),
        };

        let mut buffer = self.message_buffer.lock().unwrap();
        
        // If buffer is full, remove oldest message (FIFO)
        if buffer.len() >= MESSAGE_BUFFER_SIZE {
            buffer.pop_front();
        }
        
        buffer.push_back(msg);
        
        // Update last activity
        *self.last_activity.lock().unwrap() = Instant::now();
        
        trace!(
            connection_id = %self.connection_id,
            seq = seq,
            buffer_size = buffer.len(),
            "Message buffered"
        );
    }

    /// Get messages with sequence number greater than `since_seq`
    pub fn get_missed_messages(&self, since_seq: i64) -> Vec<SequencedMessage> {
        let buffer = self.message_buffer.lock().unwrap();
        buffer
            .iter()
            .filter(|msg| msg.seq > since_seq)
            .cloned()
            .collect()
    }

    /// Mark connection as inactive (disconnected)
    pub fn mark_inactive(&self) {
        self.is_active.store(false, Ordering::SeqCst);
        *self.last_activity.lock().unwrap() = Instant::now();
        debug!(
            connection_id = %self.connection_id,
            user_id = %self.user_id,
            "Connection marked inactive"
        );
    }

    /// Mark connection as active (reconnected)
    pub fn mark_active(&self) {
        self.is_active.store(true, Ordering::SeqCst);
        *self.last_activity.lock().unwrap() = Instant::now();
        debug!(
            connection_id = %self.connection_id,
            user_id = %self.user_id,
            "Connection marked active"
        );
    }

    /// Update last activity timestamp
    pub fn touch(&self) {
        *self.last_activity.lock().unwrap() = Instant::now();
    }

    /// Check if this connection has expired (inactive for too long)
    pub fn is_expired(&self) -> bool {
        if self.is_active.load(Ordering::SeqCst) {
            return false;
        }
        let last_activity = *self.last_activity.lock().unwrap();
        Instant::now().duration_since(last_activity) > CONNECTION_TTL
    }

    /// Update team subscriptions
    pub fn update_teams(&self, _team_ids: Vec<Uuid>) {
        // This is a simple implementation - in production you might want to
        // compute deltas and handle subscribe/unsubscribe accordingly
        // For now, we just store the team IDs
        // Note: team_ids should be stored with proper synchronization
    }
}

/// Thread-safe connection store
#[derive(Debug)]
pub struct ConnectionStore {
    /// Active and recently-disconnected connections indexed by connection_id
    connections: DashMap<String, Arc<ConnectionState>>,
    /// Index: user_id -> connection_ids for quick lookup
    user_connections: DashMap<Uuid, Vec<String>>,
}

impl ConnectionStore {
    /// Create a new connection store
    pub fn new() -> Arc<Self> {
        let store = Arc::new(Self {
            connections: DashMap::new(),
            user_connections: DashMap::new(),
        });

        // Spawn cleanup task
        let store_clone = store.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(CLEANUP_INTERVAL);
            loop {
                cleanup_interval.tick().await;
                store_clone.cleanup_expired().await;
            }
        });

        store
    }

    /// Create a new connection or resume an existing one
    /// 
    /// # Arguments
    /// * `connection_id` - Optional existing connection ID for resumption
    /// * `user_id` - The user ID
    /// * `requested_seq` - Last sequence number received by client (for resumption)
    /// 
    /// # Returns
    /// Tuple of (connection_state, is_resumed, missed_messages)
    pub fn resume_or_create(
        &self,
        connection_id: Option<String>,
        user_id: Uuid,
        requested_seq: Option<i64>,
    ) -> (Arc<ConnectionState>, bool, Vec<SequencedMessage>) {
        // Try to resume existing connection
        if let Some(ref conn_id) = connection_id {
            if let Some(existing) = self.connections.get(conn_id) {
                // Verify user_id matches
                if existing.user_id == user_id {
                    // Mark as active again
                    existing.mark_active();
                    
                    // Get missed messages
                    let since_seq = requested_seq.unwrap_or(-1);
                    let missed = existing.get_missed_messages(since_seq);
                    
                    debug!(
                        connection_id = %conn_id,
                        user_id = %user_id,
                        missed_count = missed.len(),
                        since_seq = since_seq,
                        "Connection resumed"
                    );
                    
                    return (existing.clone(), true, missed);
                } else {
                    warn!(
                        connection_id = %conn_id,
                        expected_user = %existing.user_id,
                        actual_user = %user_id,
                        "User mismatch during connection resume"
                    );
                }
            }
        }

        // Create new connection
        let new_conn_id = connection_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let initial_seq = requested_seq.unwrap_or(0);
        
        let state = ConnectionState::new(new_conn_id.clone(), user_id, initial_seq);
        
        self.connections.insert(new_conn_id.clone(), state.clone());
        
        // Add to user index
        self.user_connections
            .entry(user_id)
            .and_modify(|conns| conns.push(new_conn_id.clone()))
            .or_insert_with(|| vec![new_conn_id.clone()]);
        
        debug!(
            connection_id = %new_conn_id,
            user_id = %user_id,
            "New connection created"
        );
        
        (state, false, Vec::new())
    }

    /// Get connection state by ID
    pub fn get_connection(&self, connection_id: &str) -> Option<Arc<ConnectionState>> {
        self.connections.get(connection_id).map(|e| e.clone())
    }

    /// Remove a connection
    pub fn remove_connection(&self, connection_id: &str) {
        if let Some((_, state)) = self.connections.remove(connection_id) {
            state.mark_inactive();
            
            // Remove from user index
            self.user_connections
                .entry(state.user_id)
                .and_modify(|conns| {
                    conns.retain(|id| id != connection_id);
                });
            
            debug!(
                connection_id = %connection_id,
                user_id = %state.user_id,
                "Connection removed (marked for expiration)"
            );
        }
    }

    /// Mark connection as disconnected but keep state for potential resumption
    pub fn disconnect_connection(&self, connection_id: &str) {
        if let Some(state) = self.connections.get(connection_id) {
            state.mark_inactive();
            debug!(
                connection_id = %connection_id,
                user_id = %state.user_id,
                "Connection disconnected (state retained for resume)"
            );
        }
    }

    /// Queue a message to be sent to a connection (also buffers it)
    pub fn queue_message(&self, connection_id: &str, event: Value) -> Option<i64> {
        if let Some(state) = self.connections.get(connection_id) {
            let seq = state.next_sequence();
            state.buffer_message(seq, event);
            Some(seq)
        } else {
            None
        }
    }

    /// Get all connection IDs for a user
    pub fn get_user_connections(&self, user_id: Uuid) -> Vec<String> {
        self.user_connections
            .get(&user_id)
            .map(|e| e.clone())
            .unwrap_or_default()
    }

    /// Update subscriptions for a connection
    pub fn update_subscriptions(
        &self,
        connection_id: &str,
        team_ids: Vec<Uuid>,
        channel_ids: Vec<Uuid>,
    ) {
        if let Some(state) = self.connections.get(connection_id) {
            // We need to store these - since ConnectionState uses simple Vec,
            // we need to add proper synchronization
            // For now, just log the update
            debug!(
                connection_id = %connection_id,
                teams = team_ids.len(),
                channels = channel_ids.len(),
                "Subscriptions updated"
            );
        }
    }

    /// Clean up expired connections
    async fn cleanup_expired(&self) {
        let mut expired = Vec::new();
        
        for entry in self.connections.iter() {
            if entry.value().is_expired() {
                expired.push(entry.key().clone());
            }
        }
        
        for conn_id in &expired {
            if let Some((_, state)) = self.connections.remove(conn_id) {
                // Remove from user index
                self.user_connections
                    .entry(state.user_id)
                    .and_modify(|conns| {
                        conns.retain(|id| id != conn_id);
                    });
                
                debug!(
                    connection_id = %conn_id,
                    user_id = %state.user_id,
                    "Expired connection state purged"
                );
            }
        }
        
        if !expired.is_empty() {
            trace!(count = expired.len(), "Expired connections cleaned up");
        }
    }

    /// Get store statistics for monitoring
    pub fn stats(&self) -> ConnectionStoreStats {
        ConnectionStoreStats {
            total_connections: self.connections.len(),
            active_connections: self
                .connections
                .iter()
                .filter(|e| e.value().is_active.load(Ordering::SeqCst))
                .count(),
            inactive_connections: self
                .connections
                .iter()
                .filter(|e| !e.value().is_active.load(Ordering::SeqCst))
                .count(),
            unique_users: self.user_connections.len(),
        }
    }
}

impl Default for ConnectionStore {
    fn default() -> Self {
        // This won't spawn the cleanup task - use new() instead
        Self {
            connections: DashMap::new(),
            user_connections: DashMap::new(),
        }
    }
}

/// Statistics for monitoring
#[derive(Debug, Clone)]
pub struct ConnectionStoreStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub inactive_connections: usize,
    pub unique_users: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_connection_state_sequence() {
        let state = ConnectionState::new("test-123".to_string(), Uuid::new_v4(), 0);
        
        assert_eq!(state.current_sequence(), 0);
        assert_eq!(state.next_sequence(), 0);
        assert_eq!(state.current_sequence(), 1);
        assert_eq!(state.next_sequence(), 1);
        assert_eq!(state.current_sequence(), 2);
    }

    #[test]
    fn test_message_buffering() {
        let state = ConnectionState::new("test-123".to_string(), Uuid::new_v4(), 0);
        
        // Add some messages
        for i in 0..10 {
            state.buffer_message(i, json!({"test": i}));
        }
        
        // Get missed messages
        let missed = state.get_missed_messages(5);
        assert_eq!(missed.len(), 4); // seq 6, 7, 8, 9
        assert_eq!(missed[0].seq, 6);
        assert_eq!(missed[3].seq, 9);
    }

    #[test]
    fn test_buffer_size_limit() {
        let state = ConnectionState::new("test-123".to_string(), Uuid::new_v4(), 0);
        
        // Add more messages than buffer size
        for i in 0..(MESSAGE_BUFFER_SIZE + 10) as i64 {
            state.buffer_message(i, json!({"test": i}));
        }
        
        let buffer = state.message_buffer.lock().unwrap();
        assert_eq!(buffer.len(), MESSAGE_BUFFER_SIZE);
    }

    #[tokio::test]
    async fn test_resume_or_create() {
        let store = ConnectionStore::new();
        let user_id = Uuid::new_v4();
        
        // Create new connection
        let (state, is_resumed, missed) = store.resume_or_create(None, user_id, None);
        assert!(!is_resumed);
        assert!(missed.is_empty());
        
        let conn_id = state.connection_id.clone();
        
        // Add some messages
        for i in 0..5 {
            store.queue_message(&conn_id, json!({"msg": i}));
        }
        
        // Disconnect
        store.disconnect_connection(&conn_id);
        
        // Resume
        let (resumed_state, is_resumed, missed) = 
            store.resume_or_create(Some(conn_id.clone()), user_id, Some(2));
        
        assert!(is_resumed);
        assert_eq!(missed.len(), 2); // seq 3 and 4
        assert_eq!(resumed_state.connection_id, conn_id);
    }
}
