use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{RegistrationTokenId, UserId};

/// Maximum registration token lifetime (7 days). Tokens with `expires_at`
/// beyond this are clamped to `created_at + MAX_TOKEN_DURATION`.
pub const MAX_TOKEN_DURATION: Duration = Duration::days(7);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationToken {
    pub id: RegistrationTokenId,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by_user_id: Option<UserId>,
}

impl RegistrationToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used()
    }
}

#[derive(Debug, Clone)]
pub struct NewRegistrationToken {
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl NewRegistrationToken {
    pub fn new(token_hash: String, created_at: DateTime<Utc>, expires_at: DateTime<Utc>) -> Self {
        let max_expires = created_at + MAX_TOKEN_DURATION;
        Self {
            token_hash,
            created_at,
            expires_at: expires_at.min(max_expires),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_valid() {
        let now = Utc::now();
        let token = RegistrationToken {
            id: RegistrationTokenId::new(1),
            token_hash: "hash".to_string(),
            created_at: now,
            expires_at: now + Duration::hours(1),
            used_at: None,
            used_by_user_id: None,
        };
        assert!(token.is_valid());
    }

    #[test]
    fn token_expired() {
        let now = Utc::now();
        let token = RegistrationToken {
            id: RegistrationTokenId::new(1),
            token_hash: "hash".to_string(),
            created_at: now - Duration::hours(2),
            expires_at: now - Duration::hours(1),
            used_at: None,
            used_by_user_id: None,
        };
        assert!(!token.is_valid());
        assert!(token.is_expired());
    }

    #[test]
    fn token_used() {
        let now = Utc::now();
        let token = RegistrationToken {
            id: RegistrationTokenId::new(1),
            token_hash: "hash".to_string(),
            created_at: now,
            expires_at: now + Duration::hours(1),
            used_at: Some(now),
            used_by_user_id: Some(UserId::new(1)),
        };
        assert!(!token.is_valid());
        assert!(token.is_used());
    }

    #[test]
    fn token_expired_and_used() {
        let now = Utc::now();
        let token = RegistrationToken {
            id: RegistrationTokenId::new(1),
            token_hash: "hash".to_string(),
            created_at: now - Duration::hours(2),
            expires_at: now - Duration::hours(1),
            used_at: Some(now - Duration::minutes(30)),
            used_by_user_id: Some(UserId::new(1)),
        };
        assert!(!token.is_valid());
    }

    #[test]
    fn new_token_clamps_excessive_duration() {
        let now = Utc::now();
        let excessive_expires = now + Duration::days(30);
        let token = NewRegistrationToken::new("hash".to_string(), now, excessive_expires);
        let expected_max = now + MAX_TOKEN_DURATION;
        assert_eq!(token.expires_at, expected_max);
    }
}
