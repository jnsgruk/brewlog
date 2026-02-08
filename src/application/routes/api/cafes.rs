use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use super::macros::{define_delete_handler, define_get_handler, define_list_fragment_renderer};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::state::AppState;
use crate::domain::cafes::{Cafe, CafeSortKey, NewCafe, UpdateCafe};
use crate::domain::ids::CafeId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::infrastructure::foursquare;
use crate::presentation::web::templates::{CafeListTemplate, NearbyCafesFragment};
use crate::presentation::web::views::{CafeView, ListNavigator, NearbyCafeView, Paginated};
use tracing::info;

const CAFE_PAGE_PATH: &str = "/data?type=cafes";
const CAFE_FRAGMENT_PATH: &str = "/data?type=cafes#cafe-list";

#[tracing::instrument(skip(state))]
pub(crate) async fn load_cafe_page(
    state: &AppState,
    request: ListRequest<CafeSortKey>,
    search: Option<&str>,
) -> Result<(Paginated<CafeView>, ListNavigator<CafeSortKey>), AppError> {
    let page = state
        .cafe_repo
        .list(&request, search)
        .await
        .map_err(AppError::from)?;

    Ok(crate::application::routes::support::build_page_view(
        page,
        request,
        CafeView::from,
        CAFE_PAGE_PATH,
        CAFE_FRAGMENT_PATH,
        search.map(String::from),
    ))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn list_cafes(State(state): State<AppState>) -> Result<Json<Vec<Cafe>>, ApiError> {
    let cafes = state
        .cafe_repo
        .list_all_sorted(CafeSortKey::Name, SortDirection::Asc)
        .await
        .map_err(AppError::from)?;
    Ok(Json(cafes))
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_cafe(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewCafe>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<CafeSortKey>();
    let (new_cafe, source) = payload.into_parts();
    let new_cafe = new_cafe.normalize();
    let cafe = state
        .cafe_service
        .create(new_cafe)
        .await
        .map_err(AppError::from)?;

    info!(cafe_id = %cafe.id, name = %cafe.name, "cafe created");
    state.stats_invalidator.invalidate();

    let detail_url = format!("/cafes/{}", cafe.slug);

    if is_datastar_request(&headers) {
        let from_data_page = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|r| r.contains("type=cafes"));

        if from_data_page {
            render_cafe_list_fragment(state, request, search, true)
                .await
                .map_err(ApiError::from)
        } else {
            use axum::http::header::HeaderValue;
            let script = format!("<script>window.location.href='{detail_url}'</script>");
            let mut response = axum::response::Html(script).into_response();
            response
                .headers_mut()
                .insert("datastar-selector", HeaderValue::from_static("body"));
            response
                .headers_mut()
                .insert("datastar-mode", HeaderValue::from_static("append"));
            Ok(response)
        }
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(&detail_url).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(cafe)).into_response())
    }
}

define_get_handler!(get_cafe, CafeId, Cafe, cafe_repo);

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn update_cafe(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<CafeId>,
    Json(payload): Json<UpdateCafe>,
) -> Result<Json<Cafe>, ApiError> {
    let has_changes = payload.name.is_some()
        || payload.city.is_some()
        || payload.country.is_some()
        || payload.latitude.is_some()
        || payload.longitude.is_some()
        || payload.website.is_some()
        || payload.created_at.is_some();

    if !has_changes {
        return Err(AppError::validation("no changes provided").into());
    }

    let cafe = state
        .cafe_repo
        .update(id, payload)
        .await
        .map_err(AppError::from)?;
    info!(%id, "cafe updated");
    state.stats_invalidator.invalidate();
    Ok(Json(cafe))
}

define_delete_handler!(
    delete_cafe,
    CafeId,
    CafeSortKey,
    cafe_repo,
    render_cafe_list_fragment
);

define_list_fragment_renderer!(
    render_cafe_list_fragment,
    CafeSortKey,
    load_cafe_page,
    CafeListTemplate { cafes },
    "#cafe-list"
);

#[derive(Debug, Deserialize)]
pub struct NearbyQuery {
    lat: Option<f64>,
    lng: Option<f64>,
    q: String,
    near: Option<String>,
}

#[tracing::instrument(skip(state, _auth_user, headers))]
pub(crate) async fn nearby_cafes(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<NearbyQuery>,
) -> Result<Response, ApiError> {
    let q = query.q.trim();
    if q.is_empty() || q.len() < 2 {
        return Err(AppError::validation("q must be at least 2 characters").into());
    }

    let location = if let Some(near) = &query.near {
        let near = near.trim();
        if near.len() < 2 {
            return Err(AppError::validation("near must be at least 2 characters").into());
        }
        foursquare::SearchLocation::Near(near.to_string())
    } else {
        let lat = query
            .lat
            .ok_or_else(|| AppError::validation("lat is required when near is not provided"))?;
        let lng = query
            .lng
            .ok_or_else(|| AppError::validation("lng is required when near is not provided"))?;
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
            return Err(AppError::validation("lat must be -90..90, lng must be -180..180").into());
        }
        foursquare::SearchLocation::Coordinates { lat, lng }
    };

    let cafes = foursquare::search_nearby(
        &state.http_client,
        &state.foursquare_url,
        &state.foursquare_api_key,
        &location,
        q,
    )
    .await
    .map_err(ApiError::from)?;

    if is_datastar_request(&headers) {
        let views: Vec<NearbyCafeView> = cafes.into_iter().map(NearbyCafeView::from).collect();
        let template = NearbyCafesFragment { cafes: views };
        crate::application::routes::support::render_fragment(template, "#nearby-results")
            .map_err(ApiError::from)
    } else {
        Ok(Json(cafes).into_response())
    }
}
