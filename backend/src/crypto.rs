//! Cryptography utilities
use crate::error::AppError;
use aes::Aes256;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD, Engine};
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use rand::RngCore;
use sha2::{Digest, Sha256};

type Aes256CbcDec = cbc::Decryptor<Aes256>;

const CURRENT_PREFIX: &str = "enc:v1:";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Encrypts plaintext using AES-256-GCM with Argon2-derived keys.
pub fn encrypt(plaintext: &str, key: &str) -> Result<String, AppError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);

    let derived_key = derive_key(key, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| AppError::Internal(format!("Encryption init failed: {}", e)))?;

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|_| AppError::Internal("Encryption failed".to_string()))?;

    let mut payload = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);

    Ok(format!("{}{}", CURRENT_PREFIX, STANDARD.encode(payload)))
}

/// Decrypts a ciphertext string. Supports current AEAD format and legacy CBC data.
pub fn decrypt(ciphertext: &str, key: &str) -> Result<String, AppError> {
    if let Some(encoded) = ciphertext.strip_prefix(CURRENT_PREFIX) {
        return decrypt_current(encoded, key);
    }

    decrypt_legacy_magic_crypt(ciphertext, key)
}

fn derive_key(key_material: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], AppError> {
    let mut out = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(key_material.as_bytes(), salt, &mut out)
        .map_err(|e| AppError::Internal(format!("Key derivation failed: {}", e)))?;
    Ok(out)
}

fn decrypt_current(encoded: &str, key: &str) -> Result<String, AppError> {
    let payload = STANDARD
        .decode(encoded)
        .map_err(|e| AppError::Internal(format!("Decryption failed: invalid encoding ({})", e)))?;

    if payload.len() <= SALT_LEN + NONCE_LEN {
        return Err(AppError::Internal("Decryption failed: malformed ciphertext".to_string()));
    }

    let (salt, rest) = payload.split_at(SALT_LEN);
    let (nonce, encrypted) = rest.split_at(NONCE_LEN);

    let derived_key = derive_key(key, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| AppError::Internal(format!("Decryption init failed: {}", e)))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), encrypted)
        .map_err(|_| AppError::Internal("Decryption failed".to_string()))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))
}

// Compatibility path for secrets written by the previous implementation.
fn decrypt_legacy_magic_crypt(ciphertext: &str, key: &str) -> Result<String, AppError> {
    let encrypted = STANDARD
        .decode(ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: invalid encoding ({})", e)))?;

    let key_hash = Sha256::digest(key.as_bytes());
    let iv = [0u8; 16];
    let mut buf = encrypted;

    let plaintext = Aes256CbcDec::new_from_slices(&key_hash, &iv)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_current_format() {
        let key = "test-key";
        let plaintext = "secret-value";

        let encrypted = encrypt(plaintext, key).expect("encrypt");
        assert!(encrypted.starts_with(CURRENT_PREFIX));

        let decrypted = decrypt(&encrypted, key).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypts_legacy_ciphertext() {
        // Produced by the previous MagicCrypt256-based implementation.
        let key = "legacy-key";
        let ciphertext = "Y3UUxl/8M7j+8cypMuO+mg==";

        let decrypted = decrypt(ciphertext, key).expect("decrypt legacy");
        assert_eq!(decrypted, "hello");
    }
}
