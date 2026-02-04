use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{BagId, RoastId};
use super::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bag {
    pub id: BagId,
    pub roast_id: RoastId,
    pub roast_date: Option<NaiveDate>,
    pub amount: f64,
    pub remaining: f64,
    pub closed: bool,
    pub finished_at: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagWithRoast {
    #[serde(flatten)]
    pub bag: Bag,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBag {
    pub roast_id: RoastId,
    pub roast_date: Option<NaiveDate>,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBag {
    pub remaining: Option<f64>,
    pub closed: Option<bool>,
    pub finished_at: Option<NaiveDate>,
}

/// Filter criteria for bag queries.
#[derive(Debug, Default, Clone)]
pub struct BagFilter {
    pub closed: Option<bool>,
    pub roast_id: Option<RoastId>,
}

impl BagFilter {
    /// No filter - returns all bags.
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter for open (unclosed) bags only.
    pub fn open() -> Self {
        Self {
            closed: Some(false),
            ..Default::default()
        }
    }

    /// Filter for closed bags only.
    pub fn closed() -> Self {
        Self {
            closed: Some(true),
            ..Default::default()
        }
    }

    /// Filter for bags of a specific roast.
    pub fn for_roast(roast_id: RoastId) -> Self {
        Self {
            roast_id: Some(roast_id),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BagSortKey {
    RoastDate,
    CreatedAt,
    UpdatedAt,
    Roaster,
    Roast,
    FinishedAt,
}

impl SortKey for BagSortKey {
    fn default() -> Self {
        BagSortKey::RoastDate
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "roast-date" => Some(BagSortKey::RoastDate),
            "created-at" => Some(BagSortKey::CreatedAt),
            "updated-at" => Some(BagSortKey::UpdatedAt),
            "roaster" => Some(BagSortKey::Roaster),
            "roast" => Some(BagSortKey::Roast),
            "finished-at" => Some(BagSortKey::FinishedAt),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            BagSortKey::RoastDate => "roast-date",
            BagSortKey::CreatedAt => "created-at",
            BagSortKey::UpdatedAt => "updated-at",
            BagSortKey::Roaster => "roaster",
            BagSortKey::Roast => "roast",
            BagSortKey::FinishedAt => "finished-at",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            BagSortKey::Roaster | BagSortKey::Roast => SortDirection::Asc,
            _ => SortDirection::Desc,
        }
    }
}
