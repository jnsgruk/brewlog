use crate::helpers::{create_roaster_with_payload, spawn_app};
use brewlog::domain::roasters::NewRoaster;
use brewlog::domain::roasts::NewRoast;
use reqwest::Client;
use tokio::time::{Duration, sleep};

async fn create_roast(app: &crate::helpers::TestApp, roaster_id: &str, name: &str) {
    let client = Client::new();
    let roast = NewRoast {
        roaster_id: roaster_id.to_string(),
        name: name.to_string(),
        origin: "Ethiopia".to_string(),
        region: "Yirgacheffe".to_string(),
        producer: "Chelbesa Cooperative".to_string(),
        tasting_notes: vec!["Blueberry".to_string(), "Jasmine".to_string()],
        process: "Washed".to_string(),
    };

    let response = client
        .post(app.api_url("/roasts"))
        .json(&roast)
        .send()
        .await
        .expect("failed to create roast");

    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn timeline_page_returns_a_200_with_empty_state() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    // Assert
    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("failed to read response body");
    assert!(
        body.contains("No events yet"),
        "Expected empty timeline state message, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_roaster_surfaces_on_the_timeline() {
    // Arrange
    let app = spawn_app().await;
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
    let roaster_id = roaster.id.clone();

    // Give the database a brief moment to commit timestamps
    sleep(Duration::from_millis(10)).await;

    // Act
    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    // Assert
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
        body.contains(&format!("/roasters/{}", roaster_id)),
        "Expected roaster detail link in timeline HTML, got: {body}"
    );
}

#[tokio::test]
async fn creating_a_roast_surfaces_on_the_timeline() {
    // Arrange
    let app = spawn_app().await;
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
    // Ensure the roast event occurs after the roaster event to make ordering deterministic
    sleep(Duration::from_millis(5)).await;
    let roast_name = "Timeline Natural";
    create_roast(&app, &roaster_id, roast_name).await;

    // Act
    let response = client
        .get(format!("{}/timeline", app.address))
        .send()
        .await
        .expect("failed to fetch timeline");

    // Assert
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
