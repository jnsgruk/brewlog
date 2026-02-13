use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::{BagId, GearId, TimelineEventId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEventDetail {
    pub label: String,
    pub value: String,
}

/// Raw brew data for repeating a brew from the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineBrewData {
    pub bag_id: BagId,
    pub grinder_id: GearId,
    pub brewer_id: GearId,
    pub filter_paper_id: Option<GearId>,
    pub coffee_weight: f64,
    pub grind_setting: f64,
    pub water_volume: i32,
    pub water_temp: f64,
    pub brew_time: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: TimelineEventId,
    pub entity_type: EntityType,
    pub entity_id: i64,
    pub action: String,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    pub details: Vec<TimelineEventDetail>,
    pub tasting_notes: Vec<String>,
    pub slug: Option<String>,
    pub roaster_slug: Option<String>,
    pub brew_data: Option<TimelineBrewData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTimelineEvent {
    pub entity_type: EntityType,
    pub entity_id: i64,
    pub action: String,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    pub details: Vec<TimelineEventDetail>,
    pub tasting_notes: Vec<String>,
    pub slug: Option<String>,
    pub roaster_slug: Option<String>,
    pub brew_data: Option<TimelineBrewData>,
}

define_sort_key!(pub TimelineSortKey {
    #[default]
    OccurredAt("occurred-at", Desc),
});
