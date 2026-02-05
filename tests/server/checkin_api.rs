use brewlog::domain::cups::Cup;

use crate::helpers::{
    create_default_cafe, create_default_roast, create_default_roaster, spawn_app_with_auth,
};

#[tokio::test]
async fn checkin_with_existing_cafe_creates_cup() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let payload = serde_json::json!({
        "cafe_id": cafe.id.to_string(),
        "roast_id": roast.id.to_string(),
    });

    let response = client
        .post(app.api_url("/check-in"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let cup: Cup = response.json().await.expect("Failed to parse response");
    assert_eq!(cup.roast_id, roast.id);
    assert_eq!(cup.cafe_id, cafe.id);
}

#[tokio::test]
async fn checkin_with_new_cafe_creates_cafe_and_cup() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    let payload = serde_json::json!({
        "roast_id": roast.id.to_string(),
        "cafe_name": "New Test Cafe",
        "cafe_city": "London",
        "cafe_country": "UK",
        "cafe_lat": 51.5074,
        "cafe_lng": -0.1278,
    });

    let response = client
        .post(app.api_url("/check-in"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let cup: Cup = response.json().await.expect("Failed to parse response");
    assert_eq!(cup.roast_id, roast.id);

    // Verify the cafe was also created
    let cafes_response = client
        .get(app.api_url("/cafes"))
        .send()
        .await
        .expect("Failed to list cafes");

    let body = cafes_response.text().await.expect("Failed to read body");
    assert!(
        body.contains("New Test Cafe"),
        "Created cafe should appear in cafe list"
    );
}

#[tokio::test]
async fn checkin_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "cafe_id": "1",
        "roast_id": "1",
    });

    let response = client
        .post(app.api_url("/check-in"))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn checkin_rejects_invalid_roast_id() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "cafe_id": "1",
        "roast_id": "not-a-number",
    });

    let response = client
        .post(app.api_url("/check-in"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn checkin_requires_cafe_name_when_no_cafe_id() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    let payload = serde_json::json!({
        "roast_id": roast.id.to_string(),
    });

    let response = client
        .post(app.api_url("/check-in"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}
