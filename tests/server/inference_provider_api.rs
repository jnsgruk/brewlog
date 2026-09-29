use wiremock::matchers::{method, path};
use wiremock::{Mock, Request, ResponseTemplate};

use crate::helpers::{spawn_app_with_openai_compatible_mock, spawn_app_with_openrouter_mock};

fn mock_extraction_response() -> ResponseTemplate {
    let body = serde_json::json!({
        "id": "gen-test",
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": r#"{"name": "Square Mile"}"#
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15,
            "cost": 0.0001
        }
    });
    ResponseTemplate::new(200).set_body_json(body)
}

fn request_json(request: &Request) -> serde_json::Value {
    serde_json::from_slice(&request.body).expect("request body is valid JSON")
}

/// The `openrouter` preset sends the configured model, the
/// `openrouter:web_search` server tool, and an `Authorization` header.
#[tokio::test]
async fn openrouter_provider_sends_web_search_tool_model_and_auth() {
    let app = spawn_app_with_openrouter_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .and(|request: &Request| {
            let body = request_json(request);
            body["model"] == "openrouter/free"
                && body["tools"][0]["type"] == "openrouter:web_search"
                && request.headers.contains_key("authorization")
        })
        .respond_with(mock_extraction_response())
        .expect(1)
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(app.api_url("/extract-roaster"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({ "prompt": "Square Mile Coffee" }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
}

/// The `openai-compatible` provider sends no tools, omits `model` when
/// unconfigured so the proxy picks its own default, and omits
/// `Authorization` when no API key is configured.
#[tokio::test]
async fn openai_compatible_provider_sends_no_tools_and_omits_missing_model_and_auth() {
    let app = spawn_app_with_openai_compatible_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(|request: &Request| {
            let body = request_json(request);
            body.get("tools").is_none()
                && body.get("model").is_none()
                && !request.headers.contains_key("authorization")
        })
        .respond_with(mock_extraction_response())
        .expect(1)
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(app.api_url("/extract-roaster"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({ "prompt": "Square Mile Coffee" }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
}
