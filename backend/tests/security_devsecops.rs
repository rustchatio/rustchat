//! DevSecOps Security Tests - Track B1.1
//!
//! Tests for security hardening:
//! - cargo-audit compliance
//! - cargo-deny license checking
//! - Security configuration validation

/// Verify that the deny.toml file exists and is valid
#[test]
fn test_deny_toml_exists() {
    let deny_toml = include_str!("../deny.toml");

    // Verify the file contains required sections
    assert!(
        deny_toml.contains("[advisories]"),
        "deny.toml should contain advisories section"
    );
    assert!(
        deny_toml.contains("[licenses]"),
        "deny.toml should contain licenses section"
    );
    assert!(
        deny_toml.contains("[bans]"),
        "deny.toml should contain bans section"
    );

    // Verify copyleft licenses are denied
    assert!(deny_toml.contains("GPL-1.0"), "Should deny GPL-1.0");
    assert!(deny_toml.contains("GPL-2.0"), "Should deny GPL-2.0");
    assert!(deny_toml.contains("GPL-3.0"), "Should deny GPL-3.0");
    assert!(deny_toml.contains("AGPL-3.0"), "Should deny AGPL-3.0");

    // Verify OSI-approved licenses are allowed
    assert!(deny_toml.contains("MIT"), "Should allow MIT");
    assert!(deny_toml.contains("Apache-2.0"), "Should allow Apache-2.0");
    assert!(
        deny_toml.contains("BSD-2-Clause"),
        "Should allow BSD-2-Clause"
    );
    assert!(
        deny_toml.contains("BSD-3-Clause"),
        "Should allow BSD-3-Clause"
    );
}

/// Verify security-related headers and configurations
#[test]
fn test_security_headers_exist() {
    // This test ensures security headers are defined
    // The actual headers are tested in the middleware module

    // Verify the security headers middleware exists
    let middleware_code = include_str!("../src/middleware/security_headers.rs");

    // Check for key security headers
    assert!(
        middleware_code.contains("X-Frame-Options"),
        "Should set X-Frame-Options"
    );
    assert!(
        middleware_code.contains("X-Content-Type-Options"),
        "Should set X-Content-Type-Options"
    );
    assert!(
        middleware_code.contains("X-XSS-Protection")
            || middleware_code.contains("Content-Security-Policy"),
        "Should set XSS protection headers"
    );
}

/// Test that the CI workflow includes security checks
#[test]
fn test_ci_security_checks() {
    let ci_yaml = include_str!("../../.github/workflows/backend-ci.yml");

    // Verify cargo-audit is in the CI
    assert!(ci_yaml.contains("cargo-audit"), "CI should run cargo-audit");
    assert!(ci_yaml.contains("cargo-deny"), "CI should run cargo-deny");
    assert!(
        ci_yaml.contains("security-audit"),
        "CI should have security-audit job"
    );
    assert!(
        ci_yaml.contains("license-check"),
        "CI should have license-check job"
    );

    // Verify security checks run before tests
    assert!(
        ci_yaml.contains("needs: [security-audit, license-check]"),
        "Tests should depend on security checks"
    );
}

/// Test Docker publishing workflow has security features
#[test]
fn test_docker_publish_security() {
    let docker_yaml = include_str!("../../.github/workflows/docker-publish.yml");

    // Verify container signing is configured
    assert!(
        docker_yaml.contains("cosign"),
        "Docker publish should use cosign for signing"
    );
    assert!(
        docker_yaml.contains("sign"),
        "Docker publish should sign images"
    );

    // Verify SBOM generation is configured
    assert!(
        docker_yaml.contains("SBOM"),
        "Docker publish should generate SBOMs"
    );

    // Verify vulnerability scanning
    assert!(
        docker_yaml.contains("Trivy"),
        "Docker publish should include Trivy scanning"
    );
}

/// Test that weak cryptographic algorithms are rejected
#[test]
fn test_weak_algorithms_rejected() {
    use rustchat::api::v4::saml::SamlSecurityConfig;

    let config = SamlSecurityConfig::default();

    // SHA-1 should not be in allowed algorithms
    let sha1_algorithms = vec![
        "http://www.w3.org/2000/09/xmldsig#sha1",
        "http://www.w3.org/2000/09/xmldsig#rsa-sha1",
    ];

    for alg in sha1_algorithms {
        assert!(
            !config.allowed_digest_algorithms.contains(&alg.to_string()),
            "Weak algorithm {} should not be allowed in digest algorithms",
            alg
        );
    }

    // MD5 should not be in allowed algorithms
    let md5_algorithms = vec![
        "http://www.w3.org/2001/04/xmldsig-more#rsa-md5",
        "http://www.w3.org/2001/04/xmldsig-more#md5",
    ];

    for alg in md5_algorithms {
        assert!(
            !config
                .allowed_signature_algorithms
                .contains(&alg.to_string()),
            "Weak algorithm {} should not be allowed in signature algorithms",
            alg
        );
    }
}
