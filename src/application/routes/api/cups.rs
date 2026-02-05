use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};

use super::macros::{
    define_delete_handler, define_enriched_get_handler, define_list_fragment_renderer,
};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::cups::{CupFilter, CupSortKey, CupWithDetails, NewCup};
use crate::domain::ids::CupId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::presentation::web::templates::CupListTemplate;
use crate::presentation::web::views::{CupView, ListNavigator, Paginated};
use tracing::info;

const CUP_PAGE_PATH: &str = "/data?type=cups";
const CUP_FRAGMENT_PATH: &str = "/data?type=cups#cup-list";

#[tracing::instrument(skip(state))]
pub(in crate::application::routes) async fn load_cup_page(
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

    let cup = state
        .cup_repo
        .insert(new_cup)
        .await
        .map_err(AppError::from)?;

    info!(cup_id = %cup.id, "cup created");

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
