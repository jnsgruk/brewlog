use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::GearId;
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GearCategory {
    Grinder,
    Brewer,
    #[serde(rename = "filter_paper")]
    FilterPaper,
}

impl GearCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            GearCategory::Grinder => "grinder",
            GearCategory::Brewer => "brewer",
            GearCategory::FilterPaper => "filter_paper",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            GearCategory::Grinder => "Grinder",
            GearCategory::Brewer => "Brewer",
            GearCategory::FilterPaper => "Filter Paper",
        }
    }
}

impl FromStr for GearCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "grinder" => Ok(GearCategory::Grinder),
            "brewer" => Ok(GearCategory::Brewer),
            "filter_paper" => Ok(GearCategory::FilterPaper),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gear {
    pub id: GearId,
    pub category: GearCategory,
    pub make: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Gear {
    pub fn to_timeline_event(&self) -> NewTimelineEvent {
        NewTimelineEvent {
            entity_type: EntityType::Gear,
            entity_id: self.id.into_inner(),
            action: "added".to_string(),
            occurred_at: self.created_at,
            title: format!("{} {}", self.make, self.model),
            details: vec![
                TimelineEventDetail {
                    label: "Category".to_string(),
                    value: self.category.display_label().to_string(),
                },
                TimelineEventDetail {
                    label: "Make".to_string(),
                    value: self.make.clone(),
                },
                TimelineEventDetail {
                    label: "Model".to_string(),
                    value: self.model.clone(),
                },
            ],
            tasting_notes: vec![],
            slug: None,
            roaster_slug: None,
            brew_data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGear {
    pub category: GearCategory,
    pub make: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGear {
    pub make: Option<String>,
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone)]
pub struct GearFilter {
    pub category: Option<GearCategory>,
}

impl GearFilter {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn for_category(category: GearCategory) -> Self {
        Self {
            category: Some(category),
        }
    }
}

define_sort_key!(pub GearSortKey {
    #[default]
    CreatedAt("created-at", Desc),
    Make("make", Asc),
    Model("model", Asc),
    Category("category", Asc),
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gear_category_from_str_valid() {
        assert_eq!("grinder".parse::<GearCategory>(), Ok(GearCategory::Grinder));
    }

    #[test]
    fn gear_category_case_insensitive() {
        assert_eq!("BREWER".parse::<GearCategory>(), Ok(GearCategory::Brewer));
    }

    #[test]
    fn gear_category_invalid() {
        assert!("invalid".parse::<GearCategory>().is_err());
    }
}
