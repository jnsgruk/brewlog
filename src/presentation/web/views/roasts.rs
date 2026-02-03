use crate::domain::roasts::{Roast, RoastWithRoaster};

pub struct RoastView {
    pub id: String,
    pub full_id: String,
    pub detail_path: String,
    pub name: String,
    pub roaster_label: String,
    pub origin: String,
    pub region: String,
    pub producer: String,
    pub process: String,
    pub created_at: String,
    pub created_at_sort_key: i64,
    pub tasting_notes: Vec<String>,
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
        let origin = origin.unwrap_or_else(|| "—".to_string());
        let region = region.unwrap_or_else(|| "—".to_string());
        let producer = producer.unwrap_or_else(|| "—".to_string());
        let process = process.unwrap_or_else(|| "—".to_string());
        let created_at_sort_key = created_at.timestamp();
        let tasting_notes = tasting_notes
            .into_iter()
            .flat_map(|note| {
                note.split([',', '\n'])
                    .map(|segment| segment.trim().to_string())
                    .filter(|segment| !segment.is_empty())
                    .collect::<Vec<_>>()
            })
            .collect();
        let created_at = created_at.format("%Y-%m-%d").to_string();
        let detail_path = format!("/roasters/{roaster_slug}/roasts/{slug}");

        Self {
            id,
            full_id,
            detail_path,
            name,
            roaster_label,
            origin,
            region,
            producer,
            process,
            created_at,
            created_at_sort_key,
            tasting_notes,
        }
    }
}

pub struct RoastOptionView {
    pub id: String,
    pub label: String,
}

impl From<RoastWithRoaster> for RoastOptionView {
    fn from(roast: RoastWithRoaster) -> Self {
        Self {
            id: roast.roast.id.to_string(),
            label: format!("{} - {}", roast.roaster_name, roast.roast.name),
        }
    }
}
