use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

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
use crate::domain::ids::RoasterId;
use crate::domain::images::ImageData;
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::{NewRoaster, Roaster, RoasterSortKey, UpdateRoaster};
use crate::infrastructure::ai::{self, ExtractionInput};
use crate::presentation::web::templates::RoasterListTemplate;
use crate::presentation::web::views::{ListNavigator, Paginated, RoasterView};
use tracing::info;

const ROASTER_PAGE_PATH: &str = "/data?type=roasters";
const ROASTER_FRAGMENT_PATH: &str = "/data?type=roasters#roaster-list";

#[tracing::instrument(skip(state))]
pub(crate) async fn load_roaster_page(
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

#[derive(Debug, Deserialize)]
pub(crate) struct NewRoasterSubmission {
    name: String,
    country: String,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    image: ImageData,
}

impl NewRoasterSubmission {
    fn into_parts(self) -> (NewRoaster, Option<String>) {
        let roaster = NewRoaster {
            name: self.name,
            country: self.country,
            city: self.city,
            homepage: self.homepage,
            created_at: self.created_at,
        };
        (roaster, self.image.into_inner())
    }
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoasterSubmission>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<RoasterSortKey>();
    let (submission, source) = payload.into_parts();
    let (new_roaster, image_data_url) = submission.into_parts();
    let new_roaster = new_roaster.normalize();
    let roaster = state
        .roaster_service
        .create(new_roaster)
        .await
        .map_err(AppError::from)?;

    info!(roaster_id = %roaster.id, name = %roaster.name, "roaster created");
    state.stats_invalidator.invalidate();

    save_deferred_image(
        &state,
        EntityType::Roaster,
        i64::from(roaster.id),
        image_data_url.as_deref(),
    )
    .await;

    let detail_url = format!("/roasters/{}", roaster.slug);

    if is_datastar_request(&headers) {
        let from_data_page = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|r| r.contains("type=roasters"));

        if from_data_page {
            render_roaster_list_fragment(state, request, search, true)
                .await
                .map_err(ApiError::from)
        } else {
            render_redirect_script(&detail_url).map_err(ApiError::from)
        }
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(&detail_url).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(roaster)).into_response())
    }
}

define_get_handler!(get_roaster, RoasterId, Roaster, roaster_repo);

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateRoasterSubmission {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    image: ImageData,
}

impl UpdateRoasterSubmission {
    fn into_parts(self) -> (UpdateRoaster, Option<String>) {
        let update = UpdateRoaster {
            name: self.name,
            country: self.country,
            city: self.city,
            homepage: self.homepage,
            created_at: self.created_at,
        };
        (update, self.image.into_inner())
    }
}

impl_has_changes!(UpdateRoaster, name, country, city, homepage, created_at);

#[tracing::instrument(skip(state, _auth_user, headers))]
pub(crate) async fn update_roaster(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<RoasterId>,
    payload: FlexiblePayload<UpdateRoasterSubmission>,
) -> Result<Response, ApiError> {
    let (submission, source) = payload.into_parts();
    let (update, image_data_url) = submission.into_parts();
    let update = update.normalize();

    validate_update(&update, image_data_url.as_ref())?;

    let roaster = state
        .roaster_repo
        .update(id, update)
        .await
        .map_err(AppError::from)?;
    info!(%id, "roaster updated");
    state.stats_invalidator.invalidate();

    save_deferred_image(
        &state,
        EntityType::Roaster,
        i64::from(roaster.id),
        image_data_url.as_deref(),
    )
    .await;

    let detail_url = format!("/roasters/{}", roaster.slug);
    update_response(&headers, source, &detail_url, Json(roaster).into_response())
}

define_delete_handler!(
    delete_roaster,
    RoasterId,
    RoasterSortKey,
    roaster_repo,
    render_roaster_list_fragment,
    "type=roasters",
    "/data?type=roasters",
    image_type: crate::domain::entity_type::EntityType::Roaster
);

#[tracing::instrument(skip(state, auth_user, headers, payload))]
pub(crate) async fn extract_roaster(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    headers: HeaderMap,
    payload: FlexiblePayload<ExtractionInput>,
) -> Result<Response, ApiError> {
    let (input, _) = payload.into_parts();
    let (result, usage) = ai::extract_roaster(
        &state.http_client,
        &state.openrouter_url,
        &state.openrouter_api_key,
        &state.openrouter_model,
        &input,
    )
    .await
    .map_err(ApiError::from)?;

    crate::application::routes::support::record_ai_usage(
        state.ai_usage_repo.clone(),
        auth_user.0.id,
        &state.openrouter_model,
        "extract-roaster",
        usage,
    );

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
