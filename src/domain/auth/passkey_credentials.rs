use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{PasskeyCredentialId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub id: PasskeyCredentialId,
    pub user_id: UserId,
    pub credential_json: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct NewPasskeyCredential {
    pub user_id: UserId,
    pub credential_json: String,
    pub name: String,
}

impl NewPasskeyCredential {
    pub fn new(user_id: UserId, credential_json: String, name: String) -> Self {
        Self {
            user_id,
            credential_json,
            name,
        }
    }
}
