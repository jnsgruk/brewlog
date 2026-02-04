use askama::Template;

use super::views::{
    BagOptionView, BagView, BrewDefaultsView, BrewView, CafeOptionView, CafeView, CupView,
    GearOptionView, GearView, ListNavigator, NearbyCafeView, Paginated, RoastOptionView, RoastView,
    RoasterOptionView, RoasterView, StatsView, TimelineEventView, TimelineMonthView,
};
use crate::domain::bags::BagSortKey;
use crate::domain::brews::BrewSortKey;
use crate::domain::cafes::CafeSortKey;
use crate::domain::cups::CupSortKey;
use crate::domain::gear::GearSortKey;
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::{RoastSortKey, RoastWithRoaster};
use crate::domain::timeline::TimelineSortKey;

#[derive(Template)]
#[template(path = "partials/roaster_list.html")]
pub struct RoasterListTemplate {
    pub is_authenticated: bool,
    pub roasters: Paginated<RoasterView>,
    pub navigator: ListNavigator<RoasterSortKey>,
}

#[derive(Template)]
#[template(path = "partials/roast_list.html")]
pub struct RoastListTemplate {
    pub is_authenticated: bool,
    pub roasts: Paginated<RoastView>,
    pub navigator: ListNavigator<RoastSortKey>,
}

#[derive(Template)]
#[template(path = "timeline.html")]
pub struct TimelineTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,

    pub events: Paginated<TimelineEventView>,
    pub navigator: ListNavigator<TimelineSortKey>,
    pub months: Vec<TimelineMonthView>,
}

#[derive(Template)]
#[template(path = "partials/timeline_chunk.html")]
pub struct TimelineChunkTemplate {
    pub is_authenticated: bool,
    pub events: Paginated<TimelineEventView>,
    pub navigator: ListNavigator<TimelineSortKey>,
    pub months: Vec<TimelineMonthView>,
}

#[derive(Template)]
#[template(path = "partials/bag_list.html")]
pub struct BagListTemplate {
    pub is_authenticated: bool,
    pub bags: Paginated<BagView>,
    pub navigator: ListNavigator<BagSortKey>,
}

#[derive(Template)]
#[template(path = "partials/gear_list.html")]
pub struct GearListTemplate {
    pub is_authenticated: bool,
    pub gear: Paginated<GearView>,
    pub navigator: ListNavigator<GearSortKey>,
}

#[derive(Template)]
#[template(path = "partials/roast_options.html")]
pub struct RoastOptionsTemplate {
    pub roasts: Vec<RoastWithRoaster>,
}

#[derive(Template)]
#[template(path = "partials/brew_list.html")]
pub struct BrewListTemplate {
    pub is_authenticated: bool,
    pub brews: Paginated<BrewView>,
    pub navigator: ListNavigator<BrewSortKey>,
}

#[derive(Template)]
#[template(path = "partials/cafe_list.html")]
pub struct CafeListTemplate {
    pub is_authenticated: bool,
    pub cafes: Paginated<CafeView>,
    pub navigator: ListNavigator<CafeSortKey>,
}

#[derive(Template)]
#[template(path = "partials/cup_list.html")]
pub struct CupListTemplate {
    pub is_authenticated: bool,
    pub cups: Paginated<CupView>,
    pub navigator: ListNavigator<CupSortKey>,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,

    pub recent_brews: Vec<BrewView>,
    pub open_bags: Vec<BagView>,
    pub recent_events: Vec<TimelineEventView>,
    pub stats: StatsView,
}

#[derive(Template)]
#[template(path = "checkin.html")]
pub struct CheckInTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,

    pub roast_options: Vec<RoastOptionView>,
    pub cafe_options: Vec<CafeOptionView>,
}

#[derive(Template)]
#[template(path = "partials/nearby_cafes.html")]
pub struct NearbyCafesFragment {
    pub cafes: Vec<NearbyCafeView>,
}

#[derive(Template)]
#[template(path = "data.html")]
pub struct DataTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub active_type: String,
    pub tabs: Vec<DataTab>,
    pub content: String,
    pub search_value: String,
}

pub struct DataTab {
    pub key: &'static str,
    pub label: &'static str,
}

#[derive(Template)]
#[template(path = "add.html")]
pub struct AddTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub active_type: String,
    pub roaster_options: Vec<RoasterOptionView>,
    pub roast_options: Vec<RoastOptionView>,
    pub bag_options: Vec<BagOptionView>,
    pub grinder_options: Vec<GearOptionView>,
    pub brewer_options: Vec<GearOptionView>,
    pub filter_paper_options: Vec<GearOptionView>,
    pub cafe_options: Vec<CafeOptionView>,
    pub defaults: BrewDefaultsView,
}

pub fn render_template<T: Template>(template: T) -> Result<String, askama::Error> {
    template.render()
}
