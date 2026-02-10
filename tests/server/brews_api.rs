use crate::helpers::{
    create_default_bag, create_default_gear, create_default_roast, create_default_roaster,
    spawn_app, spawn_app_with_auth,
};
use brewlog::domain::bags::Bag;
use brewlog::domain::brews::{Brew, BrewWithDetails, NewBrew};

#[tokio::test]
async fn creating_a_brew_returns_201_for_valid_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
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

    // Act
    let response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 201);

    let brew: Brew = response.json().await.expect("Failed to parse response");
    assert_eq!(brew.bag_id, bag.id);
    assert_eq!(brew.coffee_weight, 15.0);
    assert_eq!(brew.grinder_id, grinder.id);
    assert_eq!(brew.grind_setting, 24.0);
    assert_eq!(brew.brewer_id, brewer.id);
    assert_eq!(brew.filter_paper_id, None);
    assert_eq!(brew.water_volume, 250);
    assert_eq!(brew.water_temp, 92.0);
}

#[tokio::test]
async fn creating_a_brew_with_filter_paper_returns_201() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let filter_paper = create_default_gear(&app, "filter_paper", "Hario", "V60 Tabbed 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
        bag_id: bag.id,
        coffee_weight: 15.0,
        grinder_id: grinder.id,
        grind_setting: 24.0,
        brewer_id: brewer.id,
        filter_paper_id: Some(filter_paper.id),
        water_volume: 250,
        water_temp: 92.0,
        quick_notes: Vec::new(),
        brew_time: None,
        created_at: None,
    };

    // Act
    let response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 201);

    let brew: Brew = response.json().await.expect("Failed to parse response");
    assert_eq!(brew.filter_paper_id, Some(filter_paper.id));
}

#[tokio::test]
async fn creating_a_brew_deducts_from_bag_remaining() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await; // 250g
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
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

    // Act
    client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - check bag remaining was deducted
    let bag_response = client
        .get(app.api_url(&format!("/bags/{}", bag.id)))
        .send()
        .await
        .expect("Failed to get bag");

    let updated_bag: Bag = bag_response.json().await.expect("Failed to parse bag");
    assert_eq!(updated_bag.remaining, 235.0); // 250 - 15 = 235
}

#[tokio::test]
async fn creating_a_brew_fails_if_insufficient_coffee_in_bag() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await; // 250g
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
        bag_id: bag.id,
        coffee_weight: 300.0, // More than the 250g in the bag
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

    // Act
    let response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 409); // Conflict
}

#[tokio::test]
async fn creating_a_brew_without_auth_returns_401() {
    // Arrange
    let app = spawn_app().await; // No auth
    let client = reqwest::Client::new();

    let new_brew = serde_json::json!({
        "bag_id": 1,
        "coffee_weight": 15.0,
        "grinder_id": 1,
        "grind_setting": 24.0,
        "brewer_id": 2,
        "water_volume": 250,
        "water_temp": 92.0
    });

    // Act
    let response = client
        .post(app.api_url("/brews"))
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn listing_brews_returns_200_and_enriched_data() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let filter_paper =
        create_default_gear(&app, "filter_paper", "Sibarist", "FAST Specialty 02").await;
    let client = reqwest::Client::new();

    // Create a brew with filter paper
    let new_brew = NewBrew {
        bag_id: bag.id,
        coffee_weight: 15.0,
        grinder_id: grinder.id,
        grind_setting: 24.0,
        brewer_id: brewer.id,
        filter_paper_id: Some(filter_paper.id),
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

    // Act
    let response = client
        .get(app.api_url("/brews"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let brews: Vec<BrewWithDetails> = response.json().await.expect("Failed to parse response");
    assert_eq!(brews.len(), 1);
    assert_eq!(brews[0].roast_name, roast.name);
    assert_eq!(brews[0].roaster_name, roaster.name);
    assert_eq!(brews[0].grinder_name, "Comandante C40 MK4");
    assert_eq!(brews[0].brewer_name, "Hario V60 02");
    assert_eq!(
        brews[0].filter_paper_name.as_deref(),
        Some("Sibarist FAST Specialty 02")
    );
}

#[tokio::test]
async fn listing_brews_without_filter_paper_returns_none() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "AeroPress", "Original").await;
    let client = reqwest::Client::new();

    // Create a brew without filter paper
    let new_brew = NewBrew {
        bag_id: bag.id,
        coffee_weight: 17.0,
        grinder_id: grinder.id,
        grind_setting: 20.0,
        brewer_id: brewer.id,
        filter_paper_id: None,
        water_volume: 255,
        water_temp: 88.0,
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

    // Act
    let response = client
        .get(app.api_url("/brews"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let brews: Vec<BrewWithDetails> = response.json().await.expect("Failed to parse response");
    assert_eq!(brews.len(), 1);
    assert_eq!(brews[0].filter_paper_name, None);
    assert_eq!(brews[0].brew.filter_paper_id, None);
}

#[tokio::test]
async fn listing_brews_with_bag_filter_returns_filtered_results() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag1 = create_default_bag(&app, roast.id).await;
    let bag2 = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    // Create brew for bag1
    let new_brew1 = NewBrew {
        bag_id: bag1.id,
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
        .json(&new_brew1)
        .send()
        .await
        .expect("Failed to create brew 1");

    // Create brew for bag2
    let new_brew2 = NewBrew {
        bag_id: bag2.id,
        coffee_weight: 17.0,
        grinder_id: grinder.id,
        grind_setting: 22.0,
        brewer_id: brewer.id,
        filter_paper_id: None,
        water_volume: 255,
        water_temp: 88.0,
        quick_notes: Vec::new(),
        brew_time: None,
        created_at: None,
    };

    client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew2)
        .send()
        .await
        .expect("Failed to create brew 2");

    // Act - filter by bag1
    let response = client
        .get(app.api_url(&format!("/brews?bag_id={}", bag1.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);

    let brews: Vec<BrewWithDetails> = response.json().await.expect("Failed to parse response");
    assert_eq!(brews.len(), 1);
    assert_eq!(brews[0].brew.coffee_weight, 15.0);
}

#[tokio::test]
async fn getting_a_brew_returns_200_for_valid_id() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
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

    let create_response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to create brew");

    let created_brew: Brew = create_response
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let response = client
        .get(app.api_url(&format!("/brews/{}", created_brew.id)))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 200);
    let brew: BrewWithDetails = response.json().await.expect("Failed to parse response");
    assert_eq!(brew.brew.id, created_brew.id);
    assert_eq!(brew.roast_name, roast.name);
}

#[tokio::test]
async fn deleting_a_brew_returns_204() {
    // Arrange
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
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

    let create_response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to create brew");

    let created_brew: Brew = create_response
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let response = client
        .delete(app.api_url(&format!("/brews/{}", created_brew.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), 204);

    // Verify deletion
    let get_response = client
        .get(app.api_url(&format!("/brews/{}", created_brew.id)))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn updating_a_brew_returns_200_and_updates_data() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
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

    let create_response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to create brew");

    let brew: Brew = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let update = serde_json::json!({
        "water_temp": 94.0,
        "grind_setting": 22.0,
    });

    let response = client
        .put(app.api_url(&format!("/brews/{}", brew.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to update brew");

    assert_eq!(response.status(), 200);
    let updated: Brew = response.json().await.expect("Failed to parse response");
    assert_eq!(updated.water_temp, 94.0);
    assert_eq!(updated.grind_setting, 22.0);
    assert_eq!(updated.coffee_weight, 15.0); // unchanged
}

#[tokio::test]
async fn updating_a_brew_without_auth_returns_401() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let update = serde_json::json!({
        "water_temp": 94.0,
    });

    let response = client
        .put(app.api_url("/brews/1"))
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn updating_a_brew_with_no_changes_returns_400() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60 02").await;
    let client = reqwest::Client::new();

    let new_brew = NewBrew {
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

    let create_response = client
        .post(app.api_url("/brews"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&new_brew)
        .send()
        .await
        .expect("Failed to create brew");

    let brew: Brew = create_response
        .json()
        .await
        .expect("Failed to parse response");

    let update = serde_json::json!({});

    let response = client
        .put(app.api_url(&format!("/brews/{}", brew.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to update brew");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn updating_a_nonexistent_brew_returns_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let update = serde_json::json!({
        "water_temp": 94.0,
    });

    let response = client
        .put(app.api_url("/brews/99999"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&update)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 404);
}
