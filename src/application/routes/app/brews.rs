use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::map_app_error;
use crate::application::routes::api::brews::load_brew_form_data;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::ids::BrewId;
use crate::presentation::web::templates::{BrewDetailTemplate, BrewEditTemplate};
use crate::presentation::web::views::BrewDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn brew_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<BrewId>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let brew_details = state
        .brew_repo
        .get_with_details(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let bag = state
        .bag_repo
        .get(brew_details.brew.bag_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roast = state
        .roast_repo
        .get(bag.roast_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let image_url = resolve_image_url(&state, "brew", i64::from(id))
        .await
        .or(resolve_image_url(&state, "roast", i64::from(roast.id)).await);

    let view = BrewDetailView::from_parts(brew_details, &roast, &roaster);

    let template = BrewDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        edit_url: format!("/brews/{id}/edit"),
        brew: view,
        roaster_slug: roaster.slug.clone(),
        roast_slug: roast.slug.clone(),
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn brew_edit_page(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<BrewId>,
) -> Result<Response, StatusCode> {
    let brew = state
        .brew_repo
        .get_with_details(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let form_data = load_brew_form_data(&state).await.map_err(map_app_error)?;

    let image_url = resolve_image_url(&state, "brew", i64::from(id)).await;

    let template = BrewEditTemplate {
        nav_active: "",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        id: brew.brew.id.to_string(),
        bag_id: brew.brew.bag_id.to_string(),
        bag_label: format!("{} ({})", brew.roast_name, brew.roaster_name),
        coffee_weight: brew.brew.coffee_weight,
        grinder_id: brew.brew.grinder_id.to_string(),
        grind_setting: brew.brew.grind_setting,
        brewer_id: brew.brew.brewer_id.to_string(),
        filter_paper_id: brew
            .brew
            .filter_paper_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        water_volume: brew.brew.water_volume,
        water_temp: brew.brew.water_temp,
        brew_time: brew.brew.brew_time.unwrap_or(0),
        quick_notes: brew
            .brew
            .quick_notes
            .iter()
            .map(|n| n.form_value())
            .collect::<Vec<_>>()
            .join(","),
        bag_options: form_data.bag_options,
        grinder_options: form_data.grinder_options,
        brewer_options: form_data.brewer_options,
        filter_paper_options: form_data.filter_paper_options,
        quick_note_options: form_data.quick_note_options,
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
