use axum::Json;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::state::AppState;
use crate::infrastructure::backup::BackupData;

/// GET /api/v1/backup — export all data as JSON (requires authentication)
///
/// Returns the backup with a `Content-Disposition: attachment` header so
/// browsers trigger a file download while API/CLI consumers can ignore it.
pub(crate) async fn export_backup(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
) -> Result<Response, ApiError> {
    let data = state
        .backup_service
        .export()
        .await
        .map_err(|e| AppError::unexpected(e.to_string()))?;

    let body = serde_json::to_string(&data).map_err(|e| AppError::unexpected(e.to_string()))?;

    let filename = format!(
        "brewlog-backup-{}.json",
        chrono::Utc::now().format("%Y-%m-%d")
    );

    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
        .into_response())
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
