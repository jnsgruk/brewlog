use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roast {
    pub id: RoastId,
    pub roaster_id: RoasterId,
    pub name: String,
    pub origin: Option<String>,
    pub region: Option<String>,
    pub producer: Option<String>,
    pub tasting_notes: Vec<String>,
    pub process: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastWithRoaster {
    pub roast: Roast,
    pub roaster_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoast {
    pub roaster_id: RoasterId,
    pub name: String,
    pub origin: String,
    pub region: String,
    pub producer: String,
    pub tasting_notes: Vec<String>,
    pub process: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateRoast {
    pub roaster_id: Option<RoasterId>,
    pub name: Option<String>,
    pub origin: Option<String>,
    pub region: Option<String>,
    pub producer: Option<String>,
    pub tasting_notes: Option<Vec<String>>,
    pub process: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RoastSortKey {
    CreatedAt,
    Name,
    Roaster,
    Origin,
    Producer,
}

impl SortKey for RoastSortKey {
    fn default() -> Self {
        RoastSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(RoastSortKey::CreatedAt),
            "name" => Some(RoastSortKey::Name),
            "roaster" => Some(RoastSortKey::Roaster),
            "origin" => Some(RoastSortKey::Origin),
            "producer" => Some(RoastSortKey::Producer),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            RoastSortKey::CreatedAt => "created-at",
            RoastSortKey::Name => "name",
            RoastSortKey::Roaster => "roaster",
            RoastSortKey::Origin => "origin",
            RoastSortKey::Producer => "producer",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            RoastSortKey::CreatedAt => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }
}
