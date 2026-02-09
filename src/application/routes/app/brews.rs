use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::ids::BrewId;
use crate::presentation::web::templates::BrewDetailTemplate;
use crate::presentation::web::views::BrewDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn brew_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<BrewId>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let brew_details = state
        .brew_repo
        .get_with_details(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let bag = state
        .bag_repo
        .get(brew_details.brew.bag_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roast = state
        .roast_repo
        .get(bag.roast_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let view = BrewDetailView::from_parts(brew_details, &roast, &roaster);

    let template = BrewDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        brew: view,
        roaster_slug: roaster.slug.clone(),
        roast_slug: roast.slug.clone(),
    };

    render_html(template).map(IntoResponse::into_response)
}
