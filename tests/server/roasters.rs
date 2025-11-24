use crate::server::helpers::spawn_app;
use brewlog::domain::roasters::{NewRoaster, Roaster, UpdateRoaster};

#[tokio::test]
async fn creating_a_roaster_returns_a_201_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roaster = NewRoaster {
        name: "Test Roasters".to_string(),
        country: "United Kingdom".to_string(),
        city: Some("London".to_string()),
        homepage: Some("https://example.com".to_string()),
        notes: Some("Great coffee".to_string()),
    };

    // Act
    let response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 201);

    let roaster: Roaster = response.json().await.expect("Failed to parse response");
    assert_eq!(roaster.name, "Test Roasters");
    assert_eq!(roaster.country, "United Kingdom");
    assert_eq!(roaster.city, Some("London".to_string()));
    assert_eq!(roaster.homepage, Some("https://example.com".to_string()));
    assert_eq!(roaster.notes, Some("Great coffee".to_string()));
}

#[tokio::test]
async fn creating_a_roaster_persists_the_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roaster = NewRoaster {
        name: "Persistent Roasters".to_string(),
        country: "France".to_string(),
        city: Some("Paris".to_string()),
        homepage: None,
        notes: None,
    };

    // Act
    let response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to execute request");

    let roaster: Roaster = response.json().await.expect("Failed to parse response");

    // Assert - Fetch the roaster to verify it was persisted
    let fetched_roaster = app
        .roaster_repo
        .get(roaster.id.clone())
        .await
        .expect("Failed to fetch roaster");

    assert_eq!(fetched_roaster.name, "Persistent Roasters");
    assert_eq!(fetched_roaster.country, "France");
    assert_eq!(fetched_roaster.city, Some("Paris".to_string()));
}

#[tokio::test]
async fn getting_a_roaster_returns_a_200_for_valid_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roaster = NewRoaster {
        name: "Fetchable Roasters".to_string(),
        country: "Germany".to_string(),
        city: Some("Berlin".to_string()),
        homepage: None,
        notes: None,
    };

    let create_response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to create roaster");

    let created_roaster: Roaster = create_response.json().await.expect("Failed to parse response");

    // Act
    let response = client
        .get(app.api_url(&format!("/roasters/{}", created_roaster.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roaster: Roaster = response.json().await.expect("Failed to parse response");
    assert_eq!(roaster.id, created_roaster.id);
    assert_eq!(roaster.name, "Fetchable Roasters");
}

#[tokio::test]
async fn getting_a_nonexistent_roaster_returns_a_404() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(app.api_url("/roasters/nonexistent-id"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn listing_roasters_returns_a_200_with_empty_list() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(app.api_url("/roasters"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roasters: Vec<Roaster> = response.json().await.expect("Failed to parse response");
    assert_eq!(roasters.len(), 0);
}

#[tokio::test]
async fn listing_roasters_returns_a_200_with_multiple_roasters() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Create multiple roasters
    let roaster1 = NewRoaster {
        name: "First Roasters".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };

    let roaster2 = NewRoaster {
        name: "Second Roasters".to_string(),
        country: "USA".to_string(),
        city: Some("New York".to_string()),
        homepage: None,
        notes: None,
    };

    client
        .post(app.api_url("/roasters"))
        .json(&roaster1)
        .send()
        .await
        .expect("Failed to create first roaster");

    client
        .post(app.api_url("/roasters"))
        .json(&roaster2)
        .send()
        .await
        .expect("Failed to create second roaster");

    // Act
    let response = client
        .get(app.api_url("/roasters"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let roasters: Vec<Roaster> = response.json().await.expect("Failed to parse response");
    assert_eq!(roasters.len(), 2);
}

#[tokio::test]
async fn updating_a_roaster_returns_a_200_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roaster = NewRoaster {
        name: "Original Name".to_string(),
        country: "UK".to_string(),
        city: Some("Manchester".to_string()),
        homepage: None,
        notes: None,
    };

    let create_response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to create roaster");

    let created_roaster: Roaster = create_response.json().await.expect("Failed to parse response");

    let update = UpdateRoaster {
        name: Some("Updated Name".to_string()),
        country: None,
        city: Some("Liverpool".to_string()),
        homepage: Some("https://updated.com".to_string()),
        notes: None,
    };

    // Act
    let response = client
        .put(app.api_url(&format!("/roasters/{}", created_roaster.id)))
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let updated_roaster: Roaster = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_roaster.name, "Updated Name");
    assert_eq!(updated_roaster.country, "UK"); // Should remain unchanged
    assert_eq!(updated_roaster.city, Some("Liverpool".to_string()));
    assert_eq!(updated_roaster.homepage, Some("https://updated.com".to_string()));
}

#[tokio::test]
async fn updating_a_roaster_with_no_changes_returns_a_400() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roaster = NewRoaster {
        name: "Test Roaster".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };

    let create_response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to create roaster");

    let created_roaster: Roaster = create_response.json().await.expect("Failed to parse response");

    let update = UpdateRoaster {
        name: None,
        country: None,
        city: None,
        homepage: None,
        notes: None,
    };

    // Act
    let response = client
        .put(app.api_url(&format!("/roasters/{}", created_roaster.id)))
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn updating_a_nonexistent_roaster_returns_a_404() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let update = UpdateRoaster {
        name: Some("New Name".to_string()),
        country: None,
        city: None,
        homepage: None,
        notes: None,
    };

    // Act
    let response = client
        .put(app.api_url("/roasters/nonexistent-id"))
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn deleting_a_roaster_returns_a_204_for_valid_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_roaster = NewRoaster {
        name: "To Be Deleted".to_string(),
        country: "UK".to_string(),
        city: None,
        homepage: None,
        notes: None,
    };

    let create_response = client
        .post(app.api_url("/roasters"))
        .json(&new_roaster)
        .send()
        .await
        .expect("Failed to create roaster");

    let created_roaster: Roaster = create_response.json().await.expect("Failed to parse response");

    // Act
    let response = client
        .delete(app.api_url(&format!("/roasters/{}", created_roaster.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 204);

    // Verify roaster was deleted
    let get_response = client
        .get(app.api_url(&format!("/roasters/{}", created_roaster.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn deleting_a_nonexistent_roaster_returns_a_404() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .delete(app.api_url("/roasters/nonexistent-id"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 404);
}
