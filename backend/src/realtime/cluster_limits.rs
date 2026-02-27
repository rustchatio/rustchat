//! Cluster-aware connection limits using Redis-backed presence keys.
//!
//! This module intentionally uses the same key scheme as websocket presence
//! tracking to avoid split-brain counters across parallel implementations.

use deadpool_redis::redis::AsyncCommands;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::api::AppState;
use crate::error::AppError;

const PRESENCE_HEARTBEAT_TTL_SECONDS: u64 = 90;

fn presence_connection_key(user_id: Uuid) -> String {
    format!("rustchat:presence:user:{user_id}:connections")
}

fn presence_heartbeat_key(user_id: Uuid, connection_id: &str) -> String {
    format!("rustchat:presence:user:{user_id}:connection:{connection_id}:heartbeat")
}

/// Check if user has exceeded cluster-wide connection limit.
pub async fn check_cluster_connection_limit(
    state: &AppState,
    user_id: Uuid,
    max_connections: usize,
) -> Result<bool, AppError> {
    let local_count = state.ws_hub.user_connection_count(user_id).await;
    if local_count >= max_connections {
        return Ok(false);
    }

    match get_global_connection_count(state, user_id).await {
        Ok(global_count) => {
            if global_count >= max_connections {
                debug!(
                    user_id = %user_id,
                    global_count = global_count,
                    max = max_connections,
                    "Cluster connection limit exceeded"
                );
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(e) => {
            warn!(
                user_id = %user_id,
                error = %e,
                "Failed to get global connection count, using local count only"
            );
            Ok(local_count < max_connections)
        }
    }
}

/// Get global connection count from presence registry and prune stale entries.
pub async fn get_global_connection_count(
    state: &AppState,
    user_id: Uuid,
) -> anyhow::Result<usize> {
    let mut conn = state.redis.get().await?;
    let key = presence_connection_key(user_id);

    let connections: Vec<String> = conn.smembers(&key).await?;
    if connections.is_empty() {
        return Ok(0);
    }

    let mut active_count = 0usize;
    let mut stale: Vec<String> = Vec::new();

    for connection_id in connections {
        let heartbeat_key = presence_heartbeat_key(user_id, &connection_id);
        let is_alive: bool = conn.exists(&heartbeat_key).await?;
        if is_alive {
            active_count += 1;
        } else {
            stale.push(connection_id);
        }
    }

    if !stale.is_empty() {
        let _: usize = conn.srem(&key, &stale).await?;
    }

    if active_count > 0 {
        let _: () = conn
            .expire(&key, (PRESENCE_HEARTBEAT_TTL_SECONDS * 2) as i64)
            .await?;
    } else {
        let _: usize = conn.del(&key).await?;
    }

    Ok(active_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_keys() {
        let user_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        assert_eq!(
            presence_connection_key(user_id),
            "rustchat:presence:user:11111111-1111-1111-1111-111111111111:connections"
        );
        assert_eq!(
            presence_heartbeat_key(user_id, "conn-123"),
            "rustchat:presence:user:11111111-1111-1111-1111-111111111111:connection:conn-123:heartbeat"
        );
    }
}
