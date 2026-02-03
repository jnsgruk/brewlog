use crate::helpers::{
    create_default_cafe, create_default_roast, create_default_roaster, spawn_app_with_auth,
};
use brewlog::domain::cups::{Cup, CupWithDetails, NewCup, UpdateCup};
use brewlog::domain::ids::{CafeId, RoastId};

#[tokio::test]
async fn creating_a_cup_returns_a_201_for_valid_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: Some("Excellent pour-over".to_string()),
        rating: Some(5),
    };

    let response = client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let cup: Cup = response.json().await.expect("Failed to parse response");
    assert_eq!(cup.roast_id, roast.id);
    assert_eq!(cup.cafe_id, cafe.id);
    assert_eq!(cup.notes, Some("Excellent pour-over".to_string()));
    assert_eq!(cup.rating, Some(5));
}

#[tokio::test]
async fn creating_a_cup_without_optional_fields_returns_a_201() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: None,
        rating: None,
    };

    let response = client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 201);

    let cup: Cup = response.json().await.expect("Failed to parse response");
    assert_eq!(cup.notes, None);
    assert_eq!(cup.rating, None);
}

#[tokio::test]
async fn creating_a_cup_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_cup = NewCup {
        roast_id: RoastId::new(1),
        cafe_id: CafeId::new(1),
        notes: None,
        rating: None,
    };

    let response = client
        .post(app.api_url("/cups"))
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn listing_cups_returns_a_200_with_empty_list() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let response = client
        .get(app.api_url("/cups"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let cups: Vec<CupWithDetails> = response.json().await.expect("Failed to parse response");
    assert_eq!(cups.len(), 0);
}

#[tokio::test]
async fn listing_cups_returns_a_200_with_enriched_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: None,
        rating: Some(4),
    };

    client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to create cup");

    let response = client
        .get(app.api_url("/cups"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let cups: Vec<CupWithDetails> = response.json().await.expect("Failed to parse response");
    assert_eq!(cups.len(), 1);
    assert_eq!(cups[0].roast_name, "Test Roast");
    assert_eq!(cups[0].roaster_name, "Test Roasters");
    assert_eq!(cups[0].cafe_name, "Blue Bottle");
}

#[tokio::test]
async fn getting_a_cup_returns_a_200_with_details() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: Some("Great".to_string()),
        rating: Some(3),
    };

    let create_response = client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to create cup");

    let cup: Cup = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let response = client
        .get(app.api_url(&format!("/cups/{}", cup.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let fetched: CupWithDetails = response.json().await.expect("Failed to parse response");
    assert_eq!(fetched.cup.id, cup.id);
    assert_eq!(fetched.roast_name, "Test Roast");
    assert_eq!(fetched.cafe_name, "Blue Bottle");
}

#[tokio::test]
async fn getting_a_nonexistent_cup_returns_a_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let response = client
        .get(app.api_url("/cups/999999"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn updating_a_cup_returns_a_200_for_valid_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: None,
        rating: None,
    };

    let create_response = client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to create cup");

    let cup: Cup = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let update = UpdateCup {
        notes: Some("Updated notes".to_string()),
        rating: Some(4),
    };

    let response = client
        .put(app.api_url(&format!("/cups/{}", cup.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let updated: Cup = response.json().await.expect("Failed to parse response");
    assert_eq!(updated.notes, Some("Updated notes".to_string()));
    assert_eq!(updated.rating, Some(4));
}

#[tokio::test]
async fn updating_a_cup_with_no_changes_returns_a_400() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: None,
        rating: None,
    };

    let create_response = client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to create cup");

    let cup: Cup = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let update = UpdateCup {
        notes: None,
        rating: None,
    };

    let response = client
        .put(app.api_url(&format!("/cups/{}", cup.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn deleting_a_cup_returns_a_204_for_valid_id() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        notes: None,
        rating: None,
    };

    let create_response = client
        .post(app.api_url("/cups"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_cup)
        .send()
        .await
        .expect("Failed to create cup");

    let cup: Cup = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let response = client
        .delete(app.api_url(&format!("/cups/{}", cup.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 204);

    // Verify cup was deleted
    let get_response = client
        .get(app.api_url(&format!("/cups/{}", cup.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn deleting_a_nonexistent_cup_returns_a_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let response = client
        .delete(app.api_url("/cups/999999"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);
}
