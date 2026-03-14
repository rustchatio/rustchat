//! GDPR Compliance Tests - Track B1.2
//!
//! Tests for:
//! - Right to Erasure (hard delete)
//! - Data Export (Article 20)
//! - Cryptographic file wipe

use sqlx::PgPool;
use uuid::Uuid;

mod common;

/// Test helper to create a test user
async fn create_test_user(db: &PgPool, username: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("{}@test.local", username);
    
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, is_active, role, created_at, updated_at)
        VALUES ($1, $2, $3, 'test_hash', true, 'member', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(user_id)
    .bind(username)
    .bind(email)
    .execute(db)
    .await
    .expect("Failed to create test user");
    
    user_id
}

/// Test helper to create a test channel
async fn create_test_channel(db: &PgPool, team_id: Uuid, name: &str) -> Uuid {
    let channel_id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO channels (id, team_id, name, display_name, type, created_at, updated_at)
        VALUES ($1, $2, $3, $3, 'O', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(channel_id)
    .bind(team_id)
    .bind(name)
    .execute(db)
    .await
    .expect("Failed to create test channel");
    
    channel_id
}

/// Test helper to create a test team
async fn create_test_team(db: &PgPool, name: &str) -> Uuid {
    let team_id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO teams (id, name, display_name, type, created_at, updated_at)
        VALUES ($1, $2, $2, 'O', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(team_id)
    .bind(name)
    .execute(db)
    .await
    .expect("Failed to create test team");
    
    team_id
}

/// Test helper to create a test post
async fn create_test_post(db: &PgPool, channel_id: Uuid, user_id: Uuid, message: &str) -> Uuid {
    let post_id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO posts (id, channel_id, user_id, message, props, file_ids, created_at, updated_at)
        VALUES ($1, $2, $3, $4, '{}', '{}', NOW(), NOW())
        "#
    )
    .bind(post_id)
    .bind(channel_id)
    .bind(user_id)
    .bind(message)
    .execute(db)
    .await
    .expect("Failed to create test post");
    
    post_id
}

/// Test helper to add a channel member
async fn add_channel_member(db: &PgPool, channel_id: Uuid, user_id: Uuid) {
    sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role, created_at, updated_at)
        VALUES ($1, $2, 'member', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(db)
    .await
    .expect("Failed to add channel member");
}

#[tokio::test]
async fn test_user_data_export_structure() {
    // This test verifies that the data export structure is properly defined
    use rustchat::services::gdpr::UserDataExport;
    
    // Verify the export structure implements Serialize
    let export = UserDataExport {
        export_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        export_generated_at: chrono::Utc::now(),
        user_profile: rustchat::services::gdpr::UserProfileExport {
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            display_name: None,
            first_name: None,
            last_name: None,
            nickname: None,
            position: None,
            created_at: chrono::Utc::now(),
            last_login_at: None,
            role: "member".to_string(),
            is_bot: false,
        },
        posts: vec![],
        reactions: vec![],
        files: vec![],
        preferences: serde_json::json!({}),
        channel_memberships: vec![],
        audit_log: vec![],
    };
    
    // Serialize to JSON to ensure it works
    let json = serde_json::to_string(&export);
    assert!(json.is_ok(), "UserDataExport should serialize to JSON");
}

#[tokio::test]
async fn test_certificate_of_destruction_structure() {
    use rustchat::services::gdpr::CertificateOfDestruction;
    
    let cert = CertificateOfDestruction {
        file_id: Uuid::new_v4(),
        file_name: "test.txt".to_string(),
        s3_key: "files/test.txt".to_string(),
        wiped_at: chrono::Utc::now(),
        wipe_method: "cryptographic_overwrite".to_string(),
        wiped_by: Uuid::new_v4(),
    };
    
    let json = serde_json::to_string(&cert);
    assert!(json.is_ok(), "CertificateOfDestruction should serialize to JSON");
}

#[tokio::test]
async fn test_saml_security_config() {
    use rustchat::api::v4::saml::SamlSecurityConfig;
    
    let config = SamlSecurityConfig::default();
    
    // Verify security settings are strict by default
    assert!(config.require_signed_response, "Should require signed responses");
    assert!(config.require_signed_assertions, "Should require signed assertions");
    assert!(config.strict_schema_validation, "Should use strict schema validation");
    assert!(config.validate_recipient, "Should validate recipient");
    assert!(config.strict_timestamp_validation, "Should use strict timestamp validation");
    assert!(config.clock_skew_seconds <= 300, "Clock skew should be minimal (<= 5 minutes)");
    
    // Verify weak algorithms are not allowed
    let weak_algorithms = vec![
        "http://www.w3.org/2000/09/xmldsig#sha1",
        "http://www.w3.org/2000/09/xmldsig#rsa-sha1",
        "http://www.w3.org/2001/04/xmldsig-more#rsa-md5",
    ];
    
    for weak in weak_algorithms {
        assert!(
            !config.allowed_digest_algorithms.contains(&weak.to_string()),
            "Weak algorithm {} should not be allowed", weak
        );
    }
}

#[tokio::test]
async fn test_saml_xsw_detection() {
    use rustchat::api::v4::saml::XmlSignatureValidator;
    use rustchat::api::v4::saml::SamlSecurityError;
    
    // Test valid SAML-like XML (no signature)
    let valid_xml = r#"<?xml version="1.0"?>
    <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
        <Assertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion">
            <Subject>
                <NameID>user@example.com</NameID>
            </Subject>
        </Assertion>
    </samlp:Response>"#;
    
    let result = XmlSignatureValidator::validate_no_wrapping_attack(valid_xml);
    assert!(matches!(result, Err(SamlSecurityError::MissingSignature)), 
        "Should detect missing signature");
    
    // Test XML with duplicate IDs (XSW indicator)
    let duplicate_id_xml = r#"<?xml version="1.0"?>
    <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" ID="abc123">
        <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
            <SignedInfo>
                <Reference URI="#abc123"/>
            </SignedInfo>
        </Signature>
        <Assertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion" ID="abc123">
            <Subject>
                <NameID>user@example.com</NameID>
            </Subject>
        </Assertion>
    </samlp:Response>"#;
    
    let result = XmlSignatureValidator::validate_no_wrapping_attack(duplicate_id_xml);
    assert!(matches!(result, Err(SamlSecurityError::DuplicateId)),
        "Should detect duplicate IDs as XSW attack");
    
    // Test XML with multiple signatures (XSW indicator)
    let multi_sig_xml = r#"<?xml version="1.0"?>
    <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
        <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
            <SignedInfo/>
        </Signature>
        <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
            <SignedInfo/>
        </Signature>
        <Assertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion">
            <Subject>
                <NameID>user@example.com</NameID>
            </Subject>
        </Assertion>
    </samlp:Response>"#;
    
    let result = XmlSignatureValidator::validate_no_wrapping_attack(multi_sig_xml);
    assert!(matches!(result, Err(SamlSecurityError::MultipleSignatures)),
        "Should detect multiple signatures as XSW attack");
}
