use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};

use crate::domain::ids::RoasterId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::{NewRoaster, Roaster, RoasterSortKey, UpdateRoaster};
use crate::presentation::templates::{
    RoasterDetailTemplate, RoasterListTemplate, RoastersTemplate,
};
use crate::presentation::views::{ListNavigator, Paginated, RoastView, RoasterView};
use crate::server::auth::AuthenticatedUser;
use crate::server::errors::{ApiError, AppError, map_app_error};
use crate::server::routes::render_html;
use crate::server::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::server::server::AppState;

const ROASTER_PAGE_PATH: &str = "/roasters";
const ROASTER_FRAGMENT_PATH: &str = "/roasters#roaster-list";

async fn load_roaster_page(
    state: &AppState,
    request: ListRequest<RoasterSortKey>,
) -> Result<(Paginated<RoasterView>, ListNavigator<RoasterSortKey>), AppError> {
    let page = state
        .roaster_repo
        .list(&request)
        .await
        .map_err(AppError::from)?;

    Ok(crate::server::routes::support::build_page_view(
        page,
        request,
        RoasterView::from,
        ROASTER_PAGE_PATH,
        ROASTER_FRAGMENT_PATH,
    ))
}

pub(crate) async fn roasters_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<RoasterSortKey>();

    if is_datastar_request(&headers) {
        return render_roaster_list_fragment(state, request)
            .await
            .map_err(map_app_error);
    }

    let (roasters, navigator) = load_roaster_page(&state, request)
        .await
        .map_err(map_app_error)?;

    let is_authenticated = crate::server::routes::auth::is_authenticated(&state, &cookies).await;

    let template = RoastersTemplate {
        nav_active: "roasters",
        is_authenticated,
        roasters,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

pub(crate) async fn roaster_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let roaster = state
        .roaster_repo
        .get_by_slug(&slug)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;
    let roasts = state
        .roast_repo
        .list_by_roaster(roaster.id)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let roaster_view = RoasterView::from(roaster);
    let is_authenticated = crate::server::routes::auth::is_authenticated(&state, &cookies).await;

    let template = RoasterDetailTemplate {
        nav_active: "roasters",
        is_authenticated,
        roaster: roaster_view,
        roasts: roasts.into_iter().map(RoastView::from_list_item).collect(),
    };

    render_html(template)
}

pub(crate) async fn list_roasters(
    State(state): State<AppState>,
) -> Result<Json<Vec<Roaster>>, ApiError> {
    let roasters = state
        .roaster_repo
        .list_all_sorted(RoasterSortKey::Name, SortDirection::Asc)
        .await
        .map_err(AppError::from)?;
    Ok(Json(roasters))
}

pub(crate) async fn create_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoaster>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoasterSortKey>();
    let (new_roaster, source) = payload.into_parts();
    let new_roaster = new_roaster.normalize();
    let roaster = state
        .roaster_repo
        .insert(new_roaster)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roaster_list_fragment(state, request)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target =
            ListNavigator::new(ROASTER_PAGE_PATH, ROASTER_FRAGMENT_PATH, request).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(roaster)).into_response())
    }
}

pub(crate) async fn get_roaster(
    State(state): State<AppState>,
    Path(id): Path<RoasterId>,
) -> Result<Json<Roaster>, ApiError> {
    let roaster = state.roaster_repo.get(id).await.map_err(AppError::from)?;
    Ok(Json(roaster))
}

pub(crate) async fn update_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<RoasterId>,
    Json(payload): Json<UpdateRoaster>,
) -> Result<Json<Roaster>, ApiError> {
    let has_changes = payload.name.is_some()
        || payload.country.is_some()
        || payload.city.is_some()
        || payload.homepage.is_some()
        || payload.notes.is_some();

    if !has_changes {
        return Err(AppError::validation("no changes provided").into());
    }

    let roaster = state
        .roaster_repo
        .update(id, payload)
        .await
        .map_err(AppError::from)?;
    Ok(Json(roaster))
}

pub(crate) async fn delete_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<RoasterId>,
    Query(query): Query<ListQuery>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoasterSortKey>();
    state
        .roaster_repo
        .delete(id)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roaster_list_fragment(state, request)
            .await
            .map_err(ApiError::from)
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

async fn render_roaster_list_fragment(
    state: AppState,
    request: ListRequest<RoasterSortKey>,
) -> Result<Response, AppError> {
    let (roasters, navigator) = load_roaster_page(&state, request).await?;

    let template = RoasterListTemplate {
        roasters,
        navigator,
    };

    crate::server::routes::support::render_fragment(template, "#roaster-list")
}
