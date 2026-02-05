use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{BagId, BrewId, GearId};
use super::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickNote {
    Good,
    TooFast,
    TooSlow,
    TooHot,
    UnderExtracted,
    OverExtracted,
}

impl QuickNote {
    pub fn label(self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::TooFast => "Too Fast",
            Self::TooSlow => "Too Slow",
            Self::TooHot => "Too Hot",
            Self::UnderExtracted => "Under Extracted",
            Self::OverExtracted => "Over Extracted",
        }
    }

    pub fn form_value(self) -> &'static str {
        match self {
            Self::Good => "good",
            Self::TooFast => "too-fast",
            Self::TooSlow => "too-slow",
            Self::TooHot => "too-hot",
            Self::UnderExtracted => "under-extracted",
            Self::OverExtracted => "over-extracted",
        }
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "good" | "Good" => Some(Self::Good),
            "too-fast" | "Too Fast" => Some(Self::TooFast),
            "too-slow" | "Too Slow" => Some(Self::TooSlow),
            "too-hot" | "Too Hot" => Some(Self::TooHot),
            "under-extracted" | "Under Extracted" => Some(Self::UnderExtracted),
            "over-extracted" | "Over Extracted" => Some(Self::OverExtracted),
            _ => None,
        }
    }

    pub fn is_positive(self) -> bool {
        matches!(self, Self::Good)
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Good,
            Self::TooFast,
            Self::TooSlow,
            Self::TooHot,
            Self::UnderExtracted,
            Self::OverExtracted,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brew {
    pub id: BrewId,
    pub bag_id: BagId,
    pub coffee_weight: f64,
    pub grinder_id: GearId,
    pub grind_setting: f64,
    pub brewer_id: GearId,
    pub filter_paper_id: Option<GearId>,
    pub water_volume: i32,
    pub water_temp: f64,
    pub quick_notes: Vec<QuickNote>,
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
    pub filter_paper_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBrew {
    pub bag_id: BagId,
    pub coffee_weight: f64,
    pub grinder_id: GearId,
    pub grind_setting: f64,
    pub brewer_id: GearId,
    pub filter_paper_id: Option<GearId>,
    pub water_volume: i32,
    pub water_temp: f64,
    pub quick_notes: Vec<QuickNote>,
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
