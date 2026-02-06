use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::application::errors::{AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{ListQuery, is_datastar_request};
use crate::application::state::AppState;
use crate::presentation::web::templates::{
    BagListTemplate, BrewListTemplate, CafeListTemplate, CupListTemplate, DataTemplate,
    GearListTemplate, RoastListTemplate, RoasterListTemplate, Tab, render_template,
};

const TABS: &[Tab] = &[
    Tab {
        key: "brews",
        label: "Brews",
    },
    Tab {
        key: "roasters",
        label: "Roasters",
    },
    Tab {
        key: "roasts",
        label: "Roasts",
    },
    Tab {
        key: "bags",
        label: "Bags",
    },
    Tab {
        key: "gear",
        label: "Gear",
    },
    Tab {
        key: "cafes",
        label: "Cafes",
    },
    Tab {
        key: "cups",
        label: "Cups",
    },
];

#[derive(Debug, Deserialize)]
pub(crate) struct DataType {
    #[serde(rename = "type", default = "default_type")]
    entity_type: String,
}

fn default_type() -> String {
    "brews".to_string()
}

#[tracing::instrument(skip(state, cookies, headers, data_type, list_query))]
pub(crate) async fn data_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(data_type): Query<DataType>,
    Query(list_query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let entity_type = data_type.entity_type;
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;
    let search_value = list_query.search_value();

    let content = render_entity_content(&state, &entity_type, list_query, is_authenticated)
        .await
        .map_err(map_app_error)?;

    if is_datastar_request(&headers) {
        use axum::http::header::HeaderValue;
        use axum::response::Html;

        let mut response = Html(content).into_response();
        response.headers_mut().insert(
            "datastar-selector",
            HeaderValue::from_static("#data-content"),
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

    let template = DataTemplate {
        nav_active: "data",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        active_type: entity_type,
        tabs,
        tab_signal: "_active-tab",
        tab_signal_js: "$_activeTab",
        tab_base_url: "/data?type=",
        tab_fetch_target: "#data-content",
        tab_fetch_mode: "inner",
        content,
        search_value,
    };

    render_html(template).map(IntoResponse::into_response)
}

fn render_list<T: askama::Template>(template: T, label: &str) -> Result<String, AppError> {
    render_template(template)
        .map_err(|err| AppError::unexpected(format!("failed to render {label}: {err}")))
}

async fn render_entity_content(
    state: &AppState,
    entity_type: &str,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    // Normalize unknown types to brews
    let entity_type = match entity_type {
        "brews" | "roasters" | "roasts" | "bags" | "gear" | "cafes" | "cups" => entity_type,
        _ => "brews",
    };

    match entity_type {
        "roasters" => render_roasters(state, list_query, is_authenticated).await,
        "roasts" => render_roasts(state, list_query, is_authenticated).await,
        "bags" => render_bags(state, list_query, is_authenticated).await,
        "gear" => render_gear(state, list_query, is_authenticated).await,
        "cafes" => render_cafes(state, list_query, is_authenticated).await,
        "cups" => render_cups(state, list_query, is_authenticated).await,
        _ => render_brews(state, list_query, is_authenticated).await,
    }
}

async fn render_brews(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::brews::BrewSortKey;
    let (request, search) = list_query.into_request_and_search::<BrewSortKey>();
    let data =
        crate::application::routes::api::brews::load_brew_page(state, request, search.as_deref())
            .await?;
    render_list(
        BrewListTemplate {
            is_authenticated,
            brews: data.brews,
            navigator: data.navigator,
        },
        "brews",
    )
}

async fn render_roasters(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::roasters::RoasterSortKey;
    let (request, search) = list_query.into_request_and_search::<RoasterSortKey>();
    let (roasters, navigator) = crate::application::routes::api::roasters::load_roaster_page(
        state,
        request,
        search.as_deref(),
    )
    .await?;
    render_list(
        RoasterListTemplate {
            is_authenticated,
            roasters,
            navigator,
        },
        "roasters",
    )
}

async fn render_roasts(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::roasts::RoastSortKey;
    let (request, search) = list_query.into_request_and_search::<RoastSortKey>();
    let (roasts, navigator) =
        crate::application::routes::api::roasts::load_roast_page(state, request, search.as_deref())
            .await?;
    render_list(
        RoastListTemplate {
            is_authenticated,
            roasts,
            navigator,
        },
        "roasts",
    )
}

async fn render_bags(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::bags::BagSortKey;
    let (request, search) = list_query.into_request_and_search::<BagSortKey>();
    let data =
        crate::application::routes::api::bags::load_bag_page(state, request, search.as_deref())
            .await?;
    render_list(
        BagListTemplate {
            is_authenticated,
            bags: data.bags,
            navigator: data.navigator,
        },
        "bags",
    )
}

async fn render_gear(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::gear::GearSortKey;
    let (request, search) = list_query.into_request_and_search::<GearSortKey>();
    let (gear, navigator) =
        crate::application::routes::api::gear::load_gear_page(state, request, search.as_deref())
            .await?;
    render_list(
        GearListTemplate {
            is_authenticated,
            gear,
            navigator,
        },
        "gear",
    )
}

async fn render_cafes(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::cafes::CafeSortKey;
    let (request, search) = list_query.into_request_and_search::<CafeSortKey>();
    let (cafes, navigator) =
        crate::application::routes::api::cafes::load_cafe_page(state, request, search.as_deref())
            .await?;
    render_list(
        CafeListTemplate {
            is_authenticated,
            cafes,
            navigator,
        },
        "cafes",
    )
}

async fn render_cups(
    state: &AppState,
    list_query: ListQuery,
    is_authenticated: bool,
) -> Result<String, AppError> {
    use crate::domain::cups::CupSortKey;
    let (request, search) = list_query.into_request_and_search::<CupSortKey>();
    let (cups, navigator) =
        crate::application::routes::api::cups::load_cup_page(state, request, search.as_deref())
            .await?;
    render_list(
        CupListTemplate {
            is_authenticated,
            cups,
            navigator,
        },
        "cups",
    )
}
