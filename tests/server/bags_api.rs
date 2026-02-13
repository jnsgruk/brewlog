use crate::helpers::{
    create_default_bag, create_default_gear, create_default_roast, create_default_roaster,
    spawn_app, spawn_app_with_auth,
};
use crate::test_macros::define_crud_tests;
use brewlog::domain::bags::{Bag, BagWithRoast, NewBag, UpdateBag};
use chrono::NaiveDate;

define_crud_tests!(
    entity: bag,
    path: "/bags",
    list_type: BagWithRoast
);

#[tokio::test]
async fn creating_a_bag_returns_a_201_for_valid_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let new_bag = NewBag {
        roast_id: roast.id,
        roast_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        amount: 250.0,
        created_at: None,
    };

    // Act
    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 201);

    let bag: Bag = response.json().await.expect("Failed to parse response");
    assert_eq!(bag.roast_id, roast.id);
    assert_eq!(bag.amount, 250.0);
    assert_eq!(bag.remaining, 250.0);
    assert_eq!(
        bag.roast_date,
        Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
    );
}

#[tokio::test]
async fn creating_a_bag_without_auth_returns_401() {
    // Arrange
    let app = spawn_app().await; // No auth
    let client = reqwest::Client::new();

    // We can't easily create a roast without auth, so we'll just try to create a bag
    // with a dummy ID. The auth check should happen before validation.
    let new_bag = serde_json::json!({
        "roast_id": 123,
        "amount": 250.0
    });

    // Act
    let response = client
        .post(app.api_url("/bags"))
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn listing_bags_returns_200_and_correct_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    // Create a bag
    let new_bag = NewBag {
        roast_id: roast.id,
        roast_date: None,
        amount: 500.0,
        created_at: None,
    };

    client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to create bag");

    // Act
    let response = client
        .get(app.api_url(&format!("/bags?roast_id={}", roast.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let bags: Vec<BagWithRoast> = response.json().await.expect("Failed to parse response");
    assert_eq!(bags.len(), 1);
    assert_eq!(bags[0].bag.amount, 500.0);
    assert_eq!(bags[0].roast_name, roast.name);
}

#[tokio::test]
async fn getting_a_bag_returns_200_for_valid_id() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let new_bag = NewBag {
        roast_id: roast.id,
        roast_date: None,
        amount: 250.0,
        created_at: None,
    };

    let create_response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to create bag");

    let created_bag: Bag = create_response
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let response = client
        .get(app.api_url(&format!("/bags/{}", created_bag.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);
    let bag: Bag = response.json().await.expect("Failed to parse response");
    assert_eq!(bag.id, created_bag.id);
}

#[tokio::test]
async fn updating_a_bag_returns_200_and_updates_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let new_bag = NewBag {
        roast_id: roast.id,
        roast_date: None,
        amount: 250.0,
        created_at: None,
    };

    let create_response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to create bag");

    let created_bag: Bag = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let update_payload = UpdateBag {
        remaining: Some(100.0),
        closed: Some(true),
        finished_at: Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
        ..Default::default()
    };

    // Act
    let response = client
        .put(app.api_url(&format!("/bags/{}", created_bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);
    let updated_bag: Bag = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_bag.remaining, 100.0);
    assert!(updated_bag.closed);
    assert_eq!(
        updated_bag.finished_at,
        Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap())
    );
}

#[tokio::test]
async fn deleting_a_bag_returns_204() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let new_bag = NewBag {
        roast_id: roast.id,
        roast_date: None,
        amount: 250.0,
        created_at: None,
    };

    let create_response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to create bag");

    let created_bag: Bag = create_response
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let response = client
        .delete(app.api_url(&format!("/bags/{}", created_bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 204);

    // Verify deletion
    let get_response = client
        .get(app.api_url(&format!("/bags/{}", created_bag.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn closing_a_bag_automatically_sets_finished_at() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let new_bag = NewBag {
        roast_id: roast.id,
        roast_date: None,
        amount: 250.0,
        created_at: None,
    };

    let create_response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_bag)
        .send()
        .await
        .expect("Failed to create bag");

    let created_bag: Bag = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let update_payload = serde_json::json!({
        "closed": true
    });

    // Act
    let response = client
        .put(app.api_url(&format!("/bags/{}", created_bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);
    let updated_bag: Bag = response.json().await.expect("Failed to parse response");
    assert!(updated_bag.closed);
    assert!(updated_bag.finished_at.is_some());
    // Accept today or yesterday to avoid midnight-boundary flakiness
    let today = chrono::Utc::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let finished = updated_bag.finished_at.unwrap();
    assert!(
        finished == today || finished == yesterday,
        "expected finished_at to be today or yesterday, got {finished}"
    );
}

#[tokio::test]
async fn updating_bag_amount_recomputes_remaining() {
    // Arrange: create a bag and brew from it to consume some coffee
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    // Brew 15g from the 250g bag → remaining should be 235g
    let new_brew = brewlog::domain::brews::NewBrew {
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

    client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to create brew");

    // Act: update the bag amount from 250g to 500g
    let update_payload = UpdateBag {
        amount: Some(500.0),
        ..Default::default()
    };

    let response = client
        .put(app.api_url(&format!("/bags/{}", bag.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert: remaining should be 500 - 15 = 485 (recomputed from consumed)
    assert_eq!(response.status(), 200);
    let updated_bag: Bag = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_bag.amount, 500.0);
    assert_eq!(updated_bag.remaining, 485.0);
}

#[tokio::test]
async fn creating_bag_with_zero_amount_returns_400() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({
            "roast_id": roast.id,
            "amount": 0.0
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn creating_bag_with_negative_amount_returns_400() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({
            "roast_id": roast.id,
            "amount": -100.0
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn creating_bag_with_invalid_date_returns_400() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let client = reqwest::Client::new();

    let response = client
        .post(app.api_url("/bags"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({
            "roast_id": roast.id,
            "roast_date": "not-a-date",
            "amount": 250.0
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}
