use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, PayloadSource, is_datastar_request, load_cafe_options, load_roast_options,
};
use crate::application::server::AppState;
use crate::domain::cafes::NewCafe;
use crate::domain::cups::NewCup;
use crate::domain::ids::{CafeId, RoastId};
use crate::presentation::web::templates::CheckInTemplate;
use tracing::info;

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
        nav_active: "checkin",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        roast_options,
        cafe_options,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckInSubmission {
    #[serde(default)]
    cafe_id: Option<String>,
    #[serde(default)]
    cafe_name: Option<String>,
    #[serde(default)]
    cafe_city: Option<String>,
    #[serde(default)]
    cafe_country: Option<String>,
    #[serde(default)]
    cafe_lat: f64,
    #[serde(default)]
    cafe_lng: f64,
    #[serde(default)]
    cafe_website: Option<String>,
    roast_id: String,
}

#[tracing::instrument(skip(state, _auth_user, headers, payload))]
pub(crate) async fn submit_checkin(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    payload: FlexiblePayload<CheckInSubmission>,
) -> Result<Response, ApiError> {
    let (submission, source) = payload.into_parts();

    let roast_id: i64 = submission
        .roast_id
        .parse()
        .map_err(|_| AppError::validation("invalid roast ID"))?;

    // Use existing cafe or create a new one
    let cafe_id = if let Some(id) = submission.cafe_id.as_deref().filter(|s| !s.is_empty()) {
        let parsed: i64 = id
            .parse()
            .map_err(|_| AppError::validation("invalid cafe ID"))?;
        CafeId::from(parsed)
    } else {
        let name = submission
            .cafe_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::validation("cafe name is required"))?;

        let new_cafe = NewCafe {
            name: name.to_string(),
            city: submission.cafe_city.unwrap_or_default(),
            country: submission.cafe_country.unwrap_or_default(),
            latitude: submission.cafe_lat,
            longitude: submission.cafe_lng,
            website: submission.cafe_website.filter(|s| !s.is_empty()),
        }
        .normalize();

        let cafe = state
            .cafe_repo
            .insert(new_cafe)
            .await
            .map_err(AppError::from)?;
        cafe.id
    };

    let new_cup = NewCup {
        roast_id: RoastId::from(roast_id),
        cafe_id,
    };

    let cup = state
        .cup_repo
        .insert(new_cup)
        .await
        .map_err(AppError::from)?;

    info!(cup_id = %cup.id, %cafe_id, "check-in recorded");

    if is_datastar_request(&headers) {
        crate::application::routes::support::render_signals_json(&[]).map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to("/").into_response())
    } else {
        Ok((StatusCode::CREATED, Json(cup)).into_response())
    }
}
