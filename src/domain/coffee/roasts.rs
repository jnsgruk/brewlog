use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::roasters::Roaster;
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roast {
    pub id: RoastId,
    pub roaster_id: RoasterId,
    pub name: String,
    pub slug: String,
    pub origin: Option<String>,
    pub region: Option<String>,
    pub producer: Option<String>,
    pub tasting_notes: Vec<String>,
    pub process: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastWithRoaster {
    #[serde(flatten)]
    pub roast: Roast,
    pub roaster_name: String,
    pub roaster_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoast {
    pub roaster_id: RoasterId,
    pub name: String,
    pub origin: String,
    pub region: String,
    pub producer: String,
    pub tasting_notes: Vec<String>,
    pub process: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl NewRoast {
    pub fn slug(&self) -> String {
        slug::slugify(&self.name)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateRoast {
    pub roaster_id: Option<RoasterId>,
    pub name: Option<String>,
    pub origin: Option<String>,
    pub region: Option<String>,
    pub producer: Option<String>,
    pub tasting_notes: Option<Vec<String>>,
    pub process: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

define_sort_key!(pub RoastSortKey {
    #[default]
    CreatedAt("created-at", Desc),
    Name("name", Asc),
    Roaster("roaster", Asc),
    Origin("origin", Asc),
    Producer("producer", Asc),
});

pub fn roast_timeline_event(roast: &Roast, roaster: &Roaster) -> NewTimelineEvent {
    let mut details = vec![TimelineEventDetail {
        label: "Roaster".to_string(),
        value: roaster.name.clone(),
    }];
    if let Some(ref origin) = roast.origin {
        details.push(TimelineEventDetail {
            label: "Origin".to_string(),
            value: origin.clone(),
        });
    }
    if !roast.tasting_notes.is_empty() {
        details.push(TimelineEventDetail {
            label: "Tasting Notes".to_string(),
            value: roast.tasting_notes.join(", "),
        });
    }
    NewTimelineEvent {
        entity_type: EntityType::Roast,
        entity_id: roast.id.into_inner(),
        action: "added".to_string(),
        occurred_at: roast.created_at,
        title: roast.name.clone(),
        details,
        tasting_notes: roast.tasting_notes.clone(),
        slug: Some(roast.slug.clone()),
        roaster_slug: Some(roaster.slug.clone()),
        brew_data: None,
    }
}
