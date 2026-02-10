use thirtyfour::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::server_helpers::{
    create_default_roast, create_default_roaster, spawn_app_with_openrouter_mock,
};
use crate::helpers::wait::{wait_for_text, wait_for_url_contains, wait_for_visible};

fn mock_openrouter_response(json_content: &str) -> ResponseTemplate {
    let body = serde_json::json!({
        "id": "gen-test",
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": json_content
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
async fn extract_roaster_via_text_prompt_populates_form() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    // Mount mock that returns extracted roaster data
    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"name": "Square Mile", "country": "United Kingdom", "city": "London", "homepage": "https://squaremilecoffee.com"}"#,
        ))
        .mount(mock_server)
        .await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Navigate to /add (roaster tab is default)
    session.goto("/add").await.unwrap();

    // Wait for the extraction prompt to be visible
    let prompt_input = wait_for_visible(
        &session.driver,
        "input[name='prompt'][form='roaster-extract-form']",
    )
    .await
    .unwrap();

    // Type a description and press Enter to submit the extraction form
    prompt_input.send_keys("Square Mile Coffee").await.unwrap();
    prompt_input.send_keys(Key::Enter).await.unwrap();

    // Wait for extraction to complete — the name field should be populated
    // Datastar patches signals which update form fields via data-bind
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Verify the main form fields were populated by the signal patch
    let name_input = session
        .driver
        .find(By::Css(
            "form[action='/api/v1/roasters'] input[name='name']",
        ))
        .await
        .unwrap();
    let name_value = name_input.value().await.unwrap().unwrap_or_default();
    assert_eq!(name_value, "Square Mile", "Name field should be populated");

    let country_input = session
        .driver
        .find(By::Css(
            "form[action='/api/v1/roasters'] input[name='country']",
        ))
        .await
        .unwrap();
    let country_value = country_input.value().await.unwrap().unwrap_or_default();
    assert_eq!(
        country_value, "United Kingdom",
        "Country field should be populated"
    );

    // Submit the main form to save the roaster
    let submit_button = session
        .driver
        .find(By::Css(
            "form[action='/api/v1/roasters'] button[type='submit']",
        ))
        .await
        .unwrap();
    submit_button.click().await.unwrap();

    // Should redirect to the roaster detail page
    wait_for_url_contains(&session.driver, "/roasters/")
        .await
        .unwrap();

    let heading = session.driver.find(By::Css("h1")).await.unwrap();
    assert_eq!(heading.text().await.unwrap(), "Square Mile");

    session.quit().await;
}

fn mock_bag_scan_response(json_content: &str) -> ResponseTemplate {
    mock_openrouter_response(json_content)
}

/// Trigger the homepage scan by setting the hidden image input and submitting the form.
/// The `brew-photo-capture` web component is image-only; we bypass it via JS.
async fn trigger_homepage_scan(driver: &WebDriver) {
    driver
        .execute(
            r#"
            document.getElementById('scan-image').value = 'data:image/png;base64,iVBOR';
            document.getElementById('scan-extract-form').requestSubmit();
            "#,
            vec![],
        )
        .await
        .unwrap();
}

/// Find and click a visible button whose text contains the given substring.
async fn click_button_with_text(driver: &WebDriver, text: &str) {
    let buttons = driver.find_all(By::Css("button")).await.unwrap();
    for button in buttons {
        if button.is_displayed().await.unwrap_or(false) {
            let btn_text = button.text().await.unwrap_or_default();
            if btn_text.contains(text) {
                button.click().await.unwrap();
                return;
            }
        }
    }
    panic!("No visible button containing '{text}' found");
}

#[tokio::test]
async fn homepage_scan_new_roaster_and_new_roast() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_bag_scan_response(
            r#"{"roaster": {"name": "Koppi", "country": "SE", "city": "Helsingborg"}, "roast": {"name": "Finca Vista", "origin": "Colombia", "region": "Huila", "producer": "Luis Anibal", "process": "Washed", "tasting_notes": ["Caramel", "Red Apple"]}}"#,
        ))
        .mount(mock_server)
        .await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/").await.unwrap();
    wait_for_visible(&session.driver, "brew-photo-capture")
        .await
        .unwrap();

    trigger_homepage_scan(&session.driver).await;

    // Wait for extraction to complete — the result form appears
    // Neither roaster nor roast matched → editable form with "Save Roaster & Roast" button
    wait_for_text(&session.driver, "body", "Save Roaster")
        .await
        .unwrap();

    // Verify extracted fields are populated in form inputs (not visible body text)
    let roaster_input = session
        .driver
        .find(By::Css("[data-bind\\:_roaster-name]"))
        .await
        .unwrap();
    let roaster_val = roaster_input.value().await.unwrap().unwrap_or_default();
    assert_eq!(roaster_val, "Koppi", "Roaster name input should be filled");

    let roast_input = session
        .driver
        .find(By::Css("[data-bind\\:_roast-name]"))
        .await
        .unwrap();
    let roast_val = roast_input.value().await.unwrap().unwrap_or_default();
    assert_eq!(
        roast_val, "Finca Vista",
        "Roast name input should be filled"
    );

    // Submit — creates roaster + roast + bag, then reloads
    click_button_with_text(&session.driver, "Save Roaster").await;

    // After reload, the new bag should appear in Open Bags
    wait_for_text(&session.driver, "#open-bags-section", "Finca Vista")
        .await
        .unwrap();

    session.quit().await;
}

#[tokio::test]
async fn homepage_scan_existing_roaster_new_roast() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    // Pre-create the roaster so the extraction matches it
    let _roaster = create_default_roaster(&app).await;

    // AI returns data matching the existing roaster name but a new roast
    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_bag_scan_response(
            r#"{"roaster": {"name": "Test Roasters", "country": "UK"}, "roast": {"name": "Gesha Village", "origin": "Ethiopia", "region": "Bench Maji", "producer": "Gesha Village Estate", "process": "Natural", "tasting_notes": ["Jasmine", "Peach"]}}"#,
        ))
        .mount(mock_server)
        .await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/").await.unwrap();
    wait_for_visible(&session.driver, "brew-photo-capture")
        .await
        .unwrap();

    trigger_homepage_scan(&session.driver).await;

    // Wait for extraction — roaster matched, roast not → "Save Roast" button
    wait_for_text(&session.driver, "body", "Save Roast")
        .await
        .unwrap();

    // Roaster should show as a card (matched) — visible in body text
    let body_text = session
        .driver
        .find(By::Css("body"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        body_text.contains("Test Roasters"),
        "Should show matched roaster name in card"
    );

    // Roast name is in an editable input (not matched)
    let roast_input = session
        .driver
        .find(By::Css("[data-bind\\:_roast-name]"))
        .await
        .unwrap();
    let roast_val = roast_input.value().await.unwrap().unwrap_or_default();
    assert_eq!(
        roast_val, "Gesha Village",
        "Roast name input should be filled"
    );

    // Submit — creates roast + bag under existing roaster, then reloads
    click_button_with_text(&session.driver, "Save Roast").await;

    // After reload, the new bag should appear in Open Bags
    wait_for_text(&session.driver, "#open-bags-section", "Gesha Village")
        .await
        .unwrap();

    session.quit().await;
}

#[tokio::test]
async fn homepage_scan_existing_roaster_and_existing_roast() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    // Pre-create both roaster and roast so the extraction matches both
    let roaster = create_default_roaster(&app).await;
    let _roast = create_default_roast(&app, roaster.id).await;

    // AI returns data matching both existing entities
    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_bag_scan_response(
            r#"{"roaster": {"name": "Test Roasters", "country": "UK"}, "roast": {"name": "Test Roast", "origin": "Ethiopia", "region": "Yirgacheffe", "producer": "Coop", "process": "Washed", "tasting_notes": ["Blueberry"]}}"#,
        ))
        .mount(mock_server)
        .await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/").await.unwrap();
    wait_for_visible(&session.driver, "brew-photo-capture")
        .await
        .unwrap();

    trigger_homepage_scan(&session.driver).await;

    // Wait for extraction — both matched → "Open Bag" button (open_bag defaults to true)
    wait_for_text(&session.driver, "body", "Open Bag")
        .await
        .unwrap();

    // Both roaster and roast should show as cards (matched)
    let body_text = session
        .driver
        .find(By::Css("body"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        body_text.contains("Test Roasters"),
        "Should show matched roaster"
    );
    assert!(
        body_text.contains("Test Roast"),
        "Should show matched roast"
    );

    // Submit — creates bag only for existing roast, then reloads
    click_button_with_text(&session.driver, "Open Bag").await;

    // After reload, the bag should appear in Open Bags
    wait_for_text(&session.driver, "#open-bags-section", "Test Roast")
        .await
        .unwrap();

    session.quit().await;
}
