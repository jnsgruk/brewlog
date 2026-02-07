use crate::domain::brews::{Brew, BrewWithDetails, QuickNote, format_brew_time};

use super::relative_date;

#[derive(Clone)]
pub struct QuickNoteView {
    pub label: String,
    pub pill_class: &'static str,
    pub form_value: String,
}

impl From<QuickNote> for QuickNoteView {
    fn from(note: QuickNote) -> Self {
        Self {
            label: note.label().to_string(),
            pill_class: if note.is_positive() {
                "pill pill-success"
            } else {
                "pill pill-warning"
            },
            form_value: note.form_value().to_string(),
        }
    }
}

#[derive(Clone)]
pub struct BrewView {
    pub id: String,
    pub bag_id: i64,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub coffee_weight: String,
    pub grinder_id: i64,
    pub grinder_name: String,
    pub grinder_model: String,
    pub grind_setting: String,
    pub brewer_id: i64,
    pub brewer_name: String,
    pub filter_paper_id: Option<i64>,
    pub filter_paper_name: Option<String>,
    pub water_volume: String,
    pub water_temp: String,
    pub ratio: String,
    pub brew_time: Option<String>,
    pub quick_notes: Vec<QuickNoteView>,
    pub quick_notes_label: String,
    pub quick_notes_raw: String,
    pub created_date: String,
    pub created_time: String,
    pub relative_date_label: String,
    // Raw values for "brew again" feature
    pub coffee_weight_raw: f64,
    pub grind_setting_raw: f64,
    pub water_volume_raw: i32,
    pub water_temp_raw: f64,
    pub brew_time_raw: Option<i32>,
}

impl BrewView {
    pub fn from_domain(brew: BrewWithDetails) -> Self {
        let ratio = if brew.brew.coffee_weight > 0.0 {
            format!(
                "1:{:.1}",
                f64::from(brew.brew.water_volume) / brew.brew.coffee_weight
            )
        } else {
            "\u{2014}".to_string()
        };

        let quick_notes: Vec<QuickNoteView> = brew
            .brew
            .quick_notes
            .iter()
            .copied()
            .map(QuickNoteView::from)
            .collect();
        let quick_notes_label = quick_notes
            .iter()
            .map(|n| n.label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let quick_notes_raw = quick_notes
            .iter()
            .map(|n| n.form_value.as_str())
            .collect::<Vec<_>>()
            .join(",");

        Self {
            id: brew.brew.id.to_string(),
            bag_id: brew.brew.bag_id.into_inner(),
            roast_name: brew.roast_name,
            roaster_name: brew.roaster_name,
            roast_slug: brew.roast_slug,
            roaster_slug: brew.roaster_slug,
            coffee_weight: format!("{:.1}g", brew.brew.coffee_weight),
            grinder_id: brew.brew.grinder_id.into_inner(),
            grinder_model: brew
                .grinder_name
                .split_once(' ')
                .map_or_else(|| brew.grinder_name.clone(), |(_, model)| model.to_string()),
            grinder_name: brew.grinder_name,
            grind_setting: format!("{:.1}", brew.brew.grind_setting),
            brewer_id: brew.brew.brewer_id.into_inner(),
            brewer_name: brew.brewer_name,
            filter_paper_id: brew
                .brew
                .filter_paper_id
                .map(crate::domain::ids::GearId::into_inner),
            filter_paper_name: brew.filter_paper_name,
            water_volume: format!("{}ml", brew.brew.water_volume),
            water_temp: format!("{:.1}\u{00B0}C", brew.brew.water_temp),
            ratio,
            brew_time: brew.brew.brew_time.map(format_brew_time),
            quick_notes,
            quick_notes_label,
            quick_notes_raw,
            created_date: brew.brew.created_at.format("%Y-%m-%d").to_string(),
            created_time: brew.brew.created_at.format("%H:%M").to_string(),
            relative_date_label: relative_date(brew.brew.created_at),
            coffee_weight_raw: brew.brew.coffee_weight,
            grind_setting_raw: brew.brew.grind_setting,
            water_volume_raw: brew.brew.water_volume,
            water_temp_raw: brew.brew.water_temp,
            brew_time_raw: brew.brew.brew_time,
        }
    }
}

pub struct BrewDefaultsView {
    pub bag_id: String,
    pub grinder_id: String,
    pub brewer_id: String,
    pub filter_paper_id: String,
    pub coffee_weight: f64,
    pub grind_setting: f64,
    pub water_volume: i32,
    pub water_temp: f64,
    pub brew_time: Option<i32>,
}

impl Default for BrewDefaultsView {
    fn default() -> Self {
        Self {
            bag_id: String::new(),

            grinder_id: String::new(),
            brewer_id: String::new(),
            filter_paper_id: String::new(),
            coffee_weight: 15.0,
            grind_setting: 6.0,
            water_volume: 250,
            water_temp: 91.0,
            brew_time: Some(120),
        }
    }
}

impl From<Brew> for BrewDefaultsView {
    fn from(brew: Brew) -> Self {
        Self {
            bag_id: brew.bag_id.to_string(),

            grinder_id: brew.grinder_id.to_string(),
            brewer_id: brew.brewer_id.to_string(),
            filter_paper_id: brew
                .filter_paper_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            coffee_weight: brew.coffee_weight,
            grind_setting: brew.grind_setting,
            water_volume: brew.water_volume,
            water_temp: brew.water_temp,
            brew_time: brew.brew_time,
        }
    }
}
