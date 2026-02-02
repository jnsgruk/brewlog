pub mod auth;
pub mod bags;
pub mod brews;
pub mod gear;
mod macros;
pub mod roasters;
pub mod roasts;
pub mod support;
pub mod timeline;
pub mod tokens;

pub(crate) use auth::is_authenticated;

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tracing::error;

use crate::application::server::AppState;

use crate::presentation::web::templates::render_template;

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
        .route(
            "/tokens",
            post(tokens::create_token).get(tokens::list_tokens),
        )
        .route("/tokens/:id/revoke", post(tokens::revoke_token));

    axum::Router::new()
        .route("/", get(root_redirect))
        .route("/login", get(auth::login_page).post(auth::login_submit))
        .route("/logout", post(auth::logout))
        .route("/roasters", get(roasters::roasters_page))
        .route("/roasters/:slug", get(roasters::roaster_page))
        .route("/roasts", get(roasts::roasts_page))
        .route(
            "/roasters/:roaster_slug/roasts/:roast_slug",
            get(roasts::roast_page),
        )
        .route("/bags", get(bags::bags_page))
        .route("/brews", get(brews::brews_page))
        .route("/gear", get(gear::gear_page))
        .route("/timeline", get(timeline::timeline_page))
        .route("/styles.css", get(styles))
        .route("/favicon.ico", get(favicon))
        .nest("/api/v1", api_routes)
        .layer(ServiceBuilder::new().layer(CookieManagerLayer::new()))
        .with_state(state)
}

async fn root_redirect() -> Redirect {
    Redirect::temporary("/timeline")
}

async fn styles() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        include_str!("../../../templates/styles.css"),
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
