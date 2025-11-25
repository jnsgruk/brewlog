use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type UserId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn new(
        id: UserId,
        username: String,
        password_hash: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            password_hash,
            created_at,
        }
    }
}
