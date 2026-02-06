use serde::Deserialize;

use crate::helpers::{create_default_roast, create_default_roaster, spawn_app_with_auth};

#[derive(Debug, Deserialize)]
struct ScanResult {
    redirect: String,
    roast_id: i64,
}

fn scan_payload(roaster_name: &str, roast_name: &str, tasting_notes: &str) -> serde_json::Value {
    serde_json::json!({
        "roaster_name": roaster_name,
        "roaster_country": "UK",
        "roast_name": roast_name,
        "origin": "Ethiopia",
        "region": "Yirgacheffe",
        "producer": "Test Farm",
        "process": "Washed",
        "tasting_notes": tasting_notes,
    })
}

#[tokio::test]
async fn scan_creates_roaster_and_roast() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = scan_payload("Scan Roasters", "Scan Roast", "Blueberry, Chocolate");

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let result: ScanResult = response.json().await.expect("Failed to parse response");
    assert!(result.roast_id > 0);
    assert!(
        result.redirect.contains("scan-roasters"),
        "Redirect should contain roaster slug, got: {}",
        result.redirect
    );
    assert!(
        result.redirect.contains("scan-roast"),
        "Redirect should contain roast slug, got: {}",
        result.redirect
    );
}

#[tokio::test]
async fn scan_reuses_existing_roaster_by_slug() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Create roaster first via the normal API
    let _existing = create_default_roaster(&app).await;

    // Submit scan with the same roaster name
    let payload = scan_payload("Test Roasters", "Scan Roast", "Caramel, Nutty");

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    // Verify only one roaster exists (the pre-existing one was reused)
    let roasters_response = client
        .get(app.api_url("/roasters"))
        .send()
        .await
        .expect("Failed to list roasters");

    let roasters: Vec<serde_json::Value> = roasters_response
        .json()
        .await
        .expect("Failed to parse roasters");
    assert_eq!(
        roasters.len(),
        1,
        "Should reuse existing roaster, not create a new one"
    );
}

#[tokio::test]
async fn scan_creates_bag_when_open_bag_is_true() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let mut payload = scan_payload("Bag Roasters", "Bag Roast", "Cherry");
    payload["open_bag"] = serde_json::Value::String("true".to_string());
    payload["bag_amount"] = serde_json::json!(250.0);

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    // Verify bag was created
    let bags_response = client
        .get(app.api_url("/bags"))
        .send()
        .await
        .expect("Failed to list bags");

    let bags: Vec<serde_json::Value> = bags_response.json().await.expect("Failed to parse bags");
    assert_eq!(bags.len(), 1, "A bag should have been created");
}

#[tokio::test]
async fn scan_skips_bag_when_open_bag_absent() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = scan_payload("No Bag Roasters", "No Bag Roast", "Floral");

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    // Verify no bag was created
    let bags_response = client
        .get(app.api_url("/bags"))
        .send()
        .await
        .expect("Failed to list bags");

    let bags: Vec<serde_json::Value> = bags_response.json().await.expect("Failed to parse bags");
    assert_eq!(bags.len(), 0, "No bag should have been created");
}

#[tokio::test]
async fn scan_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = scan_payload("Auth Roasters", "Auth Roast", "Berry");

    let response = client
        .post(app.api_url("/scan"))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn scan_validates_required_fields() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Empty roast_name should fail
    let payload = serde_json::json!({
        "roaster_name": "Valid Roasters",
        "roaster_country": "UK",
        "roast_name": "",
        "origin": "Ethiopia",
        "region": "Yirgacheffe",
        "producer": "Test Farm",
        "process": "Washed",
        "tasting_notes": "Blueberry",
    });

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn scan_requires_tasting_notes_for_manual_submission() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // No tasting_notes and no image/prompt → should fail validation
    let payload = serde_json::json!({
        "roaster_name": "Notes Roasters",
        "roaster_country": "UK",
        "roast_name": "Notes Roast",
        "origin": "Ethiopia",
        "region": "Yirgacheffe",
        "producer": "Test Farm",
        "process": "Washed",
    });

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn scan_with_matched_roast_id_creates_bag_only() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Pre-create roaster and roast
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    // Submit scan with matched_roast_id — should skip roaster/roast creation
    let payload = serde_json::json!({
        "matched_roast_id": roast.id.into_inner().to_string(),
        "open_bag": "true",
        "bag_amount": 250.0,
    });

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let result: ScanResult = response.json().await.expect("Failed to parse response");
    assert_eq!(result.roast_id, roast.id.into_inner());
    assert!(
        result.redirect.contains(&roast.slug),
        "Redirect should reference existing roast slug, got: {}",
        result.redirect
    );

    // Verify no new roast was created (still just the one)
    let roasts_response = client
        .get(app.api_url("/roasts"))
        .send()
        .await
        .expect("Failed to list roasts");
    let roasts: Vec<serde_json::Value> = roasts_response
        .json()
        .await
        .expect("Failed to parse roasts");
    assert_eq!(roasts.len(), 1, "Should not create a new roast");

    // Verify bag was created
    let bags_response = client
        .get(app.api_url("/bags"))
        .send()
        .await
        .expect("Failed to list bags");
    let bags: Vec<serde_json::Value> = bags_response.json().await.expect("Failed to parse bags");
    assert_eq!(bags.len(), 1, "A bag should have been created");
}

#[tokio::test]
async fn scan_with_matched_roast_id_no_bag_returns_existing() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Pre-create roaster and roast
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    // Submit scan with matched_roast_id but no open_bag
    let payload = serde_json::json!({
        "matched_roast_id": roast.id.into_inner().to_string(),
    });

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let result: ScanResult = response.json().await.expect("Failed to parse response");
    assert_eq!(result.roast_id, roast.id.into_inner());

    // Verify no bag was created
    let bags_response = client
        .get(app.api_url("/bags"))
        .send()
        .await
        .expect("Failed to list bags");
    let bags: Vec<serde_json::Value> = bags_response.json().await.expect("Failed to parse bags");
    assert_eq!(bags.len(), 0, "No bag should have been created");
}
