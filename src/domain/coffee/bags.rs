use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

use crate::define_sort_key;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::{BagId, RoastId};
use crate::domain::roasters::Roaster;
use crate::domain::roasts::Roast;
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};

/// 23:59:59 — used when converting a date-only `finished_at` to a datetime
/// so that bag "finished" events sort after same-day brews.
pub const END_OF_DAY: NaiveTime = match NaiveTime::from_hms_opt(23, 59, 59) {
    Some(t) => t,
    None => unreachable!(),
};

/// Deserializes a datetime that accepts both RFC 3339 (`2025-02-24T15:30:00Z`)
/// and date-only (`2025-02-24`) formats. Date-only values become 23:59:59 UTC
/// so bag "finished" events sort after same-day brews.
pub(crate) fn deserialize_flexible_finished_at<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                Ok(Some(dt.with_timezone(&Utc)))
            } else if let Ok(date) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                Ok(Some(date.and_time(END_OF_DAY).and_utc()))
            } else {
                Err(serde::de::Error::custom(format!(
                    "invalid finished_at: expected RFC 3339 or YYYY-MM-DD, got: {s}"
                )))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bag {
    pub id: BagId,
    pub roast_id: RoastId,
    pub roast_date: Option<NaiveDate>,
    pub amount: f64,
    pub remaining: f64,
    pub closed: bool,
    #[serde(default, deserialize_with = "deserialize_flexible_finished_at")]
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagWithRoast {
    #[serde(flatten)]
    pub bag: Bag,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBag {
    pub roast_id: RoastId,
    pub roast_date: Option<NaiveDate>,
    pub amount: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateBag {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roast_id: Option<RoastId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roast_date: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    pub remaining: Option<f64>,
    pub closed: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_flexible_finished_at")]
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

/// Filter criteria for bag queries.
#[derive(Debug, Default, Clone)]
pub struct BagFilter {
    pub closed: Option<bool>,
    pub roast_id: Option<RoastId>,
}

impl BagFilter {
    /// No filter - returns all bags.
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter for open (unclosed) bags only.
    pub fn open() -> Self {
        Self {
            closed: Some(false),
            ..Default::default()
        }
    }

    /// Filter for closed bags only.
    pub fn closed() -> Self {
        Self {
            closed: Some(true),
            ..Default::default()
        }
    }

    /// Filter for bags of a specific roast.
    pub fn for_roast(roast_id: RoastId) -> Self {
        Self {
            roast_id: Some(roast_id),
            ..Default::default()
        }
    }
}

define_sort_key!(pub BagSortKey {
    #[default]
    CreatedAt("created-at", Desc),
    RoastDate("roast-date", Desc),
    UpdatedAt("updated-at", Desc),
    Roaster("roaster", Asc),
    Roast("roast", Asc),
    Status("status", Asc),
    FinishedAt("finished-at", Desc),
});

pub fn bag_timeline_event(
    bag: &Bag,
    action: &str,
    roast: &Roast,
    roaster: &Roaster,
) -> NewTimelineEvent {
    let occurred_at = if action == "finished" {
        bag.finished_at.unwrap_or(bag.created_at)
    } else {
        bag.created_at
    };
    NewTimelineEvent {
        entity_type: EntityType::Bag,
        entity_id: bag.id.into_inner(),
        action: action.to_string(),
        occurred_at,
        title: roast.name.clone(),
        details: vec![
            TimelineEventDetail {
                label: "Roaster".to_string(),
                value: roaster.name.clone(),
            },
            TimelineEventDetail {
                label: "Amount".to_string(),
                value: crate::domain::formatting::format_weight(bag.amount),
            },
        ],
        tasting_notes: vec![],
        slug: Some(roast.slug.clone()),
        roaster_slug: Some(roaster.slug.clone()),
        brew_data: None,
    }
}
