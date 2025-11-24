use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::{NewRoast, Roast, RoastSortKey, RoastWithRoaster};
use crate::presentation::templates::{RoastDetailTemplate, RoastListTemplate, RoastsTemplate};
use crate::presentation::views::{ListNavigator, Paginated, RoastView, RoasterOptionView};
use crate::server::errors::{ApiError, AppError, map_app_error};
use crate::server::routes::render_html;
use crate::server::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request, set_datastar_patch_headers,
};
use crate::server::server::AppState;

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

    let normalized_request = crate::server::routes::support::normalize_request(request, &page);
    let roasts = Paginated::from_page(page, RoastView::from_list_item);
    let navigator = ListNavigator::new(ROAST_PAGE_PATH, ROAST_FRAGMENT_PATH, normalized_request);

    Ok((roasts, navigator))
}

pub(crate) async fn roasts_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<RoastSortKey>();

    if is_datastar_request(&headers) {
        return render_roast_list_fragment(state, request)
            .await
            .map_err(|err| map_app_error(err));
    }

    let roasters = state
        .roaster_repo
        .list_all_sorted(RoasterSortKey::Name, SortDirection::Asc)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let roaster_options = roasters.into_iter().map(RoasterOptionView::from).collect();

    let (roasts, navigator) = load_roast_page(&state, request)
        .await
        .map_err(|err| map_app_error(err))?;

    let template = RoastsTemplate {
        nav_active: "roasts",
        roasts,
        roaster_options,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

pub(crate) async fn roast_page(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let roast = state
        .roast_repo
        .get(id.clone())
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;
    let roaster = state
        .roaster_repo
        .get(roast.roaster_id.clone())
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let template = RoastDetailTemplate {
        nav_active: "roasts",
        roast: RoastView::from_domain(roast, &roaster.name),
    };

    render_html(template)
}

pub(crate) async fn create_roast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewRoastSubmission>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoastSortKey>();
    let (submission, source) = payload.into_parts();
    let new_roast = submission.into_new_roast().map_err(ApiError::from)?;

    state
        .roaster_repo
        .get(new_roast.roaster_id.clone())
        .await
        .map_err(|err| ApiError::from(AppError::from(err)))?;

    let roast = new_roast.into_roast();
    let roast = state
        .roast_repo
        .insert(roast)
        .await
        .map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roast_list_fragment(state, request)
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
    Path(id): Path<String>,
) -> Result<Json<Roast>, ApiError> {
    let roast = state.roast_repo.get(id).await.map_err(AppError::from)?;
    Ok(Json(roast))
}

pub(crate) async fn delete_roast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Response, ApiError> {
    let request = query.into_request::<RoastSortKey>();
    state.roast_repo.delete(id).await.map_err(AppError::from)?;

    if is_datastar_request(&headers) {
        render_roast_list_fragment(state, request)
            .await
            .map_err(ApiError::from)
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

#[derive(Debug, Deserialize)]
pub struct RoastsQuery {
    pub roaster_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewRoastSubmission {
    roaster_id: String,
    name: String,
    origin: String,
    region: String,
    producer: String,
    #[serde(deserialize_with = "string_or_vec")]
    tasting_notes: Vec<String>,
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

        let roaster_id = require("roaster", self.roaster_id)?;
        let name = require("name", self.name)?;
        let origin = require("origin", self.origin)?;
        let region = require("region", self.region)?;
        let producer = require("producer", self.producer)?;
        let process = require("process", self.process)?;

        let tasting_notes = self
            .tasting_notes
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

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

// TODO: If we just make sure that the repository always returns a list, even for one value, or
// no values, we can remove this deserializer, I think?
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVecVisitor;

    impl<'de> Visitor<'de> for StringOrVecVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or sequence of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let split = value
                .split(|ch| ch == ',' || ch == '\n')
                .map(|segment| segment.trim().to_string())
                .filter(|segment| !segment.is_empty())
                .collect();
            Ok(split)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    values.push(trimmed.to_string());
                }
            }
            Ok(values)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
        where
            D2: Deserializer<'de>,
        {
            string_or_vec(deserializer)
        }
    }

    deserializer.deserialize_any(StringOrVecVisitor)
}

async fn render_roast_list_fragment(
    state: AppState,
    request: ListRequest<RoastSortKey>,
) -> Result<Response, AppError> {
    let (roasts, navigator) = load_roast_page(&state, request).await?;

    let template = RoastListTemplate { roasts, navigator };

    let html = crate::presentation::templates::render_template(template)
        .map_err(|err| AppError::unexpected(format!("failed to render roast list: {err}")))?;

    let mut response = Html(html).into_response();
    set_datastar_patch_headers(response.headers_mut(), "#roast-list");
    Ok(response)
}
