//! Database replica routing for rustchat
//!
//! Provides read/write splitting between primary and read replicas.

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::{info, warn};

/// Database pool manager with replica support
pub struct DatabaseManager {
    primary: Pool<Postgres>,
    replicas: Vec<Pool<Postgres>>,
    replica_counter: std::sync::atomic::AtomicUsize,
}

/// Database operation type for routing decisions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    /// Read operation - can use replicas
    Read,
    /// Write operation - must use primary
    Write,
}

/// Query execution options
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub operation_type: OperationType,
    pub prefer_primary: bool,
    pub force_primary: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            operation_type: OperationType::Read,
            prefer_primary: false,
            force_primary: false,
        }
    }
}

impl QueryOptions {
    /// Create options for a write operation
    pub fn write() -> Self {
        Self {
            operation_type: OperationType::Write,
            prefer_primary: true,
            force_primary: true,
        }
    }

    /// Create options for a read operation
    pub fn read() -> Self {
        Self {
            operation_type: OperationType::Read,
            prefer_primary: false,
            force_primary: false,
        }
    }

    /// Prefer primary even for reads (for read-after-write consistency)
    pub fn read_consistent() -> Self {
        Self {
            operation_type: OperationType::Read,
            prefer_primary: true,
            force_primary: false,
        }
    }
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(
        primary_url: &str,
        replica_urls: &[String],
        max_connections: u32,
    ) -> anyhow::Result<Arc<Self>> {
        // Create primary connection pool
        let primary = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(max_connections / 4)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(primary_url)
            .await?;

        info!("Connected to primary database");

        // Test primary connection
        sqlx::query("SELECT 1").fetch_one(&primary).await?;

        // Create replica connection pools
        let mut replicas = Vec::new();
        for (i, url) in replica_urls.iter().enumerate() {
            match PgPoolOptions::new()
                .max_connections(max_connections / 2)
                .min_connections(2)
                .acquire_timeout(std::time::Duration::from_secs(3))
                .idle_timeout(std::time::Duration::from_secs(300))
                .max_lifetime(std::time::Duration::from_secs(1800))
                .connect(url)
                .await
            {
                Ok(pool) => {
                    // Test connection
                    match sqlx::query("SELECT 1").fetch_one(&pool).await {
                        Ok(_) => {
                            info!(replica_index = i, "Connected to read replica");
                            replicas.push(pool);
                        }
                        Err(e) => {
                            warn!(replica_index = i, error = %e, "Failed to test replica connection, skipping");
                        }
                    }
                }
                Err(e) => {
                    warn!(replica_index = i, error = %e, "Failed to connect to replica, skipping");
                }
            }
        }

        info!(
            primary = "connected",
            replicas = replicas.len(),
            "Database manager initialized"
        );

        Ok(Arc::new(Self {
            primary,
            replicas,
            replica_counter: std::sync::atomic::AtomicUsize::new(0),
        }))
    }

    /// Get the appropriate pool for the operation
    pub fn get_pool(&self, options: QueryOptions) -> &Pool<Postgres> {
        // Force primary for writes
        if options.force_primary || options.operation_type == OperationType::Write {
            return &self.primary;
        }

        // Prefer primary for consistent reads
        if options.prefer_primary {
            return &self.primary;
        }

        // Use round-robin for replicas
        if self.replicas.is_empty() {
            return &self.primary;
        }

        let counter = self
            .replica_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let index = counter % self.replicas.len();

        &self.replicas[index]
    }

    /// Get primary pool
    pub fn primary(&self) -> &Pool<Postgres> {
        &self.primary
    }

    /// Get a specific replica pool
    pub fn replica(&self, index: usize) -> Option<&Pool<Postgres>> {
        self.replicas.get(index)
    }

    /// Get number of replicas
    pub fn replica_count(&self) -> usize {
        self.replicas.len()
    }

    /// Check health of all connections
    pub async fn health_check(&self) -> DatabaseHealth {
        let primary_healthy = self.check_pool_health(&self.primary).await;

        let mut replica_health = Vec::new();
        for (i, replica) in self.replicas.iter().enumerate() {
            let healthy = self.check_pool_health(replica).await;
            replica_health.push((i, healthy));
        }

        DatabaseHealth {
            primary_healthy,
            replica_health,
        }
    }

    /// Check pool health
    async fn check_pool_health(&self, pool: &Pool<Postgres>) -> bool {
        sqlx::query("SELECT 1").fetch_one(pool).await.is_ok()
    }

    /// Close all connections
    pub async fn close(&self) {
        self.primary.close().await;
        for replica in &self.replicas {
            replica.close().await;
        }
    }
}

/// Database health status
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub primary_healthy: bool,
    pub replica_health: Vec<(usize, bool)>,
}

impl DatabaseHealth {
    /// Check if the overall system is healthy
    pub fn is_healthy(&self) -> bool {
        self.primary_healthy
    }

    /// Get the number of healthy replicas
    pub fn healthy_replica_count(&self) -> usize {
        self.replica_health.iter().filter(|(_, h)| *h).count()
    }
}

/// Macro for executing queries with automatic routing
#[macro_export]
macro_rules! query_with_options {
    ($db:expr, $options:expr, $query:expr) => {
        sqlx::query($query).fetch_all($db.get_pool($options))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_options() {
        let write_opts = QueryOptions::write();
        assert_eq!(write_opts.operation_type, OperationType::Write);
        assert!(write_opts.force_primary);

        let read_opts = QueryOptions::read();
        assert_eq!(read_opts.operation_type, OperationType::Read);
        assert!(!read_opts.force_primary);

        let consistent_opts = QueryOptions::read_consistent();
        assert_eq!(consistent_opts.operation_type, OperationType::Read);
        assert!(consistent_opts.prefer_primary);
        assert!(!consistent_opts.force_primary);
    }

    #[tokio::test]
    async fn test_health_check_without_pools() {
        // This test just verifies the structure works
        // Real tests require database connections
    }
}
