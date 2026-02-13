use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entity_type::EntityType;
use crate::domain::ids::{BagId, BrewId, GearId};
use crate::domain::listing::{SortDirection, SortKey};
use crate::domain::timeline::{NewTimelineEvent, TimelineBrewData, TimelineEventDetail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickNote {
    Good,
    TooFast,
    TooSlow,
    TooHot,
    UnderExtracted,
    OverExtracted,
}

impl QuickNote {
    pub fn label(self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::TooFast => "Too Fast",
            Self::TooSlow => "Too Slow",
            Self::TooHot => "Too Hot",
            Self::UnderExtracted => "Under Extracted",
            Self::OverExtracted => "Over Extracted",
        }
    }

    pub fn form_value(self) -> &'static str {
        match self {
            Self::Good => "good",
            Self::TooFast => "too-fast",
            Self::TooSlow => "too-slow",
            Self::TooHot => "too-hot",
            Self::UnderExtracted => "under-extracted",
            Self::OverExtracted => "over-extracted",
        }
    }

    pub fn from_str_value(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    pub fn is_positive(self) -> bool {
        matches!(self, Self::Good)
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Good,
            Self::TooFast,
            Self::TooSlow,
            Self::TooHot,
            Self::UnderExtracted,
            Self::OverExtracted,
        ]
    }
}

impl FromStr for QuickNote {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "good" | "Good" => Ok(Self::Good),
            "too-fast" | "Too Fast" => Ok(Self::TooFast),
            "too-slow" | "Too Slow" => Ok(Self::TooSlow),
            "too-hot" | "Too Hot" => Ok(Self::TooHot),
            "under-extracted" | "Under Extracted" => Ok(Self::UnderExtracted),
            "over-extracted" | "Over Extracted" => Ok(Self::OverExtracted),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brew {
    pub id: BrewId,
    pub bag_id: BagId,
    pub coffee_weight: f64,
    pub grinder_id: GearId,
    pub grind_setting: f64,
    pub brewer_id: GearId,
    pub filter_paper_id: Option<GearId>,
    pub water_volume: i32,
    pub water_temp: f64,
    pub quick_notes: Vec<QuickNote>,
    pub brew_time: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Format seconds as "M:SS" (e.g., 150 -> "2:30").
pub fn format_brew_time(seconds: i32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewWithDetails {
    #[serde(flatten)]
    pub brew: Brew,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub grinder_name: String,
    pub grinder_model: String,
    pub brewer_name: String,
    pub filter_paper_name: Option<String>,
}

impl BrewWithDetails {
    pub fn to_timeline_event(&self) -> NewTimelineEvent {
        let ratio = if self.brew.coffee_weight > 0.0 {
            format!(
                "1:{:.1}",
                f64::from(self.brew.water_volume) / self.brew.coffee_weight
            )
        } else {
            "N/A".to_string()
        };

        let mut details = vec![
            TimelineEventDetail {
                label: "Roaster".to_string(),
                value: self.roaster_name.clone(),
            },
            TimelineEventDetail {
                label: "Coffee".to_string(),
                value: crate::domain::formatting::format_weight(self.brew.coffee_weight),
            },
            TimelineEventDetail {
                label: "Water".to_string(),
                value: format!(
                    "{}ml \u{00B7} {:.1}\u{00B0}C",
                    self.brew.water_volume, self.brew.water_temp
                ),
            },
            TimelineEventDetail {
                label: "Grinder".to_string(),
                value: format!(
                    "{} \u{00B7} {:.1}",
                    self.grinder_name, self.brew.grind_setting
                ),
            },
            TimelineEventDetail {
                label: "Brewer".to_string(),
                value: self.brewer_name.clone(),
            },
        ];

        if let Some(bt) = self.brew.brew_time {
            // Insert after Water (index 2) so it appears right beneath it
            details.insert(
                3,
                TimelineEventDetail {
                    label: "Brew Time".to_string(),
                    value: format_brew_time(bt),
                },
            );
        }

        if let Some(ref fp_name) = self.filter_paper_name {
            details.push(TimelineEventDetail {
                label: "Filter".to_string(),
                value: fp_name.clone(),
            });
        }

        details.push(TimelineEventDetail {
            label: "Ratio".to_string(),
            value: ratio,
        });

        if !self.brew.quick_notes.is_empty() {
            let labels: Vec<&str> = self.brew.quick_notes.iter().map(|n| n.label()).collect();
            details.push(TimelineEventDetail {
                label: "Notes".to_string(),
                value: labels.join(", "),
            });
        }

        NewTimelineEvent {
            entity_type: EntityType::Brew,
            entity_id: self.brew.id.into_inner(),
            action: "brewed".to_string(),
            occurred_at: self.brew.created_at,
            title: self.roast_name.clone(),
            details,
            tasting_notes: vec![],
            slug: Some(self.roast_slug.clone()),
            roaster_slug: Some(self.roaster_slug.clone()),
            brew_data: Some(TimelineBrewData {
                bag_id: self.brew.bag_id,
                grinder_id: self.brew.grinder_id,
                brewer_id: self.brew.brewer_id,
                filter_paper_id: self.brew.filter_paper_id,
                coffee_weight: self.brew.coffee_weight,
                grind_setting: self.brew.grind_setting,
                water_volume: self.brew.water_volume,
                water_temp: self.brew.water_temp,
                brew_time: self.brew.brew_time,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBrew {
    pub bag_id: BagId,
    pub coffee_weight: f64,
    pub grinder_id: GearId,
    pub grind_setting: f64,
    pub brewer_id: GearId,
    pub filter_paper_id: Option<GearId>,
    pub water_volume: i32,
    pub water_temp: f64,
    pub quick_notes: Vec<QuickNote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brew_time: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateBrew {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bag_id: Option<BagId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coffee_weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grinder_id: Option<GearId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grind_setting: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brewer_id: Option<GearId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_paper_id: Option<GearId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_volume: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_temp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quick_notes: Option<Vec<QuickNote>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brew_time: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

/// Filter criteria for brew queries.
#[derive(Debug, Default, Clone)]
pub struct BrewFilter {
    pub bag_id: Option<BagId>,
}

impl BrewFilter {
    /// No filter - returns all brews.
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter for brews from a specific bag.
    pub fn for_bag(bag_id: BagId) -> Self {
        Self {
            bag_id: Some(bag_id),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BrewSortKey {
    CreatedAt,
    CoffeeWeight,
    WaterVolume,
}

impl SortKey for BrewSortKey {
    fn default() -> Self {
        BrewSortKey::CreatedAt
    }

    fn from_query(value: &str) -> Option<Self> {
        match value {
            "created-at" => Some(BrewSortKey::CreatedAt),
            "coffee-weight" => Some(BrewSortKey::CoffeeWeight),
            "water-volume" => Some(BrewSortKey::WaterVolume),
            _ => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            BrewSortKey::CreatedAt => "created-at",
            BrewSortKey::CoffeeWeight => "coffee-weight",
            BrewSortKey::WaterVolume => "water-volume",
        }
    }

    fn default_direction(self) -> SortDirection {
        SortDirection::Desc
    }
}
