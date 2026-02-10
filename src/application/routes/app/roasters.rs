use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::presentation::web::templates::RoasterDetailTemplate;
use crate::presentation::web::views::RoasterDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn roaster_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let roaster = state
        .roaster_repo
        .get_by_slug(&slug)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let image_url = resolve_image_url(&state, "roaster", i64::from(roaster.id)).await;
    let edit_url = format!("/roasters/{}/edit", roaster.id);

    let view = RoasterDetailView::from_domain(roaster);

    let template = RoasterDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        roaster: view,
        image_url,
        edit_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
