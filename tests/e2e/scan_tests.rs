use thirtyfour::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::server_helpers::spawn_app_with_openrouter_mock;
use crate::helpers::wait::{wait_for_url_contains, wait_for_visible};

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
