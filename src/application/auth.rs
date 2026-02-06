use axum::{
    extract::{FromRequestParts, Request},
    http::{StatusCode, header, request::Parts},
};
use tower_cookies::Cookies;
use tracing::warn;

use crate::application::state::AppState;
use crate::domain::users::User;
use crate::infrastructure::auth::hash_token;

const SESSION_COOKIE_NAME: &str = "brewlog_session";

/// Extension type to carry authenticated user through request handlers
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub User);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to get from extensions first (if middleware already set it)
        if let Some(user) = parts.extensions.get::<AuthenticatedUser>() {
            return Ok(user.clone());
        }

        // Try to authenticate via session cookie first
        if let Ok(cookies) = Cookies::from_request_parts(parts, state).await
            && let Some(user) = authenticate_via_session(state, &cookies).await
        {
            return Ok(AuthenticatedUser(user));
        }

        // Fall back to Bearer token authentication
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let auth_str = auth_header.to_str().map_err(|err| {
            warn!(error = %err, "authorization header contains invalid characters");
            StatusCode::UNAUTHORIZED
        })?;

        // Check for "Bearer <token>" format
        let token = auth_str
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Hash the token to look it up in the database
        let token_hash = hash_token(token);

        // Look up the token
        let token_record = state
            .token_repo
            .get_by_token_hash(&token_hash)
            .await
            .map_err(|err| {
                warn!(error = %err, "bearer token lookup failed");
                StatusCode::UNAUTHORIZED
            })?;

        // Check if token is revoked
        if token_record.is_revoked() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Update last used timestamp (fire and forget)
        let token_repo = state.token_repo.clone();
        let token_id = token_record.id;
        tokio::spawn(async move {
            if let Err(err) = token_repo.update_last_used(token_id).await {
                warn!(error = %err, %token_id, "failed to update token last_used");
            }
        });

        // Get the user
        let user = state
            .user_repo
            .get(token_record.user_id)
            .await
            .map_err(|err| {
                warn!(error = %err, user_id = %token_record.user_id, "user lookup failed for valid token");
                StatusCode::UNAUTHORIZED
            })?;

        Ok(AuthenticatedUser(user))
    }
}

/// Authenticate via session cookie
async fn authenticate_via_session(state: &AppState, cookies: &Cookies) -> Option<User> {
    let cookie = cookies.get(SESSION_COOKIE_NAME)?;
    let session_token = cookie.value();
    let session_token_hash = hash_token(session_token);

    // Check if session exists and is valid
    let session = match state
        .session_repo
        .get_by_token_hash(&session_token_hash)
        .await
    {
        Ok(s) => s,
        Err(err) => {
            warn!(error = %err, "session lookup failed during authentication");
            return None;
        }
    };

    if session.is_expired() {
        return None;
    }

    // Get the user
    match state.user_repo.get(session.user_id).await {
        Ok(user) => Some(user),
        Err(err) => {
            warn!(error = %err, user_id = %session.user_id, "user lookup failed for valid session");
            None
        }
    }
}

/// Helper to extract authenticated user from request extensions
pub fn get_authenticated_user(request: &Request) -> Option<&User> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|auth| &auth.0)
}
