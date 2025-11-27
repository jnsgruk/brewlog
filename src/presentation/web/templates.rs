use askama::Template;

use super::views::{
    BagView, ListNavigator, Paginated, RoastView, RoasterOptionView, RoasterView,
    TimelineEventView, TimelineMonthView,
};
use crate::domain::bags::BagSortKey;
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::{RoastSortKey, RoastWithRoaster};
use crate::domain::timeline::TimelineSortKey;

#[derive(Template)]
#[template(path = "roasters.html")]
pub struct RoastersTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
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
    pub roaster: RoasterView,
    pub roasts: Vec<RoastView>,
}

#[derive(Template)]
#[template(path = "roasts.html")]
pub struct RoastsTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub roasts: Paginated<RoastView>,
    pub roaster_options: Vec<RoasterOptionView>,
    pub navigator: ListNavigator<RoastSortKey>,
}

#[derive(Template)]
#[template(path = "roast_detail.html")]
pub struct RoastDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
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
    pub events: Paginated<TimelineEventView>,
    pub navigator: ListNavigator<TimelineSortKey>,
    pub months: Vec<TimelineMonthView>,
}

#[derive(Template)]
#[template(path = "partials/timeline_chunk.html")]
pub struct TimelineChunkTemplate {
    pub events: Paginated<TimelineEventView>,
    pub navigator: ListNavigator<TimelineSortKey>,
    pub months: Vec<TimelineMonthView>,
}

#[derive(Template)]
#[template(path = "bags.html")]
pub struct BagsTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
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
#[template(path = "partials/roast_options.html")]
pub struct RoastOptionsTemplate {
    pub roasts: Vec<RoastWithRoaster>,
}

pub fn render_template<T: Template>(template: T) -> Result<String, askama::Error> {
    template.render()
}
