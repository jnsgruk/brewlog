use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{BagId, BrewId, GearId};
use super::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brew {
    pub id: BrewId,
    pub bag_id: BagId,
    pub coffee_weight: f64,
    pub grinder_id: GearId,
    pub grind_setting: f64,
    pub brewer_id: GearId,
    pub water_volume: i32,
    pub water_temp: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewWithDetails {
    #[serde(flatten)]
    pub brew: Brew,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub grinder_name: String,
    pub brewer_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBrew {
    pub bag_id: BagId,
    pub coffee_weight: f64,
    pub grinder_id: GearId,
    pub grind_setting: f64,
    pub brewer_id: GearId,
    pub water_volume: i32,
    pub water_temp: f64,
}

/// Filter criteria for brew queries.
#[derive(Debug, Default, Clone)]
pub struct BrewFilter {
    pub bag_id: Option<BagId>,
}

impl BrewFilter {
    /// No filter - returns all brews.
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter for brews from a specific bag.
    pub fn for_bag(bag_id: BagId) -> Self {
        Self {
            bag_id: Some(bag_id),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BrewSortKey {
    CreatedAt,
    CoffeeWeight,
    WaterVolume,
}

impl SortKey for BrewSortKey {
    fn default() -> Self {
        BrewSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(BrewSortKey::CreatedAt),
            "coffee-weight" => Some(BrewSortKey::CoffeeWeight),
            "water-volume" => Some(BrewSortKey::WaterVolume),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            BrewSortKey::CreatedAt => "created-at",
            BrewSortKey::CoffeeWeight => "coffee-weight",
            BrewSortKey::WaterVolume => "water-volume",
        }
    }

    fn default_direction(self) -> SortDirection {
        SortDirection::Desc
    }
}
