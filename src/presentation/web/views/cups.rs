use crate::domain::cafes::Cafe;
use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};
use crate::domain::cups::CupWithDetails;
use crate::domain::roasters::Roaster;
use crate::domain::roasts::Roast;

use super::build_map_data;
use super::tasting_notes::{self, TastingNoteView};

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

impl CupView {
    pub fn from_domain(cup: CupWithDetails) -> Self {
        Self {
            id: cup.cup.id.to_string(),
            roast_name: cup.roast_name,
            roaster_name: cup.roaster_name,
            roast_slug: cup.roast_slug,
            roaster_slug: cup.roaster_slug,
            cafe_name: cup.cafe_name,
            cafe_slug: cup.cafe_slug,
            cafe_city: cup.cafe_city,
            created_date: cup.cup.created_at.format("%Y-%m-%d").to_string(),
            created_time: cup.cup.created_at.format("%H:%M").to_string(),
        }
    }
}

pub struct CupDetailView {
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
    // Map
    pub map_countries: String,
    pub map_max: u32,
    // Dates
    pub created_date: String,
    pub created_time: String,
}

impl CupDetailView {
    pub fn from_parts(cup: CupWithDetails, roast: &Roast, roaster: &Roaster, cafe: &Cafe) -> Self {
        let em_dash = "\u{2014}".to_string();

        let origin = roast.origin.clone().unwrap_or_default();
        let origin_flag = country_to_iso(&origin)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();

        let roaster_country_flag = country_to_iso(&roaster.country)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();

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

        let tasting_notes = roast
            .tasting_notes
            .iter()
            .flat_map(|note| {
                note.split([',', '\n'])
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .map(|n| tasting_notes::categorize(&n))
            .collect();

        Self {
            roast_name: cup.roast_name,
            roaster_name: cup.roaster_name,
            origin: if origin.is_empty() {
                em_dash.clone()
            } else {
                origin
            },
            origin_flag,
            region: roast
                .region
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or(em_dash.clone()),
            producer: roast
                .producer
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or(em_dash.clone()),
            process: roast
                .process
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or(em_dash),
            tasting_notes,
            roaster_country: roaster.country.clone(),
            roaster_country_flag,
            roaster_city: roaster.city.clone(),
            roaster_homepage: roaster.homepage.clone(),
            cafe_name: cafe.name.clone(),
            cafe_city: cafe.city.clone(),
            cafe_country: cafe.country.clone(),
            cafe_country_flag,
            cafe_website: cafe.website.clone(),
            map_countries,
            map_max,
            created_date: cup.cup.created_at.format("%Y-%m-%d").to_string(),
            created_time: cup.cup.created_at.format("%H:%M").to_string(),
        }
    }
}
