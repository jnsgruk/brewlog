use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use tracing::info;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::support::{FlexiblePayload, is_datastar_request, render_fragment};
use crate::application::state::AppState;
use crate::domain::images::EntityImage;
use crate::infrastructure::image_processing::process_data_url;
use crate::presentation::web::templates::ImageUploadTemplate;

const VALID_ENTITY_TYPES: &[&str] = &["roaster", "roast", "gear", "cafe", "brew", "cup"];

#[derive(Debug, Deserialize)]
pub(crate) struct ImageUpload {
    pub image: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ImagePath {
    pub entity_type: String,
    pub id: i64,
}

fn validate_entity_type(entity_type: &str) -> Result<(), ApiError> {
    if VALID_ENTITY_TYPES.contains(&entity_type) {
        Ok(())
    } else {
        Err(AppError::validation(format!("invalid entity type: {entity_type}")).into())
    }
}

async fn validate_entity_exists(
    state: &AppState,
    entity_type: &str,
    id: i64,
) -> Result<(), ApiError> {
    use crate::domain::ids::{BrewId, CafeId, CupId, GearId, RoastId, RoasterId};

    match entity_type {
        "roaster" => {
            state
                .roaster_repo
                .get(RoasterId::from(id))
                .await
                .map_err(AppError::from)?;
        }
        "roast" => {
            state
                .roast_repo
                .get(RoastId::from(id))
                .await
                .map_err(AppError::from)?;
        }
        "gear" => {
            state
                .gear_repo
                .get(GearId::from(id))
                .await
                .map_err(AppError::from)?;
        }
        "cafe" => {
            state
                .cafe_repo
                .get(CafeId::from(id))
                .await
                .map_err(AppError::from)?;
        }
        "brew" => {
            state
                .brew_repo
                .get(BrewId::from(id))
                .await
                .map_err(AppError::from)?;
        }
        "cup" => {
            state
                .cup_repo
                .get(CupId::from(id))
                .await
                .map_err(AppError::from)?;
        }
        _ => {
            return Err(AppError::validation(format!("invalid entity type: {entity_type}")).into());
        }
    }

    Ok(())
}

#[tracing::instrument(skip(state, _auth_user, headers, payload))]
pub(crate) async fn upload_image(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(path): Path<ImagePath>,
    payload: FlexiblePayload<ImageUpload>,
) -> Result<Response, ApiError> {
    validate_entity_type(&path.entity_type)?;
    validate_entity_exists(&state, &path.entity_type, path.id).await?;

    let (upload, _source) = payload.into_parts();

    let _permit = state
        .image_semaphore
        .acquire()
        .await
        .map_err(|_| AppError::unexpected("image processing unavailable"))?;

    let image_data = upload.image;
    let processed = tokio::task::spawn_blocking(move || process_data_url(&image_data))
        .await
        .map_err(|e| AppError::unexpected(format!("image processing task failed: {e}")))?
        .map_err(|e| AppError::validation(format!("invalid image: {e}")))?;

    let image = EntityImage {
        entity_type: path.entity_type.clone(),
        entity_id: path.id,
        content_type: processed.content_type,
        image_data: processed.image_data,
        thumbnail_data: processed.thumbnail_data,
    };

    state
        .image_repo
        .upsert(image)
        .await
        .map_err(AppError::from)?;

    info!(entity_type = %path.entity_type, entity_id = path.id, "image uploaded");

    if is_datastar_request(&headers) {
        let image_url = format!("/api/v1/{}/{}/image", path.entity_type, path.id);
        render_fragment(
            ImageUploadTemplate {
                entity_type: &path.entity_type,
                entity_id: path.id,
                image_url: Some(&image_url),
                is_authenticated: true,
            },
            "#entity-image",
        )
        .map_err(ApiError::from)
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn get_image(
    State(state): State<AppState>,
    Path(path): Path<ImagePath>,
) -> Result<Response, ApiError> {
    validate_entity_type(&path.entity_type)?;

    let image = state
        .image_repo
        .get(&path.entity_type, path.id)
        .await
        .map_err(AppError::from)?;

    Ok(image_response(image.image_data, &image.content_type))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn get_thumbnail(
    State(state): State<AppState>,
    Path(path): Path<ImagePath>,
) -> Result<Response, ApiError> {
    validate_entity_type(&path.entity_type)?;

    let image = state
        .image_repo
        .get_thumbnail(&path.entity_type, path.id)
        .await
        .map_err(AppError::from)?;

    Ok(image_response(image.thumbnail_data, &image.content_type))
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn delete_image(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(path): Path<ImagePath>,
) -> Result<Response, ApiError> {
    validate_entity_type(&path.entity_type)?;

    state
        .image_repo
        .delete(&path.entity_type, path.id)
        .await
        .map_err(AppError::from)?;

    info!(entity_type = %path.entity_type, entity_id = path.id, "image deleted");

    if is_datastar_request(&headers) {
        render_fragment(
            ImageUploadTemplate {
                entity_type: &path.entity_type,
                entity_id: path.id,
                image_url: None,
                is_authenticated: true,
            },
            "#entity-image",
        )
        .map_err(ApiError::from)
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

/// Check if an entity has an image and return its URL if so.
pub(crate) async fn resolve_image_url(
    state: &AppState,
    entity_type: &str,
    entity_id: i64,
) -> Option<String> {
    state
        .image_repo
        .has_image(entity_type, entity_id)
        .await
        .unwrap_or(false)
        .then(|| format!("/api/v1/{entity_type}/{entity_id}/image"))
}

/// Process and save a deferred image upload (from a create form).
/// Called after entity creation when the form included an image data URL.
/// Accepts `Option<&str>` and no-ops on `None` or empty strings.
pub(crate) async fn save_deferred_image(
    state: &AppState,
    entity_type: &str,
    entity_id: i64,
    data_url: Option<&str>,
) {
    let Some(data_url) = data_url.filter(|s| !s.is_empty()) else {
        return;
    };
    let Ok(_permit) = state.image_semaphore.acquire().await else {
        tracing::warn!(
            entity_type,
            entity_id,
            "image semaphore closed, skipping deferred image"
        );
        return;
    };

    let data_url = data_url.to_string();
    let processed = match tokio::task::spawn_blocking(move || process_data_url(&data_url)).await {
        Ok(Ok(p)) => p,
        Ok(Err(err)) => {
            tracing::warn!(entity_type, entity_id, error = %err, "failed to process deferred image");
            return;
        }
        Err(err) => {
            tracing::warn!(entity_type, entity_id, error = %err, "deferred image task panicked");
            return;
        }
    };
    let image = EntityImage {
        entity_type: entity_type.to_string(),
        entity_id,
        content_type: processed.content_type,
        image_data: processed.image_data,
        thumbnail_data: processed.thumbnail_data,
    };
    if let Err(err) = state.image_repo.upsert(image).await {
        tracing::warn!(entity_type, entity_id, error = %err, "failed to save deferred image");
    } else {
        info!(entity_type, entity_id, "deferred image saved");
    }
}

fn image_response(data: Vec<u8>, content_type: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=604800")
        .body(Body::from(data))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
