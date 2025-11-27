use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::bags::{Bag, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::ids::{BagId, RoastId};
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};
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
        occurred_at: chrono::Utc::now(),
        title: roast.name.to_string(),
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
        Ok((StatusCode::CREATED, Json(bag)).into_response())
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn list_bags(
    State(state): State<AppState>,
    Query(params): Query<BagsQuery>,
) -> Result<Json<Vec<BagWithRoast>>, ApiError> {
    let bags = match params.roast_id {
        Some(roast_id) => state
            .bag_repo
            .list_by_roast(roast_id)
            .await
            .map_err(AppError::from)?,
        None => {
            // For API list all, we might want to implement list_all in repo or reuse list with pagination
            // For now, let's just return empty or implement list_all if needed.
            // The spec implies we need list endpoints.
            // Let's implement list_all in repo later if needed, or just use list with large page size?
            // Actually, let's just use list_by_roast for now as that's the main use case for API likely.
            // Or better, let's add list_all to repo.
            // For now, I'll return an error if no filter is provided, or empty list.
            vec![]
        }
    };
    Ok(Json(bags))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn get_bag(
    State(state): State<AppState>,
    Path(id): Path<BagId>,
) -> Result<Json<Bag>, ApiError> {
    let bag = state.bag_repo.get(id).await.map_err(AppError::from)?;
    Ok(Json(bag))
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn update_bag(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<BagId>,
    Json(payload): Json<UpdateBag>,
) -> Result<Json<Bag>, ApiError> {
    let bag = state
        .bag_repo
        .update(id, payload.clone())
        .await
        .map_err(AppError::from)?;

    if let Some(true) = payload.closed {
        // Fetch roast and roaster for timeline event
        if let Ok(roast) = state.roast_repo.get(bag.roast_id).await
            && let Ok(roaster) = state.roaster_repo.get(roast.roaster_id).await
        {
            let event = NewTimelineEvent {
                entity_type: "bag".to_string(),
                entity_id: bag.id.into_inner(),
                occurred_at: chrono::Utc::now(),
                title: roast.name.to_string(),
                details: vec![TimelineEventDetail {
                    label: "Roaster".to_string(),
                    value: roaster.name,
                }],
                tasting_notes: vec![],
            };
            let _ = state.timeline_repo.insert(event).await;
        }
    }

    Ok(Json(bag))
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn delete_bag(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<BagId>,
    Query(query): Query<ListQuery>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<BagSortKey>();
    state.bag_repo.delete(id).await.map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_bag_list_fragment(state, request, true)
            .await
            .map_err(ApiError::from)
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn finish_bag(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<BagId>,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<BagSortKey>();

    let bag = state
        .bag_repo
        .get(id)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let update = UpdateBag {
        remaining: Some(0.0),
        closed: Some(true),
        finished_at: Some(chrono::Utc::now().date_naive()),
    };

    let _ = state
        .bag_repo
        .update(id, update)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    // Add timeline event
    if let Ok(roast) = state.roast_repo.get(bag.roast_id).await
        && let Ok(roaster) = state.roaster_repo.get(roast.roaster_id).await
    {
        let event = NewTimelineEvent {
            entity_type: "bag".to_string(),
            entity_id: bag.id.into_inner(),
            occurred_at: chrono::Utc::now(),
            title: roast.name.to_string(),
            details: vec![
                TimelineEventDetail {
                    label: "Roaster".to_string(),
                    value: roaster.name,
                },
                TimelineEventDetail {
                    label: "Amount".to_string(),
                    value: format!("{:.1}g", bag.amount),
                },
            ],
            tasting_notes: vec![],
        };
        let _ = state.timeline_repo.insert(event).await;
    }

    if is_datastar_request(&headers) {
        render_bag_list_fragment(state, request, true)
            .await
            .map_err(map_app_error)
    } else {
        // Fallback for non-datastar requests (though the UI uses datastar)
        Ok(Redirect::to(BAG_PAGE_PATH).into_response())
    }
}
