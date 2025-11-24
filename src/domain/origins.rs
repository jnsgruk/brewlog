use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub id: String,
    pub name: String,
    pub country: String,
    pub region: Option<String>,
    pub elevation_masl: Option<u16>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
