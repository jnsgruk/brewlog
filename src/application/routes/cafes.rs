use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};

use super::macros::{define_delete_handler, define_get_handler};
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

async fn render_cafe_list_fragment(
    state: AppState,
    request: ListRequest<CafeSortKey>,
    search: Option<String>,
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let (cafes, navigator) = load_cafe_page(&state, request, search.as_deref()).await?;

    let template = CafeListTemplate {
        is_authenticated,
        cafes,
        navigator,
    };

    crate::application::routes::support::render_fragment(template, "#cafe-list")
}
