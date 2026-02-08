use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::CafeId;
use crate::domain::listing::{SortDirection, SortKey};
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cafe {
    pub id: CafeId,
    pub name: String,
    pub slug: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Cafe {
    pub fn to_timeline_event(&self) -> NewTimelineEvent {
        NewTimelineEvent {
            entity_type: "cafe".to_string(),
            entity_id: self.id.into_inner(),
            action: "added".to_string(),
            occurred_at: self.created_at,
            title: self.name.clone(),
            details: vec![
                TimelineEventDetail {
                    label: "City".to_string(),
                    value: self.city.clone(),
                },
                TimelineEventDetail {
                    label: "Country".to_string(),
                    value: self.country.clone(),
                },
            ],
            tasting_notes: vec![],
            slug: Some(self.slug.clone()),
            roaster_slug: None,
            brew_data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCafe {
    pub name: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub website: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl NewCafe {
    pub fn normalize(mut self) -> Self {
        self.name = self.name.trim().to_string();
        self.city = self.city.trim().to_string();
        self.country = self.country.trim().to_string();
        self.website = normalize_optional_field(self.website);
        self
    }

    pub fn slug(&self) -> String {
        slug::slugify(format!("{}-{}", self.name, self.city))
    }
}

fn normalize_optional_field(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateCafe {
    pub name: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub website: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CafeSortKey {
    CreatedAt,
    Name,
    City,
    Country,
}

impl SortKey for CafeSortKey {
    fn default() -> Self {
        CafeSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(CafeSortKey::CreatedAt),
            "name" => Some(CafeSortKey::Name),
            "city" => Some(CafeSortKey::City),
            "country" => Some(CafeSortKey::Country),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            CafeSortKey::CreatedAt => "created-at",
            CafeSortKey::Name => "name",
            CafeSortKey::City => "city",
            CafeSortKey::Country => "country",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            CafeSortKey::CreatedAt => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }
}
