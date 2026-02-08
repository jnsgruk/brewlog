use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::api::macros::{define_delete_handler, define_enriched_get_handler};
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::state::AppState;
use crate::domain::bags::{BagFilter, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::ids::{BagId, RoastId};
use crate::domain::listing::{ListRequest, SortDirection};
use crate::presentation::web::templates::BagListTemplate;
use crate::presentation::web::views::{BagView, ListNavigator, Paginated};

const BAG_PAGE_PATH: &str = "/data?type=bags";
const BAG_FRAGMENT_PATH: &str = "/data?type=bags#bag-list";

pub(crate) struct BagPageData {
    pub(crate) bags: Paginated<BagView>,
    pub(crate) navigator: ListNavigator<BagSortKey>,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn load_bag_page(
    state: &AppState,
    request: ListRequest<BagSortKey>,
    search: Option<&str>,
) -> Result<BagPageData, AppError> {
    let page = state
        .bag_repo
        .list(BagFilter::all(), &request, search)
        .await
        .map_err(AppError::from)?;

    let (bags, navigator) = crate::application::routes::support::build_page_view(
        page,
        request,
        BagView::from_domain,
        BAG_PAGE_PATH,
        BAG_FRAGMENT_PATH,
        search.map(String::from),
    );

    Ok(BagPageData { bags, navigator })
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_bag(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewBagSubmission>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<BagSortKey>();
    let (submission, source) = payload.into_parts();
    let new_bag = submission.into_new_bag().map_err(ApiError::from)?;

    let bag = state
        .bag_service
        .create(new_bag)
        .await
        .map_err(AppError::from)?;

    info!(bag_id = %bag.id, "bag created");
    state.stats_invalidator.invalidate();

    let detail_url = format!("/bags/{}", bag.id);

    if is_datastar_request(&headers) {
        let from_bag_page = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|r| r.contains("type=bags"));

        if from_bag_page {
            render_bag_list_fragment(state, request, search, true)
                .await
                .map_err(ApiError::from)
        } else {
            crate::application::routes::support::render_redirect_script(&detail_url)
                .map_err(ApiError::from)
        }
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(&detail_url).into_response())
    } else {
        let enriched = state
            .bag_repo
            .get_with_roast(bag.id)
            .await
            .map_err(AppError::from)?;
        Ok((StatusCode::CREATED, Json(enriched)).into_response())
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn list_bags(
    State(state): State<AppState>,
    Query(params): Query<BagsQuery>,
) -> Result<Json<Vec<BagWithRoast>>, ApiError> {
    let filter = match params.roast_id {
        Some(roast_id) => BagFilter::for_roast(roast_id),
        None => BagFilter::all(),
    };
    let request = ListRequest::show_all(BagSortKey::RoastDate, SortDirection::Desc);
    let page = state
        .bag_repo
        .list(filter, &request, None)
        .await
        .map_err(AppError::from)?;
    Ok(Json(page.items))
}

define_enriched_get_handler!(get_bag, BagId, BagWithRoast, bag_repo, get_with_roast);

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn update_bag(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<BagId>,
    Query(query): Query<ListQuery>,
    Query(update_params): Query<UpdateBag>,
    payload: Option<Json<UpdateBag>>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<BagSortKey>();

    let body_update = payload.map_or(
        UpdateBag {
            remaining: None,
            closed: None,
            finished_at: None,
            created_at: None,
        },
        |Json(p)| p,
    );

    let update = UpdateBag {
        remaining: body_update.remaining.or(update_params.remaining),
        closed: body_update.closed.or(update_params.closed),
        finished_at: body_update.finished_at.or(update_params.finished_at),
        created_at: body_update.created_at,
    };

    let bag = if let Some(true) = update.closed {
        state
            .bag_service
            .finish(id, update.clone())
            .await
            .map_err(AppError::from)?
    } else {
        state
            .bag_repo
            .update(id, update.clone())
            .await
            .map_err(AppError::from)?
    };

    info!(%id, closed = ?update.closed, "bag updated");
    state.stats_invalidator.invalidate();

    if is_datastar_request(&headers) {
        let from_bag_page = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|r| r.contains("type=bags"));

        if from_bag_page {
            render_bag_list_fragment(state, request, search, true)
                .await
                .map_err(ApiError::from)
        } else {
            let detail_url = format!("/bags/{id}");
            crate::application::routes::support::render_redirect_script(&detail_url)
                .map_err(ApiError::from)
        }
    } else {
        let enriched = state
            .bag_repo
            .get_with_roast(bag.id)
            .await
            .map_err(AppError::from)?;
        Ok(Json(enriched).into_response())
    }
}

define_delete_handler!(
    delete_bag,
    BagId,
    BagSortKey,
    bag_repo,
    render_bag_list_fragment,
    "type=bags",
    "/data?type=bags"
);

#[derive(Debug, Deserialize)]
pub struct BagsQuery {
    pub roast_id: Option<RoastId>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewBagSubmission {
    roast_id: RoastId,
    roast_date: Option<String>,
    amount: f64,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
}

impl NewBagSubmission {
    fn into_new_bag(self) -> Result<NewBag, AppError> {
        let roast_id = self.roast_id;
        if roast_id.into_inner() <= 0 {
            return Err(AppError::validation("invalid roast id"));
        }

        let roast_date = match self.roast_date {
            Some(date_str) if !date_str.is_empty() => Some(
                chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|_| AppError::validation("invalid roast date format"))?,
            ),
            _ => None,
        };

        if self.amount <= 0.0 {
            return Err(AppError::validation("amount must be positive"));
        }

        Ok(NewBag {
            roast_id,
            roast_date,
            amount: self.amount,
            created_at: self.created_at,
        })
    }
}

async fn render_bag_list_fragment(
    state: AppState,
    request: ListRequest<BagSortKey>,
    search: Option<String>,
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let BagPageData { bags, navigator } = load_bag_page(&state, request, search.as_deref()).await?;

    let template = BagListTemplate {
        is_authenticated,
        bags,
        navigator,
    };

    crate::application::routes::support::render_fragment(template, "#bag-list")
}
