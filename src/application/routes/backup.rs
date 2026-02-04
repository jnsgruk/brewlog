use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::server::AppState;
use crate::infrastructure::backup::BackupData;

/// GET /api/v1/backup — export all data as JSON (requires authentication)
pub(crate) async fn export_backup(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
) -> Result<Json<BackupData>, ApiError> {
    let data = state
        .backup_service
        .export()
        .await
        .map_err(|e| AppError::unexpected(e.to_string()))?;
    Ok(Json(data))
}

/// POST /api/v1/backup/restore — restore from JSON backup (requires authentication)
pub(crate) async fn restore_backup(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Json(payload): Json<BackupData>,
) -> Result<Response, ApiError> {
    state.backup_service.restore(payload).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not empty") {
            ApiError::from(AppError::Conflict(msg))
        } else {
            ApiError::from(AppError::unexpected(msg))
        }
    })?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
