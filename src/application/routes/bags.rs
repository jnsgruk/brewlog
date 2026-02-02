use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use super::macros::{define_delete_handler, define_enriched_get_handler};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::bags::{BagFilter, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::ids::{BagId, RoastId};
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};
use crate::presentation::web::templates::{BagListTemplate, BagsTemplate};
use crate::presentation::web::views::{BagView, ListNavigator, Paginated, RoasterOptionView};

const BAG_PAGE_PATH: &str = "/bags";
const BAG_FRAGMENT_PATH: &str = "/bags#bag-list";

struct BagPageData {
    open_bags: Vec<BagView>,
    bags: Paginated<BagView>,
    navigator: ListNavigator<BagSortKey>,
}

#[tracing::instrument(skip(state))]
async fn load_bag_page(
    state: &AppState,
    request: ListRequest<BagSortKey>,
) -> Result<BagPageData, AppError> {
    let open_request = ListRequest::show_all(BagSortKey::RoastDate, SortDirection::Desc);
    let open_page = state
        .bag_repo
        .list(BagFilter::open(), &open_request)
        .await
        .map_err(AppError::from)?;
    let open_bags_view = open_page
        .items
        .into_iter()
        .map(BagView::from_domain)
        .collect();

    let page = state
        .bag_repo
        .list(BagFilter::closed(), &request)
        .await
        .map_err(AppError::from)?;

    let (bags, navigator) = crate::application::routes::support::build_page_view(
        page,
        request,
        BagView::from_domain,
        BAG_PAGE_PATH,
        BAG_FRAGMENT_PATH,
    );

    Ok(BagPageData {
        open_bags: open_bags_view,
        bags,
        navigator,
    })
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn bags_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<BagSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_bag_list_fragment(state, request, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let roasters = state
        .roaster_repo
        .list_all_sorted(RoasterSortKey::Name, SortDirection::Asc)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let roaster_options = roasters.into_iter().map(RoasterOptionView::from).collect();

    let BagPageData {
        open_bags,
        bags,
        navigator,
    } = load_bag_page(&state, request)
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = BagsTemplate {
        nav_active: "bags",
        is_authenticated,
        open_bags,
        bags,
        roaster_options,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_bag(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewBagSubmission>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<BagSortKey>();
    let (submission, source) = payload.into_parts();
    let new_bag = submission.into_new_bag().map_err(ApiError::from)?;

    let roast = state
        .roast_repo
        .get(new_bag.roast_id)
        .await
        .map_err(|err| ApiError::from(AppError::from(err)))?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|err| ApiError::from(AppError::from(err)))?;

    let bag = state
        .bag_repo
        .insert(new_bag)
        .await
        .map_err(AppError::from)?;

    // Add timeline event
    let event = NewTimelineEvent {
        entity_type: "bag".to_string(),
        entity_id: bag.id.into_inner(),
        action: "added".to_string(),
        occurred_at: chrono::Utc::now(),
        title: roast.name.clone(),
        details: vec![
            TimelineEventDetail {
                label: "Roaster".to_string(),
                value: roaster.name,
            },
            TimelineEventDetail {
                label: "Amount".to_string(),
                value: format!("{}g", bag.amount),
            },
        ],
        tasting_notes: vec![],
    };
    let _ = state.timeline_repo.insert(event).await;

    if is_datastar_request(&headers) {
        render_bag_list_fragment(state, request, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target = ListNavigator::new(BAG_PAGE_PATH, BAG_FRAGMENT_PATH, request).page_href(1);
        Ok(Redirect::to(&target).into_response())
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
        .list(filter, &request)
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
    let request = query.into_request::<BagSortKey>();

    let body_update = payload.map_or(
        UpdateBag {
            remaining: None,
            closed: None,
            finished_at: None,
        },
        |Json(p)| p,
    );

    let mut update = UpdateBag {
        remaining: body_update.remaining.or(update_params.remaining),
        closed: body_update.closed.or(update_params.closed),
        finished_at: body_update.finished_at.or(update_params.finished_at),
    };

    if let Some(true) = update.closed
        && update.finished_at.is_none()
    {
        update.finished_at = Some(chrono::Utc::now().date_naive());
    }

    let bag = state
        .bag_repo
        .update(id, update.clone())
        .await
        .map_err(AppError::from)?;

    if let Some(true) = update.closed {
        // Fetch roast and roaster for timeline event
        if let Ok(roast) = state.roast_repo.get(bag.roast_id).await
            && let Ok(roaster) = state.roaster_repo.get(roast.roaster_id).await
        {
            let event = NewTimelineEvent {
                entity_type: "bag".to_string(),
                entity_id: bag.id.into_inner(),
                action: "finished".to_string(),
                occurred_at: chrono::Utc::now(),
                title: roast.name.clone(),
                details: vec![
                    TimelineEventDetail {
                        label: "Roaster".to_string(),
                        value: roaster.name,
                    },
                    TimelineEventDetail {
                        label: "Amount".to_string(),
                        value: format!("{}g", bag.amount),
                    },
                ],
                tasting_notes: vec![],
            };
            let _ = state.timeline_repo.insert(event).await;
        }
    }

    if is_datastar_request(&headers) {
        render_bag_list_fragment(state, request, true)
            .await
            .map_err(ApiError::from)
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
    render_bag_list_fragment
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
        })
    }
}

async fn render_bag_list_fragment(
    state: AppState,
    request: ListRequest<BagSortKey>,
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let BagPageData {
        open_bags,
        bags,
        navigator,
    } = load_bag_page(&state, request).await?;

    let template = BagListTemplate {
        is_authenticated,
        open_bags,
        bags,
        navigator,
    };

    crate::application::routes::support::render_fragment(template, "#bag-list")
}
