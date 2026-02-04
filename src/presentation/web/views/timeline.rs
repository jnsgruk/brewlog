use crate::domain::timeline::{TimelineEvent, TimelineEventDetail};

use super::relative_date;

/// Returns `true` if the value is empty or just an em-dash placeholder.
fn is_blank(value: &str) -> bool {
    value.is_empty() || value == "\u{2014}"
}

#[derive(Clone)]
pub struct TimelineEventDetailView {
    pub label: String,
    pub value: String,
    pub link: Option<String>,
}

/// Raw brew data for repeating a brew from the timeline.
#[derive(Clone)]
pub struct TimelineBrewDataView {
    pub bag_id: i64,
    pub grinder_id: i64,
    pub brewer_id: i64,
    pub filter_paper_id: Option<i64>,
    pub coffee_weight: f64,
    pub grind_setting: f64,
    pub water_volume: i32,
    pub water_temp: f64,
}

#[derive(Clone)]
pub struct TimelineEventView {
    pub id: String,
    pub kind_label: &'static str,
    pub date_label: String,
    pub relative_date_label: String,
    pub time_label: Option<String>,
    pub iso_timestamp: String,
    pub title: String,
    pub link: String,
    pub external_link: Option<String>,
    pub details: Vec<TimelineEventDetailView>,
    pub subtitle: Option<String>,
    pub tasting_notes: Option<Vec<String>>,
    pub brew_data: Option<TimelineBrewDataView>,
}

pub struct TimelineMonthView {
    pub anchor: String,
    pub heading: String,
    pub events: Vec<TimelineEventView>,
}

impl TimelineEventView {
    pub fn from_domain(event: TimelineEvent) -> Self {
        let TimelineEvent {
            id,
            entity_type,
            entity_id,
            action,
            occurred_at,
            title,
            details,
            tasting_notes,
            slug,
            roaster_slug,
            brew_data,
        } = event;

        let kind_label = match (entity_type.as_str(), action.as_str()) {
            ("roaster", "added") => "Roaster Added",
            ("roast", "added") => "Roast Added",
            ("bag", "added") => "Bag Added",
            ("bag", "finished") => "Bag Finished",
            ("gear", "added") => "Gear Added",
            ("brew", "brewed") => "Brew Added",
            ("cafe", "added") => "Cafe Added",
            ("cup", "added") => "Cup Added",
            _ => "Event",
        };

        let link = match (entity_type.as_str(), slug, roaster_slug) {
            ("roaster", Some(slug), _) => format!("/roasters/{slug}"),
            // Roasts, bags, brews, and cups link to the roast page when we have slug info
            ("roast" | "bag" | "brew" | "cup", Some(slug), Some(roaster_slug)) => {
                format!("/roasters/{roaster_slug}/roasts/{slug}")
            }
            ("cafe", Some(slug), _) => format!("/cafes/{slug}"),
            ("cup", _, _) => "/cups".to_string(),
            ("gear", _, _) => "/gear".to_string(),
            ("brew", _, _) => "/brews".to_string(),
            ("roaster", None, _) => format!("/roasters/{entity_id}"),
            ("roast", None, _) => format!("/roasts/{entity_id}"),
            _ => String::from("#"),
        };

        let (mapped_details, external_link) = Self::map_details(details);

        let tasting_notes = if entity_type == "roast" {
            let notes = tasting_notes
                .into_iter()
                .flat_map(|note| {
                    note.split([',', '\n'])
                        .map(|segment| segment.trim().to_string())
                        .filter(|segment| !segment.is_empty())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Some(notes)
        } else {
            None
        };

        let subtitle = Self::build_subtitle(entity_type.as_str(), &mapped_details);

        let brew_data_view = brew_data.map(|bd| TimelineBrewDataView {
            bag_id: bd.bag_id,
            grinder_id: bd.grinder_id,
            brewer_id: bd.brewer_id,
            filter_paper_id: bd.filter_paper_id,
            coffee_weight: bd.coffee_weight,
            grind_setting: bd.grind_setting,
            water_volume: bd.water_volume,
            water_temp: bd.water_temp,
        });

        Self {
            id: id.to_string(),
            kind_label,
            date_label: occurred_at.format("%B %d, %Y").to_string(),
            relative_date_label: relative_date(occurred_at),
            time_label: Some(occurred_at.format("%H:%M UTC").to_string()),
            iso_timestamp: occurred_at.to_rfc3339(),
            title,
            link,
            external_link,
            details: mapped_details,
            subtitle,
            tasting_notes,
            brew_data: brew_data_view,
        }
    }

    fn build_subtitle(entity_type: &str, details: &[TimelineEventDetailView]) -> Option<String> {
        let find_value = |label: &str| {
            details
                .iter()
                .find(|d| d.label.eq_ignore_ascii_case(label))
                .map(|d| d.value.trim())
                .filter(|v| !is_blank(v))
        };

        let picks: &[&str] = match entity_type {
            "brew" => &["Roaster", "Brewer"],
            "roast" => &["Roaster", "Origin"],
            "cafe" => &["City", "Country"],
            "cup" => &["Roaster", "Cafe"],
            _ => &[],
        };

        let parts: Vec<&str> = if picks.is_empty() {
            details
                .iter()
                .take(3)
                .map(|d| d.value.trim())
                .filter(|v| !is_blank(v))
                .collect()
        } else {
            picks.iter().filter_map(|l| find_value(l)).collect()
        };

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" \u{00b7} "))
        }
    }

    fn map_details(
        details: Vec<TimelineEventDetail>,
    ) -> (Vec<TimelineEventDetailView>, Option<String>) {
        let mut mapped = Vec::new();
        let mut external_link = None;

        for detail in details {
            let label_lower = detail.label.to_ascii_lowercase();
            match label_lower.as_str() {
                // Website/homepage links are extracted, not shown as detail rows
                "homepage" | "website" => {
                    let trimmed = detail.value.trim();
                    if !is_blank(trimmed) {
                        external_link = Some(trimmed.to_string());
                    }
                }
                // Position values become clickable map links
                "position" => {
                    let trimmed = detail.value.trim();
                    if !trimmed.is_empty() {
                        let display = trimmed
                            .strip_prefix("https://www.google.com/maps?q=")
                            .unwrap_or(trimmed);
                        mapped.push(TimelineEventDetailView {
                            label: detail.label,
                            value: display.to_string(),
                            link: Some(trimmed.to_string()),
                        });
                    }
                }
                _ => {
                    mapped.push(TimelineEventDetailView {
                        label: detail.label,
                        value: detail.value,
                        link: None,
                    });
                }
            }
        }

        (mapped, external_link)
    }
}
