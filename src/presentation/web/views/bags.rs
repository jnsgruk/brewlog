use crate::domain::bags::BagWithRoast;
use crate::domain::formatting::format_weight;
use crate::domain::roasters::Roaster;
use crate::domain::roasts::Roast;

use super::tasting_notes::TastingNoteView;
use super::{build_coffee_info, build_map_data, build_roaster_info};

#[derive(Debug, Clone)]
pub struct BagView {
    pub id: String,
    pub roast_id: String,
    pub roast_date: Option<String>,
    pub amount: String,
    pub remaining: String,
    pub closed: bool,
    pub finished_date: String,
    pub created_date: String,
    pub created_time: String,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub used_percent: u8,
}

impl BagView {
    pub fn from_domain(bag: BagWithRoast) -> Self {
        let used_percent = if bag.bag.amount > 0.0 {
            (((bag.bag.amount - bag.bag.remaining) / bag.bag.amount) * 100.0).clamp(0.0, 100.0)
                as u8
        } else {
            0
        };
        Self {
            id: bag.bag.id.to_string(),
            roast_id: bag.bag.roast_id.to_string(),
            roast_date: bag.bag.roast_date.map(|d| d.to_string()),
            amount: format_weight(bag.bag.amount),
            remaining: format_weight(bag.bag.remaining),
            closed: bag.bag.closed,
            finished_date: bag
                .bag
                .finished_at
                .map_or_else(|| "—".to_string(), |d| d.format("%Y-%m-%d").to_string()),
            created_date: bag.bag.created_at.format("%Y-%m-%d").to_string(),
            created_time: bag.bag.created_at.format("%H:%M").to_string(),
            roast_name: bag.roast_name,
            roaster_name: bag.roaster_name,
            roast_slug: bag.roast_slug,
            roaster_slug: bag.roaster_slug,
            used_percent,
        }
    }
}

#[derive(Clone)]
pub struct BagOptionView {
    pub id: String,
    pub label: String,
    pub roast_name: String,
    pub roaster_name: String,
    pub remaining: String,
}

pub struct BagDetailView {
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
    // Bag-specific
    pub amount: String,
    pub remaining: String,
    pub used_percent: u8,
    pub closed: bool,
    pub roast_date: Option<String>,
    pub finished_date: Option<String>,
    // Map
    pub map_countries: String,
    pub map_max: u32,
    // Dates
    pub created_date: String,
    pub created_time: String,
}

impl BagDetailView {
    pub fn from_parts(bag: BagWithRoast, roast: &Roast, roaster: &Roaster) -> Self {
        let coffee = build_coffee_info(roast);
        let roaster_info = build_roaster_info(roaster);

        let mut map_entries: Vec<(&str, u32)> = Vec::new();
        if let Some(ref o) = roast.origin
            && !o.is_empty()
        {
            map_entries.push((o.as_str(), 2));
        }
        map_entries.push((roaster.country.as_str(), 1));
        let (map_countries, map_max) = build_map_data(&map_entries);

        Self {
            id: bag.bag.id.to_string(),
            roast_name: bag.roast_name,
            roaster_name: bag.roaster_name,
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
            amount: format_weight(bag.bag.amount),
            remaining: format_weight(bag.bag.remaining),
            used_percent: if bag.bag.amount > 0.0 {
                (((bag.bag.amount - bag.bag.remaining) / bag.bag.amount) * 100.0).clamp(0.0, 100.0)
                    as u8
            } else {
                0
            },
            closed: bag.bag.closed,
            roast_date: bag.bag.roast_date.map(|d| d.to_string()),
            finished_date: bag
                .bag
                .finished_at
                .map(|d| d.format("%Y-%m-%d").to_string()),
            map_countries,
            map_max,
            created_date: bag.bag.created_at.format("%Y-%m-%d").to_string(),
            created_time: bag.bag.created_at.format("%H:%M").to_string(),
        }
    }
}

impl From<BagWithRoast> for BagOptionView {
    fn from(bag: BagWithRoast) -> Self {
        let remaining = format_weight(bag.bag.remaining);
        Self {
            id: bag.bag.id.to_string(),
            label: format!(
                "{} - {} ({} remaining)",
                bag.roaster_name, bag.roast_name, remaining
            ),
            roast_name: bag.roast_name,
            roaster_name: bag.roaster_name,
            remaining,
        }
    }
}
