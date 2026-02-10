//! Integration tests for Datastar partial rendering.
//!
//! These tests verify that endpoints return HTML fragments with correct
//! Datastar headers when the `datastar-request: true` header is present.

use crate::helpers::{
    TestApp, assert_datastar_headers, assert_datastar_headers_with_mode, assert_full_page,
    assert_html_fragment, create_default_bag, create_default_brew, create_default_cafe,
    create_default_cup, create_default_gear, create_default_roast, create_default_roaster,
    spawn_app_with_auth,
};
use crate::test_macros::define_datastar_entity_tests;
use brewlog::domain::bags::UpdateBag;
use brewlog::domain::cafes::NewCafe;
use brewlog::domain::roasters::NewRoaster;
use brewlog::domain::roasts::NewRoast;
use reqwest::Client;

// ============================================================================
// Setup functions (return entity ID as String for delete tests)
// ============================================================================

async fn create_roaster_entity(app: &TestApp) -> String {
    create_default_roaster(app).await.id.to_string()
}

async fn create_roast_entity(app: &TestApp) -> String {
    let roaster = create_default_roaster(app).await;
    create_default_roast(app, roaster.id).await.id.to_string()
}

async fn create_bag_entity(app: &TestApp) -> String {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    create_default_bag(app, roast.id).await.id.to_string()
}

async fn create_gear_entity(app: &TestApp) -> String {
    create_default_gear(app, "grinder", "Baratza", "Encore")
        .await
        .id
        .to_string()
}

async fn create_cafe_entity(app: &TestApp) -> String {
    create_default_cafe(app).await.id.to_string()
}

// ============================================================================
// Macro-generated tests: list with/without + delete with datastar header
// ============================================================================

define_datastar_entity_tests!(
    entity: roasters,
    type_param: "roasters",
    api_path: "/roasters",
    list_element: r#"id="roaster-list""#,
    selector: "#roaster-list",
    setup: create_roaster_entity
);

define_datastar_entity_tests!(
    entity: roasts,
    type_param: "roasts",
    api_path: "/roasts",
    list_element: r#"id="roast-list""#,
    selector: "#roast-list",
    setup: create_roast_entity
);

define_datastar_entity_tests!(
    entity: bags,
    type_param: "bags",
    api_path: "/bags",
    list_element: r#"id="bag-list""#,
    selector: "#bag-list",
    setup: create_bag_entity
);

define_datastar_entity_tests!(
    entity: gear,
    type_param: "gear",
    api_path: "/gear",
    list_element: r#"id="gear-list""#,
    selector: "#gear-list",
    setup: create_gear_entity
);

define_datastar_entity_tests!(
    entity: cafes,
    type_param: "cafes",
    api_path: "/cafes",
    list_element: r#"id="cafe-list""#,
    selector: "#cafe-list",
    setup: create_cafe_entity
);

// ============================================================================
// Roasters (hand-written: create with/without, delete without)
// ============================================================================

#[tokio::test]
async fn roasters_create_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let new_roaster = NewRoaster {
        name: "Datastar Test Roasters".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        created_at: None,
    };

    let response = client
        .post(app.api_url("/roasters"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .header("referer", format!("{}/data?type=roasters", app.address))
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
        created_at: None,
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
// Roasts (hand-written: create with, roast_options with/without)
// ============================================================================

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
        created_at: None,
    };

    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .header("referer", format!("{}/data?type=roasts", app.address))
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
// Bags (hand-written: create with, update with/without)
// ============================================================================

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
        .header("referer", format!("{}/data?type=bags", app.address))
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
        ..Default::default()
    };

    let response = client
        .put(app.api_url(&format!("/bags/{}", bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .header("referer", format!("{}/data?type=bags", app.address))
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
        ..Default::default()
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
// Gear (hand-written: create with/without, update with/without)
// ============================================================================

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
        .header("referer", format!("{}/data?type=gear", app.address))
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
async fn gear_update_with_datastar_header_returns_redirect_script() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

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

    let update = serde_json::json!({
        "make": "Updated Make",
    });

    let response = client
        .put(app.api_url(&format!("/gear/{}", gear.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update gear");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "body", "append");

    let body = response.text().await.expect("failed to read body");
    assert!(
        body.contains("window.location"),
        "Expected redirect script in body"
    );
}

#[tokio::test]
async fn gear_update_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

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

    let update = brewlog::domain::gear::UpdateGear {
        make: Some("JSON Updated".to_string()),
        model: None,
        created_at: None,
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

// ============================================================================
// Cafes (hand-written: create with)
// ============================================================================

#[tokio::test]
async fn cafes_create_with_datastar_header_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let new_cafe = NewCafe {
        name: "Datastar Test Cafe".to_string(),
        city: "London".to_string(),
        country: "UK".to_string(),
        latitude: 51.5074,
        longitude: -0.1278,
        website: None,
        created_at: None,
    };

    let response = client
        .post(app.api_url("/cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .header("referer", format!("{}/data?type=cafes", app.address))
        .json(&new_cafe)
        .send()
        .await
        .expect("failed to create cafe");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#cafe-list");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
    assert!(
        body.contains("Datastar Test Cafe"),
        "Fragment should include created cafe"
    );
}

// ============================================================================
// Roasters (update with/without datastar)
// ============================================================================

#[tokio::test]
async fn roasters_update_with_datastar_header_returns_redirect_script() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let client = Client::new();

    let update = serde_json::json!({
        "name": "Updated Roaster",
    });

    let response = client
        .put(app.api_url(&format!("/roasters/{}", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update roaster");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "body", "append");

    let body = response.text().await.expect("failed to read body");
    assert!(
        body.contains("window.location"),
        "Expected redirect script in body"
    );
}

#[tokio::test]
async fn roasters_update_without_datastar_header_returns_json() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let client = Client::new();

    let update = serde_json::json!({
        "name": "JSON Updated Roaster",
    });

    let response = client
        .put(app.api_url(&format!("/roasters/{}", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("failed to update roaster");

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("datastar-selector").is_none());

    let updated: brewlog::domain::roasters::Roaster =
        response.json().await.expect("failed to parse JSON");
    assert_eq!(updated.name, "JSON Updated Roaster");
}

// ============================================================================
// Roasts (update with/without datastar)
// ============================================================================

#[tokio::test]
async fn roasts_update_with_datastar_header_returns_redirect_script() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = Client::new();

    let update = serde_json::json!({
        "name": "Updated Roast",
    });

    let response = client
        .put(app.api_url(&format!("/roasts/{}", roast.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update roast");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "body", "append");

    let body = response.text().await.expect("failed to read body");
    assert!(
        body.contains("window.location"),
        "Expected redirect script in body"
    );
}

// ============================================================================
// Cafes (update with/without datastar)
// ============================================================================

#[tokio::test]
async fn cafes_update_with_datastar_header_returns_redirect_script() {
    let app = spawn_app_with_auth().await;
    let cafe = create_default_cafe(&app).await;
    let client = Client::new();

    let update = serde_json::json!({
        "name": "Updated Cafe",
    });

    let response = client
        .put(app.api_url(&format!("/cafes/{}", cafe.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update cafe");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "body", "append");

    let body = response.text().await.expect("failed to read body");
    assert!(
        body.contains("window.location"),
        "Expected redirect script in body"
    );
}

// ============================================================================
// Brews (update with/without datastar)
// ============================================================================

#[tokio::test]
async fn brews_update_with_datastar_header_returns_redirect_script() {
    let app = spawn_app_with_auth().await;
    let brew = create_default_brew(&app).await;
    let client = Client::new();

    let update = serde_json::json!({
        "water_temp": 94.0,
    });

    let response = client
        .put(app.api_url(&format!("/brews/{}", brew.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update brew");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "body", "append");

    let body = response.text().await.expect("failed to read body");
    assert!(
        body.contains("window.location"),
        "Expected redirect script in body"
    );
}

// ============================================================================
// Cups (update with/without datastar)
// ============================================================================

#[tokio::test]
async fn cups_update_with_datastar_header_returns_redirect_script() {
    let app = spawn_app_with_auth().await;
    let cup = create_default_cup(&app).await;
    let client = Client::new();

    // Create a new cafe to update to
    let cafe2 = crate::helpers::create_cafe_with_payload(
        &app,
        brewlog::domain::cafes::NewCafe {
            name: "Updated Test Cafe".to_string(),
            city: "Tokyo".to_string(),
            country: "JP".to_string(),
            latitude: 35.6762,
            longitude: 139.6503,
            website: None,
            created_at: None,
        },
    )
    .await;

    let update = serde_json::json!({
        "cafe_id": cafe2.id,
    });

    let response = client
        .put(app.api_url(&format!("/cups/{}", cup.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&update)
        .send()
        .await
        .expect("failed to update cup");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "body", "append");

    let body = response.text().await.expect("failed to read body");
    assert!(
        body.contains("window.location"),
        "Expected redirect script in body"
    );
}
