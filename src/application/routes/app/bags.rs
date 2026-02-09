use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::ids::BagId;
use crate::presentation::web::templates::BagDetailTemplate;
use crate::presentation::web::views::BagDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn bag_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<BagId>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let bag = state
        .bag_repo
        .get_with_roast(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roast = state
        .roast_repo
        .get(bag.bag.roast_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let view = BagDetailView::from_parts(bag, &roast, &roaster);

    let template = BagDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        bag: view,
        roaster_slug: roaster.slug.clone(),
        roast_slug: roast.slug.clone(),
    };

    render_html(template).map(IntoResponse::into_response)
}
