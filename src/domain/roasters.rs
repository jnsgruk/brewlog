use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::RoasterId;
use crate::domain::listing::{SortDirection, SortKey};
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roaster {
    pub id: RoasterId,
    pub name: String,
    pub slug: String,
    pub country: String,
    pub city: Option<String>,
    pub homepage: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoaster {
    pub name: String,
    pub country: String,
    pub city: Option<String>,
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl NewRoaster {
    pub fn normalize(mut self) -> Self {
        self.name = self.name.trim().to_string();
        self.country = self.country.trim().to_string();
        self.city = normalize_optional_field(self.city);
        self.homepage =
            normalize_optional_field(self.homepage).filter(|url| is_valid_url_scheme(url));
        self
    }

    pub fn slug(&self) -> String {
        let base = match &self.city {
            Some(city) => format!("{}-{}", self.name, city),
            None => self.name.clone(),
        };
        slug::slugify(base)
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

/// Returns `true` if the URL starts with `http://` or `https://`.
/// Rejects `javascript:`, `data:`, and other potentially dangerous schemes.
fn is_valid_url_scheme(url: &str) -> bool {
    let lower = url.trim().to_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

impl Roaster {
    pub fn to_timeline_event(&self) -> NewTimelineEvent {
        let mut details = vec![TimelineEventDetail {
            label: "Country".to_string(),
            value: self.country.clone(),
        }];
        if let Some(ref city) = self.city {
            details.push(TimelineEventDetail {
                label: "City".to_string(),
                value: city.clone(),
            });
        }
        NewTimelineEvent {
            entity_type: "roaster".to_string(),
            entity_id: self.id.into_inner(),
            action: "added".to_string(),
            occurred_at: self.created_at,
            title: self.name.clone(),
            details,
            tasting_notes: vec![],
            slug: Some(self.slug.clone()),
            roaster_slug: Some(self.slug.clone()),
            brew_data: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateRoaster {
    pub name: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl UpdateRoaster {
    pub fn normalize(mut self) -> Self {
        self.homepage = self
            .homepage
            .and_then(|h| {
                let trimmed = h.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .filter(|url| is_valid_url_scheme(url));
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RoasterSortKey {
    CreatedAt,
    Name,
    Country,
    City,
}

impl SortKey for RoasterSortKey {
    fn default() -> Self {
        RoasterSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(RoasterSortKey::CreatedAt),
            "name" => Some(RoasterSortKey::Name),
            "country" => Some(RoasterSortKey::Country),
            "city" => Some(RoasterSortKey::City),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            RoasterSortKey::CreatedAt => "created-at",
            RoasterSortKey::Name => "name",
            RoasterSortKey::Country => "country",
            RoasterSortKey::City => "city",
        }
    }

    fn default_direction(self) -> SortDirection {
        match self {
            RoasterSortKey::CreatedAt => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }
}
