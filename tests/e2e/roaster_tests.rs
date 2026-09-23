use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::forms::{fill_input, submit_visible_form};
use crate::helpers::server_helpers::spawn_app_with_auth;
use crate::helpers::wait::{wait_for_element, wait_for_url_contains, wait_for_visible};

#[tokio::test]
async fn create_roaster_submit_tracks_required_and_malformed_fields() {
    let app = spawn_app_with_auth().await;
    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();
    session.goto("/add").await.unwrap();

    let submit = "form[action='/api/v1/roasters'] button[type='submit']";
    wait_for_element(&session.driver, &format!("{submit}:disabled"))
        .await
        .unwrap();
    fill_input(&session.driver, "name", "   ").await.unwrap();
    fill_input(&session.driver, "country", "United Kingdom")
        .await
        .unwrap();
    wait_for_element(&session.driver, &format!("{submit}:disabled"))
        .await
        .unwrap();

    fill_input(&session.driver, "name", "Valid Roaster")
        .await
        .unwrap();
    wait_for_element(&session.driver, &format!("{submit}:not(:disabled)"))
        .await
        .unwrap();
    fill_input(&session.driver, "homepage", "not a URL")
        .await
        .unwrap();
    wait_for_element(&session.driver, &format!("{submit}:disabled"))
        .await
        .unwrap();
    fill_input(&session.driver, "homepage", "https://example.com")
        .await
        .unwrap();
    wait_for_element(&session.driver, &format!("{submit}:not(:disabled)"))
        .await
        .unwrap();
    session.quit().await;
}

#[tokio::test]
async fn create_roaster_via_add_page() {
    let app = spawn_app_with_auth().await;
    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Navigate to /add — roaster tab is the default
    session.goto("/add").await.unwrap();

    // Wait for Datastar to initialize and show the roaster form
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // Fill the roaster form
    fill_input(&session.driver, "name", "E2E Test Roasters")
        .await
        .unwrap();
    fill_input(&session.driver, "country", "United Kingdom")
        .await
        .unwrap();

    // Submit — standard method=post, redirects to detail page
    submit_visible_form(&session.driver).await.unwrap();

    // Should redirect to the roaster detail page
    wait_for_url_contains(&session.driver, "/roasters/")
        .await
        .unwrap();

    // Verify the detail page shows the roaster name
    let heading = session.driver.find(By::Css("h1")).await.unwrap();
    let heading_text = heading.text().await.unwrap();
    assert_eq!(heading_text, "E2E Test Roasters");

    // Navigate to the data page and verify the roaster appears in the list
    session.goto("/data?type=roasters").await.unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("E2E Test Roasters"),
        "Roaster should appear in the data list"
    );

    session.quit().await;
}

#[tokio::test]
async fn create_roaster_back_restores_submit_button() {
    let app = spawn_app_with_auth().await;
    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();
    session.goto("/add").await.unwrap();

    fill_input(&session.driver, "name", "Before Back Roasters")
        .await
        .unwrap();
    fill_input(&session.driver, "country", "United Kingdom")
        .await
        .unwrap();
    let submit = "form[action='/api/v1/roasters'] button[type='submit']";
    wait_for_element(&session.driver, &format!("{submit}:not(:disabled)"))
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/roasters/")
        .await
        .unwrap();

    session.driver.back().await.unwrap();
    wait_for_url_contains(&session.driver, "/add")
        .await
        .unwrap();
    wait_for_element(&session.driver, &format!("{submit}:not(:disabled)"))
        .await
        .unwrap();
    let form = session
        .driver
        .find(By::Css("form[action='/api/v1/roasters']"))
        .await
        .unwrap();
    assert_ne!(
        form.attr("data-validation-submitting")
            .await
            .unwrap()
            .as_deref(),
        Some("true")
    );

    fill_input(&session.driver, "name", "After Back Roasters")
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/roasters/after-back-roasters")
        .await
        .unwrap();
    session.quit().await;
}

#[tokio::test]
async fn create_roaster_with_all_fields() {
    let app = spawn_app_with_auth().await;
    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/add").await.unwrap();

    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    fill_input(&session.driver, "name", "Full Detail Roasters")
        .await
        .unwrap();
    fill_input(&session.driver, "country", "Ethiopia")
        .await
        .unwrap();
    fill_input(&session.driver, "city", "Addis Ababa")
        .await
        .unwrap();
    fill_input(&session.driver, "homepage", "https://example.coffee")
        .await
        .unwrap();

    submit_visible_form(&session.driver).await.unwrap();

    wait_for_url_contains(&session.driver, "/roasters/")
        .await
        .unwrap();

    // Verify detail page shows country and city
    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(body_text.contains("Full Detail Roasters"));
    assert!(body_text.contains("Ethiopia"));
    assert!(body_text.contains("Addis Ababa"));

    session.quit().await;
}
