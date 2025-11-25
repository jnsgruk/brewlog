use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{TokenId, UserId};
use crate::domain::tokens::{NewToken, Token};
use crate::infrastructure::auth::{generate_token, hash_token, verify_password};
use crate::server::auth::AuthenticatedUser;
use crate::server::server::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub id: TokenId,
    pub name: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub id: TokenId,
    pub user_id: UserId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<Token> for TokenResponse {
    fn from(token: Token) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            name: token.name,
            created_at: token.created_at,
            last_used_at: token.last_used_at,
            revoked_at: token.revoked_at,
        }
    }
}

pub async fn create_token(
    State(state): State<AppState>,
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<CreateTokenResponse>, StatusCode> {
    // Verify username and password
    let user = state
        .user_repo
        .get_by_username(&payload.username)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let password_valid = verify_password(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate new token
    let token_value = generate_token().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token_hash = hash_token(&token_value);

    let new_token = NewToken::new(user.id, token_hash, payload.name.clone());

    // Store token
    let stored_token = state
        .token_repo
        .insert(new_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateTokenResponse {
        id: stored_token.id,
        name: stored_token.name,
        token: token_value,
    }))
}

pub async fn list_tokens(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<TokenResponse>>, StatusCode> {
    let tokens = state
        .token_repo
        .list_by_user(auth_user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token_responses: Vec<TokenResponse> = tokens.into_iter().map(TokenResponse::from).collect();

    Ok(Json(token_responses))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(token_id): Path<TokenId>,
) -> Result<Json<TokenResponse>, StatusCode> {
    // Get the token to ensure it exists and belongs to the user
    let token = state
        .token_repo
        .get(token_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Ensure the token belongs to the authenticated user
    if token.user_id != auth_user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Revoke the token
    let revoked_token = state
        .token_repo
        .revoke(token_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenResponse::from(revoked_token)))
}
