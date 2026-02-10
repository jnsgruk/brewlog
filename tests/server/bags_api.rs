use crate::helpers::{
    create_default_roast, create_default_roaster, spawn_app, spawn_app_with_auth,
};
use brewlog::domain::bags::{Bag, BagWithRoast, NewBag, UpdateBag};
use chrono::NaiveDate;

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
    assert_eq!(
        updated_bag.finished_at.unwrap(),
        chrono::Utc::now().date_naive()
    );
}
