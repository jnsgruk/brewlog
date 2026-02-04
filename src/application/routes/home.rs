use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::errors::{AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::server::AppState;
use crate::domain::bags::{BagFilter, BagSortKey};
use crate::domain::brews::{BrewFilter, BrewSortKey};
use crate::domain::cafes::CafeSortKey;
use crate::domain::cups::CupFilter;
use crate::domain::gear::GearFilter;
use crate::domain::listing::{ListRequest, PageSize, SortDirection, SortKey};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasts::RoastSortKey;
use crate::domain::timeline::TimelineSortKey;
use crate::presentation::web::templates::HomeTemplate;
use crate::presentation::web::views::{BagView, BrewView, StatsView, TimelineEventView};

#[allow(clippy::similar_names)]
#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn home_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
) -> Result<Response, StatusCode> {
    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let (content, stats) =
        tokio::try_join!(load_home_content(&state), load_stats(&state),).map_err(map_app_error)?;

    let template = HomeTemplate {
        nav_active: "home",
        is_authenticated,
        has_ai_extract: state.has_ai_extract(),
        has_foursquare: state.has_foursquare(),
        last_brew: content.last_brew,
        open_bags: content.open_bags,
        recent_events: content.recent_events,
        stats,
    };

    render_html(template).map(IntoResponse::into_response)
}

struct HomeContent {
    last_brew: Option<BrewView>,
    open_bags: Vec<BagView>,
    recent_events: Vec<TimelineEventView>,
}

/// Build a `ListRequest` that fetches page 1 with 1 item, using a sort key's
/// defaults. Used to obtain `Page.total` for entity counts.
fn count_request<K: SortKey>() -> ListRequest<K> {
    let key = K::default();
    ListRequest::new(1, PageSize::limited(1), key, key.default_direction())
}

async fn load_home_content(state: &AppState) -> Result<HomeContent, AppError> {
    let last_brew_req = ListRequest::new(
        1,
        PageSize::limited(1),
        BrewSortKey::CreatedAt,
        SortDirection::Desc,
    );
    let open_bags_req = ListRequest::show_all(BagSortKey::RoastDate, SortDirection::Desc);
    let recent_events_req = ListRequest::new(
        1,
        PageSize::limited(5),
        TimelineSortKey::default(),
        TimelineSortKey::default().default_direction(),
    );

    let (last_brew_page, open_bags_page, recent_events_page) = tokio::try_join!(
        async {
            state
                .brew_repo
                .list(BrewFilter::all(), &last_brew_req, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .bag_repo
                .list(BagFilter::open(), &open_bags_req, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .timeline_repo
                .list(&recent_events_req)
                .await
                .map_err(AppError::from)
        },
    )?;

    let last_brew = last_brew_page
        .items
        .into_iter()
        .next()
        .map(BrewView::from_domain);

    let open_bags = open_bags_page
        .items
        .into_iter()
        .map(BagView::from_domain)
        .collect();

    let recent_events = recent_events_page
        .items
        .into_iter()
        .map(TimelineEventView::from_domain)
        .collect();

    Ok(HomeContent {
        last_brew,
        open_bags,
        recent_events,
    })
}

async fn load_stats(state: &AppState) -> Result<StatsView, AppError> {
    let req_roasters: ListRequest<RoasterSortKey> = count_request();
    let req_roasts: ListRequest<RoastSortKey> = count_request();
    let req_bags: ListRequest<BagSortKey> = count_request();
    let req_brews: ListRequest<BrewSortKey> = count_request();
    let req_gear: ListRequest<crate::domain::gear::GearSortKey> = count_request();
    let req_cafes: ListRequest<CafeSortKey> = count_request();
    let req_cups: ListRequest<crate::domain::cups::CupSortKey> = count_request();

    let (roasters, roasts, bags, brews, gear, cafes, cups) = tokio::try_join!(
        async {
            state
                .roaster_repo
                .list(&req_roasters, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .roast_repo
                .list(&req_roasts, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .bag_repo
                .list(BagFilter::all(), &req_bags, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .brew_repo
                .list(BrewFilter::all(), &req_brews, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .gear_repo
                .list(GearFilter::all(), &req_gear, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .cafe_repo
                .list(&req_cafes, None)
                .await
                .map_err(AppError::from)
        },
        async {
            state
                .cup_repo
                .list(CupFilter::all(), &req_cups, None)
                .await
                .map_err(AppError::from)
        },
    )?;

    Ok(StatsView {
        brews: brews.total,
        roasts: roasts.total,
        roasters: roasters.total,
        cups: cups.total,
        cafes: cafes.total,
        bags: bags.total,
        gear: gear.total,
    })
}
