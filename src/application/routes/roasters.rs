use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};

use super::macros::{define_delete_handler, define_get_handler, define_list_fragment_renderer};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::ids::RoasterId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::{NewRoaster, Roaster, RoasterSortKey, UpdateRoaster};
use crate::infrastructure::ai::{self, ExtractionInput};
use crate::presentation::web::templates::{
    RoasterDetailTemplate, RoasterListTemplate, RoastersTemplate,
};
use crate::presentation::web::views::{ListNavigator, Paginated, RoastView, RoasterView};

const ROASTER_PAGE_PATH: &str = "/roasters";
const ROASTER_FRAGMENT_PATH: &str = "/roasters#roaster-list";

#[tracing::instrument(skip(state))]
async fn load_roaster_page(
    state: &AppState,
    request: ListRequest<RoasterSortKey>,
    search: Option<&str>,
) -> Result<(Paginated<RoasterView>, ListNavigator<RoasterSortKey>), AppError> {
    let page = state
        .roaster_repo
        .list(&request, search)
        .await
        .map_err(AppError::from)?;

    Ok(crate::application::routes::support::build_page_view(
        page,
        request,
        RoasterView::from,
        ROASTER_PAGE_PATH,
        ROASTER_FRAGMENT_PATH,
        search.map(String::from),
    ))
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn roasters_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let (request, search) = query.into_request_and_search::<RoasterSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_roaster_list_fragment(state, request, search, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let (roasters, navigator) = load_roaster_page(&state, request, search.as_deref())
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = RoastersTemplate {
        nav_active: "roasters",
        is_authenticated,
        roasters,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, cookies))]
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
    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = RoasterDetailTemplate {
        nav_active: "roasters",
        is_authenticated,
        roaster: roaster_view,
        roasts: roasts.into_iter().map(RoastView::from_list_item).collect(),
    };

    render_html(template)
}

#[tracing::instrument(skip(state))]
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

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoaster>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<RoasterSortKey>();
    let (new_roaster, source) = payload.into_parts();
    let new_roaster = new_roaster.normalize();
    let roaster = state
        .roaster_repo
        .insert(new_roaster)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roaster_list_fragment(state, request, search, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target = ListNavigator::new(ROASTER_PAGE_PATH, ROASTER_FRAGMENT_PATH, request, search)
            .page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(roaster)).into_response())
    }
}

define_get_handler!(get_roaster, RoasterId, Roaster, roaster_repo);

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn update_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<RoasterId>,
    Json(payload): Json<UpdateRoaster>,
) -> Result<Json<Roaster>, ApiError> {
    let has_changes = payload.name.is_some()
        || payload.country.is_some()
        || payload.city.is_some()
        || payload.homepage.is_some();

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

define_delete_handler!(
    delete_roaster,
    RoasterId,
    RoasterSortKey,
    roaster_repo,
    render_roaster_list_fragment
);

#[tracing::instrument(skip(state, _auth_user, headers, payload))]
pub(crate) async fn extract_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    payload: FlexiblePayload<ExtractionInput>,
) -> Result<Response, ApiError> {
    let (input, _) = payload.into_parts();
    let result = ai::extract_roaster(
        &state.http_client,
        &state.openrouter_api_key,
        &state.openrouter_model,
        &input,
    )
    .await
    .map_err(ApiError::from)?;

    if is_datastar_request(&headers) {
        use serde_json::Value;
        let signals = vec![
            (
                "_roaster-name",
                Value::String(result.name.unwrap_or_default()),
            ),
            (
                "_roaster-country",
                Value::String(result.country.unwrap_or_default()),
            ),
            (
                "_roaster-city",
                Value::String(result.city.unwrap_or_default()),
            ),
            (
                "_roaster-homepage",
                Value::String(result.homepage.unwrap_or_default()),
            ),
            ("_extracted", Value::Bool(true)),
        ];
        crate::application::routes::support::render_signals_json(&signals).map_err(ApiError::from)
    } else {
        Ok(Json(result).into_response())
    }
}

define_list_fragment_renderer!(
    render_roaster_list_fragment,
    RoasterSortKey,
    load_roaster_page,
    RoasterListTemplate { roasters },
    "#roaster-list"
);
