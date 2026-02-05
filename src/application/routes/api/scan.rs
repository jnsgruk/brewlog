use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::roasts::TastingNotesInput;
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::support::{FlexiblePayload, is_datastar_request};
use crate::application::server::AppState;
use crate::domain::bags::NewBag;
use crate::domain::errors::RepositoryError;
use crate::domain::roasters::NewRoaster;
use crate::domain::roasts::NewRoast;
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};
use crate::infrastructure::ai::{self, ExtractionInput, Usage};

#[tracing::instrument(skip(state, auth_user, headers, payload))]
pub(crate) async fn extract_bag_scan(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    headers: HeaderMap,
    payload: FlexiblePayload<ExtractionInput>,
) -> Result<Response, ApiError> {
    let (input, _) = payload.into_parts();
    let (result, usage) = ai::extract_bag_scan(
        &state.http_client,
        &state.openrouter_api_key,
        &state.openrouter_model,
        &input,
    )
    .await
    .map_err(ApiError::from)?;

    crate::application::routes::support::record_ai_usage(
        state.ai_usage_repo.clone(),
        auth_user.0.id,
        &state.openrouter_model,
        "extract-bag-scan",
        usage,
    );

    if is_datastar_request(&headers) {
        use serde_json::Value;

        let tasting_notes = result
            .roast
            .tasting_notes
            .as_ref()
            .map(|notes| notes.join(", "))
            .unwrap_or_default();

        let signals = vec![
            (
                "_roaster-name",
                Value::String(result.roaster.name.unwrap_or_default()),
            ),
            (
                "_roaster-country",
                Value::String(result.roaster.country.unwrap_or_default()),
            ),
            (
                "_roaster-city",
                Value::String(result.roaster.city.unwrap_or_default()),
            ),
            (
                "_roaster-homepage",
                Value::String(result.roaster.homepage.unwrap_or_default()),
            ),
            (
                "_roast-name",
                Value::String(result.roast.name.unwrap_or_default()),
            ),
            (
                "_origin",
                Value::String(result.roast.origin.unwrap_or_default()),
            ),
            (
                "_region",
                Value::String(result.roast.region.unwrap_or_default()),
            ),
            (
                "_producer",
                Value::String(result.roast.producer.unwrap_or_default()),
            ),
            (
                "_process",
                Value::String(result.roast.process.unwrap_or_default()),
            ),
            ("_tasting-notes", Value::String(tasting_notes)),
            ("_scan-extracted", Value::Bool(true)),
        ];
        crate::application::routes::support::render_signals_json(&signals).map_err(ApiError::from)
    } else {
        Ok(Json(result).into_response())
    }
}

fn default_tasting_notes() -> TastingNotesInput {
    TastingNotesInput::Text(String::new())
}

#[derive(Debug, Deserialize)]
pub(crate) struct BagScanSubmission {
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    roaster_name: String,
    #[serde(default)]
    roaster_country: String,
    roaster_city: Option<String>,
    roaster_homepage: Option<String>,
    #[serde(default)]
    roast_name: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    region: String,
    #[serde(default)]
    producer: String,
    #[serde(default)]
    process: String,
    #[serde(default = "default_tasting_notes")]
    tasting_notes: TastingNotesInput,
    #[serde(default)]
    open_bag: Option<String>,
    #[serde(default)]
    bag_amount: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ScanResult {
    redirect: String,
    roast_id: i64,
}

/// Populate a `BagScanSubmission` from AI extraction when image/prompt is provided.
/// Returns the usage data so the caller can record it.
async fn extract_into_submission(
    state: &AppState,
    submission: &mut BagScanSubmission,
) -> Result<Option<Usage>, ApiError> {
    let input = ExtractionInput {
        image: submission.image.take(),
        prompt: submission.prompt.take(),
    };
    let (result, usage) = ai::extract_bag_scan(
        &state.http_client,
        &state.openrouter_api_key,
        &state.openrouter_model,
        &input,
    )
    .await
    .map_err(ApiError::from)?;

    if let Some(name) = result.roaster.name {
        submission.roaster_name = name;
    }
    if let Some(country) = result.roaster.country {
        submission.roaster_country = country;
    }
    if result.roaster.city.is_some() {
        submission.roaster_city = result.roaster.city;
    }
    if result.roaster.homepage.is_some() {
        submission.roaster_homepage = result.roaster.homepage;
    }
    if let Some(name) = result.roast.name {
        submission.roast_name = name;
    }
    if let Some(origin) = result.roast.origin {
        submission.origin = origin;
    }
    if let Some(region) = result.roast.region {
        submission.region = region;
    }
    if let Some(producer) = result.roast.producer {
        submission.producer = producer;
    }
    if let Some(process) = result.roast.process {
        submission.process = process;
    }
    if let Some(notes) = result.roast.tasting_notes {
        submission.tasting_notes = TastingNotesInput::Text(notes.join(", "));
    }

    Ok(usage)
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(state, auth_user, headers, payload))]
pub(crate) async fn submit_scan(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    headers: HeaderMap,
    payload: FlexiblePayload<BagScanSubmission>,
) -> Result<Response, ApiError> {
    let (mut submission, _) = payload.into_parts();

    // Check for raw input (image/prompt triggers extraction first)
    let has_raw_input = submission.image.as_deref().is_some_and(|s| !s.is_empty())
        || submission.prompt.as_deref().is_some_and(|s| !s.is_empty());

    if has_raw_input {
        let usage = extract_into_submission(&state, &mut submission).await?;
        crate::application::routes::support::record_ai_usage(
            state.ai_usage_repo.clone(),
            auth_user.0.id,
            &state.openrouter_model,
            "extract-bag-scan",
            usage,
        );
    }

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
    if !has_raw_input && tasting_notes.is_empty() {
        return Err(AppError::validation("tasting notes are required").into());
    }

    let new_roast = if has_raw_input {
        if submission.roast_name.trim().is_empty() {
            return Err(
                AppError::validation("could not extract a roast name from the image/text").into(),
            );
        }
        NewRoast {
            roaster_id: roaster.id,
            name: submission.roast_name.trim().to_string(),
            origin: submission.origin.trim().to_string(),
            region: submission.region.trim().to_string(),
            producer: submission.producer.trim().to_string(),
            process: submission.process.trim().to_string(),
            tasting_notes,
        }
    } else {
        fn require(field: &str, value: &str) -> Result<String, AppError> {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(AppError::validation(format!("{field} is required")))
            } else {
                Ok(trimmed.to_string())
            }
        }

        NewRoast {
            roaster_id: roaster.id,
            name: require("roast name", &submission.roast_name)?,
            origin: require("origin", &submission.origin)?,
            region: require("region", &submission.region)?,
            producer: require("producer", &submission.producer)?,
            process: require("process", &submission.process)?,
            tasting_notes,
        }
    };

    let roast = state
        .roast_repo
        .insert(new_roast)
        .await
        .map_err(AppError::from)?;

    info!(roaster_id = %roaster.id, roast_id = %roast.id, roast_name = %roast.name, "scan created roast");

    // Optionally create a bag
    let wants_bag = submission
        .open_bag
        .as_deref()
        .is_some_and(|v| v == "true" || v == "on");
    if wants_bag {
        let amount = submission.bag_amount.unwrap_or(250.0);
        let new_bag = NewBag {
            roast_id: roast.id,
            roast_date: None,
            amount,
        };
        let bag = state
            .bag_repo
            .insert(new_bag)
            .await
            .map_err(AppError::from)?;

        let event = NewTimelineEvent {
            entity_type: "bag".to_string(),
            entity_id: bag.id.into_inner(),
            action: "added".to_string(),
            occurred_at: chrono::Utc::now(),
            title: roast.name.clone(),
            details: vec![
                TimelineEventDetail {
                    label: "Roaster".to_string(),
                    value: roaster.name.clone(),
                },
                TimelineEventDetail {
                    label: "Amount".to_string(),
                    value: format!("{}g", bag.amount),
                },
            ],
            tasting_notes: vec![],
            slug: Some(roast.slug.clone()),
            roaster_slug: Some(roaster.slug.clone()),
            brew_data: None,
        };
        if let Err(err) = state.timeline_repo.insert(event).await {
            warn!(error = %err, entity_type = "bag", "failed to record timeline event");
        }
    }

    let redirect = format!("/roasters/{}/roasts/{}", roaster.slug, roast.slug);
    let roast_id = roast.id.into_inner();

    if is_datastar_request(&headers) {
        use serde_json::Value;
        let signals = vec![
            ("_roast-id", Value::String(roast_id.to_string())),
            ("_scan-success", Value::String(roast.name.clone())),
            ("_roaster-name", Value::String(roaster.name.clone())),
        ];
        crate::application::routes::support::render_signals_json(&signals).map_err(ApiError::from)
    } else {
        Ok((StatusCode::CREATED, Json(ScanResult { redirect, roast_id })).into_response())
    }
}
