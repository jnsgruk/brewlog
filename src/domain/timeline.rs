use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEventDetail {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    pub details: Vec<TimelineEventDetail>,
    pub tasting_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTimelineEvent {
    pub entity_type: String,
    pub entity_id: String,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    pub details: Vec<TimelineEventDetail>,
    pub tasting_notes: Vec<String>,
}
