use crate::helpers::{spawn_app, spawn_app_with_auth};
use brewlog::domain::gear::{Gear, UpdateGear};

#[tokio::test]
async fn creating_gear_returns_201_for_valid_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    // Act
    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 201);

    let gear: Gear = response.json().await.expect("Failed to parse response");
    assert_eq!(gear.make, "Baratza");
    assert_eq!(gear.model, "Encore");
}

#[tokio::test]
async fn creating_gear_without_auth_returns_401() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    // Act
    let response = client
        .post(app.api_url("/gear"))
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn creating_gear_with_invalid_category_returns_400() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_gear = serde_json::json!({
        "category": "invalid_category",
        "make": "Baratza",
        "model": "Encore"
    });

    // Act
    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn creating_gear_with_empty_make_returns_400() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "",
        "model": "Encore"
    });

    // Act
    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn creating_gear_with_empty_model_returns_400() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": ""
    });

    // Act
    let response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn listing_gear_returns_200_and_correct_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Create a gear
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
        .expect("Failed to create gear");

    // Act
    let response = client
        .get(app.api_url("/gear"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let gear_list: Vec<Gear> = response.json().await.expect("Failed to parse response");
    assert!(!gear_list.is_empty());
    assert!(gear_list.iter().any(|g| g.make == "Baratza"));
}

#[tokio::test]
async fn listing_gear_filtered_by_category() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Create grinder
    let grinder = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&grinder)
        .send()
        .await
        .expect("Failed to create grinder");

    // Create brewer
    let brewer = serde_json::json!({
        "category": "brewer",
        "make": "Hario",
        "model": "V60"
    });

    client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&brewer)
        .send()
        .await
        .expect("Failed to create brewer");

    // Act - filter by grinder category
    let response = client
        .get(app.api_url("/gear?category=grinder"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let gear_list: Vec<Gear> = response.json().await.expect("Failed to parse response");
    assert!(gear_list.iter().all(|g| g.category.as_str() == "grinder"));
    assert!(gear_list.iter().any(|g| g.make == "Baratza"));
}

#[tokio::test]
async fn getting_gear_by_id_returns_correct_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Create gear
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    let create_response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to create gear");

    let created_gear: Gear = create_response.json().await.unwrap();

    // Act
    let response = client
        .get(app.api_url(&format!("/gear/{}", created_gear.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let gear: Gear = response.json().await.expect("Failed to parse response");
    assert_eq!(gear.id, created_gear.id);
    assert_eq!(gear.make, "Baratza");
    assert_eq!(gear.model, "Encore");
}

#[tokio::test]
async fn updating_gear_returns_updated_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Create gear
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    let create_response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to create gear");

    let created_gear: Gear = create_response.json().await.unwrap();

    // Act
    let update = UpdateGear {
        make: Some("Comandante".to_string()),
        model: Some("C40".to_string()),
        created_at: None,
    };

    let response = client
        .put(app.api_url(&format!("/gear/{}", created_gear.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let updated_gear: Gear = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_gear.id, created_gear.id);
    assert_eq!(updated_gear.make, "Comandante");
    assert_eq!(updated_gear.model, "C40");
}

#[tokio::test]
async fn updating_gear_without_auth_returns_401() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let update = UpdateGear {
        make: Some("Updated".to_string()),
        model: None,
        created_at: None,
    };

    // Act
    let response = client
        .put(app.api_url("/gear/123"))
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn deleting_gear_returns_204() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    // Create gear
    let new_gear = serde_json::json!({
        "category": "grinder",
        "make": "Baratza",
        "model": "Encore"
    });

    let create_response = client
        .post(app.api_url("/gear"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_gear)
        .send()
        .await
        .expect("Failed to create gear");

    let created_gear: Gear = create_response.json().await.unwrap();

    // Act
    let response = client
        .delete(app.api_url(&format!("/gear/{}", created_gear.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 204);

    // Verify gear is actually deleted
    let get_response = client
        .get(app.api_url(&format!("/gear/{}", created_gear.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn deleting_gear_without_auth_returns_401() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .delete(app.api_url("/gear/123"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 401);
}
