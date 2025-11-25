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
    let token_id = create_body.get("id").unwrap().as_str().unwrap();

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
    let token_id = create_body.get("id").unwrap().as_str().unwrap();

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
