mod add;
mod admin;
pub(super) mod auth;
mod brews;
mod checkin;
mod cups;
mod data;
mod home;
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
        .route("/brews/{id}", get(brews::brew_detail_page))
        .route("/cups/{id}", get(cups::cup_detail_page))
        .route("/styles.css", get(styles))
        .route("/webauthn.js", get(webauthn_js))
        .route("/components/photo-capture.js", get(photo_capture_js))
        .route(
            "/components/searchable-select.js",
            get(searchable_select_js),
        )
        .route("/components/chip-scroll.js", get(chip_scroll_js))
        .route("/components/world-map.js", get(world_map_js))
        .route("/components/donut-chart.js", get(donut_chart_js))
        .route("/favicon-light.svg", get(favicon_light))
        .route("/favicon-dark.svg", get(favicon_dark))
        .route("/health", get(health))
}

async fn scan_redirect() -> Redirect {
    Redirect::permanent("/")
}

async fn styles() -> impl IntoResponse {
    (
        [
            ("content-type", "text/css; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/css/styles.css"),
    )
}

async fn webauthn_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/js/webauthn.js"),
    )
}

async fn photo_capture_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/js/components/photo-capture.js"),
    )
}

async fn searchable_select_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/js/components/searchable-select.js"),
    )
}

async fn chip_scroll_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/js/components/chip-scroll.js"),
    )
}

async fn world_map_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/js/components/world-map.js"),
    )
}

async fn donut_chart_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/js/components/donut-chart.js"),
    )
}

async fn favicon_light() -> impl IntoResponse {
    (
        [
            ("content-type", "image/svg+xml"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/favicon-light.svg"),
    )
}

async fn favicon_dark() -> impl IntoResponse {
    (
        [
            ("content-type", "image/svg+xml"),
            ("cache-control", "public, max-age=604800"),
        ],
        include_str!("../../../../static/favicon-dark.svg"),
    )
}

async fn health() -> impl IntoResponse {
    ([("content-type", "application/json")], r#"{"status":"ok"}"#)
}
