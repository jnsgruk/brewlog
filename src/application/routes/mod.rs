pub mod api;
pub mod app;
pub mod support;

pub(crate) use app::auth::is_authenticated;

use askama::Template;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Html;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing::error;

use crate::application::state::AppState;

use crate::presentation::web::templates::render_template;

/// 5 MB request body limit.
const BODY_LIMIT_BYTES: usize = 5 * 1024 * 1024;

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
                .layer(CookieManagerLayer::new())
                .layer(RequestBodyLimitLayer::new(BODY_LIMIT_BYTES))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_CONTENT_TYPE_OPTIONS,
                    HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_FRAME_OPTIONS,
                    HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::REFERRER_POLICY,
                    HeaderValue::from_static("strict-origin-when-cross-origin"),
                ))
                // Datastar v1 evaluates expressions via Function(), requiring 'unsafe-eval'.
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static(
                        "default-src 'self'; \
                         script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net; \
                         style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                         font-src 'self' https://fonts.gstatic.com; \
                         img-src 'self' data: blob:; \
                         frame-ancestors 'none'",
                    ),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::STRICT_TRANSPORT_SECURITY,
                    HeaderValue::from_static("max-age=63072000; includeSubDomains"),
                ))
                .layer(CompressionLayer::new().gzip(true)),
        )
        .with_state(state)
}

pub(crate) fn render_html<T: Template>(template: T) -> Result<Html<String>, StatusCode> {
    render_template(template).map(Html).map_err(|err| {
        error!(error = %err, "failed to render template");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
