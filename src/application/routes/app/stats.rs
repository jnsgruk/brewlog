use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::application::errors::{AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::is_datastar_request;
use crate::application::state::AppState;
use crate::domain::country_stats::{CountryStat, GeoStats, country_to_iso, iso_to_flag_emoji};
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

    let geo_stats = load_geo_stats(&state, &entity_type)
        .await
        .map_err(map_app_error)?;

    let content = render_template(StatsMapFragment {
        geo_stats: &geo_stats,
    })
    .map_err(|err| {
        tracing::error!(error = %err, "failed to render stats fragment");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if is_datastar_request(&headers) {
        use axum::http::header::HeaderValue;
        use axum::response::Html;

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

    let tabs: Vec<Tab> = TABS
        .iter()
        .map(|t| Tab {
            key: t.key,
            label: t.label,
        })
        .collect();

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
    };

    render_html(template).map(IntoResponse::into_response)
}

async fn load_geo_stats(state: &AppState, entity_type: &str) -> Result<GeoStats, AppError> {
    let raw_counts = match entity_type {
        "roasts" => state.stats_repo.roast_origin_counts().await?,
        "cups" => state.stats_repo.cup_country_counts().await?,
        "cafes" => state.stats_repo.cafe_country_counts().await?,
        _ => state.stats_repo.roaster_country_counts().await?,
    };

    let entries: Vec<CountryStat> = raw_counts
        .into_iter()
        .map(|(name, count)| {
            let iso = country_to_iso(&name).unwrap_or("").to_string();
            let flag = if iso.is_empty() {
                String::new()
            } else {
                iso_to_flag_emoji(&iso)
            };
            CountryStat {
                country_name: name,
                iso_code: iso,
                flag_emoji: flag,
                count,
            }
        })
        .collect();

    let total_countries = entries.len();
    let max_count = entries.iter().map(|e| e.count).max().unwrap_or(0);

    Ok(GeoStats {
        entries,
        total_countries,
        max_count,
    })
}
