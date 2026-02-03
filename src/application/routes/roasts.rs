use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

use super::macros::{
    define_delete_handler, define_enriched_get_handler, define_list_fragment_renderer,
};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request, load_roaster_options,
};
use crate::application::server::AppState;
use crate::domain::bags::{BagFilter, BagSortKey};
use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasts::{NewRoast, RoastSortKey, RoastWithRoaster, UpdateRoast};
use crate::presentation::web::templates::{
    RoastDetailTemplate, RoastListTemplate, RoastOptionsTemplate, RoastsTemplate,
};
use crate::presentation::web::views::{ListNavigator, Paginated, RoastView};

const ROAST_PAGE_PATH: &str = "/roasts";
const ROAST_FRAGMENT_PATH: &str = "/roasts#roast-list";

#[tracing::instrument(skip(state))]
async fn load_roast_page(
    state: &AppState,
    request: ListRequest<RoastSortKey>,
    search: Option<&str>,
) -> Result<(Paginated<RoastView>, ListNavigator<RoastSortKey>), AppError> {
    let page = state
        .roast_repo
        .list(&request, search)
        .await
        .map_err(AppError::from)?;

    Ok(crate::application::routes::support::build_page_view(
        page,
        request,
        RoastView::from_list_item,
        ROAST_PAGE_PATH,
        ROAST_FRAGMENT_PATH,
        search.map(String::from),
    ))
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn roasts_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let (request, search) = query.into_request_and_search::<RoastSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_roast_list_fragment(state, request, search, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let roaster_options = load_roaster_options(&state).await.map_err(map_app_error)?;

    let (roasts, navigator) = load_roast_page(&state, request, search.as_deref())
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = RoastsTemplate {
        nav_active: "roasts",
        is_authenticated,
        roasts,
        roaster_options,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, cookies))]
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

    let bag_request = ListRequest::show_all(BagSortKey::RoastDate, SortDirection::Desc);
    let bags_page = state
        .bag_repo
        .list(BagFilter::for_roast(roast.id), &bag_request, None)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let bag_views = bags_page
        .items
        .into_iter()
        .map(crate::presentation::web::views::BagView::from_domain)
        .collect();

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = RoastDetailTemplate {
        nav_active: "roasts",
        is_authenticated,
        roast: RoastView::from_domain(roast, &roaster.name, &roaster.slug),
        bags: bag_views,
    };

    render_html(template)
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_roast(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoastSubmission>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<RoastSortKey>();
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
        render_roast_list_fragment(state, request, search, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target =
            ListNavigator::new(ROAST_PAGE_PATH, ROAST_FRAGMENT_PATH, request, search).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        let enriched = state
            .roast_repo
            .get_with_roaster(roast.id)
            .await
            .map_err(AppError::from)?;
        Ok((StatusCode::CREATED, Json(enriched)).into_response())
    }
}

#[tracing::instrument(skip(state, headers))]
pub(crate) async fn list_roasts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<RoastsQuery>,
) -> Result<Response, ApiError> {
    let roaster_id = match params.roaster_id.as_deref() {
        Some(s) if !s.is_empty() => {
            let s = s.trim();
            Some(s.parse::<RoasterId>().map_err(|_| {
                tracing::warn!("Invalid roaster_id: '{}'", s);
                ApiError::from(AppError::validation(format!("Invalid roaster_id: '{s}'")))
            })?)
        }
        _ => None,
    };

    let roasts = match roaster_id {
        Some(roaster_id) => state
            .roast_repo
            .list_by_roaster(roaster_id)
            .await
            .map_err(AppError::from)?,
        None => {
            if is_datastar_request(&headers) {
                vec![]
            } else {
                state.roast_repo.list_all().await.map_err(AppError::from)?
            }
        }
    };

    if is_datastar_request(&headers) {
        let template = RoastOptionsTemplate { roasts };
        crate::application::routes::support::render_fragment(template, "#roast-select-options")
            .map_err(ApiError::from)
    } else {
        Ok(Json(roasts).into_response())
    }
}

define_enriched_get_handler!(
    get_roast,
    RoastId,
    RoastWithRoaster,
    roast_repo,
    get_with_roaster
);

define_delete_handler!(
    delete_roast,
    RoastId,
    RoastSortKey,
    roast_repo,
    render_roast_list_fragment
);

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn update_roast(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<RoastId>,
    Json(payload): Json<UpdateRoast>,
) -> Result<Json<RoastWithRoaster>, ApiError> {
    let has_changes = payload.roaster_id.is_some()
        || payload.name.is_some()
        || payload.origin.is_some()
        || payload.region.is_some()
        || payload.producer.is_some()
        || payload.tasting_notes.is_some()
        || payload.process.is_some();

    if !has_changes {
        return Err(AppError::validation("no changes provided").into());
    }

    state
        .roast_repo
        .update(id, payload)
        .await
        .map_err(AppError::from)?;

    let enriched = state
        .roast_repo
        .get_with_roaster(id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(enriched))
}

#[derive(Debug, Deserialize)]
pub struct RoastsQuery {
    pub roaster_id: Option<String>,
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

define_list_fragment_renderer!(
    render_roast_list_fragment,
    RoastSortKey,
    load_roast_page,
    RoastListTemplate { roasts },
    "#roast-list"
);
