use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::ids::CafeId;
use crate::presentation::web::templates::{CafeDetailTemplate, CafeEditTemplate};
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

    let image_url = resolve_image_url(&state, "cafe", i64::from(cafe.id)).await;
    let edit_url = format!("/cafes/{}/edit", cafe.id);

    let view = CafeDetailView::from_domain(cafe);

    let template = CafeDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        edit_url,
        cafe: view,
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn cafe_edit_page(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<CafeId>,
) -> Result<Response, StatusCode> {
    let cafe = state
        .cafe_repo
        .get(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let image_url = resolve_image_url(&state, "cafe", i64::from(id)).await;

    let template = CafeEditTemplate {
        nav_active: "",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        id: cafe.id.to_string(),
        name: cafe.name,
        city: cafe.city,
        country: cafe.country,
        latitude: cafe.latitude,
        longitude: cafe.longitude,
        website: cafe.website.unwrap_or_default(),
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
