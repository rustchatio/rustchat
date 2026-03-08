//! Security configuration and validation
//!
//! Enforces security policies for secrets, tokens, and deployment settings.

use crate::config::Config;

/// Minimum entropy requirements for secrets (in bits)
pub const MIN_JWT_SECRET_ENTROPY_BITS: usize = 128;
pub const MIN_ENCRYPTION_KEY_ENTROPY_BITS: usize = 128;

/// Minimum length requirements for secrets
pub const MIN_JWT_SECRET_LENGTH: usize = 32;
pub const MIN_ENCRYPTION_KEY_LENGTH: usize = 32;

/// Validation result for secrets
#[derive(Debug, Clone)]
pub struct SecretValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl SecretValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
        self.is_valid = false;
    }

    fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }
}

/// Validate that secrets meet security requirements
///
/// In production, this will fail if secrets are weak.
/// In development, it will log warnings but allow weak secrets.
pub fn validate_secrets(config: &Config) -> SecretValidationResult {
    let mut result = SecretValidationResult::new();

    // Validate JWT secret
    validate_secret_entropy(
        &config.jwt_secret,
        "JWT_SECRET",
        MIN_JWT_SECRET_LENGTH,
        MIN_JWT_SECRET_ENTROPY_BITS,
        &mut result,
    );

    // Validate encryption key
    validate_secret_entropy(
        &config.encryption_key,
        "ENCRYPTION_KEY",
        MIN_ENCRYPTION_KEY_LENGTH,
        MIN_ENCRYPTION_KEY_ENTROPY_BITS,
        &mut result,
    );

    // Check for common weak/default values
    check_common_weak_secrets(&config.jwt_secret, "JWT_SECRET", &mut result);
    check_common_weak_secrets(&config.encryption_key, "ENCRYPTION_KEY", &mut result);

    // Validate that JWT secret and encryption key are different
    if config.jwt_secret == config.encryption_key {
        result.add_error("JWT_SECRET and ENCRYPTION_KEY must be different values".to_string());
    }

    result
}

/// Calculate Shannon entropy of a string in bits
fn calculate_entropy(input: &str) -> f64 {
    if input.is_empty() {
        return 0.0;
    }

    let mut char_counts = std::collections::HashMap::new();
    for c in input.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let len = input.len() as f64;
    char_counts.values().fold(0.0, |acc, &count| {
        let probability = count as f64 / len;
        acc - (probability * probability.log2())
    })
}

/// Validate a secret's length and entropy
fn validate_secret_entropy(
    secret: &str,
    name: &str,
    min_length: usize,
    min_entropy_bits: usize,
    result: &mut SecretValidationResult,
) {
    // Check length
    if secret.len() < min_length {
        result.add_error(format!(
            "{} is too short: {} characters (minimum {} required)",
            name,
            secret.len(),
            min_length
        ));
    }

    // Check entropy
    let entropy = calculate_entropy(secret);
    let entropy_bits = entropy * secret.len() as f64;

    if entropy_bits < min_entropy_bits as f64 {
        // Check if it looks like a base64 or hex encoded value (which may have lower entropy)
        let is_encoded = looks_like_encoded(secret);

        if is_encoded {
            result.add_warning(format!(
                "{} may have low entropy: {:.1} bits (encoded values should be at least {} characters)",
                name,
                entropy_bits,
                min_entropy_bits / 4 // Rough estimate for hex
            ));
        } else {
            result.add_error(format!(
                "{} has insufficient entropy: {:.1} bits (minimum {} required). \
                 Use a cryptographically secure random generator.",
                name, entropy_bits, min_entropy_bits
            ));
        }
    }

    // Check for repeated characters (low entropy indicator)
    let unique_chars: std::collections::HashSet<char> = secret.chars().collect();
    let uniqueness_ratio = unique_chars.len() as f64 / secret.len() as f64;

    if uniqueness_ratio < 0.5 && secret.len() > 16 {
        result.add_warning(format!(
            "{} has low character diversity ({:.0}% unique). Consider using more varied characters.",
            name,
            uniqueness_ratio * 100.0
        ));
    }
}

/// Check if a string looks like base64 or hex encoded
fn looks_like_encoded(s: &str) -> bool {
    // Check for hex (only hex chars and even length)
    let is_hex = s.len().is_multiple_of(2) && s.chars().all(|c| c.is_ascii_hexdigit());

    // Check for base64 (alphanumeric + / + = padding)
    let is_base64 = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '+' || c == '=')
        && s.len().is_multiple_of(4);

    is_hex || is_base64
}

/// Check for common weak/default secret values
fn check_common_weak_secrets(secret: &str, name: &str, result: &mut SecretValidationResult) {
    let weak_patterns = [
        "secret",
        "password",
        "admin",
        "123",
        "default",
        "changeme",
        "test",
        "dev",
        "local",
        "localhost",
    ];

    let lower_secret = secret.to_lowercase();

    for pattern in &weak_patterns {
        if lower_secret.contains(pattern) {
            result.add_error(format!(
                "{} contains weak pattern '{}' - this is a security risk",
                name, pattern
            ));
            break; // Only report once per secret
        }
    }

    // Check for sequential characters
    if has_sequential_chars(secret) {
        result.add_warning(format!(
            "{} contains sequential characters (e.g., 'abc', '123')",
            name
        ));
    }
}

/// Check for sequential characters (common in weak passwords)
fn has_sequential_chars(s: &str) -> bool {
    let bytes = s.as_bytes();
    for window in bytes.windows(3) {
        // Check for ascending sequences like "abc", "123"
        if window[1] == window[0] + 1 && window[2] == window[1] + 1 {
            return true;
        }
        // Check for descending sequences like "cba", "321"
        if window[1] == window[0] - 1 && window[2] == window[1] - 1 {
            return true;
        }
        // Check for repeated sequences like "aaa", "111"
        if window[0] == window[1] && window[1] == window[2] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        // "aaaa" has very low entropy
        let low_entropy = calculate_entropy("aaaa");
        assert!(low_entropy < 1.0);

        // Mixed characters have higher entropy
        let high_entropy = calculate_entropy("abcdefgh");
        assert!(high_entropy > low_entropy);
    }

    #[test]
    fn test_looks_like_encoded() {
        assert!(looks_like_encoded("deadbeef")); // hex
        assert!(looks_like_encoded("dGVzdA==")); // base64
        assert!(!looks_like_encoded("hello world"));
    }

    #[test]
    fn test_has_sequential_chars() {
        assert!(has_sequential_chars("abc123"));
        assert!(has_sequential_chars("321cba"));
        assert!(has_sequential_chars("aaa"));
        assert!(!has_sequential_chars("random"));
    }
}
