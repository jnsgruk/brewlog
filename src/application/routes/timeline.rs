use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::application::errors::{AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{is_datastar_request, normalize_request};
use crate::application::server::AppState;
use crate::domain::listing::{ListRequest, PageSize, SortDirection, SortKey};
use crate::domain::timeline::{TimelineEvent, TimelineSortKey};
use crate::presentation::web::templates::{TimelineChunkTemplate, TimelineTemplate};
use crate::presentation::web::views::{
    ListNavigator, Paginated, TimelineEventView, TimelineMonthView,
};

const TIMELINE_PAGE_PATH: &str = "/timeline";
const TIMELINE_FRAGMENT_PATH: &str = "/timeline";
const TIMELINE_DEFAULT_PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PageSizeParam {
    Number(u32),
    Text(String),
}

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    page: Option<u32>,
    #[serde(default)]
    page_size: Option<PageSizeParam>,
    #[serde(default, rename = "sort")]
    sort_key: Option<String>,
    #[serde(default, rename = "dir")]
    sort_dir: Option<String>,
}

impl TimelineQuery {
    fn to_request(&self) -> ListRequest<TimelineSortKey> {
        let page = self.page.unwrap_or(1);
        let page_size = match &self.page_size {
            Some(PageSizeParam::Number(value)) => PageSize::limited(*value),
            Some(PageSizeParam::Text(text)) if text.eq_ignore_ascii_case("all") => PageSize::All,
            Some(PageSizeParam::Text(text)) => text
                .parse::<u32>()
                .map(PageSize::limited)
                .unwrap_or(PageSize::limited(TIMELINE_DEFAULT_PAGE_SIZE)),
            None => PageSize::limited(TIMELINE_DEFAULT_PAGE_SIZE),
        };

        let sort_key = self
            .sort_key
            .as_deref()
            .and_then(TimelineSortKey::from_query)
            .unwrap_or_else(TimelineSortKey::default);

        let sort_direction = self
            .sort_dir
            .as_deref()
            .and_then(|dir| match dir.to_ascii_lowercase().as_str() {
                "asc" => Some(SortDirection::Asc),
                "desc" => Some(SortDirection::Desc),
                _ => None,
            })
            .unwrap_or_else(|| sort_key.default_direction());

        ListRequest::new(page, page_size, sort_key, sort_direction)
    }
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn timeline_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<TimelineQuery>,
) -> Result<Response, StatusCode> {
    let request = query.to_request();
    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    if is_datastar_request(&headers) {
        return render_timeline_chunk(state, request, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    let data = load_timeline_page(&state, request)
        .await
        .map_err(map_app_error)?;

    let template = TimelineTemplate {
        nav_active: "timeline",
        is_authenticated,
        has_ai_extract: state.has_ai_extract(),
        events: data.events,
        navigator: data.navigator,
        months: data.months,
    };

    render_html(template).map(IntoResponse::into_response)
}

struct TimelinePreparedEvent {
    anchor: String,
    heading: String,
    view: TimelineEventView,
}

async fn render_timeline_chunk(
    state: AppState,
    request: ListRequest<TimelineSortKey>,
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let data = load_timeline_page(&state, request).await?;
    let template = TimelineChunkTemplate {
        is_authenticated,
        events: data.events,
        navigator: data.navigator,
        months: data.months,
    };

    crate::application::routes::support::render_fragment(template, "#timeline-loader")
}

struct TimelinePageData {
    events: Paginated<TimelineEventView>,
    navigator: ListNavigator<TimelineSortKey>,
    months: Vec<TimelineMonthView>,
}

#[tracing::instrument(skip(state))]
async fn load_timeline_page(
    state: &AppState,
    request: ListRequest<TimelineSortKey>,
) -> Result<TimelinePageData, AppError> {
    let page = state
        .timeline_repo
        .list(&request)
        .await
        .map_err(AppError::from)?;

    let normalized_request = normalize_request(request, &page);

    let prepared_events = page
        .items
        .into_iter()
        .map(prepare_event)
        .collect::<Vec<_>>();

    let views = prepared_events
        .iter()
        .map(|prepared| prepared.view.clone())
        .collect::<Vec<_>>();

    let events = Paginated::new(
        views,
        page.page,
        page.page_size,
        page.total,
        page.showing_all,
    );
    let months = build_months(prepared_events);
    let navigator = ListNavigator::new(
        TIMELINE_PAGE_PATH,
        TIMELINE_FRAGMENT_PATH,
        normalized_request,
        None,
    );

    Ok(TimelinePageData {
        events,
        navigator,
        months,
    })
}

fn prepare_event(event: TimelineEvent) -> TimelinePreparedEvent {
    let anchor = event.occurred_at.format("%Y-%m").to_string();
    let heading = event.occurred_at.format("%B %Y").to_string();
    let view = TimelineEventView::from_domain(event);

    TimelinePreparedEvent {
        anchor,
        heading,
        view,
    }
}

fn build_months(prepared_events: Vec<TimelinePreparedEvent>) -> Vec<TimelineMonthView> {
    let mut months: Vec<TimelineMonthView> = Vec::new();

    for prepared in prepared_events {
        if let Some(last) = months.last_mut()
            && last.anchor == prepared.anchor
        {
            last.events.push(prepared.view);
            continue;
        }

        months.push(TimelineMonthView {
            anchor: prepared.anchor,
            heading: prepared.heading,
            events: vec![prepared.view],
        });
    }

    months
}
