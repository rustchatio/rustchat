//! Tests for entity type enums and models
//!
//! Following TDD approach - these tests verify the entity type enum
//! behavior and API key model functionality.

#[cfg(test)]
mod entity_tests {
    use rustchat::models::entity::{EntityType, RateLimitTier};

    #[test]
    fn test_entity_type_variants() {
        // Verify all variants exist
        let _human = EntityType::Human;
        let _agent = EntityType::Agent;
        let _service = EntityType::Service;
        let _ci = EntityType::CI;
    }

    #[test]
    fn test_entity_type_default() {
        // Default should be Human
        let default_entity = EntityType::default();
        assert!(matches!(default_entity, EntityType::Human));
    }

    #[test]
    fn test_entity_type_serialization() {
        // Test JSON serialization matches expected format
        assert_eq!(
            serde_json::to_string(&EntityType::Human).unwrap(),
            r#""human""#
        );
        assert_eq!(
            serde_json::to_string(&EntityType::Agent).unwrap(),
            r#""agent""#
        );
        assert_eq!(
            serde_json::to_string(&EntityType::Service).unwrap(),
            r#""service""#
        );
        assert_eq!(serde_json::to_string(&EntityType::CI).unwrap(), r#""ci""#);
    }

    #[test]
    fn test_entity_type_deserialization() {
        // Test JSON deserialization
        let human: EntityType = serde_json::from_str(r#""human""#).unwrap();
        assert!(matches!(human, EntityType::Human));

        let agent: EntityType = serde_json::from_str(r#""agent""#).unwrap();
        assert!(matches!(agent, EntityType::Agent));

        let service: EntityType = serde_json::from_str(r#""service""#).unwrap();
        assert!(matches!(service, EntityType::Service));

        let ci: EntityType = serde_json::from_str(r#""ci""#).unwrap();
        assert!(matches!(ci, EntityType::CI));
    }

    #[test]
    fn test_rate_limit_tier_variants() {
        // Verify all variants exist
        let _human = RateLimitTier::HumanStandard;
        let _agent = RateLimitTier::AgentHigh;
        let _service = RateLimitTier::ServiceUnlimited;
        let _ci = RateLimitTier::CIStandard;
    }

    #[test]
    fn test_rate_limit_tier_default() {
        // Default should be HumanStandard
        let default_tier = RateLimitTier::default();
        assert!(matches!(default_tier, RateLimitTier::HumanStandard));
    }

    #[test]
    fn test_rate_limit_tier_serialization() {
        // Test JSON serialization matches expected format
        assert_eq!(
            serde_json::to_string(&RateLimitTier::HumanStandard).unwrap(),
            r#""human_standard""#
        );
        assert_eq!(
            serde_json::to_string(&RateLimitTier::AgentHigh).unwrap(),
            r#""agent_high""#
        );
        assert_eq!(
            serde_json::to_string(&RateLimitTier::ServiceUnlimited).unwrap(),
            r#""service_unlimited""#
        );
        assert_eq!(
            serde_json::to_string(&RateLimitTier::CIStandard).unwrap(),
            r#""ci_standard""#
        );
    }

    #[test]
    fn test_rate_limit_tier_deserialization() {
        // Test JSON deserialization
        let human: RateLimitTier = serde_json::from_str(r#""human_standard""#).unwrap();
        assert!(matches!(human, RateLimitTier::HumanStandard));

        let agent: RateLimitTier = serde_json::from_str(r#""agent_high""#).unwrap();
        assert!(matches!(agent, RateLimitTier::AgentHigh));

        let service: RateLimitTier = serde_json::from_str(r#""service_unlimited""#).unwrap();
        assert!(matches!(service, RateLimitTier::ServiceUnlimited));

        let ci: RateLimitTier = serde_json::from_str(r#""ci_standard""#).unwrap();
        assert!(matches!(ci, RateLimitTier::CIStandard));
    }

    #[test]
    fn test_entity_type_is_non_human() {
        // Test the is_non_human helper method
        assert!(!EntityType::Human.is_non_human());
        assert!(EntityType::Agent.is_non_human());
        assert!(EntityType::Service.is_non_human());
        assert!(EntityType::CI.is_non_human());
    }

    #[test]
    fn test_entity_type_default_rate_limit() {
        // Test default rate limit tier for each entity type
        assert_eq!(
            EntityType::Human.default_rate_limit(),
            RateLimitTier::HumanStandard
        );
        assert_eq!(
            EntityType::Agent.default_rate_limit(),
            RateLimitTier::AgentHigh
        );
        assert_eq!(
            EntityType::Service.default_rate_limit(),
            RateLimitTier::ServiceUnlimited
        );
        assert_eq!(
            EntityType::CI.default_rate_limit(),
            RateLimitTier::CIStandard
        );
    }
}

#[cfg(test)]
mod api_key_tests {
    use rustchat::models::api_key::ApiKey;
    use uuid::Uuid;

    #[test]
    fn test_api_key_new() {
        // Test creating a new API key
        let user_id = Uuid::new_v4();
        let api_key = ApiKey::new(user_id, "test_key".to_string());

        assert_eq!(api_key.user_id, user_id);
        assert_eq!(api_key.key_hash, "test_key");
        assert!(api_key.name.is_none());
        assert!(api_key.description.is_none());
        assert!(api_key.expires_at.is_none());
        assert!(api_key.last_used_at.is_none());
        assert!(api_key.is_active);
        assert!(api_key.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_api_key_with_name() {
        // Test builder pattern for setting name
        let user_id = Uuid::new_v4();
        let api_key = ApiKey::new(user_id, "test_key".to_string())
            .with_name("Production API Key".to_string());

        assert_eq!(api_key.name, Some("Production API Key".to_string()));
    }

    #[test]
    fn test_api_key_with_description() {
        // Test builder pattern for setting description
        let user_id = Uuid::new_v4();
        let api_key = ApiKey::new(user_id, "test_key".to_string())
            .with_description("Key for production deployments".to_string());

        assert_eq!(
            api_key.description,
            Some("Key for production deployments".to_string())
        );
    }

    #[test]
    fn test_api_key_is_expired() {
        // Test expiration checking
        let user_id = Uuid::new_v4();

        // Key without expiration should not be expired
        let no_expiry = ApiKey::new(user_id, "test_key".to_string());
        assert!(!no_expiry.is_expired());

        // Key with future expiration should not be expired
        let future_expiry = ApiKey::new(user_id, "test_key".to_string())
            .with_expiry(chrono::Utc::now() + chrono::Duration::days(30));
        assert!(!future_expiry.is_expired());

        // Key with past expiration should be expired
        let past_expiry = ApiKey::new(user_id, "test_key".to_string())
            .with_expiry(chrono::Utc::now() - chrono::Duration::days(1));
        assert!(past_expiry.is_expired());
    }

    #[test]
    fn test_api_key_is_valid() {
        // Test validity checking (active and not expired)
        let user_id = Uuid::new_v4();

        // Active and not expired = valid
        let valid_key = ApiKey::new(user_id, "test_key".to_string());
        assert!(valid_key.is_valid());

        // Inactive key = invalid
        let mut inactive_key = ApiKey::new(user_id, "test_key".to_string());
        inactive_key.is_active = false;
        assert!(!inactive_key.is_valid());

        // Expired key = invalid
        let expired_key = ApiKey::new(user_id, "test_key".to_string())
            .with_expiry(chrono::Utc::now() - chrono::Duration::days(1));
        assert!(!expired_key.is_valid());
    }
}

#[cfg(test)]
mod user_entity_tests {
    use chrono::Utc;
    use rustchat::models::entity::{EntityType, RateLimitTier};
    use rustchat::models::user::{User, UserResponse};
    use uuid::Uuid;

    fn create_test_user(entity_type: EntityType) -> User {
        User {
            id: Uuid::new_v4(),
            org_id: None,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: Some("hash".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            first_name: None,
            last_name: None,
            nickname: None,
            position: None,
            is_bot: false,
            is_active: true,
            role: "member".to_string(),
            presence: "online".to_string(),
            status_text: None,
            status_emoji: None,
            status_expires_at: None,
            custom_status: None,
            notify_props: serde_json::json!({}),
            timezone: None,
            last_login_at: None,
            email_verified: true,
            email_verified_at: Some(Utc::now()),
            deleted_at: None,
            deleted_by: None,
            delete_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            entity_type,
            api_key_hash: None,
            entity_metadata: serde_json::json!({}),
            rate_limit_tier: None,
        }
    }

    #[test]
    fn test_user_is_non_human() {
        // Human user
        let human_user = create_test_user(EntityType::Human);
        assert!(!human_user.is_non_human());

        // Agent user
        let agent_user = create_test_user(EntityType::Agent);
        assert!(agent_user.is_non_human());

        // Service user
        let service_user = create_test_user(EntityType::Service);
        assert!(service_user.is_non_human());

        // CI user
        let ci_user = create_test_user(EntityType::CI);
        assert!(ci_user.is_non_human());
    }

    #[test]
    fn test_user_requires_api_key() {
        // Human user should not require API key
        let human_user = create_test_user(EntityType::Human);
        assert!(!human_user.requires_api_key());

        // Non-human users should require API key
        let agent_user = create_test_user(EntityType::Agent);
        assert!(agent_user.requires_api_key());
    }

    #[test]
    fn test_user_effective_rate_limit_tier() {
        // Test with explicit rate limit tier set
        let mut user = create_test_user(EntityType::Human);
        user.rate_limit_tier = Some(RateLimitTier::AgentHigh);
        assert_eq!(user.effective_rate_limit_tier(), RateLimitTier::AgentHigh);

        // Test with no explicit tier (falls back to entity type default)
        let human_user = create_test_user(EntityType::Human);
        assert_eq!(
            human_user.effective_rate_limit_tier(),
            RateLimitTier::HumanStandard
        );

        let agent_user = create_test_user(EntityType::Agent);
        assert_eq!(
            agent_user.effective_rate_limit_tier(),
            RateLimitTier::AgentHigh
        );

        let service_user = create_test_user(EntityType::Service);
        assert_eq!(
            service_user.effective_rate_limit_tier(),
            RateLimitTier::ServiceUnlimited
        );

        let ci_user = create_test_user(EntityType::CI);
        assert_eq!(
            ci_user.effective_rate_limit_tier(),
            RateLimitTier::CIStandard
        );
    }

    #[test]
    fn test_user_response_does_not_include_entity_type() {
        // Test that UserResponse does NOT include entity_type for backward compatibility
        let user = create_test_user(EntityType::Agent);
        let response: UserResponse = user.into();

        // Serialize to JSON and verify entity_type is not present
        let json = serde_json::to_value(&response).unwrap();
        assert!(
            !json.as_object().unwrap().contains_key("entity_type"),
            "UserResponse should not include entity_type field for mobile compatibility"
        );
        assert!(
            !json.as_object().unwrap().contains_key("api_key_hash"),
            "UserResponse should not include api_key_hash (sensitive field)"
        );
        assert!(
            !json.as_object().unwrap().contains_key("rate_limit_tier"),
            "UserResponse should not include rate_limit_tier (internal field)"
        );
    }

    #[test]
    fn test_user_serialization_skips_api_key_hash() {
        // Test that User serialization skips api_key_hash
        let mut user = create_test_user(EntityType::Agent);
        user.api_key_hash = Some("secret_hash".to_string());

        let json = serde_json::to_value(&user).unwrap();
        assert!(
            !json.as_object().unwrap().contains_key("api_key_hash"),
            "User serialization should skip api_key_hash field"
        );
    }
}
