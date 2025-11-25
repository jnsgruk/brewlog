use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::RoasterId;
use crate::domain::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roaster {
    pub id: RoasterId,
    pub name: String,
    pub country: String,
    pub city: Option<String>,
    pub homepage: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoaster {
    pub name: String,
    pub country: String,
    pub city: Option<String>,
    pub homepage: Option<String>,
    pub notes: Option<String>,
}

impl NewRoaster {
    pub fn normalize(mut self) -> Self {
        self.name = self.name.trim().to_string();
        self.country = self.country.trim().to_string();
        self.city = normalize_optional_field(self.city);
        self.homepage = normalize_optional_field(self.homepage);
        self.notes = normalize_optional_field(self.notes);
        self
    }
}

fn normalize_optional_field(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateRoaster {
    pub name: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub homepage: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RoasterSortKey {
    CreatedAt,
    Name,
    Country,
    City,
}

impl SortKey for RoasterSortKey {
    fn default() -> Self {
        RoasterSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(RoasterSortKey::CreatedAt),
            "name" => Some(RoasterSortKey::Name),
            "country" => Some(RoasterSortKey::Country),
            "city" => Some(RoasterSortKey::City),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            RoasterSortKey::CreatedAt => "created-at",
            RoasterSortKey::Name => "name",
            RoasterSortKey::Country => "country",
            RoasterSortKey::City => "city",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            RoasterSortKey::CreatedAt => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }
}
