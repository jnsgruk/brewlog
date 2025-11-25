use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::domain::listing::ListRequest;
use crate::domain::timeline::{TimelineEvent, TimelineSortKey};
use crate::presentation::templates::{TimelineChunkTemplate, TimelineTemplate};
use crate::presentation::views::{ListNavigator, Paginated, TimelineEventView, TimelineMonthView};
use crate::server::errors::{AppError, map_app_error};
use crate::server::routes::render_html;
use crate::server::routes::support::{
    ListQuery, is_datastar_request, normalize_request, set_datastar_patch_headers,
};
use crate::server::server::AppState;

const TIMELINE_PAGE_PATH: &str = "/timeline";
const TIMELINE_FRAGMENT_PATH: &str = "/timeline";
const TIMELINE_DEFAULT_PAGE_SIZE: u32 = 5;

pub(crate) async fn timeline_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request_with_default::<TimelineSortKey>(TIMELINE_DEFAULT_PAGE_SIZE);
    if is_datastar_request(&headers) {
        return render_timeline_chunk(state, request)
            .await
            .map_err(map_app_error);
    }

    let data = load_timeline_page(&state, request)
        .await
        .map_err(map_app_error)?;

    let is_authenticated = crate::server::routes::auth::is_authenticated(&state, &cookies).await;

    let template = TimelineTemplate {
        nav_active: "timeline",
        is_authenticated,
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
) -> Result<Response, AppError> {
    let data = load_timeline_page(&state, request).await?;
    let template = TimelineChunkTemplate {
        events: data.events,
        navigator: data.navigator,
        months: data.months,
    };

    let html = crate::presentation::templates::render_template(template)
        .map_err(|err| AppError::unexpected(format!("failed to render timeline chunk: {err}")))?;

    let mut response = Html(html).into_response();
    set_datastar_patch_headers(response.headers_mut(), "#timeline-loader");
    Ok(response)
}

struct TimelinePageData {
    events: Paginated<TimelineEventView>,
    navigator: ListNavigator<TimelineSortKey>,
    months: Vec<TimelineMonthView>,
}

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
