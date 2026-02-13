use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::errors::{AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{ListQuery, is_datastar_request, normalize_request};
use crate::application::state::AppState;
use crate::domain::listing::ListRequest;
use crate::domain::timeline::{TimelineEvent, TimelineSortKey};
use crate::presentation::web::templates::{TimelineChunkTemplate, TimelineTemplate};
use crate::presentation::web::views::{
    ListNavigator, Paginated, TimelineEventView, TimelineMonthView,
};

const TIMELINE_PAGE_PATH: &str = "/timeline";
const TIMELINE_FRAGMENT_PATH: &str = "/timeline";
const TIMELINE_DEFAULT_PAGE_SIZE: u32 = 20;

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn timeline_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let (request, _search) =
        query.into_request_and_search_with_default::<TimelineSortKey>(TIMELINE_DEFAULT_PAGE_SIZE);
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

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
        version_info: &crate::VERSION_INFO,
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
