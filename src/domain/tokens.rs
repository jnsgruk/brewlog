use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{TokenId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: TokenId,
    pub user_id: UserId,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct NewToken {
    pub user_id: UserId,
    pub token_hash: String,
    pub name: String,
}

impl Token {
    pub fn new(
        id: TokenId,
        user_id: UserId,
        token_hash: String,
        name: String,
        created_at: DateTime<Utc>,
        last_used_at: Option<DateTime<Utc>>,
        revoked_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            user_id,
            token_hash,
            name,
            created_at,
            last_used_at,
            revoked_at,
        }
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_active(&self) -> bool {
        !self.is_revoked()
    }
}

impl NewToken {
    pub fn new(user_id: UserId, token_hash: String, name: String) -> Self {
        Self {
            user_id,
            token_hash,
            name,
        }
    }
}
