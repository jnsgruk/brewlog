use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::support::{FlexiblePayload, PayloadSource, is_datastar_request};
use crate::application::state::AppState;
use crate::domain::cafes::NewCafe;
use crate::domain::cups::NewCup;
use crate::domain::ids::{CafeId, RoastId};

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
            created_at: None,
        }
        .normalize();

        let cafe = state
            .cafe_service
            .create(new_cafe)
            .await
            .map_err(AppError::from)?;
        cafe.id
    };

    let new_cup = NewCup {
        roast_id: RoastId::from(roast_id),
        cafe_id,
        created_at: None,
    };

    let cup = state
        .cup_service
        .create(new_cup)
        .await
        .map_err(AppError::from)?;

    let detail_url = format!("/cups/{}", cup.id);

    if is_datastar_request(&headers) {
        use axum::http::header::HeaderValue;
        let script = format!("<script>window.location.href='{detail_url}'</script>");
        let mut response = axum::response::Html(script).into_response();
        response
            .headers_mut()
            .insert("datastar-selector", HeaderValue::from_static("body"));
        response
            .headers_mut()
            .insert("datastar-mode", HeaderValue::from_static("append"));
        Ok(response)
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(&detail_url).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(cup)).into_response())
    }
}
