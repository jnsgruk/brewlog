use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use super::macros::{define_delete_handler, define_get_handler, define_list_fragment_renderer};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::cafes::{Cafe, CafeSortKey, NewCafe, UpdateCafe};
use crate::domain::ids::CafeId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::infrastructure::foursquare::{self, NearbyCafe};
use crate::presentation::web::templates::{CafeDetailTemplate, CafeListTemplate, CafesTemplate};
use crate::presentation::web::views::{CafeView, ListNavigator, Paginated};

const CAFE_PAGE_PATH: &str = "/cafes";
const CAFE_FRAGMENT_PATH: &str = "/cafes#cafe-list";

#[tracing::instrument(skip(state))]
async fn load_cafe_page(
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

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn cafes_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let (request, search) = query.into_request_and_search::<CafeSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_cafe_list_fragment(state, request, search, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let (cafes, navigator) = load_cafe_page(&state, request, search.as_deref())
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = CafesTemplate {
        nav_active: "cafes",
        is_authenticated,
        has_ai_extract: state.has_ai_extract(),
        cafes,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn cafe_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    let cafe = state
        .cafe_repo
        .get_by_slug(&slug)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let cafe_view = CafeView::from(cafe);
    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = CafeDetailTemplate {
        nav_active: "cafes",
        is_authenticated,
        has_ai_extract: state.has_ai_extract(),
        cafe: cafe_view,
    };

    render_html(template).map(IntoResponse::into_response)
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
        .cafe_repo
        .insert(new_cafe)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_cafe_list_fragment(state, request, search, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target =
            ListNavigator::new(CAFE_PAGE_PATH, CAFE_FRAGMENT_PATH, request, search).page_href(1);
        Ok(Redirect::to(&target).into_response())
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
        || payload.notes.is_some();

    if !has_changes {
        return Err(AppError::validation("no changes provided").into());
    }

    let cafe = state
        .cafe_repo
        .update(id, payload)
        .await
        .map_err(AppError::from)?;
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
    lat: f64,
    lng: f64,
    q: String,
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn nearby_cafes(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Query(query): Query<NearbyQuery>,
) -> Result<Json<Vec<NearbyCafe>>, ApiError> {
    if !(-90.0..=90.0).contains(&query.lat) || !(-180.0..=180.0).contains(&query.lng) {
        return Err(AppError::validation("lat must be -90..90, lng must be -180..180").into());
    }

    let q = query.q.trim();
    if q.is_empty() || q.len() < 2 {
        return Err(AppError::validation("q must be at least 2 characters").into());
    }

    let api_key = state
        .foursquare_api_key
        .as_deref()
        .ok_or_else(|| AppError::unexpected("Foursquare API key not configured"))?;

    let cafes = foursquare::search_nearby(
        &state.http_client,
        &state.foursquare_url,
        api_key,
        query.lat,
        query.lng,
        q,
    )
    .await
    .map_err(ApiError::from)?;
    Ok(Json(cafes))
}
