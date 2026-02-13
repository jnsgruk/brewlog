use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::errors::{AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::bags::{BagFilter, BagSortKey};
use crate::domain::brews::{BrewFilter, BrewSortKey};
use crate::domain::listing::{ListRequest, PageSize, SortDirection, SortKey};
use crate::domain::timeline::TimelineSortKey;
use rand::seq::SliceRandom;

use crate::domain::stats::CachedStats;
use crate::presentation::web::templates::HomeTemplate;
use crate::presentation::web::views::{BagView, BrewView, StatCard, StatsView, TimelineEventView};

#[allow(clippy::similar_names)]
#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn home_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let content = load_home_content(&state).await.map_err(map_app_error)?;

    let cached = state.stats_repo.get_cached().await.ok().flatten();

    let stats = if let Some(ref cs) = cached {
        StatsView {
            brews: cs.entity_counts.brews,
            roasts: cs.entity_counts.roasts,
            roasters: cs.entity_counts.roasters,
            cups: cs.entity_counts.cups,
            cafes: cs.entity_counts.cafes,
            bags: cs.entity_counts.bags,
        }
    } else {
        // Fallback: compute counts directly when cache is empty (e.g. first page load)
        match state.stats_repo.entity_counts().await {
            Ok(ec) => StatsView {
                brews: ec.brews,
                roasts: ec.roasts,
                roasters: ec.roasters,
                cups: ec.cups,
                cafes: ec.cafes,
                bags: ec.bags,
            },
            Err(_) => StatsView::default(),
        }
    };

    let stat_cards = cached.map(build_stat_cards).unwrap_or_default();

    let template = HomeTemplate {
        nav_active: "home",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        recent_brews: content.recent_brews,
        open_bags: content.open_bags,
        recent_events: content.recent_events,
        stats,
        stat_cards,
    };

    render_html(template).map(IntoResponse::into_response)
}

struct HomeContent {
    recent_brews: Vec<BrewView>,
    open_bags: Vec<BagView>,
    recent_events: Vec<TimelineEventView>,
}

async fn load_home_content(state: &AppState) -> Result<HomeContent, AppError> {
    let recent_brews_req = ListRequest::new(
        1,
        PageSize::limited(10),
        BrewSortKey::CreatedAt,
        SortDirection::Desc,
    );
    let open_bags_req =
        ListRequest::new(1, PageSize::All, BagSortKey::UpdatedAt, SortDirection::Desc);
    let recent_events_req = ListRequest::new(
        1,
        PageSize::limited(5),
        TimelineSortKey::default(),
        TimelineSortKey::default().default_direction(),
    );

    let (recent_brews_page, open_bags_page, recent_events_page) = tokio::try_join!(
        async {
            state
                .brew_repo
                .list(BrewFilter::all(), &recent_brews_req, None)
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

    let recent_brews: Vec<BrewView> = recent_brews_page
        .items
        .into_iter()
        .map(BrewView::from)
        .collect();

    let open_bags = open_bags_page
        .items
        .into_iter()
        .map(BagView::from)
        .collect();

    let recent_events = recent_events_page
        .items
        .into_iter()
        .map(TimelineEventView::from)
        .collect();

    Ok(HomeContent {
        recent_brews,
        open_bags,
        recent_events,
    })
}

fn build_stat_cards(cs: CachedStats) -> Vec<StatCard> {
    let mut cards = vec![
        StatCard {
            icon: "coffee_bean",
            value: crate::domain::formatting::format_weight(cs.consumption.last_30_days_grams),
            label: "Coffee (30d)",
        },
        StatCard {
            icon: "coffee_bean",
            value: crate::domain::formatting::format_weight(cs.consumption.all_time_grams),
            label: "All Time",
        },
        StatCard {
            icon: "beaker",
            value: cs.consumption.brews_last_30_days.to_string(),
            label: "Brews (30d)",
        },
        StatCard {
            icon: "map",
            value: cs.roast_summary.unique_origins.to_string(),
            label: "Origins",
        },
        StatCard {
            icon: "location",
            value: cs
                .roast_summary
                .top_origin
                .unwrap_or_else(|| "\u{2014}".into()),
            label: "Top Origin",
        },
        StatCard {
            icon: "fire",
            value: cs
                .roast_summary
                .top_roaster
                .unwrap_or_else(|| "\u{2014}".into()),
            label: "Top Roaster",
        },
    ];
    cards.shuffle(&mut rand::thread_rng());
    cards
}
