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
            created_date: cup.cup.created_at.format("%Y-%m-%d").to_string(),
            created_time: cup.cup.created_at.format("%H:%M").to_string(),
        }
    }
}
