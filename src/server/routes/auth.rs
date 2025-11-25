use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, warn};

use crate::domain::sessions::NewSession;
use crate::infrastructure::auth::{generate_session_token, hash_token, verify_password};
use crate::server::routes::render_html;
use crate::server::server::AppState;

const SESSION_COOKIE_NAME: &str = "brewlog_session";

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    nav_active: &'static str,
    is_authenticated: bool,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub(crate) async fn login_page(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, StatusCode> {
    // Check if already authenticated
    if is_authenticated(&state, &cookies).await {
        return Ok(Redirect::to("/timeline").into_response());
    }

    let template = LoginTemplate {
        nav_active: "login",
        is_authenticated: false,
        error: None,
    };

    render_html(template).map(IntoResponse::into_response)
}

pub(crate) async fn login_submit(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Result<Response, StatusCode> {
    // Validate credentials
    let user = match state.user_repo.get_by_username(&form.username).await {
        Ok(user) => user,
        Err(err) => {
            warn!(username = %form.username, error = %err, "login attempt with non-existent username or error");
            return show_login_error("Invalid username or password");
        }
    };

    // Verify password
    if !verify_password(&form.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        warn!(username = %form.username, "login attempt with incorrect password");
        return show_login_error("Invalid username or password");
    }

    // Create session token
    let session_token = generate_session_token();
    let session_token_hash = hash_token(&session_token);

    // Create session in database (valid for 30 days)
    let new_session = NewSession::new(
        user.id,
        session_token_hash,
        Utc::now(),
        Utc::now() + Duration::days(30),
    );

    if let Err(err) = state.session_repo.insert(new_session).await {
        error!(error = %err, "failed to create session");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Set secure cookie
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);

    // Enable secure flag if BREWLOG_SECURE_COOKIES is set to "true"
    // This should be enabled in production when serving over HTTPS
    if std::env::var("BREWLOG_SECURE_COOKIES").unwrap_or_default() == "true" {
        cookie.set_secure(true);
    }

    cookies.add(cookie);

    Ok(Redirect::to("/timeline").into_response())
}

pub(crate) async fn logout(State(state): State<AppState>, cookies: Cookies) -> Redirect {
    // Try to delete session from database if cookie exists
    if let Some(cookie) = cookies.get(SESSION_COOKIE_NAME) {
        let session_token = cookie.value();
        let session_token_hash = hash_token(session_token);

        // Try to find and delete the session
        if let Ok(session) = state
            .session_repo
            .get_by_token_hash(&session_token_hash)
            .await
        {
            let _ = state.session_repo.delete(session.id).await;
        }
    }

    cookies.remove(Cookie::from(SESSION_COOKIE_NAME));
    Redirect::to("/timeline")
}

fn show_login_error(message: &str) -> Result<Response, StatusCode> {
    let template = LoginTemplate {
        nav_active: "login",
        is_authenticated: false,
        error: Some(message.to_string()),
    };

    render_html(template).map(IntoResponse::into_response)
}

/// Check if user is authenticated based on session cookie
/// Validates the session token against the database
pub async fn is_authenticated(state: &AppState, cookies: &Cookies) -> bool {
    let Some(cookie) = cookies.get(SESSION_COOKIE_NAME) else {
        return false;
    };

    let session_token = cookie.value();
    let session_token_hash = hash_token(session_token);

    // Check if session exists and is valid
    match state
        .session_repo
        .get_by_token_hash(&session_token_hash)
        .await
    {
        Ok(session) => !session.is_expired(),
        Err(_) => false,
    }
}
