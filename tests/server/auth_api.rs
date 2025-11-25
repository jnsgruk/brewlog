use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::helpers::spawn_app_with_auth;

#[tokio::test]
async fn test_create_token_with_valid_credentials() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Create a token
    let response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({
            "username": "admin",
            "password": "test_password",
            "name": "test-token"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body.get("id").is_some());
    assert_eq!(body.get("name").unwrap(), "test-token");
    assert!(body.get("token").is_some());

    // Token should be a non-empty string
    let token = body.get("token").unwrap().as_str().unwrap();
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_create_token_with_invalid_credentials() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Try to create a token with wrong password
    let response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({
            "username": "admin",
            "password": "wrong_password",
            "name": "test-token"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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

    // First, create a token
    let create_response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({
            "username": "admin",
            "password": "test_password",
            "name": "test-token"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let token = create_body.get("token").unwrap().as_str().unwrap();

    // Now list tokens with authentication
    let response = client
        .get(&app.api_url("/tokens"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let tokens: Vec<serde_json::Value> = response.json().await.expect("Failed to parse response");
    // We expect 2 tokens: the one created by spawn_app_with_auth() and the one we just created
    assert_eq!(tokens.len(), 2);
    // Find the token we created
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

    // First, create a token
    let create_response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({
            "username": "admin",
            "password": "test_password",
            "name": "test-token"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let token = create_body.get("token").unwrap().as_str().unwrap();
    let token_id = create_body.get("id").unwrap().as_i64().unwrap();

    // Revoke the token
    let response = client
        .post(&app.api_url(&format!("/tokens/{}/revoke", token_id)))
        .header("Authorization", format!("Bearer {}", token))
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

    // Create a token
    let create_response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({
            "username": "admin",
            "password": "test_password",
            "name": "test-token"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let token = create_body.get("token").unwrap().as_str().unwrap();
    let token_id = create_body.get("id").unwrap().as_i64().unwrap();

    // Revoke the token
    client
        .post(&app.api_url(&format!("/tokens/{}/revoke", token_id)))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");

    // Try to use the revoked token
    let response = client
        .get(&app.api_url("/tokens"))
        .header("Authorization", format!("Bearer {}", token))
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

    // Create a token
    let create_response = client
        .post(&app.api_url("/tokens"))
        .json(&json!({
            "username": "admin",
            "password": "test_password",
            "name": "test-token"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let create_body: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let token = create_body.get("token").unwrap().as_str().unwrap();

    // Create a roaster with authentication
    let response = client
        .post(&app.api_url("/roasters"))
        .header("Authorization", format!("Bearer {}", token))
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
async fn test_session_authentication_via_login() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // Login to create a session
    let login_response = client
        .post(&format!("{}/login", app.address))
        .form(&[("username", "admin"), ("password", "test_password")])
        .send()
        .await
        .expect("Failed to send login request");

    assert!(
        login_response.status().is_redirection() || login_response.status().is_success(),
        "Login should succeed"
    );

    // Use the session cookie to create a roaster (no Bearer token needed)
    let response = client
        .post(&app.api_url("/roasters"))
        .json(&json!({
            "name": "Session Test Roaster",
            "country": "US"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Session cookie should authenticate API request"
    );
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
async fn test_logout_invalidates_session() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // Login to create a session
    let login_response = client
        .post(&format!("{}/login", app.address))
        .form(&[("username", "admin"), ("password", "test_password")])
        .send()
        .await
        .expect("Failed to send login request");

    assert!(login_response.status().is_redirection() || login_response.status().is_success());

    // Verify the session works
    let auth_response = client
        .post(&app.api_url("/roasters"))
        .json(&json!({
            "name": "Pre-Logout Roaster",
            "country": "FR"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(auth_response.status(), StatusCode::CREATED);

    // Logout
    let logout_response = client
        .post(&format!("{}/logout", app.address))
        .send()
        .await
        .expect("Failed to send logout request");

    assert!(logout_response.status().is_redirection() || logout_response.status().is_success());

    // Try to use the session after logout
    let post_logout_response = client
        .post(&app.api_url("/roasters"))
        .json(&json!({
            "name": "Post-Logout Roaster",
            "country": "DE"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        post_logout_response.status(),
        StatusCode::UNAUTHORIZED,
        "Session should be invalidated after logout"
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
