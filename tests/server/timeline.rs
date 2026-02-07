use crate::helpers::{
    create_cafe_with_payload, create_default_bag, create_default_cafe, create_default_gear,
    create_default_roast, create_default_roaster, create_roaster_with_payload, spawn_app_with_auth,
};
use brewlog::domain::brews::NewBrew;
use brewlog::domain::cafes::NewCafe;
use brewlog::domain::ids::RoasterId;
use brewlog::domain::roasters::NewRoaster;
use brewlog::domain::roasts::NewRoast;
use reqwest::Client;
use tokio::time::{Duration, sleep};

async fn create_roast(app: &crate::helpers::TestApp, roaster_id: RoasterId, name: &str) {
    let client = Client::new();
    let roast = NewRoast {
        roaster_id,
        name: name.to_string(),
        origin: "Ethiopia".to_string(),
        region: "Yirgacheffe".to_string(),
        producer: "Chelbesa Cooperative".to_string(),
        tasting_notes: vec!["Blueberry".to_string(), "Jasmine".to_string()],
        process: "Washed".to_string(),
        created_at: None,
    };

    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&roast)
        .send()
        .await
        .expect("failed to create roast");

    assert_eq!(response.status(), 201);
}

async fn seed_timeline_with_roasts(
    app: &crate::helpers::TestApp,
    roast_count: usize,
) -> (String, Vec<String>) {
    let roaster_name = "Timeline Seed Roasters";
    let roaster = create_roaster_with_payload(
        app,
        NewRoaster {
            name: roaster_name.to_string(),
            country: "UK".to_string(),
            city: Some("Bristol".to_string()),
            homepage: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await;

    // Ensure the roaster event predates the roast events.
    sleep(Duration::from_millis(5)).await;

    let mut roast_names = Vec::new();
    for index in 0..roast_count {
        let roast_name = format!("Seed Roast {index:02}");
        create_roast(app, roaster.id, &roast_name).await;
        roast_names.push(roast_name);
        // Space out timestamps to keep ordering deterministic.
        sleep(Duration::from_millis(2)).await;
    }

    (roaster_name.to_string(), roast_names)
}

#[tokio::test]
async fn timeline_page_returns_a_200_with_empty_state() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("No events yet"),
        "Expected empty timeline state message, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_roaster_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster_name = "Timeline Roasters";
    create_roaster_with_payload(
        &app,
        NewRoaster {
            name: roaster_name.to_string(),
            country: "UK".to_string(),
            city: Some("Bristol".to_string()),
            homepage: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await;

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Roaster Added"),
        "Expected 'Roaster Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains(roaster_name),
        "Expected roaster name to appear in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("/data?type=roasters"),
        "Expected roaster link in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_roast_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster_id = create_roaster_with_payload(
        &app,
        NewRoaster {
            name: "Timeline Roast Roasters".to_string(),
            country: "UK".to_string(),
            city: Some("Bristol".to_string()),
            homepage: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await
    .id;

    sleep(Duration::from_millis(5)).await;
    let roast_name = "Timeline Natural";
    create_roast(&app, roaster_id, roast_name).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Roast Added"),
        "Expected 'Roast Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains(roast_name),
        "Expected roast name to appear in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("Blueberry"),
        "Expected tasting notes to appear in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_bag_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster_id = create_roaster_with_payload(
        &app,
        NewRoaster {
            name: "Bag Timeline Roasters".to_string(),
            country: "UK".to_string(),
            city: Some("Bristol".to_string()),
            homepage: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await
    .id;

    sleep(Duration::from_millis(5)).await;
    let roast_name = "Bag Timeline Roast";
    create_roast(&app, roaster_id, roast_name).await;

    // Fetch the roast to get its ID
    let roasts_response = client
        .get(app.api_url("/roasts"))
        .send()
        .await
        .expect("failed to fetch roasts");

    assert_eq!(roasts_response.status(), 200);
    let roasts: Vec<brewlog::domain::roasts::RoastWithRoaster> = roasts_response
        .json()
        .await
        .expect("failed to parse roasts");
    let roast_id = roasts.first().unwrap().roast.id;

    // Create a bag
    let bag_submission = serde_json::json!({
        "roast_id": roast_id,
        "roast_date": "2023-01-01",
        "amount": 250.0
    });

    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&bag_submission)
        .send()
        .await
        .expect("failed to create bag");
    assert_eq!(response.status(), 201);

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Bag Added"),
        "Expected 'Bag Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains(&format!("{}", roast_name)),
        "Expected bag title to appear in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn timeline_page_signals_more_results_when_over_page_size() {
    let app = spawn_app_with_auth().await;
    let (_, roast_names) = seed_timeline_with_roasts(&app, 6).await;
    assert_eq!(roast_names.len(), 6);

    let client = Client::new();

    // Explicitly request page_size=5 to test pagination with 6 events
    let response = client
        .get(format!("{}/timeline?page_size=5", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("failed to read response body");

    assert!(
        body.contains(
            "data-next-url=\"/timeline?page=2&#38;page_size=5&#38;sort=occurred-at&#38;dir=desc\""
        ),
        "Expected loader next-page URL missing from timeline HTML:\n{}",
        body
    );
    assert!(
        body.contains("data-has-more=\"true\""),
        "Expected loader to signal additional pages"
    );

    let latest_roast = roast_names.last().unwrap();
    assert!(
        body.contains(latest_roast),
        "Expected most recent roast '{latest_roast}' to appear in first page HTML"
    );

    let event_occurrences = body.matches("data-timeline-event").count();
    assert_eq!(
        event_occurrences, 5,
        "Expected exactly 5 events on first page"
    );
}

#[tokio::test]
async fn timeline_chunk_endpoint_serves_remaining_events() {
    let app = spawn_app_with_auth().await;
    let (roaster_name, roast_names) = seed_timeline_with_roasts(&app, 6).await;
    let oldest_roast = roast_names
        .first()
        .expect("missing seeded roast name")
        .clone();
    let client = Client::new();

    // page_size=5 to test pagination with 6 events
    let chunk_url = format!(
        "{}/timeline?page=2&page_size=5&sort=occurred-at&dir=desc",
        app.address
    );

    let response = client
        .get(chunk_url)
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to fetch timeline chunk");

    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("failed to read response body");

    assert!(
        body.contains(&oldest_roast),
        "Expected chunk payload to include oldest roast '{oldest_roast}':\n{}",
        body
    );
    assert!(
        body.contains(&roaster_name),
        "Expected chunk payload to include the roaster event: {body}"
    );
    assert!(
        body.contains("data-has-more=\"false\""),
        "Expected chunk to disable further pagination"
    );
    assert!(
        body.contains("data-next-url=\"\""),
        "Expected chunk to clear next URL once exhausted"
    );
}

#[tokio::test]
async fn closing_a_bag_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster_id = create_roaster_with_payload(
        &app,
        NewRoaster {
            name: "Bag Finish Timeline Roasters".to_string(),
            country: "UK".to_string(),
            city: Some("Bristol".to_string()),
            homepage: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await
    .id;

    sleep(Duration::from_millis(5)).await;
    let roast_name = "Bag Finish Timeline Roast";
    create_roast(&app, roaster_id, roast_name).await;

    // Fetch the roast to get its ID
    let roasts_response = client
        .get(app.api_url("/roasts"))
        .send()
        .await
        .expect("failed to fetch roasts");

    let roasts: Vec<brewlog::domain::roasts::RoastWithRoaster> = roasts_response
        .json()
        .await
        .expect("failed to parse roasts");
    let roast_id = roasts.first().unwrap().roast.id;

    // Create a bag
    let bag_submission = serde_json::json!({
        "roast_id": roast_id,
        "roast_date": "2023-01-01",
        "amount": 250.0
    });

    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&bag_submission)
        .send()
        .await
        .expect("failed to create bag");

    let bag: brewlog::domain::bags::Bag = response.json().await.expect("failed to parse bag");

    sleep(Duration::from_millis(10)).await;

    // Close the bag
    let update_submission = serde_json::json!({
        "closed": true
    });

    let response = client
        .put(app.api_url(&format!("/bags/{}", bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update_submission)
        .send()
        .await
        .expect("failed to update bag");
    assert_eq!(response.status(), 200);

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Bag Finished"),
        "Expected 'Bag Finished' badge in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn creating_gear_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    // Create gear
    let gear_submission = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&gear_submission)
        .send()
        .await
        .expect("failed to create gear");
    assert_eq!(response.status(), 201);

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Gear Added"),
        "Expected 'Gear Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("Baratza Encore"),
        "Expected gear title to appear in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("Grinder"),
        "Expected gear category to appear in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_cafe_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let cafe_name = "Timeline Test Cafe";
    create_cafe_with_payload(
        &app,
        NewCafe {
            name: cafe_name.to_string(),
            city: "Bristol".to_string(),
            country: "UK".to_string(),
            latitude: 51.4545,
            longitude: -2.5879,
            website: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await;

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Cafe Added"),
        "Expected 'Cafe Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains(cafe_name),
        "Expected cafe name to appear in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("/data?type=cafes"),
        "Expected cafe link in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn checkin_surfaces_cup_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    sleep(Duration::from_millis(10)).await;

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
        .expect("failed to create check-in");
    assert_eq!(response.status(), 201);

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Cup Added"),
        "Expected 'Cup Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("Test Roast"),
        "Expected roast name to appear in cup timeline event, got: {body}"
    );
}

#[tokio::test]
async fn checkin_with_new_cafe_surfaces_both_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    sleep(Duration::from_millis(10)).await;

    let cafe_name = "Checkin Timeline Cafe";
    let payload = serde_json::json!({
        "roast_id": roast.id.to_string(),
        "cafe_name": cafe_name,
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
        .expect("failed to create check-in");
    assert_eq!(response.status(), 201);

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Cafe Added"),
        "Expected 'Cafe Added' badge in timeline HTML for new cafe, got: {body}"
    );
    assert!(
        body.contains("Cup Added"),
        "Expected 'Cup Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains(cafe_name),
        "Expected cafe name to appear in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_brew_surfaces_on_the_timeline() {
    let app = spawn_app_with_auth().await;
    let client = Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;

    sleep(Duration::from_millis(10)).await;

    let new_brew = NewBrew {
        bag_id: bag.id,
        coffee_weight: 15.0,
        grinder_id: grinder.id,
        grind_setting: 24.0,
        brewer_id: brewer.id,
        filter_paper_id: None,
        water_volume: 250,
        water_temp: 92.0,
        quick_notes: Vec::new(),
        brew_time: None,
        created_at: None,
    };

    let response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("failed to create brew");
    assert_eq!(response.status(), 201);

    sleep(Duration::from_millis(10)).await;

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("Brew Added"),
        "Expected 'Brew Added' badge in timeline HTML, got: {body}"
    );
    assert!(
        body.contains("Test Roast"),
        "Expected roast name to appear in brew timeline event, got: {body}"
    );
}
