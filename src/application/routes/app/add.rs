use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::routes::support::{
    load_cafe_options, load_roast_options, load_roaster_options,
};
use crate::application::state::AppState;
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
    bag_id: Option<String>,
    // Brew-again overrides
    coffee_weight: Option<f64>,
    grinder_id: Option<String>,
    grind_setting: Option<f64>,
    brewer_id: Option<String>,
    filter_paper_id: Option<String>,
    water_volume: Option<i32>,
    water_temp: Option<f64>,
    brew_time: Option<i32>,
    quick_notes: Option<String>,
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

    let mut defaults = brew_form.defaults;

    // Apply brew-again query param overrides
    if let Some(cw) = query.coffee_weight {
        defaults.coffee_weight = cw;
    }
    if let Some(ref gid) = query.grinder_id {
        if let Some(opt) = brew_form.grinder_options.iter().find(|o| o.id == *gid) {
            defaults.grinder_name = opt.label.clone();
        }
        defaults.grinder_id = gid.clone();
    }
    if let Some(gs) = query.grind_setting {
        defaults.grind_setting = gs;
    }
    if let Some(ref bid) = query.brewer_id {
        if let Some(opt) = brew_form.brewer_options.iter().find(|o| o.id == *bid) {
            defaults.brewer_name = opt.label.clone();
        }
        defaults.brewer_id = bid.clone();
    }
    if let Some(ref fpid) = query.filter_paper_id {
        if let Some(opt) = brew_form
            .filter_paper_options
            .iter()
            .find(|o| o.id == *fpid)
        {
            defaults.filter_paper_name = opt.label.clone();
        }
        defaults.filter_paper_id = fpid.clone();
    }
    if let Some(wv) = query.water_volume {
        defaults.water_volume = wv;
    }
    if let Some(wt) = query.water_temp {
        defaults.water_temp = wt;
    }
    if let Some(bt) = query.brew_time {
        defaults.brew_time = Some(bt);
    }
    if let Some(ref qn) = query.quick_notes {
        defaults.quick_notes_raw.clone_from(qn);
    }

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
        defaults,
        quick_note_options: brew_form.quick_note_options,
        pre_select_bag_id: query.bag_id,
    };

    render_html(template).map(IntoResponse::into_response)
}
