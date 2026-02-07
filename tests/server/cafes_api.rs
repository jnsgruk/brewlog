use crate::helpers::{create_default_cafe, spawn_app_with_auth};
use crate::test_macros::define_crud_tests;
use brewlog::domain::cafes::{Cafe, NewCafe, UpdateCafe};

define_crud_tests!(
    entity: cafe,
    path: "/cafes",
    list_type: Cafe,
    malformed_json: r#"{"name": "Test", "city": }"#,
    missing_fields: r#"{"name": "Test Cafe"}"#
);

#[tokio::test]
async fn creating_a_cafe_returns_a_201_for_valid_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_cafe = NewCafe {
        name: "Blue Bottle".to_string(),
        city: "San Francisco".to_string(),
        country: "United States".to_string(),
        latitude: 37.7749,
        longitude: -122.4194,
        website: Some("https://bluebottlecoffee.com".to_string()),
        created_at: None,
    };

    let response = client
        .post(app.api_url("/cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cafe)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let cafe: Cafe = response.json().await.expect("Failed to parse response");
    assert_eq!(cafe.name, "Blue Bottle");
    assert_eq!(cafe.city, "San Francisco");
    assert_eq!(cafe.country, "United States");
    assert_eq!(cafe.latitude, 37.7749);
    assert_eq!(cafe.longitude, -122.4194);
    assert_eq!(
        cafe.website,
        Some("https://bluebottlecoffee.com".to_string())
    );
}

#[tokio::test]
async fn creating_a_cafe_persists_the_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_cafe = NewCafe {
        name: "Persistent Cafe".to_string(),
        city: "Paris".to_string(),
        country: "France".to_string(),
        latitude: 48.8566,
        longitude: 2.3522,
        website: None,
        created_at: None,
    };

    let response = client
        .post(app.api_url("/cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cafe)
        .send()
        .await
        .expect("Failed to execute request");

    let cafe: Cafe = response.json().await.expect("Failed to parse response");

    let fetched_cafe = app
        .cafe_repo
        .get(cafe.id)
        .await
        .expect("Failed to fetch cafe");

    assert_eq!(fetched_cafe.name, "Persistent Cafe");
    assert_eq!(fetched_cafe.city, "Paris");
    assert_eq!(fetched_cafe.country, "France");
}

#[tokio::test]
async fn creating_a_cafe_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_cafe = NewCafe {
        name: "Auth Test Cafe".to_string(),
        city: "London".to_string(),
        country: "UK".to_string(),
        latitude: 51.5074,
        longitude: -0.1278,
        website: None,
        created_at: None,
    };

    let response = client
        .post(app.api_url("/cafes"))
        .json(&new_cafe)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn listing_cafes_returns_a_200_with_multiple_cafes() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let cafe1 = NewCafe {
        name: "First Cafe".to_string(),
        city: "London".to_string(),
        country: "UK".to_string(),
        latitude: 51.5074,
        longitude: -0.1278,
        website: None,
        created_at: None,
    };

    let cafe2 = NewCafe {
        name: "Second Cafe".to_string(),
        city: "Berlin".to_string(),
        country: "Germany".to_string(),
        latitude: 52.52,
        longitude: 13.405,
        website: None,
        created_at: None,
    };

    client
        .post(app.api_url("/cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&cafe1)
        .send()
        .await
        .expect("Failed to create first cafe");

    client
        .post(app.api_url("/cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&cafe2)
        .send()
        .await
        .expect("Failed to create second cafe");

    let response = client
        .get(app.api_url("/cafes"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let cafes: Vec<Cafe> = response.json().await.expect("Failed to parse response");
    assert_eq!(cafes.len(), 2);
}

#[tokio::test]
async fn getting_a_cafe_returns_a_200_for_valid_id() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let cafe = create_default_cafe(&app).await;

    let response = client
        .get(app.api_url(&format!("/cafes/{}", cafe.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let fetched: Cafe = response.json().await.expect("Failed to parse response");
    assert_eq!(fetched.id, cafe.id);
    assert_eq!(fetched.name, "Blue Bottle");
}

#[tokio::test]
async fn updating_a_cafe_returns_a_200_for_valid_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let cafe = create_default_cafe(&app).await;

    let update = UpdateCafe {
        name: Some("Updated Cafe".to_string()),
        city: None,
        country: None,
        latitude: None,
        longitude: None,
        website: Some("https://updated.com".to_string()),
        created_at: None,
    };

    let response = client
        .put(app.api_url(&format!("/cafes/{}", cafe.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let updated: Cafe = response.json().await.expect("Failed to parse response");
    assert_eq!(updated.name, "Updated Cafe");
    assert_eq!(updated.city, "San Francisco"); // unchanged
    assert_eq!(updated.website, Some("https://updated.com".to_string()));
}

#[tokio::test]
async fn updating_a_cafe_with_no_changes_returns_a_400() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let cafe = create_default_cafe(&app).await;

    let update = UpdateCafe {
        name: None,
        city: None,
        country: None,
        latitude: None,
        longitude: None,
        website: None,
        created_at: None,
    };

    let response = client
        .put(app.api_url(&format!("/cafes/{}", cafe.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn updating_a_nonexistent_cafe_returns_a_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let update = UpdateCafe {
        name: Some("New Name".to_string()),
        city: None,
        country: None,
        latitude: None,
        longitude: None,
        website: None,
        created_at: None,
    };

    let response = client
        .put(app.api_url("/cafes/999999"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn deleting_a_cafe_returns_a_204_for_valid_id() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let cafe = create_default_cafe(&app).await;

    let response = client
        .delete(app.api_url(&format!("/cafes/{}", cafe.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 204);

    // Verify cafe was deleted
    let get_response = client
        .get(app.api_url(&format!("/cafes/{}", cafe.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}
