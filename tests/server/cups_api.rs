use crate::helpers::{
    create_cafe_with_payload, create_default_cafe, create_default_roast, create_default_roaster,
    spawn_app_with_auth,
};
use crate::test_macros::define_crud_tests;
use brewlog::domain::cups::{Cup, CupWithDetails, NewCup};
use brewlog::domain::ids::{CafeId, RoastId};

define_crud_tests!(
    entity: cup,
    path: "/cups",
    list_type: CupWithDetails
);

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
        created_at: None,
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
}

#[tokio::test]
async fn creating_a_cup_requires_authentication() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let new_cup = NewCup {
        roast_id: RoastId::new(1),
        cafe_id: CafeId::new(1),
        created_at: None,
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
async fn listing_cups_returns_a_200_with_enriched_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        created_at: None,
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
        created_at: None,
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
async fn deleting_a_cup_returns_a_204_for_valid_id() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        created_at: None,
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
async fn updating_a_cup_returns_a_200_for_valid_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe1 = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe1.id,
        created_at: None,
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

    // Create a second cafe to update to
    let cafe2 = create_cafe_with_payload(
        &app,
        brewlog::domain::cafes::NewCafe {
            name: "Updated Cafe".to_string(),
            city: "Tokyo".to_string(),
            country: "JP".to_string(),
            latitude: 35.6762,
            longitude: 139.6503,
            website: None,
            created_at: None,
        },
    )
    .await;

    let update = serde_json::json!({
        "cafe_id": cafe2.id,
    });

    let response = client
        .put(app.api_url(&format!("/cups/{}", cup.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to update cup");

    assert_eq!(response.status(), 200);
    let updated: Cup = response.json().await.expect("Failed to parse response");
    assert_eq!(updated.cafe_id, cafe2.id);
    assert_eq!(updated.roast_id, roast.id); // unchanged
}

#[tokio::test]
async fn updating_a_cup_without_auth_returns_401() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let update = serde_json::json!({
        "cafe_id": 2,
    });

    let response = client
        .put(app.api_url("/cups/1"))
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn updating_a_cup_with_no_changes_returns_400() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let new_cup = NewCup {
        roast_id: roast.id,
        cafe_id: cafe.id,
        created_at: None,
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

    let update = serde_json::json!({});

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
async fn updating_a_nonexistent_cup_returns_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let update = serde_json::json!({
        "cafe_id": 1,
    });

    let response = client
        .put(app.api_url("/cups/99999"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);
}
