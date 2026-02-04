use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};

use super::macros::{
    define_delete_handler, define_enriched_get_handler, define_list_fragment_renderer,
};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request, load_cafe_options,
    load_roast_options,
};
use crate::application::server::AppState;
use crate::domain::cups::{Cup, CupFilter, CupSortKey, CupWithDetails, NewCup, UpdateCup};
use crate::domain::ids::CupId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::presentation::web::templates::{CupListTemplate, CupsTemplate};
use crate::presentation::web::views::{CupView, ListNavigator, Paginated};

const CUP_PAGE_PATH: &str = "/cups";
const CUP_FRAGMENT_PATH: &str = "/cups#cup-list";

#[tracing::instrument(skip(state))]
async fn load_cup_page(
    state: &AppState,
    request: ListRequest<CupSortKey>,
    search: Option<&str>,
) -> Result<(Paginated<CupView>, ListNavigator<CupSortKey>), AppError> {
    let page = state
        .cup_repo
        .list(CupFilter::all(), &request, search)
        .await
        .map_err(AppError::from)?;

    Ok(crate::application::routes::support::build_page_view(
        page,
        request,
        CupView::from_domain,
        CUP_PAGE_PATH,
        CUP_FRAGMENT_PATH,
        search.map(String::from),
    ))
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn cups_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let (request, search) = query.into_request_and_search::<CupSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_cup_list_fragment(state, request, search, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let (cups, navigator) = load_cup_page(&state, request, search.as_deref())
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let roast_options = load_roast_options(&state).await.map_err(map_app_error)?;

    let cafe_options = load_cafe_options(&state).await.map_err(map_app_error)?;

    let template = CupsTemplate {
        nav_active: "cups",
        is_authenticated,
        cups,
        roast_options,
        cafe_options,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_cup(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewCup>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<CupSortKey>();
    let (new_cup, source) = payload.into_parts();

    if let Some(rating) = new_cup.rating
        && !(1..=5).contains(&rating)
    {
        return Err(AppError::validation("rating must be between 1 and 5").into());
    }

    let cup = state
        .cup_repo
        .insert(new_cup)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_cup_list_fragment(state, request, search, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target =
            ListNavigator::new(CUP_PAGE_PATH, CUP_FRAGMENT_PATH, request, search).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(cup)).into_response())
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn list_cups(
    State(state): State<AppState>,
) -> Result<Json<Vec<CupWithDetails>>, ApiError> {
    let request = ListRequest::show_all(CupSortKey::CreatedAt, SortDirection::Desc);
    let page = state
        .cup_repo
        .list(CupFilter::all(), &request, None)
        .await
        .map_err(AppError::from)?;
    Ok(Json(page.items))
}

define_enriched_get_handler!(get_cup, CupId, CupWithDetails, cup_repo, get_with_details);

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn update_cup(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<CupId>,
    Json(payload): Json<UpdateCup>,
) -> Result<Json<Cup>, ApiError> {
    let has_changes = payload.rating.is_some();

    if !has_changes {
        return Err(AppError::validation("no changes provided").into());
    }

    if let Some(rating) = payload.rating.as_ref()
        && !(1..=5).contains(rating)
    {
        return Err(AppError::validation("rating must be between 1 and 5").into());
    }

    let cup = state
        .cup_repo
        .update(id, payload)
        .await
        .map_err(AppError::from)?;
    Ok(Json(cup))
}

define_delete_handler!(
    delete_cup,
    CupId,
    CupSortKey,
    cup_repo,
    render_cup_list_fragment
);

define_list_fragment_renderer!(
    render_cup_list_fragment,
    CupSortKey,
    load_cup_page,
    CupListTemplate { cups },
    "#cup-list"
);
