use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use base64::{Engine as _, engine::general_purpose};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

/// Hashes a password using Argon2id with secure defaults
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {e}"))?
        .to_string();

    Ok(password_hash)
}

/// Verifies a password against a hash
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| anyhow::anyhow!("failed to parse password hash: {e}"))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generates a cryptographically secure random token
/// Returns a base64-encoded token string
pub fn generate_token() -> Result<String> {
    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    Ok(general_purpose::STANDARD.encode(token_bytes))
}

/// Hashes a token for storage using SHA-256
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

/// Generates a session token for cookie-based authentication
pub fn generate_session_token() -> String {
    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(token_bytes)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests: unwrap is acceptable for test assertions
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_password_hashing_different_salts() {
        let password = "test_password_123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify the same password
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_token_generation() {
        let token1 = generate_token().unwrap();
        let token2 = generate_token().unwrap();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Tokens should be base64 encoded (at least 40 chars for 32 bytes)
        assert!(token1.len() >= 40);
        assert!(token2.len() >= 40);
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token_12345";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);

        // Different token should produce different hash
        let different_token = "different_token";
        let hash3 = hash_token(different_token);
        assert_ne!(hash1, hash3);
    }
}
