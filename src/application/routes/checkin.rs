use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

use crate::application::errors::map_app_error;
use crate::application::routes::render_html;
use crate::application::routes::support::{load_cafe_options, load_roast_options};
use crate::application::server::AppState;
use crate::presentation::web::templates::CheckInTemplate;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn checkin_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
) -> Result<Response, StatusCode> {
    let is_authenticated = super::is_authenticated(&state, &cookies).await;
    if !is_authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    let roast_options = load_roast_options(&state).await.map_err(map_app_error)?;
    let cafe_options = load_cafe_options(&state).await.map_err(map_app_error)?;

    let template = CheckInTemplate {
        nav_active: "home",
        is_authenticated: true,
        has_ai_extract: state.has_ai_extract(),
        has_foursquare: state.has_foursquare(),
        roast_options,
        cafe_options,
    };

    render_html(template).map(IntoResponse::into_response)
}
