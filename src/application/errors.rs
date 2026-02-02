use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::domain::RepositoryError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    pub fn new<T: ToString>(message: T) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

pub struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::NotFound => (StatusCode::NOT_FOUND, "entity not found".to_string()),
            AppError::Unexpected(message) => {
                error!(error = %message, "unexpected application error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "unexpected error".to_string(),
                )
            }
        };

        (status, Json(ErrorResponse::new(message))).into_response()
    }
}

pub fn map_app_error(err: AppError) -> StatusCode {
    match err {
        AppError::Validation(_) => StatusCode::BAD_REQUEST,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::NotFound => StatusCode::NOT_FOUND,
        AppError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("entity not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

impl AppError {
    pub fn validation<T: ToString>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }

    pub fn unexpected<T: ToString>(msg: T) -> Self {
        Self::Unexpected(msg.to_string())
    }
}
impl From<RepositoryError> for AppError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound => Self::NotFound,
            RepositoryError::Conflict(msg) => Self::Conflict(msg),
            RepositoryError::Unexpected(msg) => Self::Unexpected(msg),
        }
    }
}
