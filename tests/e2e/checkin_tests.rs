use thirtyfour::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::forms::select_searchable;
use crate::helpers::server_helpers::{
    create_default_cafe, create_default_roast, create_default_roaster, spawn_app_with_all_mocks,
    spawn_app_with_auth,
};
use crate::helpers::wait::{wait_for_text, wait_for_url_contains, wait_for_visible};

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

fn mock_foursquare_response() -> ResponseTemplate {
    let body = serde_json::json!({
        "results": [{
            "name": "Prufrock Coffee",
            "latitude": 51.5246,
            "longitude": -0.1098,
            "location": {
                "locality": "London",
                "country": "GB"
            },
            "website": "https://www.prufrockcoffee.com",
            "distance": 2800
        }]
    });
    ResponseTemplate::new(200).set_body_json(body)
}

fn mock_openrouter_scan_response() -> ResponseTemplate {
    let extracted = serde_json::json!({
        "roaster": {
            "name": "Prufrock Roasters",
            "country": "GB",
            "city": "London"
        },
        "roast": {
            "name": "Kiandu AA",
            "origin": "Kenya",
            "region": "Nyeri",
            "producer": "Kiandu Factory",
            "process": "Washed",
            "tasting_notes": ["Blackcurrant", "Grapefruit"]
        }
    });
    let body = serde_json::json!({
        "id": "gen-test",
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": extracted.to_string()
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 100,
            "completion_tokens": 50,
            "total_tokens": 150,
            "cost": 0.001
        }
    });
    ResponseTemplate::new(200).set_body_json(body)
}

#[tokio::test]
async fn checkin_with_new_cafe_and_scanned_roast() {
    let app = spawn_app_with_all_mocks().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/places/search"))
        .respond_with(mock_foursquare_response())
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_scan_response())
        .mount(mock_server)
        .await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/check-in").await.unwrap();
    wait_for_visible(&session.driver, "#checkin-root")
        .await
        .unwrap();

    // Step 1: Search for a new cafe via location search
    // Type city name in the location input
    let city_input = session
        .driver
        .find(By::Css("[data-bind\\:_city-name]"))
        .await
        .unwrap();
    city_input.send_keys("London").await.unwrap();

    // Type cafe name — triggers debounced Foursquare search when both >= 2 chars
    let cafe_search = session
        .driver
        .find(By::Css("[data-bind\\:_cafe-search]"))
        .await
        .unwrap();
    cafe_search.send_keys("Prufrock").await.unwrap();

    // Wait for Foursquare results to appear
    wait_for_visible(&session.driver, "#nearby-results button")
        .await
        .unwrap();

    // Click the first result — use JS to avoid StaleElementReference from Datastar updates
    session
        .driver
        .execute(
            "document.querySelector('#nearby-results button').click()",
            vec![],
        )
        .await
        .unwrap();

    // Wait for the review form, then click "Next" to advance to step 2
    wait_for_text(&session.driver, "body", "Confirm cafe details")
        .await
        .unwrap();
    // Use JS to find+click atomically — avoids StaleElementReference from DOM updates
    session
        .driver
        .execute(
            r#"
            for (const btn of document.querySelectorAll('button')) {
                if (btn.offsetParent !== null && btn.textContent.includes('Next')) {
                    btn.click();
                    return;
                }
            }
            "#,
            vec![],
        )
        .await
        .unwrap();

    // Step 2: Scan a coffee bag via text prompt
    let prompt_input = wait_for_visible(&session.driver, "#checkin-scan-form input[name='prompt']")
        .await
        .unwrap();
    prompt_input
        .send_keys("Kiandu AA from Prufrock Roasters")
        .await
        .unwrap();
    prompt_input.send_keys(Key::Enter).await.unwrap();

    // Wait for scan to complete — step 3 becomes visible with the submit button
    let submit_btn = wait_for_visible(&session.driver, "button[type='submit']")
        .await
        .unwrap();

    // Verify the review section shows the new cafe and scanned coffee
    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("Prufrock Coffee"),
        "Review should show the new cafe name"
    );
    assert!(
        body_text.contains("Kiandu AA"),
        "Review should show the scanned roast name"
    );

    // Submit the check-in
    submit_btn.click().await.unwrap();

    // Should redirect to the cup detail page
    wait_for_url_contains(&session.driver, "/cups/")
        .await
        .unwrap();

    let detail_body = session.driver.find(By::Css("body")).await.unwrap();
    let detail_text = detail_body.text().await.unwrap();
    assert!(
        detail_text.contains("Prufrock Coffee"),
        "Cup detail should show the cafe name"
    );
    assert!(
        detail_text.contains("Kiandu AA"),
        "Cup detail should show the roast name"
    );

    session.quit().await;
}
