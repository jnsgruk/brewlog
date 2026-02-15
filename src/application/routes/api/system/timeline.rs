use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::info;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::ApiError;
use crate::application::state::AppState;

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn rebuild_timeline(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
) -> Result<Response, ApiError> {
    info!("timeline rebuild requested");
    state.timeline_invalidator.rebuild_all();
    Ok(StatusCode::NO_CONTENT.into_response())
}
