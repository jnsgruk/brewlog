use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{SessionId, UserId};

/// Maximum session duration (30 days). Sessions with `expires_at` beyond this
/// are clamped to `created_at + MAX_SESSION_DURATION`.
pub const MAX_SESSION_DURATION: Duration = Duration::days(30);

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    #[serde(skip_serializing)]
    pub session_token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("session_token_hash", &"<redacted>")
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

impl Session {
    pub fn new(
        id: SessionId,
        user_id: UserId,
        session_token_hash: String,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            session_token_hash,
            created_at,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, Clone)]
pub struct NewSession {
    pub user_id: UserId,
    pub session_token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl NewSession {
    pub fn new(
        user_id: UserId,
        session_token_hash: String,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let max_expires = created_at + MAX_SESSION_DURATION;
        Self {
            user_id,
            session_token_hash,
            created_at,
            expires_at: expires_at.min(max_expires),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_not_expired() {
        let now = Utc::now();
        let session = Session::new(
            SessionId::new(1),
            UserId::new(1),
            "hash".to_string(),
            now,
            now + Duration::hours(1),
        );
        assert!(!session.is_expired());
    }

    #[test]
    fn session_expired() {
        let now = Utc::now();
        let session = Session::new(
            SessionId::new(1),
            UserId::new(1),
            "hash".to_string(),
            now - Duration::hours(2),
            now - Duration::hours(1),
        );
        assert!(session.is_expired());
    }

    #[test]
    fn new_session_clamps_excessive_duration() {
        let now = Utc::now();
        let excessive_expires = now + Duration::days(60);
        let session = NewSession::new(UserId::new(1), "hash".to_string(), now, excessive_expires);
        let expected_max = now + MAX_SESSION_DURATION;
        assert_eq!(session.expires_at, expected_max);
    }
}
