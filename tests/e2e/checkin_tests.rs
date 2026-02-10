use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::forms::select_searchable;
use crate::helpers::server_helpers::{
    create_default_cafe, create_default_roast, create_default_roaster, spawn_app_with_auth,
};
use crate::helpers::wait::{wait_for_url_contains, wait_for_visible};

#[tokio::test]
async fn checkin_with_saved_cafe_and_existing_roast() {
    let app = spawn_app_with_auth().await;

    // Pre-create entities via API
    let _cafe = create_default_cafe(&app).await;
    let roaster = create_default_roaster(&app).await;
    let _roast = create_default_roast(&app, roaster.id).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Navigate to check-in page
    session.goto("/check-in").await.unwrap();

    // Step 1: Select a saved cafe
    // Wait for the saved cafes searchable-select to be visible
    wait_for_visible(&session.driver, "searchable-select[name='saved_cafe_id']")
        .await
        .unwrap();

    // Select "Blue Bottle" from saved cafes
    // This triggers data-on:change which sets $_step = 2
    select_searchable(&session.driver, "saved_cafe_id", "Blue Bottle")
        .await
        .unwrap();

    // Step 2: Select an existing roast
    // Wait for step 2 content to become visible
    wait_for_visible(&session.driver, "searchable-select[name='roast_id']")
        .await
        .unwrap();

    // Select "Test Roast" from existing roasts
    // This triggers data-on:change which sets $_step = 3
    select_searchable(&session.driver, "roast_id", "Test Roast")
        .await
        .unwrap();

    // Step 3: Submit the check-in
    // Wait for the submit button to be visible
    let submit_btn = wait_for_visible(&session.driver, "button[type='submit']")
        .await
        .unwrap();

    // Verify the review section shows the selected cafe and coffee
    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("Blue Bottle"),
        "Review should show the selected cafe"
    );
    assert!(
        body_text.contains("Test Roast"),
        "Review should show the selected roast"
    );

    // Click the Check In button — Datastar @post → redirect script → /cups/...
    submit_btn.click().await.unwrap();

    // Should redirect to the cup detail page
    wait_for_url_contains(&session.driver, "/cups/")
        .await
        .unwrap();

    // Verify the cup detail page shows the cafe and roast
    let detail_body = session.driver.find(By::Css("body")).await.unwrap();
    let detail_text = detail_body.text().await.unwrap();
    assert!(
        detail_text.contains("Blue Bottle"),
        "Cup detail should show the cafe name"
    );
    assert!(
        detail_text.contains("Test Roast"),
        "Cup detail should show the roast name"
    );

    session.quit().await;
}
