use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use tracing::warn;
use crate::infrastructure::auth::{generate_session_token, verify_password};
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

pub(crate) async fn login_page(cookies: Cookies) -> Result<Response, StatusCode> {
    // Check if already authenticated
    if cookies.get(SESSION_COOKIE_NAME).is_some() {
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

    // Set secure cookie
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    // In production, set secure flag: cookie.set_secure(true);

    cookies.add(cookie);

    Ok(Redirect::to("/timeline").into_response())
}

pub(crate) async fn logout(cookies: Cookies) -> Redirect {
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
pub fn is_authenticated(cookies: &Cookies) -> bool {
    cookies.get(SESSION_COOKIE_NAME).is_some()
}
