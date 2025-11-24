use crate::server::helpers::spawn_app;
use brewlog::domain::roasters::NewRoaster;
use brewlog::domain::roasts::{NewRoast, Roast, RoastWithRoaster};

async fn create_test_roaster(app: &crate::server::helpers::TestApp) -> String {
    let new_roaster = NewRoaster {
        name: "Test Roasters".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to create roaster");

    let roaster: brewlog::domain::roasters::Roaster = response
        .json()
        .await
        .expect("Failed to parse roaster");

    roaster.id
}

#[tokio::test]
async fn creating_a_roast_returns_a_201_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let roaster_id = create_test_roaster(&app).await;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id.clone(),
        name: "Ethiopian Yirgacheffe".to_string(),
        origin: "Ethiopia".to_string(),
        region: "Yirgacheffe".to_string(),
        producer: "Local Cooperative".to_string(),
        tasting_notes: vec![
            "Blueberry".to_string(),
            "Chocolate".to_string(),
            "Citrus".to_string(),
        ],
        process: "Washed".to_string(),
    };

    // Act
    let response = client
        .post(app.api_url("/roasts"))
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 201);

    let roast: Roast = response.json().await.expect("Failed to parse response");
    assert_eq!(roast.name, "Ethiopian Yirgacheffe");
    assert_eq!(roast.roaster_id, roaster_id);
    assert_eq!(roast.origin, Some("Ethiopia".to_string()));
    assert_eq!(roast.region, Some("Yirgacheffe".to_string()));
    assert_eq!(roast.producer, Some("Local Cooperative".to_string()));
    assert_eq!(roast.tasting_notes.len(), 3);
    assert_eq!(roast.process, Some("Washed".to_string()));
}

#[tokio::test]
async fn creating_a_roast_persists_the_data() {
    // Arrange
    let app = spawn_app().await;
    let roaster_id = create_test_roaster(&app).await;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id.clone(),
        name: "Colombian Supremo".to_string(),
        origin: "Colombia".to_string(),
        region: "Huila".to_string(),
        producer: "Farm Co-op".to_string(),
        tasting_notes: vec!["Caramel".to_string(), "Nuts".to_string()],
        process: "Natural".to_string(),
    };

    // Act
    let response = client
        .post(app.api_url("/roasts"))
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to execute request");

    let roast: Roast = response.json().await.expect("Failed to parse response");

    // Assert - Fetch the roast to verify it was persisted
    let fetched_roast = app
        .roast_repo
        .get(roast.id.clone())
        .await
        .expect("Failed to fetch roast");

    assert_eq!(fetched_roast.name, "Colombian Supremo");
    assert_eq!(fetched_roast.roaster_id, roaster_id);
    assert_eq!(fetched_roast.origin, Some("Colombia".to_string()));
}

#[tokio::test]
async fn creating_a_roast_with_nonexistent_roaster_returns_a_404() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: "nonexistent-roaster-id".to_string(),
        name: "Test Roast".to_string(),
        origin: "Test".to_string(),
        region: "Test".to_string(),
        producer: "Test".to_string(),
        tasting_notes: vec!["Test".to_string()],
        process: "Test".to_string(),
    };

    // Act
    let response = client
        .post(app.api_url("/roasts"))
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn getting_a_roast_returns_a_200_for_valid_id() {
    // Arrange
    let app = spawn_app().await;
    let roaster_id = create_test_roaster(&app).await;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id.clone(),
        name: "Kenyan AA".to_string(),
        origin: "Kenya".to_string(),
        region: "Nyeri".to_string(),
        producer: "Estate".to_string(),
        tasting_notes: vec!["Blackcurrant".to_string()],
        process: "Washed".to_string(),
    };

    let create_response = client
        .post(app.api_url("/roasts"))
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to create roast");

    let created_roast: Roast = create_response.json().await.expect("Failed to parse response");

    // Act
    let response = client
        .get(app.api_url(&format!("/roasts/{}", created_roast.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roast: Roast = response.json().await.expect("Failed to parse response");
    assert_eq!(roast.id, created_roast.id);
    assert_eq!(roast.name, "Kenyan AA");
}

#[tokio::test]
async fn getting_a_nonexistent_roast_returns_a_404() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(app.api_url("/roasts/nonexistent-id"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn listing_roasts_returns_a_200_with_empty_list() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(app.api_url("/roasts"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roasts: Vec<RoastWithRoaster> = response.json().await.expect("Failed to parse response");
    assert_eq!(roasts.len(), 0);
}

#[tokio::test]
async fn listing_roasts_returns_a_200_with_multiple_roasts() {
    // Arrange
    let app = spawn_app().await;
    let roaster_id = create_test_roaster(&app).await;
    let client = reqwest::Client::new();

    // Create multiple roasts
    let roast1 = NewRoast {
        roaster_id: roaster_id.clone(),
        name: "First Roast".to_string(),
        origin: "Brazil".to_string(),
        region: "Santos".to_string(),
        producer: "Farm A".to_string(),
        tasting_notes: vec!["Chocolate".to_string()],
        process: "Natural".to_string(),
    };

    let roast2 = NewRoast {
        roaster_id: roaster_id.clone(),
        name: "Second Roast".to_string(),
        origin: "Guatemala".to_string(),
        region: "Antigua".to_string(),
        producer: "Farm B".to_string(),
        tasting_notes: vec!["Caramel".to_string()],
        process: "Washed".to_string(),
    };

    client
        .post(app.api_url("/roasts"))
        .json(&roast1)
        .send()
        .await
        .expect("Failed to create first roast");

    client
        .post(app.api_url("/roasts"))
        .json(&roast2)
        .send()
        .await
        .expect("Failed to create second roast");

    // Act
    let response = client
        .get(app.api_url("/roasts"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roasts: Vec<RoastWithRoaster> = response.json().await.expect("Failed to parse response");
    assert_eq!(roasts.len(), 2);
}

#[tokio::test]
async fn listing_roasts_by_roaster_returns_a_200_with_filtered_list() {
    // Arrange
    let app = spawn_app().await;
    let roaster1_id = create_test_roaster(&app).await;
    
    // Create a second roaster
    let new_roaster2 = NewRoaster {
        name: "Second Roasters".to_string(),
        country: "USA".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster2)
        .send()
        .await
        .expect("Failed to create second roaster");
    
    let roaster2: brewlog::domain::roasters::Roaster = response
        .json()
        .await
        .expect("Failed to parse roaster");
    let roaster2_id = roaster2.id;

    // Create roasts for both roasters
    let roast1 = NewRoast {
        roaster_id: roaster1_id.clone(),
        name: "Roaster 1 Roast".to_string(),
        origin: "Brazil".to_string(),
        region: "Santos".to_string(),
        producer: "Farm A".to_string(),
        tasting_notes: vec!["Chocolate".to_string()],
        process: "Natural".to_string(),
    };

    let roast2 = NewRoast {
        roaster_id: roaster2_id.clone(),
        name: "Roaster 2 Roast".to_string(),
        origin: "Guatemala".to_string(),
        region: "Antigua".to_string(),
        producer: "Farm B".to_string(),
        tasting_notes: vec!["Caramel".to_string()],
        process: "Washed".to_string(),
    };

    client
        .post(app.api_url("/roasts"))
        .json(&roast1)
        .send()
        .await
        .expect("Failed to create first roast");

    client
        .post(app.api_url("/roasts"))
        .json(&roast2)
        .send()
        .await
        .expect("Failed to create second roast");

    // Act
    let response = client
        .get(app.api_url(&format!("/roasts?roaster_id={}", roaster1_id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roasts: Vec<RoastWithRoaster> = response.json().await.expect("Failed to parse response");
    assert_eq!(roasts.len(), 1);
    assert_eq!(roasts[0].roast.roaster_id, roaster1_id);
}

#[tokio::test]
async fn deleting_a_roast_returns_a_204_for_valid_id() {
    // Arrange
    let app = spawn_app().await;
    let roaster_id = create_test_roaster(&app).await;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id.clone(),
        name: "To Be Deleted".to_string(),
        origin: "Test".to_string(),
        region: "Test".to_string(),
        producer: "Test".to_string(),
        tasting_notes: vec!["Test".to_string()],
        process: "Test".to_string(),
    };

    let create_response = client
        .post(app.api_url("/roasts"))
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to create roast");

    let created_roast: Roast = create_response.json().await.expect("Failed to parse response");

    // Act
    let response = client
        .delete(app.api_url(&format!("/roasts/{}", created_roast.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 204);

    // Verify roast was deleted
    let get_response = client
        .get(app.api_url(&format!("/roasts/{}", created_roast.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn deleting_a_nonexistent_roast_returns_a_404() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .delete(app.api_url("/roasts/nonexistent-id"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 404);
}
