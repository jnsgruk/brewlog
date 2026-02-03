use crate::domain::cups::CupWithDetails;

#[derive(Clone)]
pub struct CupView {
    pub id: String,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub cafe_name: String,
    pub cafe_slug: String,
    pub notes: String,
    pub has_notes: bool,
    pub rating: String,
    pub has_rating: bool,
    pub created_at: String,
}

impl CupView {
    pub fn from_domain(cup: CupWithDetails) -> Self {
        let notes = cup.cup.notes.clone().unwrap_or_default();
        let has_notes = !notes.is_empty();
        let rating = cup
            .cup
            .rating
            .map_or_else(|| "—".to_string(), |r| format!("{r}/5"));
        let has_rating = cup.cup.rating.is_some();

        Self {
            id: cup.cup.id.to_string(),
            roast_name: cup.roast_name,
            roaster_name: cup.roaster_name,
            roast_slug: cup.roast_slug,
            roaster_slug: cup.roaster_slug,
            cafe_name: cup.cafe_name,
            cafe_slug: cup.cafe_slug,
            notes,
            has_notes,
            rating,
            has_rating,
            created_at: cup.cup.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
