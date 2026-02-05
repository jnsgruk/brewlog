use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use crate::application::routes::render_html;
use crate::application::server::AppState;
use crate::infrastructure::auth::hash_token;

const SESSION_COOKIE_NAME: &str = "brewlog_session";

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub cli_callback: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    nav_active: &'static str,
    is_authenticated: bool,
}

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn login_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<LoginQuery>,
) -> Result<Response, StatusCode> {
    // Don't redirect when CLI callback params are present — the user needs
    // to authenticate with their passkey to generate a bearer token for the CLI.
    if query.cli_callback.is_none() && is_authenticated(&state, &cookies).await {
        return Ok(Redirect::to("/").into_response());
    }

    let template = LoginTemplate {
        nav_active: "login",
        is_authenticated: false,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn logout(State(state): State<AppState>, cookies: Cookies) -> Redirect {
    if let Some(cookie) = cookies.get(SESSION_COOKIE_NAME) {
        let session_token = cookie.value();
        let session_token_hash = hash_token(session_token);

        if let Ok(session) = state
            .session_repo
            .get_by_token_hash(&session_token_hash)
            .await
        {
            let _ = state.session_repo.delete(session.id).await;
        }
    }

    cookies.remove(Cookie::from(SESSION_COOKIE_NAME));
    Redirect::to("/")
}

#[tracing::instrument(skip(state, cookies))]
pub async fn is_authenticated(state: &AppState, cookies: &Cookies) -> bool {
    let Some(cookie) = cookies.get(SESSION_COOKIE_NAME) else {
        return false;
    };

    let session_token = cookie.value();
    let session_token_hash = hash_token(session_token);

    match state
        .session_repo
        .get_by_token_hash(&session_token_hash)
        .await
    {
        Ok(session) => !session.is_expired(),
        Err(_) => false,
    }
}
