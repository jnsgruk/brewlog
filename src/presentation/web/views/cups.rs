use crate::domain::cafes::Cafe;
use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};
use crate::domain::cups::CupWithDetails;
use crate::domain::roasters::Roaster;
use crate::domain::roasts::Roast;

use super::tasting_notes::TastingNoteView;
use super::{LegendEntry, build_coffee_info, build_map_data, build_roaster_info, format_datetime};

#[derive(Clone)]
pub struct CupView {
    pub id: String,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub cafe_name: String,
    pub cafe_slug: String,
    pub cafe_city: String,
    pub created_date: String,
    pub created_time: String,
}

impl From<CupWithDetails> for CupView {
    fn from(cup: CupWithDetails) -> Self {
        let (created_date, created_time) = format_datetime(cup.cup.created_at);
        Self {
            id: cup.cup.id.to_string(),
            roast_name: cup.roast_name,
            roaster_name: cup.roaster_name,
            roast_slug: cup.roast_slug,
            roaster_slug: cup.roaster_slug,
            cafe_name: cup.cafe_name,
            cafe_slug: cup.cafe_slug,
            cafe_city: cup.cafe_city,
            created_date,
            created_time,
        }
    }
}

pub struct CupDetailView {
    pub id: String,
    // Coffee info
    pub roast_name: String,
    pub roaster_name: String,
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
    // Cafe info
    pub cafe_name: String,
    pub cafe_city: String,
    pub cafe_country: String,
    pub cafe_country_flag: String,
    pub cafe_website: Option<String>,
    pub cafe_map_url: String,
    // Map
    pub map_countries: String,
    pub map_max: u32,
    pub legend_entries: Vec<LegendEntry>,
    // Slugs (for breadcrumbs)
    pub roaster_slug: String,
    pub roast_slug: String,
    pub cafe_slug: String,
    // Dates
    pub created_date: String,
    pub created_time: String,
}

impl CupDetailView {
    pub fn from_parts(cup: CupWithDetails, roast: &Roast, roaster: &Roaster, cafe: &Cafe) -> Self {
        let coffee = build_coffee_info(roast);
        let roaster_info = build_roaster_info(roaster);

        let cafe_country_flag = country_to_iso(&cafe.country)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();

        let mut map_entries: Vec<(&str, u32)> = Vec::new();
        map_entries.push((cafe.country.as_str(), 3));
        if let Some(ref o) = roast.origin
            && !o.is_empty()
        {
            map_entries.push((o.as_str(), 2));
        }
        map_entries.push((roaster.country.as_str(), 1));
        let (map_countries, map_max) = build_map_data(&map_entries);
        let (created_date, created_time) = format_datetime(cup.cup.created_at);

        Self {
            id: cup.cup.id.to_string(),
            roast_name: cup.roast_name,
            roaster_name: cup.roaster_name,
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
            cafe_name: cafe.name.clone(),
            cafe_city: cafe.city.clone(),
            cafe_country: cafe.country.clone(),
            cafe_country_flag,
            cafe_website: cafe.website.clone(),
            cafe_map_url: format!(
                "https://www.google.com/maps?q={},{}",
                cafe.latitude, cafe.longitude
            ),
            roaster_slug: roaster.slug.clone(),
            roast_slug: roast.slug.clone(),
            cafe_slug: cafe.slug.clone(),
            map_countries,
            map_max,
            legend_entries: vec![
                LegendEntry {
                    label: "Cafe",
                    opacity: "",
                },
                LegendEntry {
                    label: "Origin",
                    opacity: "opacity-65",
                },
                LegendEntry {
                    label: "Roaster",
                    opacity: "opacity-35",
                },
            ],
            created_date,
            created_time,
        }
    }
}
