use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::helpers::{spawn_app, spawn_app_with_auth};

#[tokio::test]
async fn test_create_token_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Token creation now requires session or bearer token auth
    let response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({ "name": "test-token" }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_token_with_bearer_auth() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();
    let auth_token = app.auth_token.as_ref().unwrap();

    // Create a token using bearer auth
    let response = client
        .post(&app.api_url("/tokens"))
        .bearer_auth(auth_token)
        .json(&json!({ "name": "new-token" }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body.get("id").is_some());
    assert_eq!(body.get("name").unwrap(), "new-token");
    assert!(body.get("token").is_some());

    // Token should be a non-empty string
    let token = body.get("token").unwrap().as_str().unwrap();
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_list_tokens_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Try to list tokens without authentication
    let response = client
        .get(&app.api_url("/tokens"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_tokens_with_authentication() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();
    let auth_token = app.auth_token.as_ref().unwrap();

    // List tokens with authentication
    let response = client
        .get(&app.api_url("/tokens"))
        .bearer_auth(auth_token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let tokens: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    // We expect at least 1 token (the one created by spawn_app_with_auth)
    assert!(
        !tokens.is_empty(),
        "Should have at least one token from test setup"
    );
    let test_token = tokens
        .iter()
        .find(|t| t.get("name").unwrap() == "test-token")
        .expect("Could not find test-token");
    assert_eq!(test_token.get("name").unwrap(), "test-token");
}

#[tokio::test]
async fn test_revoke_token() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();
    let auth_token = app.auth_token.as_ref().unwrap();

    // Create a new token to revoke
    let create_response = client
        .post(&app.api_url("/tokens"))
        .bearer_auth(auth_token)
        .json(&json!({ "name": "token-to-revoke" }))
        .send()
        .await
        .expect("Failed to send request");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let token_id = create_body.get("id").unwrap().as_i64().unwrap();

    // Revoke the token
    let response = client
        .post(&app.api_url(&format!("/tokens/{}/revoke", token_id)))
        .bearer_auth(auth_token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body.get("revoked_at").is_some());
}

#[tokio::test]
async fn test_revoked_token_cannot_be_used() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();
    let auth_token = app.auth_token.as_ref().unwrap();

    // Create a new token
    let create_response = client
        .post(&app.api_url("/tokens"))
        .bearer_auth(auth_token)
        .json(&json!({ "name": "will-be-revoked" }))
        .send()
        .await
        .expect("Failed to send request");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let new_token = create_body.get("token").unwrap().as_str().unwrap();
    let token_id = create_body.get("id").unwrap().as_i64().unwrap();

    // Revoke it using the original auth token
    client
        .post(&app.api_url(&format!("/tokens/{}/revoke", token_id)))
        .bearer_auth(auth_token)
        .send()
        .await
        .expect("Failed to send request");

    // Try to use the revoked token
    let response = client
        .get(&app.api_url("/tokens"))
        .header("Authorization", format!("Bearer {}", new_token))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoints_require_authentication() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Try to create a roaster without authentication
    let response = client
        .post(&app.api_url("/roasters"))
        .json(&json!({
            "name": "Test Roaster",
            "country": "UK"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoints_work_with_authentication() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();
    let auth_token = app.auth_token.as_ref().unwrap();

    // Create a roaster with authentication
    let response = client
        .post(&app.api_url("/roasters"))
        .bearer_auth(auth_token)
        .json(&json!({
            "name": "Test Roaster",
            "country": "UK"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_invalid_session_cookie_fails() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // Try to create a roaster with no session (unauthenticated)
    let response = client
        .post(&app.api_url("/roasters"))
        .json(&json!({
            "name": "Invalid Session Roaster",
            "country": "UK"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Request without valid session should fail"
    );
}

#[tokio::test]
async fn test_fake_session_cookie_fails() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // Try to use a fake/forged session cookie
    let response = client
        .post(&app.api_url("/roasters"))
        .header("Cookie", "brewlog_session=fake_session_token_12345")
        .json(&json!({
            "name": "Fake Session Roaster",
            "country": "IT"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Fake session cookie should not authenticate"
    );
}

#[tokio::test]
async fn test_read_endpoints_dont_require_authentication() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // List roasters without authentication should work
    let response = client
        .get(&app.api_url("/roasters"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
}

// --- Admin passkey endpoint tests ---

#[tokio::test]
async fn list_passkeys_requires_auth() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(&app.api_url("/passkeys"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_passkeys_returns_empty_for_user_without_passkeys() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .get(&app.api_url("/passkeys"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let passkeys: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    assert!(passkeys.is_empty(), "expected empty passkeys list");
}

#[tokio::test]
async fn delete_passkey_nonexistent_returns_404() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .delete(&app.api_url("/passkeys/99999"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_passkey_requires_auth() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .delete(&app.api_url("/passkeys/1"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
