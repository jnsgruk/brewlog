use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::warn;

use crate::application::routes::render_html;
use crate::application::server::AppState;
use crate::infrastructure::auth::hash_token;

// --- Templates ---

#[derive(Template)]
#[template(path = "pages/register.html")]
struct RegisterTemplate {
    nav_active: &'static str,
    is_authenticated: bool,
    version_info: &'static crate::VersionInfo,
    token: String,
}

#[derive(Template)]
#[template(path = "pages/cli_callback.html")]
struct CliCallbackTemplate {
    nav_active: &'static str,
    is_authenticated: bool,
    version_info: &'static crate::VersionInfo,
    token: Option<String>,
    error: Option<String>,
}

// --- Registration page (bootstrap flow) ---

#[tracing::instrument(skip(state))]
pub(crate) async fn register_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response, StatusCode> {
    // Validate the token exists and is usable
    let token_hash = hash_token(&token);
    let reg_token = state
        .registration_token_repo
        .get_by_token_hash(&token_hash)
        .await
        .map_err(|err| {
            warn!(
                %err,
                token_hash_prefix = &token_hash[..8],
                "registration token lookup failed"
            );
            StatusCode::NOT_FOUND
        })?;

    if !reg_token.is_valid() {
        return Err(StatusCode::GONE);
    }

    let template = RegisterTemplate {
        nav_active: "",
        is_authenticated: false,
        version_info: &crate::VERSION_INFO,
        token,
    };

    render_html(template).map(IntoResponse::into_response)
}

// --- CLI callback page ---

pub(crate) async fn cli_callback_page() -> Result<Response, StatusCode> {
    let template = CliCallbackTemplate {
        nav_active: "",
        is_authenticated: false,
        version_info: &crate::VERSION_INFO,
        token: None,
        error: None,
    };

    render_html(template).map(IntoResponse::into_response)
}
