use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{StatusCode, header, request::Parts},
};
use tower_cookies::Cookies;

use crate::domain::users::User;
use crate::infrastructure::auth::hash_token;
use crate::server::server::AppState;

const SESSION_COOKIE_NAME: &str = "brewlog_session";

/// Extension type to carry authenticated user through request handlers
#[derive(Clone)]
pub struct AuthenticatedUser(pub User);

#[async_trait]
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

        let auth_str = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

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
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Check if token is revoked
        if token_record.is_revoked() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Update last used timestamp (fire and forget)
        let token_repo = state.token_repo.clone();
        let token_id = token_record.id;
        tokio::spawn(async move {
            let _ = token_repo.update_last_used(token_id).await;
        });

        // Get the user
        let user = state
            .user_repo
            .get(token_record.user_id)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser(user))
    }
}

/// Authenticate via session cookie
async fn authenticate_via_session(state: &AppState, cookies: &Cookies) -> Option<User> {
    let cookie = cookies.get(SESSION_COOKIE_NAME)?;
    let session_token = cookie.value();
    let session_token_hash = hash_token(session_token);

    // Check if session exists and is valid
    let session = state
        .session_repo
        .get_by_token_hash(&session_token_hash)
        .await
        .ok()?;

    if session.is_expired() {
        return None;
    }

    // Get the user
    state.user_repo.get(session.user_id).await.ok()
}

/// Helper to extract authenticated user from request extensions
pub fn get_authenticated_user(request: &Request) -> Option<&User> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|auth| &auth.0)
}
