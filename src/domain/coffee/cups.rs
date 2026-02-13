use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::{CafeId, CupId, RoastId};
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
            entity_type: EntityType::Cup,
            entity_id: self.cup.id.into_inner(),
            action: "added".to_string(),
            occurred_at: self.cup.created_at,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateCup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roast_id: Option<RoastId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cafe_id: Option<CafeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
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

define_sort_key!(pub CupSortKey {
    #[default]
    CreatedAt("created-at", Desc),
    CafeName("cafe", Asc),
    CafeCity("city", Asc),
    RoastName("roast", Asc),
    RoasterName("roaster", Asc),
});
