use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::domain::users::User;
use crate::infrastructure::auth::hash_token;
use crate::server::server::AppState;

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

        // Otherwise, extract from Authorization header directly
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
        let token_id = token_record.id.clone();
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

/// Middleware that optionally extracts authentication from the request
/// Sets AuthenticatedUser extension if valid token is found
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(user) = extract_user_from_request(&state, &request).await {
        request.extensions_mut().insert(AuthenticatedUser(user));
    }
    next.run(request).await
}

/// Middleware that requires authentication
/// Returns 401 if no valid token is found
pub async fn require_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    match extract_user_from_request(&state, &request).await {
        Some(user) => {
            request.extensions_mut().insert(AuthenticatedUser(user));
            next.run(request).await
        }
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn extract_user_from_request(state: &AppState, request: &Request) -> Option<User> {
    // Extract token from Authorization header
    let auth_header = request.headers().get(header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;

    // Check for "Bearer <token>" format
    let token = auth_str.strip_prefix("Bearer ")?;

    // Hash the token to look it up in the database
    let token_hash = hash_token(token);

    // Look up the token
    let token_record = state.token_repo.get_by_token_hash(&token_hash).await.ok()?;

    // Check if token is revoked
    if token_record.is_revoked() {
        return None;
    }

    // Update last used timestamp (fire and forget - don't block on this)
    let token_repo = state.token_repo.clone();
    let token_id = token_record.id.clone();
    tokio::spawn(async move {
        let _ = token_repo.update_last_used(token_id).await;
    });

    // Get the user
    state.user_repo.get(token_record.user_id).await.ok()
}

/// Helper to extract authenticated user from request extensions
pub fn get_authenticated_user(request: &Request) -> Option<&User> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|auth| &auth.0)
}
