use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{RegistrationTokenId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationToken {
    pub id: RegistrationTokenId,
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
        Self {
            token_hash,
            created_at,
            expires_at,
        }
    }
}
