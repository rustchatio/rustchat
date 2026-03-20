//! Entity type and rate limiting enums
//!
//! Defines entity types (human, agent, service, CI) and their associated
//! rate limiting tiers for the rustchat platform.

use serde::{Deserialize, Serialize};
use sqlx;

/// Entity type for users in the system
///
/// Distinguishes between human users, AI agents, services, and CI systems.
/// Maps to database VARCHAR column with CHECK constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Human user (default)
    #[default]
    Human,
    /// AI agent or bot
    Agent,
    /// Service account for system integrations
    Service,
    /// Continuous Integration system
    #[serde(rename = "ci")]
    #[sqlx(rename = "ci")]
    CI,
}

impl EntityType {
    /// Check if this entity type is non-human (requires API key)
    pub fn is_non_human(&self) -> bool {
        !matches!(self, EntityType::Human)
    }

    /// Get the default rate limit tier for this entity type
    pub fn default_rate_limit(&self) -> RateLimitTier {
        match self {
            EntityType::Human => RateLimitTier::HumanStandard,
            EntityType::Agent => RateLimitTier::AgentHigh,
            EntityType::Service => RateLimitTier::ServiceUnlimited,
            EntityType::CI => RateLimitTier::CIStandard,
        }
    }
}

/// Rate limiting tier for entities
///
/// Determines the rate limits applied to different entity types.
/// Maps to database VARCHAR column with CHECK constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RateLimitTier {
    /// Standard limits for human users (60 req/min)
    #[default]
    HumanStandard,
    /// High limits for AI agents (300 req/min)
    AgentHigh,
    /// Unlimited for trusted services
    ServiceUnlimited,
    /// Standard limits for CI systems (100 req/min)
    #[serde(rename = "ci_standard")]
    #[sqlx(rename = "ci_standard")]
    CIStandard,
}
