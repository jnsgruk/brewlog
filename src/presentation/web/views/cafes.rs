use crate::domain::cafes::Cafe;

pub struct CafeView {
    pub id: String,
    pub detail_path: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub map_url: String,
    pub has_website: bool,
    pub website_url: String,
    pub website_label: String,
    pub notes: String,
    pub created_at: String,
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
            notes,
            created_at,
            updated_at: _,
        } = cafe;

        let website = website.unwrap_or_default();
        let has_website = !website.is_empty();
        let detail_path = format!("/cafes/{slug}");
        let map_url = format!("https://www.google.com/maps?q={latitude},{longitude}");

        let created_at_sort_key = created_at.timestamp();
        let created_at_label = created_at.format("%Y-%m-%d").to_string();

        Self {
            detail_path,
            id: id.to_string(),
            name,
            city,
            country,
            latitude,
            longitude,
            map_url,
            has_website,
            website_url: website.clone(),
            website_label: website,
            notes: notes.unwrap_or_else(|| "This cafe has no notes yet.".to_string()),
            created_at: created_at_label,
            created_at_sort_key,
        }
    }
}

pub struct CafeOptionView {
    pub id: String,
    pub label: String,
}

impl From<Cafe> for CafeOptionView {
    fn from(cafe: Cafe) -> Self {
        Self {
            id: cafe.id.to_string(),
            label: format!("{} ({})", cafe.name, cafe.city),
        }
    }
}
