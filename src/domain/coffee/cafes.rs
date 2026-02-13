use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::normalize_optional_field;
use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::CafeId;
use crate::domain::roasters::is_valid_url_scheme;
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
            entity_type: EntityType::Cafe,
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
        self.website =
            normalize_optional_field(self.website).filter(|url| is_valid_url_scheme(url));
        self
    }

    pub fn slug(&self) -> String {
        slug::slugify(format!("{}-{}", self.name, self.city))
    }
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

impl UpdateCafe {
    pub fn normalize(mut self) -> Self {
        self.website =
            normalize_optional_field(self.website).filter(|url| is_valid_url_scheme(url));
        self
    }
}

define_sort_key!(pub CafeSortKey {
    #[default]
    CreatedAt("created-at", Desc),
    Name("name", Asc),
    City("city", Asc),
    Country("country", Asc),
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_fields() {
        let cafe = NewCafe {
            name: "  Test Cafe  ".to_string(),
            city: "  Portland  ".to_string(),
            country: "  US  ".to_string(),
            latitude: 45.5,
            longitude: -122.6,
            website: None,
            created_at: None,
        };
        let normalized = cafe.normalize();
        assert_eq!(normalized.name, "Test Cafe");
        assert_eq!(normalized.city, "Portland");
        assert_eq!(normalized.country, "US");
    }

    #[test]
    fn normalize_filters_bad_website() {
        let cafe = NewCafe {
            name: "Test Cafe".to_string(),
            city: "Portland".to_string(),
            country: "US".to_string(),
            latitude: 45.5,
            longitude: -122.6,
            website: Some("javascript:alert(1)".to_string()),
            created_at: None,
        };
        let normalized = cafe.normalize();
        assert_eq!(normalized.website, None);
    }

    #[test]
    fn normalize_keeps_valid_website() {
        let cafe = NewCafe {
            name: "Test Cafe".to_string(),
            city: "Portland".to_string(),
            country: "US".to_string(),
            latitude: 45.5,
            longitude: -122.6,
            website: Some("https://testcafe.com".to_string()),
            created_at: None,
        };
        let normalized = cafe.normalize();
        assert_eq!(normalized.website, Some("https://testcafe.com".to_string()));
    }

    #[test]
    fn slug_generation() {
        let cafe = NewCafe {
            name: "Test Cafe".to_string(),
            city: "Portland".to_string(),
            country: "US".to_string(),
            latitude: 45.5,
            longitude: -122.6,
            website: None,
            created_at: None,
        };
        assert_eq!(cafe.slug(), "test-cafe-portland");
    }
}
