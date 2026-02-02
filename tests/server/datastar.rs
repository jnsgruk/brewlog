//! Integration tests for Datastar partial rendering.
//!
//! These tests verify that endpoints return HTML fragments with correct
//! Datastar headers when the `datastar-request: true` header is present.

use crate::helpers::{
    assert_datastar_headers, assert_full_page, assert_html_fragment, create_default_bag,
    create_default_roast, create_default_roaster, spawn_app_with_auth,
};
use brewlog::domain::bags::UpdateBag;
use brewlog::domain::roasters::NewRoaster;
use brewlog::domain::roasts::NewRoast;
use reqwest::Client;

// ============================================================================
// Roasters
// ============================================================================

#[tokio::test]
async fn roasters_list_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    create_default_roaster(&app).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/roasters", app.address))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch roasters");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roaster-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("id=\"roaster-list\""),
        "Fragment should contain the selector element"
    );
}

#[tokio::test]
async fn roasters_list_without_datastar_header_returns_full_page() {
    let app = spawn_app_with_auth().await;
    create_default_roaster(&app).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/roasters", app.address))
        .send()
        .await
        .expect("failed to fetch roasters");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let body = response.text().await.expect("failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn roasters_create_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let new_roaster = NewRoaster {
        name: "Datastar Test Roasters".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };

    let response = client
        .post(app.api_url("/roasters"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&new_roaster)
        .send()
        .await
        .expect("failed to create roaster");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roaster-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("Datastar Test Roasters"),
        "Fragment should include created roaster"
    );
}

#[tokio::test]
async fn roasters_create_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let new_roaster = NewRoaster {
        name: "JSON Test Roasters".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };

    let response = client
        .post(app.api_url("/roasters"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_roaster)
        .send()
        .await
        .expect("failed to create roaster");

    assert_eq!(response.status(), 201);
    assert!(response.headers().get("datastar-selector").is_none());

    let roaster: brewlog::domain::roasters::Roaster =
        response.json().await.expect("failed to parse JSON");
    assert_eq!(roaster.name, "JSON Test Roasters");
}

#[tokio::test]
async fn roasters_delete_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let client = Client::new();

    let response = client
        .delete(app.api_url(&format!("/roasters/{}", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to delete roaster");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roaster-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn roasters_delete_without_datastar_header_returns_204() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let client = Client::new();

    let response = client
        .delete(app.api_url(&format!("/roasters/{}", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("failed to delete roaster");

    assert_eq!(response.status(), 204);
    assert!(response.headers().get("datastar-selector").is_none());
}

// ============================================================================
// Roasts
// ============================================================================

#[tokio::test]
async fn roasts_list_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/roasts", app.address))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch roasts");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roast-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("id=\"roast-list\""),
        "Fragment should contain the selector element"
    );
}

#[tokio::test]
async fn roasts_list_without_datastar_header_returns_full_page() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/roasts", app.address))
        .send()
        .await
        .expect("failed to fetch roasts");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let body = response.text().await.expect("failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn roasts_create_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let client = Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster.id,
        name: "Datastar Roast".to_string(),
        origin: "Ethiopia".to_string(),
        region: "Yirgacheffe".to_string(),
        producer: "Test Farm".to_string(),
        tasting_notes: vec!["Blueberry".to_string()],
        process: "Washed".to_string(),
    };

    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&new_roast)
        .send()
        .await
        .expect("failed to create roast");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roast-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("Datastar Roast"),
        "Fragment should include created roast"
    );
}

#[tokio::test]
async fn roasts_delete_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let response = client
        .delete(app.api_url(&format!("/roasts/{}", roast.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to delete roast");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roast-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn roast_options_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let response = client
        .get(app.api_url(&format!("/roasts?roaster_id={}", roaster.id)))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch roast options");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#roast-select-options");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn roast_options_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let response = client
        .get(app.api_url(&format!("/roasts?roaster_id={}", roaster.id)))
        .send()
        .await
        .expect("failed to fetch roast options");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let roasts: Vec<brewlog::domain::roasts::RoastWithRoaster> =
        response.json().await.expect("failed to parse JSON");
    assert_eq!(roasts.len(), 1);
}

// ============================================================================
// Bags
// ============================================================================

#[tokio::test]
async fn bags_list_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    create_default_bag(&app, roast.id).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/bags", app.address))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch bags");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#bag-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("id=\"bag-list\""),
        "Fragment should contain the selector element"
    );
}

#[tokio::test]
async fn bags_list_without_datastar_header_returns_full_page() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    create_default_bag(&app, roast.id).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/bags", app.address))
        .send()
        .await
        .expect("failed to fetch bags");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let body = response.text().await.expect("failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn bags_create_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let new_bag = serde_json::json!({
        "roast_id": roast.id,
        "roast_date": "2023-06-15",
        "amount": 350.0
    });

    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&new_bag)
        .send()
        .await
        .expect("failed to create bag");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#bag-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn bags_update_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let client = Client::new();

    let update = UpdateBag {
        remaining: Some(100.0),
        closed: None,
        finished_at: None,
    };

    let response = client
        .put(app.api_url(&format!("/bags/{}", bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update bag");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#bag-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn bags_update_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let client = Client::new();

    let update = UpdateBag {
        remaining: Some(100.0),
        closed: None,
        finished_at: None,
    };

    let response = client
        .put(app.api_url(&format!("/bags/{}", bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("failed to update bag");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let updated_bag: brewlog::domain::bags::Bag =
        response.json().await.expect("failed to parse JSON");
    assert_eq!(updated_bag.remaining, 100.0);
}

#[tokio::test]
async fn bags_delete_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let client = Client::new();

    let response = client
        .delete(app.api_url(&format!("/bags/{}", bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to delete bag");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#bag-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

// ============================================================================
// Timeline
// ============================================================================

#[tokio::test]
async fn timeline_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/timeline", app.address))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#timeline-loader");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn timeline_without_datastar_header_returns_full_page() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let body = response.text().await.expect("failed to read body");
    assert_full_page(&body);
}

// ============================================================================
// Gear
// ============================================================================

#[tokio::test]
async fn gear_list_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Create gear
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("failed to create gear");

    let response = client
        .get(format!("{}/gear", app.address))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch gear");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#gear-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("id=\"gear-list\""),
        "Fragment should contain the selector element"
    );
}

#[tokio::test]
async fn gear_list_without_datastar_header_returns_full_page() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/gear", app.address))
        .send()
        .await
        .expect("failed to fetch gear");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let body = response.text().await.expect("failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn gear_create_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Datastar Grinder",
        "model": "Test Model"
    });

    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&new_gear)
        .send()
        .await
        .expect("failed to create gear");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#gear-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("Datastar Grinder"),
        "Fragment should include created gear"
    );
}

#[tokio::test]
async fn gear_create_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let new_gear = serde_json::json!({
        "category": "brewer",
        "make": "JSON Test Brewer",
        "model": "V60"
    });

    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("failed to create gear");

    assert_eq!(response.status(), 201);
    assert!(response.headers().get("datastar-selector").is_none());

    let gear: brewlog::domain::gear::Gear = response.json().await.expect("failed to parse JSON");
    assert_eq!(gear.make, "JSON Test Brewer");
}

#[tokio::test]
async fn gear_update_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Create gear first
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Original Make",
        "model": "Original Model"
    });

    let create_response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("failed to create gear");

    let gear: brewlog::domain::gear::Gear = create_response.json().await.unwrap();

    // Update gear
    let update = brewlog::domain::gear::UpdateGear {
        make: Some("Updated Make".to_string()),
        model: None,
    };

    let response = client
        .put(app.api_url(&format!("/gear/{}", gear.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update gear");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#gear-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn gear_update_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Create gear first
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Original Make",
        "model": "Original Model"
    });

    let create_response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("failed to create gear");

    let gear: brewlog::domain::gear::Gear = create_response.json().await.unwrap();

    // Update gear
    let update = brewlog::domain::gear::UpdateGear {
        make: Some("JSON Updated".to_string()),
        model: None,
    };

    let response = client
        .put(app.api_url(&format!("/gear/{}", gear.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("failed to update gear");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let updated_gear: brewlog::domain::gear::Gear =
        response.json().await.expect("failed to parse JSON");
    assert_eq!(updated_gear.make, "JSON Updated");
}

#[tokio::test]
async fn gear_delete_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Create gear first
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "To Delete",
        "model": "Test Model"
    });

    let create_response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("failed to create gear");

    let gear: brewlog::domain::gear::Gear = create_response.json().await.unwrap();

    let response = client
        .delete(app.api_url(&format!("/gear/{}", gear.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to delete gear");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#gear-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}
