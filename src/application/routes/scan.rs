use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::roasts::TastingNotesInput;
use crate::application::server::AppState;
use crate::domain::errors::RepositoryError;
use crate::domain::roasters::NewRoaster;
use crate::domain::roasts::NewRoast;
use crate::infrastructure::ai::{self, ExtractedBagScan, ExtractionInput};

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn extract_bag_scan(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Json(input): Json<ExtractionInput>,
) -> Result<Json<ExtractedBagScan>, ApiError> {
    let api_key = state
        .openrouter_api_key
        .as_deref()
        .ok_or_else(|| AppError::validation("AI extraction is not configured"))?;

    let result = ai::extract_bag_scan(&state.http_client, api_key, &state.openrouter_model, &input)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub(crate) struct BagScanSubmission {
    roaster_name: String,
    roaster_country: String,
    roaster_city: Option<String>,
    roaster_homepage: Option<String>,
    roast_name: String,
    origin: String,
    region: String,
    producer: String,
    process: String,
    tasting_notes: TastingNotesInput,
}

#[derive(Debug, Serialize)]
struct ScanResult {
    redirect: String,
    roast_id: i64,
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn submit_scan(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Json(submission): Json<BagScanSubmission>,
) -> Result<Response, ApiError> {
    // Build and normalize the roaster
    let new_roaster = NewRoaster {
        name: submission.roaster_name,
        country: submission.roaster_country,
        city: submission.roaster_city,
        homepage: submission.roaster_homepage,
    }
    .normalize();

    let slug = new_roaster.slug();

    // Try to find existing roaster by slug, otherwise create
    let roaster = match state.roaster_repo.get_by_slug(&slug).await {
        Ok(existing) => existing,
        Err(RepositoryError::NotFound) => state
            .roaster_repo
            .insert(new_roaster)
            .await
            .map_err(AppError::from)?,
        Err(err) => return Err(AppError::from(err).into()),
    };

    // Validate and build the roast
    let tasting_notes = submission.tasting_notes.into_vec();
    if tasting_notes.is_empty() {
        return Err(AppError::validation("tasting notes are required").into());
    }

    fn require(field: &str, value: &str) -> Result<String, AppError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(AppError::validation(format!("{field} is required")))
        } else {
            Ok(trimmed.to_string())
        }
    }

    let new_roast = NewRoast {
        roaster_id: roaster.id,
        name: require("roast name", &submission.roast_name)?,
        origin: require("origin", &submission.origin)?,
        region: require("region", &submission.region)?,
        producer: require("producer", &submission.producer)?,
        process: require("process", &submission.process)?,
        tasting_notes,
    };

    let roast = state
        .roast_repo
        .insert(new_roast)
        .await
        .map_err(AppError::from)?;

    let redirect = format!("/roasters/{}/roasts/{}", roaster.slug, roast.slug);
    let roast_id = roast.id.into_inner();
    Ok((StatusCode::CREATED, Json(ScanResult { redirect, roast_id })).into_response())
}
