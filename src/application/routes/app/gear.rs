use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::ids::GearId;
use crate::presentation::web::templates::GearDetailTemplate;
use crate::presentation::web::views::GearDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn gear_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<GearId>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let gear = state
        .gear_repo
        .get(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let image_url = resolve_image_url(&state, "gear", i64::from(id)).await;

    let view = GearDetailView::from_domain(gear);

    let template = GearDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        gear: view,
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
