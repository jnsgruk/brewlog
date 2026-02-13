use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};
use crate::domain::roasters::Roaster;

use super::{LegendEntry, build_map_data, format_datetime};

pub struct RoasterDetailView {
    pub id: String,
    pub name: String,
    pub country: String,
    pub country_flag: String,
    pub city: Option<String>,
    pub homepage: Option<String>,
    pub map_countries: String,
    pub map_max: u32,
    pub legend_entries: Vec<LegendEntry>,
    pub created_date: String,
    pub created_time: String,
}

impl RoasterDetailView {
    pub fn from_domain(roaster: Roaster) -> Self {
        let country_flag = country_to_iso(&roaster.country)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();
        let (map_countries, map_max) = build_map_data(&[(&roaster.country, 1)]);
        let (created_date, created_time) = format_datetime(roaster.created_at);

        Self {
            id: roaster.id.to_string(),
            name: roaster.name,
            country_flag,
            country: roaster.country,
            city: roaster.city,
            homepage: roaster.homepage,
            map_countries,
            map_max,
            legend_entries: vec![LegendEntry {
                label: "Roaster",
                opacity: "",
            }],
            created_date,
            created_time,
        }
    }
}

pub struct RoasterOptionView {
    pub id: String,
    pub name: String,
}

impl From<Roaster> for RoasterOptionView {
    fn from(roaster: Roaster) -> Self {
        Self {
            id: roaster.id.to_string(),
            name: roaster.name,
        }
    }
}

impl From<&Roaster> for RoasterOptionView {
    fn from(roaster: &Roaster) -> Self {
        Self {
            id: roaster.id.to_string(),
            name: roaster.name.clone(),
        }
    }
}

pub struct RoasterView {
    pub id: String,
    pub detail_path: String,
    pub name: String,
    pub country: String,
    pub country_flag: String,
    pub city: String,
    pub has_homepage: bool,
    pub homepage_url: String,
    pub homepage_label: String,
    pub created_date: String,
    pub created_time: String,
    pub created_at_sort_key: i64,
}

impl From<Roaster> for RoasterView {
    fn from(roaster: Roaster) -> Self {
        let Roaster {
            id,
            slug,
            name,
            country,
            city,
            homepage,
            created_at,
        } = roaster;

        let homepage = homepage.unwrap_or_default();
        let has_homepage = !homepage.is_empty();
        let detail_path = format!("/roasters/{slug}");
        let country_flag = country_to_iso(&country)
            .map(iso_to_flag_emoji)
            .unwrap_or_default();

        let created_at_sort_key = created_at.timestamp();
        let (created_date, created_time) = format_datetime(created_at);

        Self {
            detail_path,
            id: id.to_string(),
            name,
            country_flag,
            country,
            city: city.unwrap_or_else(|| "—".to_string()),
            has_homepage,
            homepage_url: homepage.clone(),
            homepage_label: homepage,
            created_date,
            created_time,
            created_at_sort_key,
        }
    }
}
