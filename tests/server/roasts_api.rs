use crate::helpers::{create_default_roaster, create_roaster_with_name, spawn_app_with_auth};
use crate::test_macros::define_crud_tests;
use brewlog::domain::ids::RoasterId;
use brewlog::domain::roasts::{NewRoast, Roast, RoastWithRoaster};

define_crud_tests!(
    entity: roast,
    path: "/roasts",
    list_type: RoastWithRoaster,
    malformed_json: r#"{"name": "Test", "roaster_id": }"#
);

#[tokio::test]
async fn creating_a_roast_returns_a_201_for_valid_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id,
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
        .bearer_auth(app.auth_token.as_ref().unwrap())
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
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id,
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
        .bearer_auth(app.auth_token.as_ref().unwrap())
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
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: RoasterId::new(999999),
        name: "Orphaned Roast".to_string(),
        origin: "Unknown".to_string(),
        region: "Unknown".to_string(),
        producer: "Unknown Producer".to_string(),
        tasting_notes: vec!["Bitter".to_string()],
        process: "Unknown".to_string(),
    };

    // Act
    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
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
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id,
        name: "Kenyan AA".to_string(),
        origin: "Kenya".to_string(),
        region: "Nyeri".to_string(),
        producer: "Estate".to_string(),
        tasting_notes: vec!["Blackcurrant".to_string()],
        process: "Washed".to_string(),
    };

    let create_response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to create roast");

    let created_roast: Roast = create_response
        .json()
        .await
        .expect("Failed to parse response");

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
async fn listing_roasts_returns_a_200_with_multiple_roasts() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    // Create multiple roasts
    let roast1 = NewRoast {
        roaster_id: roaster_id,
        name: "First Roast".to_string(),
        origin: "Brazil".to_string(),
        region: "Santos".to_string(),
        producer: "Farm A".to_string(),
        tasting_notes: vec!["Chocolate".to_string()],
        process: "Natural".to_string(),
    };

    let roast2 = NewRoast {
        roaster_id: roaster_id,
        name: "Second Roast".to_string(),
        origin: "Guatemala".to_string(),
        region: "Antigua".to_string(),
        producer: "Farm B".to_string(),
        tasting_notes: vec!["Caramel".to_string()],
        process: "Washed".to_string(),
    };

    client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&roast1)
        .send()
        .await
        .expect("Failed to create first roast");

    client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
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
    let app = spawn_app_with_auth().await;
    let roaster1_id = create_default_roaster(&app).await.id;
    let roaster2_id = create_roaster_with_name(&app, "Second Roasters").await.id;

    let client = reqwest::Client::new();

    // Create roasts for both roasters
    let roast1 = NewRoast {
        roaster_id: roaster1_id,
        name: "Roaster 1 Roast".to_string(),
        origin: "Brazil".to_string(),
        region: "Santos".to_string(),
        producer: "Farm A".to_string(),
        tasting_notes: vec!["Chocolate".to_string()],
        process: "Natural".to_string(),
    };

    let roast2 = NewRoast {
        roaster_id: roaster2_id,
        name: "Roaster 2 Roast".to_string(),
        origin: "Guatemala".to_string(),
        region: "Antigua".to_string(),
        producer: "Farm B".to_string(),
        tasting_notes: vec!["Caramel".to_string()],
        process: "Washed".to_string(),
    };

    client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&roast1)
        .send()
        .await
        .expect("Failed to create first roast");

    client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
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
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    let new_roast = NewRoast {
        roaster_id: roaster_id,
        name: "Temporary Roast".to_string(),
        origin: "Peru".to_string(),
        region: "Cusco".to_string(),
        producer: "Temporary Co-op".to_string(),
        tasting_notes: vec!["Fleeting".to_string()],
        process: "Washed".to_string(),
    };

    let create_response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_roast)
        .send()
        .await
        .expect("Failed to create roast");

    let created_roast: Roast = create_response
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let response = client
        .delete(app.api_url(&format!("/roasts/{}", created_roast.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
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
async fn creating_a_roast_with_empty_name_returns_a_400() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    // Act - Create roast with empty name (after trim)
    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("content-type", "application/json")
        .body(format!(
            r#"{{
                "roaster_id": {},
                "name": "   ",
                "origin": "Ethiopia",
                "region": "Yirgacheffe",
                "producer": "Co-op",
                "tasting_notes": "Blueberry",
                "process": "Washed"
            }}"#,
            i64::from(roaster_id)
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn creating_a_roast_with_missing_required_fields_returns_a_400() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    // Act - Missing 'origin' field
    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("content-type", "application/json")
        .body(format!(
            r#"{{
                "roaster_id": {},
                "name": "Test Roast",
                "region": "Yirgacheffe",
                "producer": "Co-op",
                "tasting_notes": "Blueberry",
                "process": "Washed"
            }}"#,
            i64::from(roaster_id)
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn creating_a_roast_with_empty_tasting_notes_returns_a_400() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster_id = create_default_roaster(&app).await.id;
    let client = reqwest::Client::new();

    // Act - Empty tasting notes
    let response = client
        .post(app.api_url("/roasts"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("content-type", "application/json")
        .body(format!(
            r#"{{
                "roaster_id": {},
                "name": "Test Roast",
                "origin": "Ethiopia",
                "region": "Yirgacheffe",
                "producer": "Co-op",
                "tasting_notes": "",
                "process": "Washed"
            }}"#,
            i64::from(roaster_id)
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 400);
}
