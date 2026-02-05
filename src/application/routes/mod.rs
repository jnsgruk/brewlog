pub mod account;
pub mod add;
pub mod auth;
pub mod backup;
pub mod bags;
pub mod brews;
pub mod cafes;
pub mod checkin;
pub mod cups;
pub mod data;
pub mod gear;
pub mod home;
mod macros;
pub mod roasters;
pub mod roasts;
pub mod scan;
pub mod support;
pub mod timeline;
pub mod tokens;
pub mod webauthn;

pub(crate) use auth::is_authenticated;

use askama::Template;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tracing::error;

use crate::application::server::AppState;

use crate::presentation::web::templates::render_template;

#[allow(clippy::too_many_lines)]
pub fn app_router(state: AppState) -> axum::Router {
    let api_routes = axum::Router::new()
        // Public API routes
        .route(
            "/roasters",
            get(roasters::list_roasters).post(roasters::create_roaster),
        )
        .route(
            "/roasters/:id",
            get(roasters::get_roaster)
                .put(roasters::update_roaster)
                .delete(roasters::delete_roaster),
        )
        .route(
            "/roasts",
            get(roasts::list_roasts).post(roasts::create_roast),
        )
        .route(
            "/roasts/:id",
            get(roasts::get_roast)
                .put(roasts::update_roast)
                .delete(roasts::delete_roast),
        )
        .route("/bags", get(bags::list_bags).post(bags::create_bag))
        .route(
            "/bags/:id",
            get(bags::get_bag)
                .put(bags::update_bag)
                .delete(bags::delete_bag),
        )
        .route("/gear", get(gear::list_gear).post(gear::create_gear))
        .route(
            "/gear/:id",
            get(gear::get_gear)
                .put(gear::update_gear)
                .delete(gear::delete_gear),
        )
        .route("/brews", get(brews::list_brews).post(brews::create_brew))
        .route(
            "/brews/:id",
            get(brews::get_brew).delete(brews::delete_brew),
        )
        .route("/cafes", get(cafes::list_cafes).post(cafes::create_cafe))
        .route(
            "/cafes/:id",
            get(cafes::get_cafe)
                .put(cafes::update_cafe)
                .delete(cafes::delete_cafe),
        )
        .route("/nearby-cafes", get(cafes::nearby_cafes))
        .route("/extract-roaster", post(roasters::extract_roaster))
        .route("/extract-roast", post(roasts::extract_roast_info))
        .route("/extract-bag-scan", post(scan::extract_bag_scan))
        .route("/scan", post(scan::submit_scan))
        .route("/check-in", post(checkin::submit_checkin))
        .route("/cups", get(cups::list_cups).post(cups::create_cup))
        .route("/cups/:id", get(cups::get_cup).delete(cups::delete_cup))
        .route(
            "/tokens",
            post(tokens::create_token).get(tokens::list_tokens),
        )
        .route("/tokens/:id/revoke", post(tokens::revoke_token))
        .route("/passkeys", get(account::list_passkeys))
        .route(
            "/passkeys/:id",
            axum::routing::delete(account::delete_passkey),
        )
        .route("/backup", get(backup::export_backup))
        .route(
            "/backup/restore",
            post(backup::restore_backup).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        );

    let webauthn_routes = axum::Router::new()
        .route("/register/start", post(webauthn::register_start))
        .route("/register/finish", post(webauthn::register_finish))
        .route("/auth/start", get(webauthn::auth_start))
        .route("/auth/finish", post(webauthn::auth_finish))
        .route("/passkey/start", post(webauthn::passkey_add_start))
        .route("/passkey/finish", post(webauthn::passkey_add_finish));

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
        .nest("/api/v1", api_routes)
        .nest("/api/v1/webauthn", webauthn_routes)
        .layer(ServiceBuilder::new().layer(CookieManagerLayer::new()))
        .with_state(state)
}

async fn scan_redirect() -> Redirect {
    Redirect::permanent("/")
}

async fn styles() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../../../templates/styles.css"),
    )
}

async fn webauthn_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        include_str!("../../../templates/webauthn.js"),
    )
}

async fn photo_capture_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        include_str!("../../../templates/components/photo-capture.js"),
    )
}

async fn searchable_select_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        include_str!("../../../templates/components/searchable-select.js"),
    )
}

async fn favicon() -> impl IntoResponse {
    (
        [("content-type", "image/x-icon")],
        include_bytes!("../../../templates/favicon.ico").as_ref(),
    )
}

pub(crate) fn render_html<T: Template>(template: T) -> Result<Html<String>, StatusCode> {
    render_template(template).map(Html).map_err(|err| {
        error!(error = %err, "failed to render template");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
