use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::presentation::web::templates::RoastDetailTemplate;
use crate::presentation::web::views::RoastDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn roast_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((roaster_slug, roast_slug)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let roaster = state
        .roaster_repo
        .get_by_slug(&roaster_slug)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roast = state
        .roast_repo
        .get_by_slug(roaster.id, &roast_slug)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let image_url = resolve_image_url(&state, "roast", i64::from(roast.id)).await;

    let view = RoastDetailView::from_parts(roast, &roaster);

    let template = RoastDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        roast: view,
        roaster_slug,
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
