use thirtyfour::prelude::*;

use super::browser::BrowserSession;
use super::server_helpers::{TestApp, create_session};

/// Authenticate the browser session by injecting a session cookie.
///
/// WebAuthn login can't be automated in headless Chrome, so we create a valid
/// session directly in the database and set the cookie in the browser.
pub async fn authenticate_browser(session: &BrowserSession, app: &TestApp) -> WebDriverResult<()> {
    // Must visit the domain first before setting cookies
    session.goto("/login").await?;

    let session_token = create_session(app).await;

    let mut cookie = Cookie::new("brewlog_session", &session_token);
    cookie.set_path("/");
    session.driver.add_cookie(cookie).await?;

    Ok(())
}
