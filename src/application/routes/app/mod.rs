mod account;
mod add;
pub(super) mod auth;
mod checkin;
mod data;
mod home;
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
        .route("/account", get(account::account_page))
        .route("/register/:token", get(webauthn::register_page))
        .route("/auth/cli-callback", get(webauthn::cli_callback_page))
        .route("/data", get(data::data_page))
        .route("/add", get(add::add_page))
        .route("/scan", get(scan_redirect))
        .route("/check-in", get(checkin::checkin_page))
        .route("/timeline", get(timeline::timeline_page))
        .route("/styles.css", get(styles))
        .route("/webauthn.js", get(webauthn_js))
        .route("/components/photo-capture.js", get(photo_capture_js))
        .route(
            "/components/searchable-select.js",
            get(searchable_select_js),
        )
        .route("/favicon.ico", get(favicon))
        .route("/favicon-light.svg", get(favicon_light))
        .route("/favicon-dark.svg", get(favicon_dark))
}

async fn scan_redirect() -> Redirect {
    Redirect::permanent("/")
}

async fn styles() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../../../../static/css/styles.css"),
    )
}

async fn webauthn_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        include_str!("../../../../static/js/webauthn.js"),
    )
}

async fn photo_capture_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        include_str!("../../../../static/js/components/photo-capture.js"),
    )
}

async fn searchable_select_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        include_str!("../../../../static/js/components/searchable-select.js"),
    )
}

async fn favicon() -> impl IntoResponse {
    (
        [("content-type", "image/x-icon")],
        include_bytes!("../../../../static/favicon.ico").as_ref(),
    )
}

async fn favicon_light() -> impl IntoResponse {
    (
        [("content-type", "image/svg+xml")],
        include_str!("../../../../static/favicon-light.svg"),
    )
}

async fn favicon_dark() -> impl IntoResponse {
    (
        [("content-type", "image/svg+xml")],
        include_str!("../../../../static/favicon-dark.svg"),
    )
}
