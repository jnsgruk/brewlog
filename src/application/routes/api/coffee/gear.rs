use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::api::images::save_deferred_image;
use crate::application::routes::api::macros::{
    define_delete_handler, define_get_handler, define_list_fragment_renderer,
};
use crate::application::routes::support::impl_has_changes;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request, render_redirect_script,
    update_response, validate_update,
};
use crate::application::state::AppState;
use crate::domain::entity_type::EntityType;
use crate::domain::gear::{Gear, GearCategory, GearFilter, GearSortKey, NewGear, UpdateGear};
use crate::domain::ids::GearId;
use crate::domain::images::ImageData;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::presentation::web::templates::GearListTemplate;
use crate::presentation::web::views::{GearView, ListNavigator, Paginated};

const GEAR_PAGE_PATH: &str = "/data?type=gear";
const GEAR_FRAGMENT_PATH: &str = "/data?type=gear#gear-list";

#[tracing::instrument(skip(state))]
pub(crate) async fn load_gear_page(
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
        GearView::from,
        GEAR_PAGE_PATH,
        GEAR_FRAGMENT_PATH,
        search.map(String::from),
    ))
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
    let (new_gear, image_data_url) = submission.into_parts().map_err(ApiError::from)?;

    let gear = state
        .gear_service
        .create(new_gear)
        .await
        .map_err(AppError::from)?;

    info!(gear_id = %gear.id, make = %gear.make, model = %gear.model, "gear created");
    state.stats_invalidator.invalidate();

    save_deferred_image(
        &state,
        EntityType::Gear,
        i64::from(gear.id),
        image_data_url.as_deref(),
    )
    .await;

    let detail_url = format!("/gear/{}", gear.id);

    if is_datastar_request(&headers) {
        let from_data_page = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|r| r.contains("type=gear"));

        if from_data_page {
            render_gear_list_fragment(state, request, search, true)
                .await
                .map_err(ApiError::from)
        } else {
            render_redirect_script(&detail_url).map_err(ApiError::from)
        }
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(&detail_url).into_response())
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

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateGearSubmission {
    #[serde(default)]
    make: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    image: ImageData,
}

impl UpdateGearSubmission {
    fn into_parts(self) -> (UpdateGear, Option<String>) {
        let update = UpdateGear {
            make: self.make,
            model: self.model,
            created_at: self.created_at,
        };
        (update, self.image.into_inner())
    }
}

impl_has_changes!(UpdateGear, make, model, created_at);

#[tracing::instrument(skip(state, _auth_user, headers))]
pub(crate) async fn update_gear(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<GearId>,
    payload: FlexiblePayload<UpdateGearSubmission>,
) -> Result<Response, ApiError> {
    let (submission, source) = payload.into_parts();
    let (update, image_data_url) = submission.into_parts();

    validate_update(&update, image_data_url.as_ref())?;

    let gear = state
        .gear_repo
        .update(id, update)
        .await
        .map_err(AppError::from)?;

    info!(%id, "gear updated");
    state.stats_invalidator.invalidate();

    save_deferred_image(
        &state,
        EntityType::Gear,
        i64::from(gear.id),
        image_data_url.as_deref(),
    )
    .await;

    let detail_url = format!("/gear/{}", gear.id);
    update_response(&headers, source, &detail_url, Json(gear).into_response())
}

define_delete_handler!(
    delete_gear,
    GearId,
    GearSortKey,
    gear_repo,
    render_gear_list_fragment,
    "type=gear",
    "/data?type=gear",
    image_type: crate::domain::entity_type::EntityType::Gear
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
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    image: ImageData,
}

impl NewGearSubmission {
    fn into_parts(self) -> Result<(NewGear, Option<String>), AppError> {
        let category = GearCategory::from_str(&self.category)
            .map_err(|()| AppError::validation("invalid category"))?;

        if self.make.trim().is_empty() {
            return Err(AppError::validation("make cannot be empty"));
        }

        if self.model.trim().is_empty() {
            return Err(AppError::validation("model cannot be empty"));
        }

        Ok((
            NewGear {
                category,
                make: self.make,
                model: self.model,
                created_at: self.created_at,
            },
            self.image.into_inner(),
        ))
    }
}

define_list_fragment_renderer!(
    render_gear_list_fragment,
    GearSortKey,
    load_gear_page,
    GearListTemplate { gear },
    "#gear-list"
);
