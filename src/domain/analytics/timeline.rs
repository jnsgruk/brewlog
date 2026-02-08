use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::TimelineEventId;
use crate::domain::listing::{SortDirection, SortKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEventDetail {
    pub label: String,
    pub value: String,
}

/// Raw brew data for repeating a brew from the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineBrewData {
    pub bag_id: i64,
    pub grinder_id: i64,
    pub brewer_id: i64,
    pub filter_paper_id: Option<i64>,
    pub coffee_weight: f64,
    pub grind_setting: f64,
    pub water_volume: i32,
    pub water_temp: f64,
    pub brew_time: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: TimelineEventId,
    pub entity_type: String,
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
    pub entity_type: String,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TimelineSortKey {
    OccurredAt,
}

impl SortKey for TimelineSortKey {
    fn default() -> Self {
        TimelineSortKey::OccurredAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "occurred-at" => Some(TimelineSortKey::OccurredAt),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            TimelineSortKey::OccurredAt => "occurred-at",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            TimelineSortKey::OccurredAt => SortDirection::Desc,
        }
    }
}
