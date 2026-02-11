use askama::Template;

use super::views::{
    BagDetailView, BagOptionView, BagView, BrewDefaultsView, BrewDetailView, BrewView,
    CafeDetailView, CafeOptionView, CafeView, CupDetailView, CupView, GearDetailView,
    GearOptionView, GearView, ListNavigator, NearbyCafeView, Paginated, QuickNoteView,
    RoastDetailView, RoastOptionView, RoastView, RoasterDetailView, RoasterOptionView, RoasterView,
    StatCard, StatsView, TimelineEventView, TimelineMonthView,
};
use crate::domain::bags::BagSortKey;
use crate::domain::brews::BrewSortKey;
use crate::domain::cafes::CafeSortKey;
use crate::domain::cups::CupSortKey;
use crate::domain::gear::GearSortKey;
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::{RoastSortKey, RoastWithRoaster};
use crate::domain::stats::{BrewingSummaryStats, ConsumptionStats, RoastSummaryStats};
use crate::domain::timeline::TimelineSortKey;

#[derive(Template)]
#[template(path = "partials/lists/roaster_list.html")]
pub struct RoasterListTemplate {
    pub is_authenticated: bool,
    pub roasters: Paginated<RoasterView>,
    pub navigator: ListNavigator<RoasterSortKey>,
}

#[derive(Template)]
#[template(path = "partials/lists/roast_list.html")]
pub struct RoastListTemplate {
    pub is_authenticated: bool,
    pub roasts: Paginated<RoastView>,
    pub navigator: ListNavigator<RoastSortKey>,
}

#[derive(Template)]
#[template(path = "pages/timeline.html")]
pub struct TimelineTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,

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
#[template(path = "partials/lists/bag_list.html")]
pub struct BagListTemplate {
    pub is_authenticated: bool,
    pub bags: Paginated<BagView>,
    pub navigator: ListNavigator<BagSortKey>,
}

#[derive(Template)]
#[template(path = "partials/lists/gear_list.html")]
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
#[template(path = "partials/lists/brew_list.html")]
pub struct BrewListTemplate {
    pub is_authenticated: bool,
    pub brews: Paginated<BrewView>,
    pub navigator: ListNavigator<BrewSortKey>,
}

#[derive(Template)]
#[template(path = "partials/lists/cafe_list.html")]
pub struct CafeListTemplate {
    pub is_authenticated: bool,
    pub cafes: Paginated<CafeView>,
    pub navigator: ListNavigator<CafeSortKey>,
}

#[derive(Template)]
#[template(path = "partials/lists/cup_list.html")]
pub struct CupListTemplate {
    pub is_authenticated: bool,
    pub cups: Paginated<CupView>,
    pub navigator: ListNavigator<CupSortKey>,
}

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,

    pub recent_brews: Vec<BrewView>,
    pub open_bags: Vec<BagView>,
    pub recent_events: Vec<TimelineEventView>,
    pub stats: StatsView,
    pub stat_cards: Vec<StatCard>,
}

#[derive(Template)]
#[template(path = "pages/checkin.html")]
pub struct CheckInTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,

    pub roast_options: Vec<RoastOptionView>,
    pub cafe_options: Vec<CafeOptionView>,
}

#[derive(Template)]
#[template(path = "partials/nearby_cafes.html")]
pub struct NearbyCafesFragment {
    pub cafes: Vec<NearbyCafeView>,
}

#[derive(Template)]
#[template(path = "pages/data.html")]
pub struct DataTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub active_type: String,
    pub tabs: Vec<Tab>,
    pub tab_signal: &'static str,
    pub tab_signal_js: &'static str,
    pub tab_base_url: &'static str,
    pub tab_fetch_target: &'static str,
    pub tab_fetch_mode: &'static str,
    pub content: String,
    pub search_value: String,
}

pub struct Tab {
    pub key: &'static str,
    pub label: &'static str,
}

#[derive(Template)]
#[template(path = "pages/add.html")]
pub struct AddTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub active_type: String,
    pub tabs: Vec<Tab>,
    pub tab_signal: &'static str,
    pub tab_signal_js: &'static str,
    pub tab_base_url: &'static str,
    pub tab_fetch_target: &'static str,
    pub tab_fetch_mode: &'static str,
    pub roaster_options: Vec<RoasterOptionView>,
    pub roast_options: Vec<RoastOptionView>,
    pub bag_options: Vec<BagOptionView>,
    pub grinder_options: Vec<GearOptionView>,
    pub brewer_options: Vec<GearOptionView>,
    pub filter_paper_options: Vec<GearOptionView>,
    pub cafe_options: Vec<CafeOptionView>,
    pub defaults: BrewDefaultsView,
    pub quick_note_options: Vec<QuickNoteView>,
    pub pre_select_bag_id: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/stats.html")]
pub struct StatsPageTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub active_type: String,
    pub tabs: Vec<Tab>,
    pub tab_signal: &'static str,
    pub tab_signal_js: &'static str,
    pub tab_base_url: &'static str,
    pub tab_fetch_target: &'static str,
    pub tab_fetch_mode: &'static str,
    pub content: String,
    pub roast_summary: RoastSummaryStats,
    pub consumption: ConsumptionStats,
    pub brewing_summary: BrewingSummaryStats,
    pub grinder_weights: Vec<(String, f64, String)>,
    pub max_grinder_weight: f64,
    pub consumption_30d_weight: String,
    pub consumption_all_time_weight: String,
    pub cache_age: String,
    pub has_data: bool,
}

#[derive(Template)]
#[template(path = "partials/stats_map.html")]
pub struct StatsMapFragment<'a> {
    pub geo_stats: &'a crate::domain::country_stats::GeoStats,
}

#[derive(Template)]
#[template(path = "pages/bag.html")]
pub struct BagDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub bag: BagDetailView,
    pub roaster_slug: String,
    pub roast_slug: String,
    pub image_url: Option<String>,
    pub edit_url: String,
}

#[derive(Template)]
#[template(path = "pages/brew.html")]
pub struct BrewDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub brew: BrewDetailView,
    pub roaster_slug: String,
    pub roast_slug: String,
    pub image_url: Option<String>,
    pub edit_url: String,
}

#[derive(Template)]
#[template(path = "pages/cup.html")]
pub struct CupDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub cup: CupDetailView,
    pub roaster_slug: String,
    pub roast_slug: String,
    pub cafe_slug: String,
    pub image_url: Option<String>,
    pub edit_url: String,
}

#[derive(Template)]
#[template(path = "pages/roast.html")]
pub struct RoastDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub roast: RoastDetailView,
    pub roaster_slug: String,
    pub image_url: Option<String>,
    pub edit_url: String,
}

#[derive(Template)]
#[template(path = "pages/roaster.html")]
pub struct RoasterDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub roaster: RoasterDetailView,
    pub image_url: Option<String>,
    pub edit_url: String,
}

#[derive(Template)]
#[template(path = "pages/cafe.html")]
pub struct CafeDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub cafe: CafeDetailView,
    pub image_url: Option<String>,
    pub edit_url: String,
}

#[derive(Template)]
#[template(path = "pages/gear.html")]
pub struct GearDetailTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub base_url: &'static str,
    pub gear: GearDetailView,
    pub image_url: Option<String>,
    pub edit_url: String,
}

// ── Edit page templates ──

#[derive(Template)]
#[template(path = "pages/edit_roaster.html")]
pub struct RoasterEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub name: String,
    pub country: String,
    pub city: String,
    pub homepage: String,
    pub image_url: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/edit_roast.html")]
pub struct RoastEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub roaster_id: String,
    pub roaster_name: String,
    pub name: String,
    pub origin: String,
    pub region: String,
    pub producer: String,
    pub process: String,
    pub tasting_notes: String,
    pub roaster_options: Vec<RoasterOptionView>,
    pub image_url: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/edit_bag.html")]
pub struct BagEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub roast_id: String,
    pub roast_label: String,
    pub roast_date: String,
    pub amount: f64,
    pub remaining: f64,
    pub roast_options: Vec<RoastOptionView>,
}

#[derive(Template)]
#[template(path = "pages/edit_brew.html")]
pub struct BrewEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub bag_id: String,
    pub bag_label: String,
    pub coffee_weight: f64,
    pub grinder_id: String,
    pub grind_setting: f64,
    pub brewer_id: String,
    pub filter_paper_id: String,
    pub water_volume: i32,
    pub water_temp: f64,
    pub brew_time: i32,
    pub quick_notes: String,
    pub bag_options: Vec<BagOptionView>,
    pub grinder_options: Vec<GearOptionView>,
    pub brewer_options: Vec<GearOptionView>,
    pub filter_paper_options: Vec<GearOptionView>,
    pub quick_note_options: Vec<QuickNoteView>,
    pub image_url: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/edit_cafe.html")]
pub struct CafeEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub website: String,
    pub image_url: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/edit_cup.html")]
pub struct CupEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub roast_id: String,
    pub roast_label: String,
    pub cafe_id: String,
    pub cafe_label: String,
    pub roast_options: Vec<RoastOptionView>,
    pub cafe_options: Vec<CafeOptionView>,
    pub image_url: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/edit_gear.html")]
pub struct GearEditTemplate {
    pub nav_active: &'static str,
    pub is_authenticated: bool,
    pub version_info: &'static crate::VersionInfo,
    pub id: String,
    pub category: String,
    pub make: String,
    pub model: String,
    pub image_url: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/image_upload.html")]
pub struct ImageUploadTemplate<'a> {
    pub entity_type: &'a str,
    pub entity_id: i64,
    pub image_url: Option<&'a str>,
    pub is_authenticated: bool,
}

pub fn render_template<T: Template>(template: T) -> Result<String, askama::Error> {
    template.render()
}
