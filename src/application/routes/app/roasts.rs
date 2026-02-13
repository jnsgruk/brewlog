use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::routes::support::load_roaster_options;
use crate::application::state::AppState;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::RoastId;
use crate::presentation::web::templates::{RoastDetailTemplate, RoastEditTemplate};
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

    let image_url = resolve_image_url(&state, EntityType::Roast, i64::from(roast.id)).await;
    let edit_url = format!("/roasts/{}/edit", roast.id);

    let view = RoastDetailView::from_parts(roast, &roaster);

    let template = RoastDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        roast: view,
        roaster_slug,
        image_url,
        edit_url,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn roast_edit_page(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<RoastId>,
) -> Result<Response, StatusCode> {
    let roast = state
        .roast_repo
        .get(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roaster_options = load_roaster_options(&state).await.map_err(map_app_error)?;

    let image_url = resolve_image_url(&state, EntityType::Roast, i64::from(id)).await;

    let template = RoastEditTemplate {
        nav_active: "",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        id: roast.id.to_string(),
        roaster_id: roast.roaster_id.to_string(),
        roaster_name: roaster.name,
        name: roast.name,
        origin: roast.origin.unwrap_or_default(),
        region: roast.region.unwrap_or_default(),
        producer: roast.producer.unwrap_or_default(),
        process: roast.process.unwrap_or_default(),
        tasting_notes: roast.tasting_notes.join(", "),
        roaster_options,
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
