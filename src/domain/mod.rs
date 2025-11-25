pub mod ids;
pub mod listing;
pub mod repositories;
pub mod roasters;
pub mod roasts;
pub mod sessions;
pub mod timeline;
pub mod tokens;
pub mod users;
use std::fmt::Display;

use thiserror::Error;

// TODO: This should probably be in a file named errors.rs
// so that it's domain::errors::RepositoryError.
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("entity not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unexpected data store error: {0}")]
    Unexpected(String),
}

impl RepositoryError {
    pub fn conflict<T: Display>(message: T) -> Self {
        Self::Conflict(message.to_string())
    }

    pub fn unexpected<T: Display>(message: T) -> Self {
        Self::Unexpected(message.to_string())
    }
}
