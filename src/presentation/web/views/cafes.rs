use crate::domain::cafes::Cafe;
use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};
use crate::domain::nearby_cafes::NearbyCafeResult;

use super::{LegendEntry, build_map_data, format_datetime};

pub struct CafeDetailView {
    pub id: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub country_flag: String,
    pub website: Option<String>,
    pub map_url: String,
    pub map_countries: String,
    pub map_max: u32,
    pub legend_entries: Vec<LegendEntry>,
    pub created_date: String,
    pub created_time: String,
}

impl From<Cafe> for CafeDetailView {
    fn from(cafe: Cafe) -> Self {
        let (created_date, created_time) = format_datetime(cafe.created_at);
        let country_flag = country_to_iso(&cafe.country)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();
        let (map_countries, map_max) = build_map_data(&[(&cafe.country, 1)]);
        let map_url = format!(
            "https://www.google.com/maps?q={},{}",
            cafe.latitude, cafe.longitude
        );

        Self {
            id: cafe.id.to_string(),
            name: cafe.name,
            city: cafe.city,
            country_flag,
            country: cafe.country,
            website: cafe.website,
            map_url,
            map_countries,
            map_max,
            legend_entries: vec![LegendEntry {
                label: "Cafe",
                opacity: "",
            }],
            created_date,
            created_time,
        }
    }
}

pub struct CafeView {
    pub id: String,
    pub detail_path: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub country_flag: String,
    pub latitude: f64,
    pub longitude: f64,
    pub map_url: String,
    pub has_website: bool,
    pub website_url: String,
    pub website_label: String,
    pub created_date: String,
    pub created_time: String,
    pub created_at_sort_key: i64,
}

impl From<Cafe> for CafeView {
    fn from(cafe: Cafe) -> Self {
        let Cafe {
            id,
            slug,
            name,
            city,
            country,
            latitude,
            longitude,
            website,
            created_at,
            updated_at: _,
        } = cafe;

        let website = website.unwrap_or_default();
        let has_website = !website.is_empty();
        let detail_path = format!("/cafes/{slug}");
        let map_url = format!("https://www.google.com/maps?q={latitude},{longitude}");
        let country_flag = country_to_iso(&country)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();

        let created_at_sort_key = created_at.timestamp();
        let (created_date, created_time) = format_datetime(created_at);

        Self {
            detail_path,
            id: id.to_string(),
            name,
            city,
            country_flag,
            country,
            latitude,
            longitude,
            map_url,
            has_website,
            website_url: website.clone(),
            website_label: website,
            created_date,
            created_time,
            created_at_sort_key,
        }
    }
}

pub struct CafeOptionView {
    pub id: String,
    pub label: String,
    pub name: String,
    pub city: String,
}

impl From<Cafe> for CafeOptionView {
    fn from(cafe: Cafe) -> Self {
        Self {
            id: cafe.id.to_string(),
            label: format!("{} ({})", cafe.name, cafe.city),
            name: cafe.name,
            city: cafe.city,
        }
    }
}

pub struct NearbyCafeView {
    pub name: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub website: String,
    pub distance: String,
    pub location: String,
}

impl From<NearbyCafeResult> for NearbyCafeView {
    fn from(cafe: NearbyCafeResult) -> Self {
        let distance = if cafe.distance_meters < 1000 {
            format!("{} m", cafe.distance_meters)
        } else {
            format!("{:.1} km", f64::from(cafe.distance_meters) / 1000.0)
        };
        let location = [cafe.city.as_str(), cafe.country.as_str()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        Self {
            name: cafe.name,
            city: cafe.city,
            country: cafe.country,
            latitude: cafe.latitude,
            longitude: cafe.longitude,
            website: cafe.website.unwrap_or_default(),
            distance,
            location,
        }
    }
}
