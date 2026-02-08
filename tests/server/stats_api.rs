use reqwest::Client;

use crate::helpers::{
    assert_datastar_headers_with_mode, assert_full_page, assert_html_fragment, create_default_cafe,
    create_default_roast, create_default_roaster, spawn_app, spawn_app_with_auth,
};

#[tokio::test]
async fn stats_page_returns_200_with_empty_database() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(app.page_url("/stats"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
    assert!(body.contains("Stats"), "Page should contain title");
}

#[tokio::test]
async fn stats_page_returns_200_with_data() {
    let app = spawn_app_with_auth().await;

    let roaster = create_default_roaster(&app).await;
    let _roast = create_default_roast(&app, roaster.id).await;

    let client = Client::new();
    let response = client
        .get(app.page_url("/stats"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
    assert!(
        body.contains("Ethiopia"),
        "Stats page should contain roast origin"
    );
}

#[tokio::test]
async fn stats_page_datastar_tab_switch_returns_fragment() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(app.page_url("/stats?type=roasts"))
        .header("datastar-request", "true")
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    assert_datastar_headers_with_mode(&response, "#stats-content", "inner");

    let body = response.text().await.expect("Failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn recompute_stats_requires_authentication() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .post(app.api_url("/stats/recompute"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn recompute_stats_returns_json() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .post(app.api_url("/stats/recompute"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(
        body.get("computed_at").is_some(),
        "Response should contain computed_at"
    );
    assert!(
        body.get("roast_summary").is_some(),
        "Response should contain roast_summary"
    );
    assert!(
        body.get("consumption").is_some(),
        "Response should contain consumption"
    );
    assert!(
        body.get("brewing_summary").is_some(),
        "Response should contain brewing_summary"
    );
}

#[tokio::test]
async fn recompute_stats_reflects_created_data() {
    let app = spawn_app_with_auth().await;

    let roaster = create_default_roaster(&app).await;
    let _roast = create_default_roast(&app, roaster.id).await;
    let _cafe = create_default_cafe(&app).await;

    let client = Client::new();
    let response = client
        .post(app.api_url("/stats/recompute"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    // Roaster is in UK
    let geo_roasters = &body["geo_roasters"]["entries"];
    assert!(
        geo_roasters
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["country_name"] == "UK"),
        "geo_roasters should contain UK: {geo_roasters}"
    );

    // Roast origin is Ethiopia
    let geo_roasts = &body["geo_roasts"]["entries"];
    assert!(
        geo_roasts
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["country_name"] == "Ethiopia"),
        "geo_roasts should contain Ethiopia: {geo_roasts}"
    );

    // Cafe is in US
    let geo_cafes = &body["geo_cafes"]["entries"];
    assert!(
        geo_cafes
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["country_name"] == "US"),
        "geo_cafes should contain US: {geo_cafes}"
    );
}

#[tokio::test]
async fn stats_page_loads_after_recompute() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Populate the cache
    let recompute_response = client
        .post(app.api_url("/stats/recompute"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to recompute");
    assert_eq!(recompute_response.status(), 200);

    // Load the page — should read from cache
    let response = client
        .get(app.page_url("/stats"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
}
