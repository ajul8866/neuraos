// neuraos-shield/src/crypto.rs
// Cryptographic utilities for NeuraOS

use crate::{ShieldError, ShieldResult};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use sha2::{Digest, Sha256, Sha512};

pub struct CryptoUtils;

impl CryptoUtils {
    /// SHA-256 hash of input bytes, returned as hex string
    pub fn sha256_hex(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// SHA-512 hash, returned as hex string
    pub fn sha512_hex(data: &[u8]) -> String {
        let mut hasher = Sha512::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Base64-encode bytes
    pub fn b64_encode(data: &[u8]) -> String {
        B64.encode(data)
    }

    /// Base64-decode string
    pub fn b64_decode(input: &str) -> ShieldResult<Vec<u8>> {
        B64.decode(input).map_err(|e| ShieldError::CryptoError(e.to_string()))
    }

    /// Generate a random token of `n` bytes, hex-encoded
    pub fn random_token(n: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        // Deterministic-enough for non-security-critical tokens
        // In production use `rand` crate instead
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();

        let mut hasher = Sha256::new();
        hasher.update(nanos.to_le_bytes());
        hasher.update(uuid::Uuid::new_v4().as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..n.min(32)])
    }

    /// Constant-time string comparison to prevent timing attacks
    pub fn constant_time_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
    }

    /// Hash a password-like string with a salt
    pub fn hash_with_salt(data: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(b":");
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Mask a secret — show only first/last 4 chars
    pub fn mask_secret(secret: &str) -> String {
        let len = secret.len();
        if len <= 8 {
            return "*".repeat(len);
        }
        format!("{}...{}", &secret[..4], &secret[len - 4..])
    }
}
