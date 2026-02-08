use axum::Json;
use axum::extract::State;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::services::stats::compute_all_stats;
use crate::application::state::AppState;
use crate::domain::stats::CachedStats;

/// Force an immediate stats recomputation, bypassing the debounce timer.
#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn recompute_stats(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
) -> Result<Json<CachedStats>, ApiError> {
    let cached = compute_all_stats(&*state.stats_repo)
        .await
        .map_err(AppError::from)?;

    state
        .stats_repo
        .store_cached(&cached)
        .await
        .map_err(AppError::from)?;

    Ok(Json(cached))
}
