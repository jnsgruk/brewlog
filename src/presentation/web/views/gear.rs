use crate::domain::gear::Gear;

#[derive(Clone)]
pub struct GearView {
    pub id: String,
    pub category: String,
    pub category_label: String,
    pub make: String,
    pub model: String,
    pub full_name: String,
    pub created_at: String,
}

impl GearView {
    pub fn from_domain(gear: Gear) -> Self {
        Self {
            id: gear.id.to_string(),
            category: gear.category.as_str().to_string(),
            category_label: gear.category.display_label().to_string(),
            make: gear.make.clone(),
            model: gear.model.clone(),
            full_name: format!("{} {}", gear.make, gear.model),
            created_at: gear.created_at.format("%Y-%m-%d").to_string(),
        }
    }
}

#[derive(Clone)]
pub struct GearOptionView {
    pub id: String,
    pub label: String,
}

impl From<Gear> for GearOptionView {
    fn from(gear: Gear) -> Self {
        Self {
            id: gear.id.to_string(),
            label: format!("{} {}", gear.make, gear.model),
        }
    }
}
