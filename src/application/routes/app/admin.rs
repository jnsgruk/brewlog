use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tower_cookies::Cookies;
use tracing::{error, warn};

use crate::application::routes::render_html;
use crate::application::state::AppState;

// --- View types ---

#[derive(Serialize)]
pub struct AiUsageView {
    pub calls: i64,
    pub tokens: String,
    pub cost: String,
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
#[template(path = "pages/admin.html")]
struct AdminTemplate {
    nav_active: &'static str,
    is_authenticated: bool,
    version_info: &'static crate::VersionInfo,
    ai_usage: Option<AiUsageView>,
    passkeys: Vec<PasskeyView>,
    tokens: Vec<TokenView>,
}

// --- Page handler ---

pub(crate) async fn admin_page(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, StatusCode> {
    if !crate::application::routes::is_authenticated(&state, &cookies).await {
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
        .map_err(|err| {
            error!(error = %err, "failed to list passkeys for admin page");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
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
        .map_err(|err| {
            error!(error = %err, "failed to list tokens for admin page");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .filter(crate::domain::tokens::Token::is_active)
        .map(|t| TokenView {
            id: i64::from(t.id),
            name: t.name,
            created_at: format_date(t.created_at),
            last_used_at: t.last_used_at.map(format_date),
        })
        .collect();

    let ai_usage = match state.ai_usage_repo.summary_for_user(auth_user.id).await {
        Ok(summary) => Some(summary),
        Err(err) => {
            warn!(error = %err, "failed to load AI usage summary");
            None
        }
    };
    let ai_usage = ai_usage.filter(|s| s.total_calls > 0).map(|s| AiUsageView {
        calls: s.total_calls,
        tokens: format_number(s.total_tokens),
        cost: format_cost(s.total_cost),
    });

    let template = AdminTemplate {
        nav_active: "admin",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        ai_usage,
        passkeys,
        tokens,
    };

    render_html(template).map(IntoResponse::into_response)
}

// --- Helpers ---

async fn extract_user_from_session(
    state: &AppState,
    cookies: &Cookies,
) -> Option<crate::domain::users::User> {
    let cookie = cookies.get("brewlog_session")?;
    let session_token = cookie.value();
    let session_token_hash = crate::infrastructure::auth::hash_token(session_token);

    let session = match state
        .session_repo
        .get_by_token_hash(&session_token_hash)
        .await
    {
        Ok(s) => s,
        Err(err) => {
            warn!(error = %err, "session lookup failed on admin page");
            return None;
        }
    };

    if session.is_expired() {
        return None;
    }

    match state.user_repo.get(session.user_id).await {
        Ok(user) => Some(user),
        Err(err) => {
            warn!(error = %err, user_id = %session.user_id, "user lookup failed for valid session");
            None
        }
    }
}
