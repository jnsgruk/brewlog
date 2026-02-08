use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::error;

use crate::application::auth::AuthenticatedUser;
use crate::application::state::AppState;
use crate::domain::ids::PasskeyCredentialId;

#[derive(Serialize)]
pub struct PasskeyResponse {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

pub(crate) async fn list_passkeys(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<PasskeyResponse>>, StatusCode> {
    let passkeys = state
        .passkey_repo
        .list_by_user(auth_user.0.id)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to list passkeys");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let responses: Vec<PasskeyResponse> = passkeys
        .into_iter()
        .map(|p| PasskeyResponse {
            id: i64::from(p.id),
            name: p.name,
            created_at: p.created_at,
            last_used_at: p.last_used_at,
        })
        .collect();

    Ok(Json(responses))
}

pub(crate) async fn delete_passkey(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(passkey_id): Path<PasskeyCredentialId>,
) -> Result<StatusCode, StatusCode> {
    // Verify the passkey belongs to the user
    let passkey = state.passkey_repo.get(passkey_id).await.map_err(|err| {
        error!(error = %err, %passkey_id, "failed to get passkey for deletion");
        StatusCode::NOT_FOUND
    })?;

    if passkey.user_id != auth_user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Ensure the user has more than one passkey
    let all_passkeys = state
        .passkey_repo
        .list_by_user(auth_user.0.id)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to list passkeys for deletion check");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if all_passkeys.len() <= 1 {
        return Err(StatusCode::CONFLICT);
    }

    state.passkey_repo.delete(passkey_id).await.map_err(|err| {
        error!(error = %err, %passkey_id, "failed to delete passkey");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}
