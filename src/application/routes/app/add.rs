use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::routes::support::{
    load_cafe_options, load_roast_options, load_roaster_options,
};
use crate::application::server::AppState;
use crate::presentation::web::templates::{AddTemplate, Tab};

use crate::application::routes::api::brews::load_brew_form_data;

const ADD_TABS: &[Tab] = &[
    Tab {
        key: "roaster",
        label: "Roaster",
    },
    Tab {
        key: "roast",
        label: "Roast",
    },
    Tab {
        key: "bag",
        label: "Bag",
    },
    Tab {
        key: "brew",
        label: "Brew",
    },
    Tab {
        key: "gear",
        label: "Gear",
    },
    Tab {
        key: "cafe",
        label: "Cafe",
    },
    Tab {
        key: "cup",
        label: "Cup",
    },
];

#[derive(Debug, Deserialize)]
pub(crate) struct AddQuery {
    #[serde(rename = "type", default = "default_type")]
    entity_type: String,
}

fn default_type() -> String {
    "roaster".to_string()
}

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn add_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    Query(query): Query<AddQuery>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    if !is_authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    let (roaster_options, roast_options, cafe_options, brew_form) = tokio::try_join!(
        async { load_roaster_options(&state).await },
        async { load_roast_options(&state).await },
        async { load_cafe_options(&state).await },
        async { load_brew_form_data(&state).await },
    )
    .map_err(map_app_error)?;

    let template = AddTemplate {
        nav_active: "data",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        active_type: query.entity_type,
        tabs: ADD_TABS
            .iter()
            .map(|t| Tab {
                key: t.key,
                label: t.label,
            })
            .collect(),
        tab_signal: "_add-type",
        tab_signal_js: "$_addType",
        tab_base_url: "",
        tab_fetch_target: "",
        tab_fetch_mode: "",
        roaster_options,
        roast_options,
        bag_options: brew_form.bag_options,
        grinder_options: brew_form.grinder_options,
        brewer_options: brew_form.brewer_options,
        filter_paper_options: brew_form.filter_paper_options,
        cafe_options,
        defaults: brew_form.defaults,
        quick_note_options: brew_form.quick_note_options,
    };

    render_html(template).map(IntoResponse::into_response)
}
