use std::str::FromStr;

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
use crate::domain::gear::{Gear, GearCategory, GearFilter, GearSortKey, NewGear, UpdateGear};
use crate::domain::ids::GearId;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};
use crate::presentation::web::templates::{GearListTemplate, GearTemplate};
use crate::presentation::web::views::{GearView, ListNavigator, Paginated};

const GEAR_PAGE_PATH: &str = "/gear";
const GEAR_FRAGMENT_PATH: &str = "/gear#gear-list";

#[tracing::instrument(skip(state))]
async fn load_gear_page(
    state: &AppState,
    request: ListRequest<GearSortKey>,
    search: Option<&str>,
) -> Result<(Paginated<GearView>, ListNavigator<GearSortKey>), AppError> {
    let page = state
        .gear_repo
        .list(GearFilter::all(), &request, search)
        .await
        .map_err(AppError::from)?;

    Ok(crate::application::routes::support::build_page_view(
        page,
        request,
        GearView::from_domain,
        GEAR_PAGE_PATH,
        GEAR_FRAGMENT_PATH,
        search.map(String::from),
    ))
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn gear_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let (request, search) = query.into_request_and_search::<GearSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_gear_list_fragment(state, request, search, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let (gear, navigator) = load_gear_page(&state, request, search.as_deref())
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = GearTemplate {
        nav_active: "gear",
        is_authenticated,
        gear,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_gear(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewGearSubmission>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<GearSortKey>();
    let (submission, source) = payload.into_parts();
    let new_gear = submission.into_new_gear().map_err(ApiError::from)?;

    let gear = state
        .gear_repo
        .insert(new_gear)
        .await
        .map_err(AppError::from)?;

    // Add timeline event
    let event = NewTimelineEvent {
        entity_type: "gear".to_string(),
        entity_id: gear.id.into_inner(),
        action: "added".to_string(),
        occurred_at: chrono::Utc::now(),
        title: format!("{} {}", gear.make, gear.model),
        details: vec![
            TimelineEventDetail {
                label: "Category".to_string(),
                value: gear.category.display_label().to_string(),
            },
            TimelineEventDetail {
                label: "Make".to_string(),
                value: gear.make.clone(),
            },
            TimelineEventDetail {
                label: "Model".to_string(),
                value: gear.model.clone(),
            },
        ],
        tasting_notes: vec![],
        slug: None,         // Gear has no slug
        roaster_slug: None, // Gear is not related to roasters
        brew_data: None,
    };
    let _ = state.timeline_repo.insert(event).await;

    if is_datastar_request(&headers) {
        render_gear_list_fragment(state, request, search, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target =
            ListNavigator::new(GEAR_PAGE_PATH, GEAR_FRAGMENT_PATH, request, search).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(gear)).into_response())
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn list_gear(
    State(state): State<AppState>,
    Query(params): Query<GearQuery>,
) -> Result<Json<Vec<Gear>>, ApiError> {
    let filter = match params.category {
        Some(ref cat_str) => {
            let category = GearCategory::from_str(cat_str)
                .map_err(|()| AppError::validation("invalid category"))?;
            GearFilter::for_category(category)
        }
        None => GearFilter::all(),
    };
    let request = ListRequest::show_all(GearSortKey::Make, SortDirection::Asc);
    let page = state
        .gear_repo
        .list(filter, &request, None)
        .await
        .map_err(AppError::from)?;
    Ok(Json(page.items))
}

define_get_handler!(get_gear, GearId, Gear, gear_repo);

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn update_gear(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<GearId>,
    Query(query): Query<ListQuery>,
    payload: Json<UpdateGear>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<GearSortKey>();

    let gear = state
        .gear_repo
        .update(id, payload.0)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_gear_list_fragment(state, request, search, true)
            .await
            .map_err(ApiError::from)
    } else {
        Ok(Json(gear).into_response())
    }
}

define_delete_handler!(
    delete_gear,
    GearId,
    GearSortKey,
    gear_repo,
    render_gear_list_fragment
);

#[derive(Debug, Deserialize)]
pub struct GearQuery {
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewGearSubmission {
    category: String,
    make: String,
    model: String,
}

impl NewGearSubmission {
    fn into_new_gear(self) -> Result<NewGear, AppError> {
        let category = GearCategory::from_str(&self.category)
            .map_err(|()| AppError::validation("invalid category"))?;

        if self.make.trim().is_empty() {
            return Err(AppError::validation("make cannot be empty"));
        }

        if self.model.trim().is_empty() {
            return Err(AppError::validation("model cannot be empty"));
        }

        Ok(NewGear {
            category,
            make: self.make,
            model: self.model,
        })
    }
}

define_list_fragment_renderer!(
    render_gear_list_fragment,
    GearSortKey,
    load_gear_page,
    GearListTemplate { gear },
    "#gear-list"
);
