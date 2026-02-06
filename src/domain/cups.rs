use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{CafeId, CupId, RoastId};
use super::listing::{SortDirection, SortKey};
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cup {
    pub id: CupId,
    pub roast_id: RoastId,
    pub cafe_id: CafeId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CupWithDetails {
    #[serde(flatten)]
    pub cup: Cup,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub cafe_name: String,
    pub cafe_slug: String,
    pub cafe_city: String,
}

impl CupWithDetails {
    pub fn to_timeline_event(&self) -> NewTimelineEvent {
        NewTimelineEvent {
            entity_type: "cup".to_string(),
            entity_id: self.cup.id.into_inner(),
            action: "added".to_string(),
            occurred_at: Utc::now(),
            title: self.roast_name.clone(),
            details: vec![
                TimelineEventDetail {
                    label: "Coffee".to_string(),
                    value: self.roast_name.clone(),
                },
                TimelineEventDetail {
                    label: "Roaster".to_string(),
                    value: self.roaster_name.clone(),
                },
                TimelineEventDetail {
                    label: "Cafe".to_string(),
                    value: self.cafe_name.clone(),
                },
            ],
            tasting_notes: vec![],
            slug: Some(self.roast_slug.clone()),
            roaster_slug: Some(self.roaster_slug.clone()),
            brew_data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCup {
    pub roast_id: RoastId,
    pub cafe_id: CafeId,
}

/// Filter criteria for cup queries.
#[derive(Debug, Default, Clone)]
pub struct CupFilter {
    pub cafe_id: Option<CafeId>,
    pub roast_id: Option<RoastId>,
}

impl CupFilter {
    /// No filter - returns all cups.
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter for cups at a specific cafe.
    pub fn for_cafe(cafe_id: CafeId) -> Self {
        Self {
            cafe_id: Some(cafe_id),
            ..Self::default()
        }
    }

    /// Filter for cups of a specific roast.
    pub fn for_roast(roast_id: RoastId) -> Self {
        Self {
            roast_id: Some(roast_id),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CupSortKey {
    CreatedAt,
    CafeName,
    CafeCity,
    RoastName,
    RoasterName,
}

impl SortKey for CupSortKey {
    fn default() -> Self {
        CupSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(CupSortKey::CreatedAt),
            "cafe" => Some(CupSortKey::CafeName),
            "city" => Some(CupSortKey::CafeCity),
            "roast" => Some(CupSortKey::RoastName),
            "roaster" => Some(CupSortKey::RoasterName),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            CupSortKey::CreatedAt => "created-at",
            CupSortKey::CafeName => "cafe",
            CupSortKey::CafeCity => "city",
            CupSortKey::RoastName => "roast",
            CupSortKey::RoasterName => "roaster",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            CupSortKey::CreatedAt => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }
}
