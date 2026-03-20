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
use tracing::{debug, warn};

/// Redis key prefix for rate limiting
const RATE_LIMIT_KEY_PREFIX: &str = "ratelimit";

/// Lua script for atomic INCR+EXPIRE operation
///
/// This script atomically increments a counter and sets expiry if it's the first increment.
/// Returns the new count value.
///
/// KEYS[1]: The rate limit key
/// ARGV[1]: The rate limit threshold
/// ARGV[2]: The TTL in seconds
const RATE_LIMIT_SCRIPT: &str = r#"
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local ttl = tonumber(ARGV[2])
local count = redis.call('INCR', key)
if count == 1 then
    redis.call('EXPIRE', key, ttl)
end
return count
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

/// Rate limiting service
///
/// Provides atomic rate limit checks using Redis Lua scripts.
pub struct RateLimitService {
    redis: Pool,
    script: Arc<Script>,
}

impl RateLimitService {
    /// Create a new rate limiting service
    ///
    /// # Arguments
    /// * `redis` - Redis connection pool
    pub fn new(redis: Pool) -> Self {
        let script = Arc::new(Script::new(RATE_LIMIT_SCRIPT));
        Self { redis, script }
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
    ///
    /// # Arguments
    /// * `entity_id` - The entity UUID to check
    /// * `tier` - The rate limit tier to apply
    ///
    /// # Returns
    /// * `Ok(())` - Request is allowed
    /// * `Err(AppError::RateLimitExceeded)` - Rate limit exceeded
    ///
    /// # Errors
    /// * Redis connection errors
    /// * Rate limit exceeded errors
    pub async fn check_rate_limit(
        &self,
        entity_id: &uuid::Uuid,
        tier: RateLimitTier,
    ) -> ApiResult<()> {
        let config = RateLimitConfig::for_tier(tier);

        // ServiceUnlimited tier bypasses rate limiting
        if config.is_unlimited() {
            debug!(
                entity_id = %entity_id,
                tier = ?tier,
                "Rate limit bypassed for unlimited tier"
            );
            return Ok(());
        }

        let key = self.build_key(entity_id, tier);

        // Execute atomic INCR+EXPIRE via Lua script
        let mut conn = self.redis.get().await.map_err(|e| {
            warn!(
                error = %e,
                entity_id = %entity_id,
                tier = ?tier,
                "Failed to get Redis connection"
            );
            AppError::Internal(format!("Redis connection error: {}", e))
        })?;

        let count: u64 = self
            .script
            .key(&key)
            .arg(config.limit)
            .arg(config.window_secs)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                warn!(
                    error = %e,
                    entity_id = %entity_id,
                    tier = ?tier,
                    "Redis error during rate limit check"
                );
                AppError::Redis(e)
            })?;

        if count > config.limit {
            warn!(
                entity_id = %entity_id,
                tier = ?tier,
                count = count,
                limit = config.limit,
                "Rate limit exceeded"
            );
            return Err(AppError::RateLimitExceeded(format!(
                "Rate limit exceeded: {} requests in {}s window (limit: {})",
                count, config.window_secs, config.limit
            )));
        }

        debug!(
            entity_id = %entity_id,
            tier = ?tier,
            count = count,
            limit = config.limit,
            "Rate limit check passed"
        );

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
}
