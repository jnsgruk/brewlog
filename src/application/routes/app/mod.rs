mod add;
mod admin;
pub(super) mod auth;
mod bags;
mod brews;
mod cafes;
mod checkin;
mod cups;
mod data;
mod gear;
mod home;
mod roasters;
mod roasts;
mod stats;
mod timeline;
mod webauthn;

use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};

use crate::application::state::AppState;

pub(super) fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(home::home_page))
        .route("/login", get(auth::login_page))
        .route("/logout", post(auth::logout))
        .route("/admin", get(admin::admin_page))
        .route("/register/{token}", get(webauthn::register_page))
        .route("/auth/cli-callback", get(webauthn::cli_callback_page))
        .route("/data", get(data::data_page))
        .route("/add", get(add::add_page))
        .route("/scan", get(scan_redirect))
        .route("/check-in", get(checkin::checkin_page))
        .route("/timeline", get(timeline::timeline_page))
        .route("/stats", get(stats::stats_page))
        .route("/bags/{id}", get(bags::bag_detail_page))
        .route("/bags/{id}/edit", get(bags::bag_edit_page))
        .route("/brews/{id}", get(brews::brew_detail_page))
        .route("/brews/{id}/edit", get(brews::brew_edit_page))
        .route("/cafes/{slug}", get(cafes::cafe_detail_page))
        .route("/cafes/{id}/edit", get(cafes::cafe_edit_page))
        .route("/cups/{id}", get(cups::cup_detail_page))
        .route("/cups/{id}/edit", get(cups::cup_edit_page))
        .route("/gear/{id}", get(gear::gear_detail_page))
        .route("/gear/{id}/edit", get(gear::gear_edit_page))
        .route("/roasters/{slug}", get(roasters::roaster_detail_page))
        .route("/roasters/{id}/edit", get(roasters::roaster_edit_page))
        .route(
            "/roasters/{roaster_slug}/roasts/{roast_slug}",
            get(roasts::roast_detail_page),
        )
        .route("/roasts/{id}/edit", get(roasts::roast_edit_page))
        .route("/static/css/styles.css", get(styles))
        .route("/static/js/webauthn.js", get(webauthn_js))
        .route(
            "/static/js/components/photo-capture.js",
            get(photo_capture_js),
        )
        .route(
            "/static/js/components/searchable-select.js",
            get(searchable_select_js),
        )
        .route("/static/js/components/chip-scroll.js", get(chip_scroll_js))
        .route("/static/js/location.js", get(location_js))
        .route("/static/js/image-utils.js", get(image_utils_js))
        .route("/static/js/components/world-map.js", get(world_map_js))
        .route("/static/js/components/donut-chart.js", get(donut_chart_js))
        .route(
            "/static/js/components/image-upload.js",
            get(image_upload_js),
        )
        .route("/static/favicon-light.svg", get(favicon_light))
        .route("/static/favicon-dark.svg", get(favicon_dark))
        .route("/static/og-image.png", get(og_image))
        .route("/static/app-icon-192.png", get(app_icon_192))
        .route("/static/app-icon-512.png", get(app_icon_512))
        .route("/static/site.webmanifest", get(site_webmanifest))
        .route("/health", get(health))
}

async fn scan_redirect() -> Redirect {
    Redirect::permanent("/")
}

/// Generate a static-asset handler that returns embedded file content with a
/// one-week cache header.
macro_rules! static_asset {
    ($name:ident, $content_type:expr, str $path:expr) => {
        async fn $name() -> impl IntoResponse {
            (
                [
                    ("content-type", $content_type),
                    ("cache-control", "public, max-age=604800"),
                ],
                include_str!($path),
            )
        }
    };
    ($name:ident, $content_type:expr, bytes $path:expr) => {
        async fn $name() -> impl IntoResponse {
            (
                [
                    ("content-type", $content_type),
                    ("cache-control", "public, max-age=604800"),
                ],
                include_bytes!($path).as_slice(),
            )
        }
    };
}

static_asset!(styles, "text/css; charset=utf-8", str "../../../../static/css/styles.css");
static_asset!(webauthn_js, "application/javascript; charset=utf-8", str "../../../../static/js/webauthn.js");
static_asset!(location_js, "application/javascript; charset=utf-8", str "../../../../static/js/location.js");
static_asset!(image_utils_js, "application/javascript; charset=utf-8", str "../../../../static/js/image-utils.js");
static_asset!(photo_capture_js, "application/javascript; charset=utf-8", str "../../../../static/js/components/photo-capture.js");
static_asset!(searchable_select_js, "application/javascript; charset=utf-8", str "../../../../static/js/components/searchable-select.js");
static_asset!(chip_scroll_js, "application/javascript; charset=utf-8", str "../../../../static/js/components/chip-scroll.js");
static_asset!(world_map_js, "application/javascript; charset=utf-8", str "../../../../static/js/components/world-map.js");
static_asset!(donut_chart_js, "application/javascript; charset=utf-8", str "../../../../static/js/components/donut-chart.js");
static_asset!(image_upload_js, "application/javascript; charset=utf-8", str "../../../../static/js/components/image-upload.js");
static_asset!(favicon_light, "image/svg+xml", str "../../../../static/favicon-light.svg");
static_asset!(favicon_dark, "image/svg+xml", str "../../../../static/favicon-dark.svg");
static_asset!(og_image, "image/png", bytes "../../../../static/og-image.png");
static_asset!(app_icon_192, "image/png", bytes "../../../../static/app-icon-192.png");
static_asset!(app_icon_512, "image/png", bytes "../../../../static/app-icon-512.png");
static_asset!(site_webmanifest, "application/manifest+json; charset=utf-8", str "../../../../static/site.webmanifest");

async fn health() -> impl IntoResponse {
    ([("content-type", "application/json")], r#"{"status":"ok"}"#)
}
