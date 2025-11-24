use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};

use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::{NewRoaster, Roaster, RoasterSortKey, UpdateRoaster};
use crate::presentation::templates::{
    RoasterDetailTemplate, RoasterListTemplate, RoastersTemplate,
};
use crate::presentation::views::{ListNavigator, Paginated, RoastView, RoasterView};
use crate::server::errors::{ApiError, AppError, map_app_error};
use crate::server::routes::render_html;
use crate::server::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request, set_datastar_patch_headers,
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

    let normalized_request = crate::server::routes::support::normalize_request(request, &page);
    let roasters = Paginated::from_page(page, RoasterView::from);
    let navigator =
        ListNavigator::new(ROASTER_PAGE_PATH, ROASTER_FRAGMENT_PATH, normalized_request);

    Ok((roasters, navigator))
}

pub(crate) async fn roasters_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<RoasterSortKey>();

    if is_datastar_request(&headers) {
        return render_roaster_list_fragment(state, request)
            .await
            .map_err(|err| map_app_error(err));
    }

    let (roasters, navigator) = load_roaster_page(&state, request)
        .await
        .map_err(|err| map_app_error(err))?;

    let template = RoastersTemplate {
        nav_active: "roasters",
        roasters,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

pub(crate) async fn roaster_page(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let roaster = state
        .roaster_repo
        .get(id.clone())
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;
    let roasts = state
        .roast_repo
        .list_by_roaster(id)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let roaster_view = RoasterView::from(roaster);

    let template = RoasterDetailTemplate {
        nav_active: "roasters",
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
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoaster>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoasterSortKey>();
    let (new_roaster, source) = payload.into_parts();
    let roaster = new_roaster.normalize().into_roaster();
    let roaster = state
        .roaster_repo
        .insert(roaster)
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
    Path(id): Path<String>,
) -> Result<Json<Roaster>, ApiError> {
    let roaster = state.roaster_repo.get(id).await.map_err(AppError::from)?;
    Ok(Json(roaster))
}

pub(crate) async fn update_roaster(
    State(state): State<AppState>,
    Path(id): Path<String>,
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
    headers: HeaderMap,
    Path(id): Path<String>,
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

    let html = crate::presentation::templates::render_template(template)
        .map_err(|err| AppError::unexpected(format!("failed to render roaster list: {err}")))?;

    let mut response = Html(html).into_response();
    set_datastar_patch_headers(response.headers_mut(), "#roaster-list");
    Ok(response)
}
