use crate::domain::gear::Gear;

pub struct GearDetailView {
    pub id: String,
    pub category_label: String,
    pub make: String,
    pub model: String,
    pub created_date: String,
    pub created_time: String,
}

impl GearDetailView {
    pub fn from_domain(gear: Gear) -> Self {
        Self {
            id: gear.id.to_string(),
            category_label: gear.category.display_label().to_string(),
            make: gear.make,
            model: gear.model,
            created_date: gear.created_at.format("%Y-%m-%d").to_string(),
            created_time: gear.created_at.format("%H:%M").to_string(),
        }
    }
}

#[derive(Clone)]
pub struct GearView {
    pub id: String,
    pub category: String,
    pub category_label: String,
    pub make: String,
    pub model: String,
    pub full_name: String,
    pub created_date: String,
    pub created_time: String,
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
            created_date: gear.created_at.format("%Y-%m-%d").to_string(),
            created_time: gear.created_at.format("%H:%M").to_string(),
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
