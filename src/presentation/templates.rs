use askama::Template;

use super::views::{
    ListNavigator, Paginated, RoastView, RoasterOptionView, RoasterView, TimelineEventView,
    TimelineMonthView,
};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::RoastSortKey;
use crate::domain::timeline::TimelineSortKey;

#[derive(Template)]
#[template(path = "roasters.html")]
pub struct RoastersTemplate {
    pub nav_active: &'static str,
    pub roasters: Paginated<RoasterView>,
    pub navigator: ListNavigator<RoasterSortKey>,
}

#[derive(Template)]
#[template(path = "partials/roaster_list.html")]
pub struct RoasterListTemplate {
    pub roasters: Paginated<RoasterView>,
    pub navigator: ListNavigator<RoasterSortKey>,
}

#[derive(Template)]
#[template(path = "roaster_detail.html")]
pub struct RoasterDetailTemplate {
    pub nav_active: &'static str,
    pub roaster: RoasterView,
    pub roasts: Vec<RoastView>,
}

#[derive(Template)]
#[template(path = "roasts.html")]
pub struct RoastsTemplate {
    pub nav_active: &'static str,
    pub roasts: Paginated<RoastView>,
    pub roaster_options: Vec<RoasterOptionView>,
    pub navigator: ListNavigator<RoastSortKey>,
}

#[derive(Template)]
#[template(path = "roast_detail.html")]
pub struct RoastDetailTemplate {
    pub nav_active: &'static str,
    pub roast: RoastView,
}

#[derive(Template)]
#[template(path = "partials/roast_list.html")]
pub struct RoastListTemplate {
    pub roasts: Paginated<RoastView>,
    pub navigator: ListNavigator<RoastSortKey>,
}

#[derive(Template)]
#[template(path = "timeline.html")]
pub struct TimelineTemplate {
    pub nav_active: &'static str,
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

pub fn render_template<T: Template>(template: T) -> Result<String, askama::Error> {
    template.render()
}
