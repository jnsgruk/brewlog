use crate::domain::gear::Gear;

use super::format_datetime;

pub struct GearDetailView {
    pub id: String,
    pub category_label: String,
    pub make: String,
    pub model: String,
    pub created_date: String,
    pub created_time: String,
}

impl From<Gear> for GearDetailView {
    fn from(gear: Gear) -> Self {
        let (created_date, created_time) = format_datetime(gear.created_at);
        Self {
            id: gear.id.to_string(),
            category_label: gear.category.display_label().to_string(),
            make: gear.make,
            model: gear.model,
            created_date,
            created_time,
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

impl From<Gear> for GearView {
    fn from(gear: Gear) -> Self {
        let (created_date, created_time) = format_datetime(gear.created_at);
        Self {
            id: gear.id.to_string(),
            category: gear.category.as_str().to_string(),
            category_label: gear.category.display_label().to_string(),
            make: gear.make.clone(),
            model: gear.model.clone(),
            full_name: format!("{} {}", gear.make, gear.model),
            created_date,
            created_time,
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
