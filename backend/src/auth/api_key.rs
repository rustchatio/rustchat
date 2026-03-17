//! API key generation and validation for non-human entities
//!
//! This module provides secure API key generation and bcrypt-based hashing for
//! authentication of agents, services, and CI/CD systems. API keys are 32-byte
//! random hex strings (64 characters) hashed with bcrypt cost factor 12.
//!
//! # Security Features
//!
//! - Cryptographically secure random generation using `rand::thread_rng()`
//! - Bcrypt hashing with cost factor 12 for defense against brute force
//! - Constant-time validation to resist timing attacks
//! - Async-safe operations using `tokio::task::spawn_blocking` for CPU-intensive work
//!
//! # Example
//!
//! ```no_run
//! use rustchat::auth::api_key::{generate_api_key, hash_api_key, validate_api_key};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Generate new API key for a service account
//!     let api_key = generate_api_key();
//!     println!("Generated API key: {}", api_key);
//!
//!     // Hash it for storage in database
//!     let hash = hash_api_key(&api_key).await?;
//!     println!("Hash for storage: {}", hash);
//!
//!     // Later, validate an incoming API key
//!     let is_valid = validate_api_key(&api_key, &hash).await?;
//!     assert!(is_valid);
//!
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use rand::Rng;

/// Generate a new API key as a 32-byte random hex string (64 characters)
///
/// This function generates cryptographically secure random bytes using
/// `rand::thread_rng()` and encodes them as lowercase hexadecimal.
///
/// # Returns
///
/// A 64-character hex string representing 32 random bytes
///
/// # Example
///
/// ```no_run
/// use rustchat::auth::api_key::generate_api_key;
///
/// let api_key = generate_api_key();
/// assert_eq!(api_key.len(), 64);
/// ```
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Extract the first 16 characters as the API key prefix
///
/// The prefix consists of:
/// - "rck_" (4 chars)
/// - First 12 hex characters (12 chars)
///
/// # Arguments
/// * `key` - Full API key (68 chars: rck_ + 64 hex)
///
/// # Returns
/// * `Ok(String)` - The 16-character prefix
/// * `Err` - If key format is invalid
pub fn extract_prefix(key: &str) -> Result<String, String> {
    if key.len() != 68 {
        return Err(format!("Invalid key length: expected 68, got {}", key.len()));
    }

    if !key.starts_with("rck_") {
        return Err("Invalid key format: must start with rck_".to_string());
    }

    let prefix = &key[..16];
    Ok(prefix.to_string())
}

/// Hash an API key using bcrypt with cost factor 12
///
/// This function performs CPU-intensive bcrypt hashing in a blocking thread
/// pool to avoid blocking the async runtime. The bcrypt cost factor is set
/// to 12 (4096 iterations), providing strong defense against brute force
/// attacks while maintaining acceptable performance.
///
/// # Arguments
///
/// * `api_key` - The plaintext API key to hash (typically 64 hex characters)
///
/// # Returns
///
/// A bcrypt hash string (60 characters, format: `$2b$12$...`) or an error
///
/// # Errors
///
/// Returns an error if bcrypt hashing fails
///
/// # Example
///
/// ```no_run
/// use rustchat::auth::api_key::{generate_api_key, hash_api_key};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let api_key = generate_api_key();
///     let hash = hash_api_key(&api_key).await?;
///     println!("Hash: {}", hash);
///     Ok(())
/// }
/// ```
pub async fn hash_api_key(api_key: &str) -> Result<String> {
    let api_key = api_key.to_string();

    // Perform CPU-intensive bcrypt hashing in blocking thread pool
    tokio::task::spawn_blocking(move || {
        bcrypt::hash(api_key, bcrypt::DEFAULT_COST).context("Failed to hash API key")
    })
    .await
    .context("Bcrypt hashing task panicked")?
}

/// Validate an API key against its bcrypt hash
///
/// This function performs constant-time comparison using bcrypt's built-in
/// verification to resist timing attacks. The CPU-intensive verification is
/// performed in a blocking thread pool to avoid blocking the async runtime.
///
/// # Arguments
///
/// * `api_key` - The plaintext API key to validate
/// * `hash` - The bcrypt hash to verify against (60 characters)
///
/// # Returns
///
/// `Ok(true)` if the API key matches the hash, `Ok(false)` if it doesn't match,
/// or an error if the hash format is invalid or verification fails
///
/// # Errors
///
/// Returns an error if:
/// - The hash format is invalid
/// - The verification process fails
/// - The blocking task panics
///
/// # Security
///
/// Uses constant-time comparison via bcrypt to prevent timing attacks that
/// could leak information about the hash.
///
/// # Example
///
/// ```no_run
/// use rustchat::auth::api_key::{generate_api_key, hash_api_key, validate_api_key};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let api_key = generate_api_key();
///     let hash = hash_api_key(&api_key).await?;
///
///     let is_valid = validate_api_key(&api_key, &hash).await?;
///     assert!(is_valid);
///
///     let wrong_key = generate_api_key();
///     let is_invalid = validate_api_key(&wrong_key, &hash).await?;
///     assert!(!is_invalid);
///
///     Ok(())
/// }
/// ```
pub async fn validate_api_key(api_key: &str, hash: &str) -> Result<bool> {
    let api_key = api_key.to_string();
    let hash = hash.to_string();

    // Perform CPU-intensive bcrypt verification in blocking thread pool
    tokio::task::spawn_blocking(move || {
        bcrypt::verify(&api_key, &hash).context("Failed to verify API key")
    })
    .await
    .context("Bcrypt verification task panicked")?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_sync() {
        let key = generate_api_key();
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_api_key_uniqueness_sync() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_hash_and_validate() {
        let key = generate_api_key();
        let hash = hash_api_key(&key).await.unwrap();

        assert!(hash.starts_with("$2b$") || hash.starts_with("$2a$") || hash.starts_with("$2y$"));
        assert_eq!(hash.len(), 60);

        let is_valid = validate_api_key(&key, &hash).await.unwrap();
        assert!(is_valid);

        let wrong_key = generate_api_key();
        let is_invalid = validate_api_key(&wrong_key, &hash).await.unwrap();
        assert!(!is_invalid);
    }
}
