use brewlog::infrastructure::ai::{ExtractedBagScan, ExtractedRoast, ExtractedRoaster};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app_with_openrouter_mock;

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

// --- extract-roaster ---

#[tokio::test]
async fn extract_roaster_returns_json() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"name": "Square Mile", "country": "United Kingdom", "city": "London"}"#,
        ))
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "prompt": "Square Mile Coffee" });

    let response = client
        .post(app.api_url("/extract-roaster"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let result: ExtractedRoaster = response.json().await.expect("Failed to parse response");
    assert_eq!(result.name.as_deref(), Some("Square Mile"));
    assert_eq!(result.country.as_deref(), Some("United Kingdom"));
    assert_eq!(result.city.as_deref(), Some("London"));
}

#[tokio::test]
async fn extract_roaster_returns_datastar_signals() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"name": "Has Bean", "country": "UK"}"#,
        ))
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "prompt": "Has Bean" });

    let response = client
        .post(app.api_url("/extract-roaster"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["_roasterName"], "Has Bean");
    assert_eq!(body["_roasterCountry"], "UK");
}

#[tokio::test]
async fn extract_roaster_requires_auth() {
    let app = spawn_app_with_openrouter_mock().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({ "prompt": "test" });

    let response = client
        .post(app.api_url("/extract-roaster"))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn extract_roaster_rejects_empty_input() {
    let app = spawn_app_with_openrouter_mock().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({});

    let response = client
        .post(app.api_url("/extract-roaster"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

// --- extract-roast ---

#[tokio::test]
async fn extract_roast_returns_json() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"name": "Red Brick", "origin": "Brazil", "tasting_notes": ["Chocolate", "Hazelnut"]}"#,
        ))
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "prompt": "Red Brick coffee" });

    let response = client
        .post(app.api_url("/extract-roast"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let result: ExtractedRoast = response.json().await.expect("Failed to parse response");
    assert_eq!(result.name.as_deref(), Some("Red Brick"));
    assert_eq!(result.origin.as_deref(), Some("Brazil"));
    assert_eq!(
        result.tasting_notes.as_deref(),
        Some(&["Chocolate".to_string(), "Hazelnut".to_string()][..])
    );
}

#[tokio::test]
async fn extract_roast_returns_datastar_signals() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"name": "Ethiopia Natural", "origin": "Ethiopia", "process": "Natural"}"#,
        ))
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "prompt": "Ethiopia Natural" });

    let response = client
        .post(app.api_url("/extract-roast"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["_roastName"], "Ethiopia Natural");
    assert_eq!(body["_origin"], "Ethiopia");
    assert_eq!(body["_process"], "Natural");
}

// --- extract-bag-scan ---

#[tokio::test]
async fn extract_bag_scan_returns_json() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"roaster": {"name": "Origin", "country": "UK"}, "roast": {"name": "Blend One", "origin": "Colombia"}}"#,
        ))
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "prompt": "Origin Blend One" });

    let response = client
        .post(app.api_url("/extract-bag-scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let result: ExtractedBagScan = response.json().await.expect("Failed to parse response");
    assert_eq!(result.roaster.name.as_deref(), Some("Origin"));
    assert_eq!(result.roast.name.as_deref(), Some("Blend One"));
}

#[tokio::test]
async fn extract_bag_scan_returns_datastar_signals() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(mock_openrouter_response(
            r#"{"roaster": {"name": "Allpress"}, "roast": {"name": "Redchurch", "tasting_notes": ["Chocolate", "Caramel"]}}"#,
        ))
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "prompt": "Allpress Redchurch" });

    let response = client
        .post(app.api_url("/extract-bag-scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["_roasterName"], "Allpress");
    assert_eq!(body["_roastName"], "Redchurch");
    assert_eq!(body["_tastingNotes"], "Chocolate, Caramel");
    assert_eq!(body["_scanExtracted"], true);
}

#[tokio::test]
async fn extract_bag_scan_requires_auth() {
    let app = spawn_app_with_openrouter_mock().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({ "prompt": "test" });

    let response = client
        .post(app.api_url("/extract-bag-scan"))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}
