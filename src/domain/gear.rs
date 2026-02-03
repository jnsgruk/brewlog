use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::GearId;
use super::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GearCategory {
    Grinder,
    Brewer,
    #[serde(rename = "filter_paper")]
    FilterPaper,
}

impl GearCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            GearCategory::Grinder => "grinder",
            GearCategory::Brewer => "brewer",
            GearCategory::FilterPaper => "filter_paper",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            GearCategory::Grinder => "Grinder",
            GearCategory::Brewer => "Brewer",
            GearCategory::FilterPaper => "Filter Paper",
        }
    }
}

impl FromStr for GearCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "grinder" => Ok(GearCategory::Grinder),
            "brewer" => Ok(GearCategory::Brewer),
            "filter_paper" => Ok(GearCategory::FilterPaper),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gear {
    pub id: GearId,
    pub category: GearCategory,
    pub make: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGear {
    pub category: GearCategory,
    pub make: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGear {
    pub make: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct GearFilter {
    pub category: Option<GearCategory>,
}

impl GearFilter {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn for_category(category: GearCategory) -> Self {
        Self {
            category: Some(category),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GearSortKey {
    Make,
    Model,
    Category,
    CreatedAt,
}

impl SortKey for GearSortKey {
    fn default() -> Self {
        GearSortKey::Make
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "make" => Some(GearSortKey::Make),
            "model" => Some(GearSortKey::Model),
            "category" => Some(GearSortKey::Category),
            "created-at" => Some(GearSortKey::CreatedAt),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            GearSortKey::Make => "make",
            GearSortKey::Model => "model",
            GearSortKey::Category => "category",
            GearSortKey::CreatedAt => "created-at",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            GearSortKey::Make | GearSortKey::Model | GearSortKey::Category => SortDirection::Asc,
            GearSortKey::CreatedAt => SortDirection::Desc,
        }
    }
}
