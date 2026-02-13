use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::UserId;

pub const MIN_USERNAME_LEN: usize = 3;
pub const MAX_USERNAME_LEN: usize = 32;

/// Returns `true` if the username contains only allowed characters
/// (alphanumeric, underscore, hyphen) and is within length bounds.
pub fn is_valid_username(s: &str) -> bool {
    let len = s.len();
    (MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&len)
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub uuid: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub uuid: String,
}

impl User {
    pub fn new(id: UserId, username: String, uuid: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            username,
            uuid,
            created_at,
        }
    }
}

impl NewUser {
    pub fn new(username: String, uuid: String) -> Self {
        Self { username, uuid }
    }

    /// Validates that the username meets length and character constraints.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !is_valid_username(&self.username) {
            return Err(
                "Username must be 3-32 characters and contain only alphanumeric, underscore, or hyphen characters",
            );
        }
        Ok(())
    }
}
