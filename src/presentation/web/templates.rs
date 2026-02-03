use askama::Template;

use super::views::{
    BagOptionView, BagView, BrewDefaultsView, BrewView, CafeOptionView, CafeView, CupView,
    GearOptionView, GearView, ListNavigator, Paginated, RoastOptionView, RoastView,
    RoasterOptionView, RoasterView, TimelineEventView, TimelineMonthView,
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
#[template(path = "roasters.html")]
pub struct RoastersTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub roasters: Paginated<RoasterView>,
    pub navigator: ListNavigator<RoasterSortKey>,
}

#[derive(Template)]
#[template(path = "partials/roaster_list.html")]
pub struct RoasterListTemplate {
    pub is_authenticated: bool,
    pub roasters: Paginated<RoasterView>,
    pub navigator: ListNavigator<RoasterSortKey>,
}

#[derive(Template)]
#[template(path = "roaster_detail.html")]
pub struct RoasterDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub roaster: RoasterView,
    pub roasts: Vec<RoastView>,
}

#[derive(Template)]
#[template(path = "roasts.html")]
pub struct RoastsTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub roasts: Paginated<RoastView>,
    pub roaster_options: Vec<RoasterOptionView>,
    pub navigator: ListNavigator<RoastSortKey>,
}

#[derive(Template)]
#[template(path = "roast_detail.html")]
pub struct RoastDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub roast: RoastView,
    pub bags: Vec<BagView>,
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
    pub has_ai_extract: bool,
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
#[template(path = "bags.html")]
pub struct BagsTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub open_bags: Vec<BagView>,
    pub bags: Paginated<BagView>,
    pub roaster_options: Vec<RoasterOptionView>,
    pub navigator: ListNavigator<BagSortKey>,
}

#[derive(Template)]
#[template(path = "partials/bag_list.html")]
pub struct BagListTemplate {
    pub is_authenticated: bool,
    pub open_bags: Vec<BagView>,
    pub bags: Paginated<BagView>,
    pub navigator: ListNavigator<BagSortKey>,
}

#[derive(Template)]
#[template(path = "gear.html")]
pub struct GearTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub gear: Paginated<GearView>,
    pub navigator: ListNavigator<GearSortKey>,
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
#[template(path = "brews.html")]
pub struct BrewsTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub brews: Paginated<BrewView>,
    pub bag_options: Vec<BagOptionView>,
    pub grinder_options: Vec<GearOptionView>,
    pub brewer_options: Vec<GearOptionView>,
    pub filter_paper_options: Vec<GearOptionView>,
    pub defaults: BrewDefaultsView,
    pub navigator: ListNavigator<BrewSortKey>,
}

#[derive(Template)]
#[template(path = "partials/brew_list.html")]
pub struct BrewListTemplate {
    pub is_authenticated: bool,
    pub brews: Paginated<BrewView>,
    pub navigator: ListNavigator<BrewSortKey>,
}

#[derive(Template)]
#[template(path = "cafes.html")]
pub struct CafesTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub cafes: Paginated<CafeView>,
    pub navigator: ListNavigator<CafeSortKey>,
}

#[derive(Template)]
#[template(path = "partials/cafe_list.html")]
pub struct CafeListTemplate {
    pub is_authenticated: bool,
    pub cafes: Paginated<CafeView>,
    pub navigator: ListNavigator<CafeSortKey>,
}

#[derive(Template)]
#[template(path = "cafe_detail.html")]
pub struct CafeDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub cafe: CafeView,
}

#[derive(Template)]
#[template(path = "cups.html")]
pub struct CupsTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
    pub cups: Paginated<CupView>,
    pub roast_options: Vec<RoastOptionView>,
    pub cafe_options: Vec<CafeOptionView>,
    pub navigator: ListNavigator<CupSortKey>,
}

#[derive(Template)]
#[template(path = "partials/cup_list.html")]
pub struct CupListTemplate {
    pub is_authenticated: bool,
    pub cups: Paginated<CupView>,
    pub navigator: ListNavigator<CupSortKey>,
}

#[derive(Template)]
#[template(path = "scan.html")]
pub struct ScanTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub has_ai_extract: bool,
}

pub fn render_template<T: Template>(template: T) -> Result<String, askama::Error> {
    template.render()
}
