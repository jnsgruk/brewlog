use crate::domain::bags::BagWithRoast;

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
            amount: format!("{:.1}", bag.bag.amount),
            remaining: format!("{:.1}", bag.bag.remaining),
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

impl From<BagWithRoast> for BagOptionView {
    fn from(bag: BagWithRoast) -> Self {
        let remaining = format!("{:.0}g", bag.bag.remaining);
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
