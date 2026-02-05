use askama::Template;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tower_cookies::Cookies;

use crate::application::auth::AuthenticatedUser;
use crate::application::routes::render_html;
use crate::application::server::AppState;
use crate::domain::ids::PasskeyCredentialId;

use super::auth::is_authenticated;

// --- View types ---

#[derive(Serialize)]
pub struct AiUsageView {
    pub total_calls: i64,
    pub total_tokens: String,
    pub total_cost: String,
}

fn format_cost(cost: f64) -> String {
    if cost < 0.01 {
        format!("${cost:.4}")
    } else {
        format!("${cost:.2}")
    }
}

fn format_number(n: i64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[derive(Serialize)]
pub struct PasskeyView {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Serialize)]
pub struct TokenView {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

fn format_date(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

// --- Templates ---

#[derive(Template)]
#[template(path = "pages/account.html")]
struct AccountTemplate {
    nav_active: &'static str,
    is_authenticated: bool,
    version_info: &'static crate::VersionInfo,
    ai_usage: Option<AiUsageView>,
    passkeys: Vec<PasskeyView>,
    tokens: Vec<TokenView>,
}

// --- Page handler ---

pub(crate) async fn account_page(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, StatusCode> {
    if !is_authenticated(&state, &cookies).await {
        return Ok(Redirect::to("/login").into_response());
    }

    // We need the authenticated user for repo queries — re-extract from session
    let auth_user = extract_user_from_session(&state, &cookies)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let passkeys = state
        .passkey_repo
        .list_by_user(auth_user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|p| PasskeyView {
            id: i64::from(p.id),
            name: p.name,
            created_at: format_date(p.created_at),
            last_used_at: p.last_used_at.map(format_date),
        })
        .collect();

    let tokens = state
        .token_repo
        .list_by_user(auth_user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(crate::domain::tokens::Token::is_active)
        .map(|t| TokenView {
            id: i64::from(t.id),
            name: t.name,
            created_at: format_date(t.created_at),
            last_used_at: t.last_used_at.map(format_date),
        })
        .collect();

    let ai_usage = state
        .ai_usage_repo
        .summary_for_user(auth_user.id)
        .await
        .ok()
        .filter(|s| s.total_calls > 0)
        .map(|s| AiUsageView {
            total_calls: s.total_calls,
            total_tokens: format_number(s.total_tokens),
            total_cost: format_cost(s.total_cost),
        });

    let template = AccountTemplate {
        nav_active: "account",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        ai_usage,
        passkeys,
        tokens,
    };

    render_html(template).map(IntoResponse::into_response)
}

// --- Passkey API ---

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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    let passkey = state
        .passkey_repo
        .get(passkey_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if passkey.user_id != auth_user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Ensure the user has more than one passkey
    let all_passkeys = state
        .passkey_repo
        .list_by_user(auth_user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if all_passkeys.len() <= 1 {
        return Err(StatusCode::CONFLICT);
    }

    state
        .passkey_repo
        .delete(passkey_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Helpers ---

async fn extract_user_from_session(
    state: &AppState,
    cookies: &Cookies,
) -> Option<crate::domain::users::User> {
    let cookie = cookies.get("brewlog_session")?;
    let session_token = cookie.value();
    let session_token_hash = crate::infrastructure::auth::hash_token(session_token);

    let session = state
        .session_repo
        .get_by_token_hash(&session_token_hash)
        .await
        .ok()?;

    if session.is_expired() {
        return None;
    }

    state.user_repo.get(session.user_id).await.ok()
}
