use axum::extract::{Query, State};
use axum::http::header::HeaderValue;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use chrono::Utc;
use serde::Deserialize;

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::routes::support::is_datastar_request;
use crate::application::services::stats::compute_all_stats;
use crate::application::state::AppState;
use crate::domain::country_stats::GeoStats;
use crate::domain::stats::CachedStats;
use crate::presentation::web::templates::{
    StatsMapFragment, StatsPageTemplate, Tab, render_template,
};

const TABS: &[Tab] = &[
    Tab {
        key: "roasters",
        label: "Roasters",
    },
    Tab {
        key: "roasts",
        label: "Roasts",
    },
    Tab {
        key: "cups",
        label: "Cups",
    },
    Tab {
        key: "cafes",
        label: "Cafes",
    },
];

#[derive(Debug, Deserialize)]
pub(crate) struct StatsQuery {
    #[serde(rename = "type", default = "default_type")]
    entity_type: String,
}

fn default_type() -> String {
    "roasters".to_string()
}

#[tracing::instrument(skip(state, cookies, headers, stats_query))]
pub(crate) async fn stats_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(stats_query): Query<StatsQuery>,
) -> Result<Response, StatusCode> {
    let entity_type = stats_query.entity_type;
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    // Datastar tab switch: only need geo stats for the selected tab
    if is_datastar_request(&headers) {
        let geo_stats = geo_for_type(&load_or_compute(&state).await?, &entity_type);

        let content = render_template(StatsMapFragment {
            geo_stats: &geo_stats,
        })
        .map_err(|err| {
            tracing::error!(error = %err, "failed to render stats fragment");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let mut response = Html(content).into_response();
        response.headers_mut().insert(
            "datastar-selector",
            HeaderValue::from_static("#stats-content"),
        );
        response
            .headers_mut()
            .insert("datastar-mode", HeaderValue::from_static("inner"));
        return Ok(response);
    }

    // Full page load: use cached stats or compute on the fly
    let cached = load_or_compute(&state).await?;
    let geo_stats = geo_for_type(&cached, &entity_type);

    let content = render_template(StatsMapFragment {
        geo_stats: &geo_stats,
    })
    .map_err(|err| {
        tracing::error!(error = %err, "failed to render stats fragment");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let tabs: Vec<Tab> = TABS
        .iter()
        .map(|t| Tab {
            key: t.key,
            label: t.label,
        })
        .collect();

    let cache_age = format_cache_age(&cached.computed_at);
    let consumption_30d_weight =
        crate::domain::formatting::format_weight(cached.consumption.last_30_days_grams);
    let consumption_all_time_weight =
        crate::domain::formatting::format_weight(cached.consumption.all_time_grams);
    let grinder_weights: Vec<(String, f64, String)> = cached
        .brewing_summary
        .grinder_weight_counts
        .iter()
        .map(|(name, grams)| {
            (
                name.clone(),
                *grams,
                crate::domain::formatting::format_weight(*grams),
            )
        })
        .collect();
    let max_grinder_weight = cached.brewing_summary.max_grinder_weight;

    let template = StatsPageTemplate {
        nav_active: "stats",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        active_type: entity_type,
        tabs,
        tab_signal: "_active-tab",
        tab_signal_js: "$_activeTab",
        tab_base_url: "/stats?type=",
        tab_fetch_target: "#stats-content",
        tab_fetch_mode: "inner",
        content,
        roast_summary: cached.roast_summary,
        consumption: cached.consumption,
        brewing_summary: cached.brewing_summary,
        grinder_weights,
        max_grinder_weight,
        consumption_30d_weight,
        consumption_all_time_weight,
        cache_age,
    };

    render_html(template).map(IntoResponse::into_response)
}

/// Load stats from cache, falling back to live computation on cache miss.
async fn load_or_compute(state: &AppState) -> Result<CachedStats, StatusCode> {
    if let Ok(Some(cached)) = state.stats_repo.get_cached().await {
        return Ok(cached);
    }
    tracing::debug!("stats cache miss, computing live");
    compute_all_stats(&*state.stats_repo)
        .await
        .map_err(|e| map_app_error(e.into()))
}

/// Select the geo stats for a given entity type from the cached snapshot.
fn geo_for_type(cached: &CachedStats, entity_type: &str) -> GeoStats {
    match entity_type {
        "roasts" => cached.geo_roasts.clone(),
        "cups" => cached.geo_cups.clone(),
        "cafes" => cached.geo_cafes.clone(),
        _ => cached.geo_roasters.clone(),
    }
}

/// Format the cache timestamp as a relative age string (e.g. "Just now", "2m ago").
fn format_cache_age(computed_at: &str) -> String {
    let Ok(ts) = chrono::DateTime::parse_from_rfc3339(computed_at) else {
        return String::new();
    };
    crate::domain::formatting::format_relative_time(ts.with_timezone(&Utc), Utc::now())
}
