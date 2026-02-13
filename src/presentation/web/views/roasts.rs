use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};
use crate::domain::roasters::Roaster;
use crate::domain::roasts::{Roast, RoastWithRoaster};

use super::tasting_notes::{self, TastingNoteView};
use super::{
    LegendEntry, build_coffee_info, build_origin_roaster_map, build_roaster_info, format_datetime,
};

pub struct RoastView {
    pub id: String,
    pub full_id: String,
    pub detail_path: String,
    pub name: String,
    pub roaster_label: String,
    pub origin: String,
    pub origin_flag: String,
    pub region: String,
    pub producer: String,
    pub process: String,
    pub created_date: String,
    pub created_time: String,
    pub created_at_sort_key: i64,
    pub tasting_notes: Vec<TastingNoteView>,
}

impl RoastView {
    pub fn from_domain(roast: Roast, roaster_name: &str, roaster_slug: &str) -> Self {
        Self::from_parts(roast, roaster_name, roaster_slug)
    }

    pub fn from_list_item(item: RoastWithRoaster) -> Self {
        let RoastWithRoaster {
            roast,
            roaster_name,
            roaster_slug,
        } = item;
        Self::from_parts(roast, &roaster_name, &roaster_slug)
    }

    fn from_parts(roast: Roast, roaster_name: &str, roaster_slug: &str) -> Self {
        let Roast {
            id: roast_id,
            roaster_id: _,
            name,
            slug,
            origin,
            region,
            producer,
            tasting_notes,
            process,
            created_at,
        } = roast;

        let full_id = roast_id.to_string();
        let id: String = full_id.chars().take(6).collect();
        let roaster_label = if roaster_name.trim().is_empty() {
            "Unknown roaster".to_string()
        } else {
            roaster_name.to_string()
        };
        let origin_flag = origin
            .as_deref()
            .and_then(country_to_iso)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();
        let origin = origin.unwrap_or_else(|| "—".to_string());
        let region = region.unwrap_or_else(|| "—".to_string());
        let producer = producer.unwrap_or_else(|| "—".to_string());
        let process = process.unwrap_or_else(|| "—".to_string());
        let created_at_sort_key = created_at.timestamp();
        let tasting_notes = tasting_notes::parse_and_categorize(&tasting_notes);
        let (created_date, created_time) = format_datetime(created_at);
        let detail_path = format!("/roasters/{roaster_slug}/roasts/{slug}");

        Self {
            id,
            full_id,
            detail_path,
            name,
            roaster_label,
            origin,
            origin_flag,
            region,
            producer,
            process,
            created_date,
            created_time,
            created_at_sort_key,
            tasting_notes,
        }
    }
}

pub struct RoastDetailView {
    pub id: String,
    pub name: String,
    pub roaster_name: String,
    pub roaster_slug: String,
    // Coffee info
    pub origin: String,
    pub origin_flag: String,
    pub region: String,
    pub producer: String,
    pub process: String,
    pub tasting_notes: Vec<TastingNoteView>,
    // Roaster info
    pub roaster_country: String,
    pub roaster_country_flag: String,
    pub roaster_city: Option<String>,
    pub roaster_homepage: Option<String>,
    // Map
    pub map_countries: String,
    pub map_max: u32,
    pub legend_entries: Vec<LegendEntry>,
    // Dates
    pub created_date: String,
    pub created_time: String,
}

impl RoastDetailView {
    pub fn from_parts(roast: Roast, roaster: &Roaster) -> Self {
        let coffee = build_coffee_info(&roast);
        let roaster_info = build_roaster_info(roaster);

        let (map_countries, map_max, legend_entries) =
            build_origin_roaster_map(roast.origin.as_deref(), &roaster.country);
        let (created_date, created_time) = format_datetime(roast.created_at);

        Self {
            id: roast.id.to_string(),
            name: roast.name,
            roaster_name: roaster.name.clone(),
            roaster_slug: roaster.slug.clone(),
            origin: coffee.origin,
            origin_flag: coffee.origin_flag,
            region: coffee.region,
            producer: coffee.producer,
            process: coffee.process,
            tasting_notes: coffee.tasting_notes,
            roaster_country: roaster_info.country,
            roaster_country_flag: roaster_info.country_flag,
            roaster_city: roaster_info.city,
            roaster_homepage: roaster_info.homepage,
            map_countries,
            map_max,
            legend_entries,
            created_date,
            created_time,
        }
    }
}

pub struct RoastOptionView {
    pub id: String,
    pub label: String,
    pub name: String,
    pub roaster_name: String,
}

impl From<RoastWithRoaster> for RoastOptionView {
    fn from(roast: RoastWithRoaster) -> Self {
        Self {
            id: roast.roast.id.to_string(),
            label: format!("{} - {}", roast.roaster_name, roast.roast.name),
            name: roast.roast.name,
            roaster_name: roast.roaster_name,
        }
    }
}
