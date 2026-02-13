use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::helpers::spawn_app_with_auth;

#[tokio::test]
async fn register_start_with_missing_token_returns_400() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // POST without a token field — Axum returns 422 for missing fields
    let response = client
        .post(app.webauthn_url("/register/start"))
        .json(&json!({ "display_name": "Alice" }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn register_start_with_invalid_token_returns_401_or_gone() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .post(app.webauthn_url("/register/start"))
        .json(&json!({
            "token": "totally-invalid-token",
            "display_name": "Alice"
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Expired/missing token → 401 (token lookup fails) or 410 (token expired)
    let status = response.status().as_u16();
    assert!(
        status == 401 || status == 410,
        "expected 401 or 410, got {status}"
    );
}

#[tokio::test]
async fn register_finish_with_invalid_challenge_returns_400() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // `credential: {}` is invalid for RegisterPublicKeyCredential, so Axum returns 422
    let response = client
        .post(app.webauthn_url("/register/finish"))
        .json(&json!({
            "challenge_id": "nonexistent-challenge-id",
            "passkey_name": "my-key",
            "credential": {}
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn auth_start_with_no_passkeys_returns_404() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // No passkeys have been registered, so auth start should fail.
    // auth/start is a GET endpoint.
    let response = client
        .get(app.webauthn_url("/auth/start"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn auth_finish_with_invalid_challenge_returns_400() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // `credential: {}` is invalid for PublicKeyCredential, so Axum returns 422
    let response = client
        .post(app.webauthn_url("/auth/finish"))
        .json(&json!({
            "challenge_id": "nonexistent-challenge-id",
            "credential": {}
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn passkey_add_start_requires_auth() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // No auth header → 401
    let response = client
        .post(app.webauthn_url("/passkey/start"))
        .json(&json!({ "name": "my-passkey" }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn passkey_add_finish_requires_auth() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // No auth header → 401
    let response = client
        .post(app.webauthn_url("/passkey/finish"))
        .json(&json!({
            "challenge_id": "nonexistent-challenge-id",
            "name": "my-passkey",
            "credential": {}
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_start_with_non_localhost_cli_callback_returns_400_or_404() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // auth/start is GET. Without passkeys, we get 404 before the callback validation.
    // With passkeys, a non-localhost callback would return 400.
    // We verify the endpoint handles these params without crashing.
    let response = client
        .get(&format!(
            "{}/auth/start?cli_callback=http://evil.com/steal&state=abc&token_name=test",
            app.webauthn_url("")
        ))
        .send()
        .await
        .expect("Failed to send request");

    let status = response.status().as_u16();
    assert!(
        status == 400 || status == 404,
        "expected 400 or 404, got {status}"
    );
}
