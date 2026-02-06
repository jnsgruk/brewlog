pub mod api;
pub mod app;
pub mod support;

pub(crate) use app::auth::is_authenticated;

use askama::Template;
use axum::http::StatusCode;
use axum::response::Html;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing::error;

use crate::application::state::AppState;

use crate::presentation::web::templates::render_template;

pub fn app_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .merge(app::router())
        .nest("/api/v1", api::router())
        .nest("/api/v1/webauthn", api::webauthn_router())
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(CookieManagerLayer::new()),
        )
        .with_state(state)
}

pub(crate) fn render_html<T: Template>(template: T) -> Result<Html<String>, StatusCode> {
    render_template(template).map(Html).map_err(|err| {
        error!(error = %err, "failed to render template");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
