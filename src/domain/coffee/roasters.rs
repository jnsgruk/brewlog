use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::normalize_optional_field;
use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::RoasterId;
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

/// Returns `true` if the URL starts with `http://` or `https://`.
/// Rejects `javascript:`, `data:`, and other potentially dangerous schemes.
pub(crate) fn is_valid_url_scheme(url: &str) -> bool {
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
            entity_type: EntityType::Roaster,
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
        self.homepage =
            normalize_optional_field(self.homepage).filter(|url| is_valid_url_scheme(url));
        self
    }
}

define_sort_key!(pub RoasterSortKey {
    #[default]
    CreatedAt("created-at", Desc),
    Name("name", Asc),
    Country("country", Asc),
    City("city", Asc),
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_http_scheme() {
        assert!(is_valid_url_scheme("http://example.com"));
    }

    #[test]
    fn valid_https_scheme() {
        assert!(is_valid_url_scheme("https://example.com"));
    }

    #[test]
    fn rejects_javascript_scheme() {
        assert!(!is_valid_url_scheme("javascript:alert(1)"));
    }

    #[test]
    fn rejects_data_scheme() {
        assert!(!is_valid_url_scheme("data:text/html,<h1>hi</h1>"));
    }

    #[test]
    fn case_insensitive_http() {
        assert!(is_valid_url_scheme("HTTP://EXAMPLE.COM"));
    }

    #[test]
    fn rejects_ftp_scheme() {
        assert!(!is_valid_url_scheme("ftp://example.com"));
    }

    #[test]
    fn rejects_empty_string() {
        assert!(!is_valid_url_scheme(""));
    }

    #[test]
    fn trims_whitespace() {
        assert!(is_valid_url_scheme("  https://example.com  "));
    }

    #[test]
    fn normalize_trims_name() {
        let roaster = NewRoaster {
            name: "  Test  ".to_string(),
            country: "US".to_string(),
            city: Some("Portland".to_string()),
            homepage: Some("https://test.com".to_string()),
            created_at: None,
        };
        let normalized = roaster.normalize();
        assert_eq!(normalized.name, "Test");
    }

    #[test]
    fn normalize_filters_bad_scheme() {
        let roaster = NewRoaster {
            name: "Test".to_string(),
            country: "US".to_string(),
            city: Some("Portland".to_string()),
            homepage: Some("javascript:alert(1)".to_string()),
            created_at: None,
        };
        let normalized = roaster.normalize();
        assert_eq!(normalized.homepage, None);
    }

    #[test]
    fn normalize_keeps_valid_homepage() {
        let roaster = NewRoaster {
            name: "Test".to_string(),
            country: "US".to_string(),
            city: Some("Portland".to_string()),
            homepage: Some("https://test.com".to_string()),
            created_at: None,
        };
        let normalized = roaster.normalize();
        assert_eq!(normalized.homepage, Some("https://test.com".to_string()));
    }

    #[test]
    fn slug_with_city() {
        let roaster = NewRoaster {
            name: "Test".to_string(),
            country: "US".to_string(),
            city: Some("Portland".to_string()),
            homepage: Some("https://test.com".to_string()),
            created_at: None,
        };
        assert_eq!(roaster.slug(), "test-portland");
    }

    #[test]
    fn slug_without_city() {
        let roaster = NewRoaster {
            name: "Test".to_string(),
            country: "US".to_string(),
            city: None,
            homepage: Some("https://test.com".to_string()),
            created_at: None,
        };
        assert_eq!(roaster.slug(), "test");
    }
}
