//! Rate limiting service with Redis-backed atomic operations
//!
//! Implements rate limiting using Redis Lua scripts for atomic INCR+EXPIRE operations.
//! Supports tiered limits based on entity type (human, agent, service, CI).

use crate::error::{ApiResult, AppError};
use crate::models::entity::RateLimitTier;
use deadpool_redis::redis::{AsyncCommands, Script};
use deadpool_redis::Pool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Redis key prefix for rate limiting
const RATE_LIMIT_KEY_PREFIX: &str = "ratelimit";

/// Lua script for atomic INCR+EXPIRE.
/// Returns a two-element array: [current_count, ttl_remaining_secs]
///
/// KEYS[1]: Redis key
/// ARGV[1]: Window TTL in seconds
const RATE_LIMIT_SCRIPT: &str = r#"
local key = KEYS[1]
local ttl = tonumber(ARGV[1])
local count = redis.call('INCR', key)
if count == 1 then
    redis.call('EXPIRE', key, ttl)
end
local remaining_ttl = redis.call('TTL', key)
return {count, remaining_ttl}
"#;

/// Rate limiting configuration for a tier
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Maximum requests allowed in the window
    pub limit: u64,
    /// Time window in seconds
    pub window_secs: u64,
}

impl RateLimitConfig {
    /// Get rate limit configuration for a tier
    pub fn for_tier(tier: RateLimitTier) -> Self {
        match tier {
            RateLimitTier::HumanStandard => Self {
                limit: 1000,       // 1k requests
                window_secs: 3600, // per hour
            },
            RateLimitTier::AgentHigh => Self {
                limit: 10000,      // 10k requests
                window_secs: 3600, // per hour
            },
            RateLimitTier::ServiceUnlimited => Self {
                limit: u64::MAX, // effectively unlimited
                window_secs: 3600,
            },
            RateLimitTier::CIStandard => Self {
                limit: 5000,       // 5k requests
                window_secs: 3600, // per hour
            },
        }
    }

    /// Check if this tier has unlimited access
    pub fn is_unlimited(&self) -> bool {
        self.limit == u64::MAX
    }
}

/// Rate limit config for IP/key-based limits (admin-configurable, hot-reloadable)
#[derive(Debug, Clone, Copy)]
pub struct IpRateLimitConfig {
    /// Max requests allowed per window
    pub limit: u64,
    /// Window duration in seconds
    pub window_secs: u64,
    /// If false, this limit is skipped entirely
    pub enabled: bool,
}

/// Full set of hot-reloadable IP rate limit configs, populated from the `rate_limits` DB table
#[derive(Debug, Clone)]
pub struct RateLimitLimits {
    pub auth_ip: IpRateLimitConfig,
    pub auth_user: IpRateLimitConfig,
    pub register_ip: IpRateLimitConfig,
    pub password_reset_ip: IpRateLimitConfig,
    pub websocket_ip: IpRateLimitConfig,
}

impl Default for RateLimitLimits {
    fn default() -> Self {
        Self {
            auth_ip:            IpRateLimitConfig { limit: 20, window_secs: 60, enabled: true },
            auth_user:          IpRateLimitConfig { limit: 10, window_secs: 60, enabled: true },
            register_ip:        IpRateLimitConfig { limit: 10, window_secs: 60, enabled: true },
            password_reset_ip:  IpRateLimitConfig { limit:  5, window_secs: 60, enabled: true },
            websocket_ip:       IpRateLimitConfig { limit: 30, window_secs: 60, enabled: true },
        }
    }
}

/// Rate limiting service
///
/// Provides atomic rate limit checks using Redis Lua scripts.
pub struct RateLimitService {
    redis: Pool,
    script: Arc<Script>,
    /// PostgreSQL pool used by reload() to read the rate_limits table
    db: sqlx::PgPool,
    /// Hot-reloadable IP/key rate limit configuration
    ip_limits: Arc<RwLock<RateLimitLimits>>,
}

impl RateLimitService {
    /// Create service with default IP limits (call `reload()` to populate from DB).
    pub fn new(redis: Pool, db: sqlx::PgPool) -> Self {
        Self {
            redis,
            script: Arc::new(Script::new(RATE_LIMIT_SCRIPT)),
            db,
            ip_limits: Arc::new(RwLock::new(RateLimitLimits::default())),
        }
    }

    /// Create service with pre-loaded IP limits.
    pub fn new_with_limits(redis: Pool, db: sqlx::PgPool, initial_limits: RateLimitLimits) -> Self {
        Self {
            redis,
            script: Arc::new(Script::new(RATE_LIMIT_SCRIPT)),
            db,
            ip_limits: Arc::new(RwLock::new(initial_limits)),
        }
    }

    /// Returns a clone of the current IP rate limit configuration.
    pub async fn ip_limits(&self) -> RateLimitLimits {
        self.ip_limits.read().await.clone()
    }

    /// Build the Redis key for rate limiting
    ///
    /// # Arguments
    /// * `entity_id` - The entity UUID
    /// * `tier` - The rate limit tier
    ///
    /// # Returns
    /// Redis key in the format: `ratelimit:{tier}:{entity_id}`
    fn build_key(&self, entity_id: &uuid::Uuid, tier: RateLimitTier) -> String {
        let tier_str = match tier {
            RateLimitTier::HumanStandard => "human_standard",
            RateLimitTier::AgentHigh => "agent_high",
            RateLimitTier::ServiceUnlimited => "service_unlimited",
            RateLimitTier::CIStandard => "ci_standard",
        };
        format!("{}:{}:{}", RATE_LIMIT_KEY_PREFIX, tier_str, entity_id)
    }

    /// Check rate limit for an entity
    pub async fn check_rate_limit(
        &self,
        entity_id: &uuid::Uuid,
        tier: RateLimitTier,
    ) -> ApiResult<()> {
        let config = RateLimitConfig::for_tier(tier);

        if config.is_unlimited() {
            debug!(entity_id = %entity_id, tier = ?tier, "Rate limit bypassed for unlimited tier");
            return Ok(());
        }

        let key = self.build_key(entity_id, tier);

        let mut conn = self.redis.get().await.map_err(|e| {
            warn!(error = %e, entity_id = %entity_id, "Redis connection failed for entity rate limit");
            AppError::Internal(format!("Redis connection error: {}", e))
        })?;

        // Script returns [count, ttl]; take only count for entity-level check
        let result: Vec<u64> = self
            .script
            .key(&key)
            .arg(config.window_secs)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                warn!(error = %e, entity_id = %entity_id, "Redis error during entity rate limit");
                AppError::Redis(e)
            })?;

        let count = result[0];

        if count > config.limit {
            warn!(entity_id = %entity_id, tier = ?tier, count, limit = config.limit, "Entity rate limit exceeded");
            return Err(AppError::RateLimitExceeded {
                message: format!(
                    "Rate limit exceeded: {} requests in {}s window (limit: {})",
                    count, config.window_secs, config.limit
                ),
                retry_after_secs: config.window_secs as i64,
            });
        }

        debug!(entity_id = %entity_id, tier = ?tier, count, limit = config.limit, "Entity rate limit passed");
        Ok(())
    }

    /// Check rate limit for an arbitrary string key.
    /// Fails open on Redis errors to prevent cascading outages.
    pub async fn check_key(&self, key: &str, config: &IpRateLimitConfig) -> ApiResult<()> {
        if !config.enabled {
            return Ok(());
        }

        let redis_key = format!("{}:ip_limit:{}", RATE_LIMIT_KEY_PREFIX, key);

        let mut conn = match self.redis.get().await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, key, "Redis unavailable for IP rate limit; failing open");
                return Ok(());
            }
        };

        let result: Vec<i64> = match self
            .script
            .key(&redis_key)
            .arg(config.window_secs)
            .invoke_async(&mut *conn)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, key, "Redis Lua error during IP rate limit; failing open");
                return Ok(());
            }
        };

        let count = result[0] as u64;
        let ttl = result[1];

        if count > config.limit {
            warn!(key, count, limit = config.limit, ttl, "IP rate limit exceeded");
            return Err(AppError::RateLimitExceeded {
                message: format!("Too many requests. Retry after {}s.", ttl.max(0)),
                retry_after_secs: ttl.max(0),
            });
        }

        debug!(key, count, limit = config.limit, "IP rate limit passed");
        Ok(())
    }

    pub async fn check_auth_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.auth_ip;
        self.check_key(&format!("auth:ip:{}", ip), &config).await
    }

    pub async fn check_auth_user(&self, user_id: uuid::Uuid) -> ApiResult<()> {
        let config = self.ip_limits.read().await.auth_user;
        self.check_key(&format!("auth:user:{}", user_id), &config).await
    }

    pub async fn check_register_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.register_ip;
        self.check_key(&format!("register:ip:{}", ip), &config).await
    }

    pub async fn check_password_reset_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.password_reset_ip;
        self.check_key(&format!("password_reset:ip:{}", ip), &config).await
    }

    pub async fn check_websocket_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.websocket_ip;
        self.check_key(&format!("websocket:ip:{}", ip), &config).await
    }

    /// Hot-reload IP rate limits from the `rate_limits` database table.
    /// Applies a 5-second query timeout to avoid blocking callers.
    pub async fn reload(&self) -> ApiResult<()> {
        #[derive(sqlx::FromRow)]
        struct Row {
            key: String,
            limit_value: i32,
            window_secs: i32,
        }

        let rows: Vec<Row> = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            sqlx::query_as::<_, Row>(
                "SELECT key, limit_value, window_secs FROM rate_limits"
            )
            .fetch_all(&self.db),
        )
        .await
        .map_err(|_| AppError::Internal("Rate limit DB reload timed out".into()))?
        .map_err(|e| AppError::Internal(format!("Rate limit DB reload failed: {}", e)))?;

        let mut limits = RateLimitLimits::default();

        // First pass: apply limit rows (window_secs > 0)
        for row in &rows {
            if row.window_secs > 0 {
                let limit = row.limit_value as u64;
                let window_secs = row.window_secs as u64;
                match row.key.as_str() {
                    "auth_ip_per_minute" =>
                        limits.auth_ip = IpRateLimitConfig { limit, window_secs, ..limits.auth_ip },
                    "auth_user_per_minute" =>
                        limits.auth_user = IpRateLimitConfig { limit, window_secs, ..limits.auth_user },
                    "register_ip_per_minute" =>
                        limits.register_ip = IpRateLimitConfig { limit, window_secs, ..limits.register_ip },
                    "password_reset_ip_per_minute" =>
                        limits.password_reset_ip = IpRateLimitConfig { limit, window_secs, ..limits.password_reset_ip },
                    "websocket_ip_per_minute" =>
                        limits.websocket_ip = IpRateLimitConfig { limit, window_secs, ..limits.websocket_ip },
                    _ => {}
                }
            }
        }

        // Second pass: apply enabled flag rows (window_secs == 0)
        for row in &rows {
            if row.window_secs == 0 {
                let enabled = row.limit_value != 0;
                match row.key.as_str() {
                    "auth_ip_enabled"           => limits.auth_ip.enabled = enabled,
                    "auth_user_enabled"         => limits.auth_user.enabled = enabled,
                    "register_ip_enabled"       => limits.register_ip.enabled = enabled,
                    "password_reset_ip_enabled" => limits.password_reset_ip.enabled = enabled,
                    "websocket_ip_enabled"      => limits.websocket_ip.enabled = enabled,
                    _ => {}
                }
            }
        }

        *self.ip_limits.write().await = limits;
        tracing::info!("IP rate limits reloaded from database");
        Ok(())
    }

    /// Get current rate limit status for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity UUID to check
    /// * `tier` - The rate limit tier
    ///
    /// # Returns
    /// `RateLimitStatus` containing current count and limit information
    pub async fn get_rate_limit_status(
        &self,
        entity_id: &uuid::Uuid,
        tier: RateLimitTier,
    ) -> ApiResult<RateLimitStatus> {
        let config = RateLimitConfig::for_tier(tier);

        if config.is_unlimited() {
            return Ok(RateLimitStatus {
                current_count: 0,
                limit: u64::MAX,
                window_secs: config.window_secs,
                remaining: u64::MAX,
                reset_at: None,
            });
        }

        let key = self.build_key(entity_id, tier);

        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;

        // Get current count and TTL
        let count: Option<u64> = conn.get(&key).await?;
        let ttl: i64 = conn.ttl(&key).await?;

        let current_count = count.unwrap_or(0);
        let remaining = config.limit.saturating_sub(current_count);

        let reset_at = if ttl > 0 {
            Some(std::time::SystemTime::now() + Duration::from_secs(ttl as u64))
        } else {
            None
        };

        Ok(RateLimitStatus {
            current_count,
            limit: config.limit,
            window_secs: config.window_secs,
            remaining,
            reset_at,
        })
    }

    /// Reset rate limit for an entity (admin operation)
    ///
    /// # Arguments
    /// * `entity_id` - The entity UUID to reset
    /// * `tier` - The rate limit tier
    pub async fn reset_rate_limit(
        &self,
        entity_id: &uuid::Uuid,
        tier: RateLimitTier,
    ) -> ApiResult<()> {
        let key = self.build_key(entity_id, tier);
        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;
        conn.del::<_, ()>(&key).await?;

        debug!(
            entity_id = %entity_id,
            tier = ?tier,
            "Rate limit reset"
        );

        Ok(())
    }
}

/// Rate limit status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    /// Current request count in the window
    pub current_count: u64,
    /// Maximum allowed requests
    pub limit: u64,
    /// Window size in seconds
    pub window_secs: u64,
    /// Remaining requests in the current window
    pub remaining: u64,
    /// When the rate limit window resets
    pub reset_at: Option<std::time::SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_for_tier() {
        let human = RateLimitConfig::for_tier(RateLimitTier::HumanStandard);
        assert_eq!(human.limit, 1000);
        assert_eq!(human.window_secs, 3600);
        assert!(!human.is_unlimited());

        let agent = RateLimitConfig::for_tier(RateLimitTier::AgentHigh);
        assert_eq!(agent.limit, 10000);
        assert_eq!(agent.window_secs, 3600);
        assert!(!agent.is_unlimited());

        let service = RateLimitConfig::for_tier(RateLimitTier::ServiceUnlimited);
        assert_eq!(service.limit, u64::MAX);
        assert!(service.is_unlimited());

        let ci = RateLimitConfig::for_tier(RateLimitTier::CIStandard);
        assert_eq!(ci.limit, 5000);
        assert_eq!(ci.window_secs, 3600);
        assert!(!ci.is_unlimited());
    }

    #[test]
    fn test_ip_rate_limit_config_defaults() {
        let limits = RateLimitLimits::default();
        assert_eq!(limits.auth_ip.limit, 20);
        assert_eq!(limits.auth_ip.window_secs, 60);
        assert!(limits.auth_ip.enabled);
        assert_eq!(limits.password_reset_ip.limit, 5);
        assert!(limits.password_reset_ip.enabled);
    }

    #[tokio::test]
    async fn test_check_key_disabled_bypasses() {
        // When enabled=false, check_key returns Ok without touching Redis.
        // Port 1 is unreachable — if Redis were contacted it would fail.
        let cfg = deadpool_redis::Config::from_url("redis://127.0.0.1:1");
        let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).expect("pool");
        let db = sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://x").unwrap();
        let svc = RateLimitService::new(pool, db);
        let config = IpRateLimitConfig { limit: 1, window_secs: 60, enabled: false };
        let result = svc.check_key("test_key", &config).await;
        assert!(result.is_ok(), "disabled limit should always pass: {:?}", result);
    }
}
