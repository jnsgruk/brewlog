use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::presentation::web::templates::CafeDetailTemplate;
use crate::presentation::web::views::CafeDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn cafe_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let cafe = state
        .cafe_repo
        .get_by_slug(&slug)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let view = CafeDetailView::from_domain(cafe);

    let template = CafeDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        cafe: view,
    };

    render_html(template).map(IntoResponse::into_response)
}
