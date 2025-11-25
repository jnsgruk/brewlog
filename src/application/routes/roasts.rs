use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::{NewRoast, Roast, RoastSortKey, RoastWithRoaster};
use crate::presentation::templates::{RoastDetailTemplate, RoastListTemplate, RoastsTemplate};
use crate::presentation::views::{ListNavigator, Paginated, RoastView, RoasterOptionView};

const ROAST_PAGE_PATH: &str = "/roasts";
const ROAST_FRAGMENT_PATH: &str = "/roasts#roast-list";

async fn load_roast_page(
    state: &AppState,
    request: ListRequest<RoastSortKey>,
) -> Result<(Paginated<RoastView>, ListNavigator<RoastSortKey>), AppError> {
    let page = state
        .roast_repo
        .list(&request)
        .await
        .map_err(AppError::from)?;

    Ok(crate::application::routes::support::build_page_view(
        page,
        request,
        RoastView::from_list_item,
        ROAST_PAGE_PATH,
        ROAST_FRAGMENT_PATH,
    ))
}

pub(crate) async fn roasts_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<RoastSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated =
            crate::application::routes::auth::is_authenticated(&state, &cookies).await;
        return render_roast_list_fragment(state, request, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let roasters = state
        .roaster_repo
        .list_all_sorted(RoasterSortKey::Name, SortDirection::Asc)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let roaster_options = roasters.into_iter().map(RoasterOptionView::from).collect();

    let (roasts, navigator) = load_roast_page(&state, request)
        .await
        .map_err(map_app_error)?;

    let is_authenticated =
        crate::application::routes::auth::is_authenticated(&state, &cookies).await;

    let template = RoastsTemplate {
        nav_active: "roasts",
        is_authenticated,
        roasts,
        roaster_options,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

pub(crate) async fn roast_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    Path((roaster_slug, roast_slug)): Path<(String, String)>,
) -> Result<Html<String>, StatusCode> {
    let roaster = state
        .roaster_repo
        .get_by_slug(&roaster_slug)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let roast = state
        .roast_repo
        .get_by_slug(roaster.id, &roast_slug)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let is_authenticated =
        crate::application::routes::auth::is_authenticated(&state, &cookies).await;

    let template = RoastDetailTemplate {
        nav_active: "roasts",
        is_authenticated,
        roast: RoastView::from_domain(roast, &roaster.name, &roaster.slug),
    };

    render_html(template)
}

pub(crate) async fn create_roast(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoastSubmission>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoastSortKey>();
    let (submission, source) = payload.into_parts();
    let new_roast = submission.into_new_roast().map_err(ApiError::from)?;

    state
        .roaster_repo
        .get(new_roast.roaster_id)
        .await
        .map_err(|err| ApiError::from(AppError::from(err)))?;

    let roast = state
        .roast_repo
        .insert(new_roast)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roast_list_fragment(state, request, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target = ListNavigator::new(ROAST_PAGE_PATH, ROAST_FRAGMENT_PATH, request).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(roast)).into_response())
    }
}

pub(crate) async fn list_roasts(
    State(state): State<AppState>,
    Query(params): Query<RoastsQuery>,
) -> Result<Json<Vec<RoastWithRoaster>>, ApiError> {
    let roasts = match params.roaster_id {
        Some(roaster_id) => state
            .roast_repo
            .list_by_roaster(roaster_id)
            .await
            .map_err(AppError::from)?,
        None => state.roast_repo.list_all().await.map_err(AppError::from)?,
    };
    Ok(Json(roasts))
}

pub(crate) async fn get_roast(
    State(state): State<AppState>,
    Path(id): Path<RoastId>,
) -> Result<Json<Roast>, ApiError> {
    let roast = state.roast_repo.get(id).await.map_err(AppError::from)?;
    Ok(Json(roast))
}

pub(crate) async fn delete_roast(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(id): Path<RoastId>,
    Query(query): Query<ListQuery>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoastSortKey>();
    state.roast_repo.delete(id).await.map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roast_list_fragment(state, request, true)
            .await
            .map_err(ApiError::from)
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

#[derive(Debug, Deserialize)]
pub struct RoastsQuery {
    pub roaster_id: Option<RoasterId>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewRoastSubmission {
    roaster_id: RoasterId,
    name: String,
    origin: String,
    region: String,
    producer: String,
    tasting_notes: TastingNotesInput,
    process: String,
}

impl NewRoastSubmission {
    fn into_new_roast(self) -> Result<NewRoast, AppError> {
        fn require(field: &str, value: String) -> Result<String, AppError> {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(AppError::validation(format!("{field} is required")))
            } else {
                Ok(trimmed.to_string())
            }
        }

        let roaster_id = self.roaster_id;
        if roaster_id.into_inner() <= 0 {
            return Err(AppError::validation("invalid roaster id"));
        }
        let name = require("name", self.name)?;
        let origin = require("origin", self.origin)?;
        let region = require("region", self.region)?;
        let producer = require("producer", self.producer)?;
        let process = require("process", self.process)?;

        let tasting_notes = self.tasting_notes.into_vec();

        if tasting_notes.is_empty() {
            return Err(AppError::validation("tasting notes are required"));
        }

        Ok(NewRoast {
            roaster_id,
            name,
            origin,
            region,
            producer,
            tasting_notes,
            process,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TastingNotesInput {
    List(Vec<String>),
    Text(String),
}

impl TastingNotesInput {
    fn into_vec(self) -> Vec<String> {
        match self {
            TastingNotesInput::List(values) => values
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect(),
            TastingNotesInput::Text(value) => value
                .split([',', '\n'])
                .map(|segment| segment.trim().to_string())
                .filter(|segment| !segment.is_empty())
                .collect(),
        }
    }
}

async fn render_roast_list_fragment(
    state: AppState,
    request: ListRequest<RoastSortKey>,
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let (roasts, navigator) = load_roast_page(&state, request).await?;

    let template = RoastListTemplate {
        is_authenticated,
        roasts,
        navigator,
    };

    crate::application::routes::support::render_fragment(template, "#roast-list")
}
