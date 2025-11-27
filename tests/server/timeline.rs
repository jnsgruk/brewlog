use crate::helpers::{create_roaster_with_payload, spawn_app_with_auth};
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
            notes: None,
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
    let roaster = create_roaster_with_payload(
        &app,
        NewRoaster {
            name: roaster_name.to_string(),
            country: "UK".to_string(),
            city: Some("Bristol".to_string()),
            homepage: Some("https://example.com".to_string()),
            notes: None,
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
        body.contains(&format!("/roasters/{}", roaster.slug)),
        "Expected roaster detail link in timeline HTML, got: {body}"
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
            notes: None,
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
            notes: None,
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

    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("failed to read response body");

    assert!(
        body.contains(
            "data-next-url=\"/timeline?page=2&amp;page_size=5&amp;sort=occurred-at&amp;dir=desc\""
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
            notes: None,
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
    assert!(
        body.contains(&format!("Finished: {}", roast_name)),
        "Expected bag finished title to appear in timeline HTML, got: {body}"
    );
}
