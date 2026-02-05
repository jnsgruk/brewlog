use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token1 = generate_token().unwrap();
        let token2 = generate_token().unwrap();

        assert_ne!(token1, token2);
        assert!(token1.len() >= 40);
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token_12345";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);

        let hash3 = hash_token("different_token");
        assert_ne!(hash1, hash3);
    }
}
