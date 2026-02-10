use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::server_helpers::{
    create_default_bag, create_default_cafe, create_default_gear, create_default_roast,
    create_default_roaster, create_roaster_with_payload, spawn_app_with_auth,
};
use crate::helpers::wait::wait_for_visible;

use brewlog::domain::coffee::roasters::NewRoaster;

#[tokio::test]
async fn roaster_detail_shows_all_fields() {
    let app = spawn_app_with_auth().await;
    let roaster = create_roaster_with_payload(
        &app,
        NewRoaster {
            name: "Detail Roasters".to_string(),
            country: "JP".to_string(),
            city: Some("Tokyo".to_string()),
            homepage: Some("https://example.com".to_string()),
            created_at: None,
        },
    )
    .await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/roasters/{}", roaster.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(text.contains("Detail Roasters"), "Should show roaster name");
    assert!(text.contains("JP"), "Should show country code");
    assert!(text.contains("Tokyo"), "Should show city");
    assert!(text.contains("Visit Website"), "Should show homepage link");

    session.quit().await;
}

#[tokio::test]
async fn roast_detail_shows_coffee_info() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/roasters/{}/roasts/{}", roaster.slug, roast.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(text.contains("Test Roast"), "Should show roast name");
    assert!(text.contains("Ethiopia"), "Should show origin");
    assert!(text.contains("Yirgacheffe"), "Should show region");
    assert!(text.contains("Washed"), "Should show process");
    assert!(text.contains("Blueberry"), "Should show tasting notes");

    session.quit().await;
}

#[tokio::test]
async fn bag_detail_shows_status_and_close_button() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto(&format!("/bags/{}", bag.id)).await.unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(text.contains("Test Roast"), "Should show roast name");
    assert!(text.contains("250"), "Should show bag amount");

    // Close Bag button should be visible (bag is open)
    assert!(text.contains("Close Bag"), "Should show Close Bag button");

    session.quit().await;
}

#[tokio::test]
async fn gear_detail_shows_fields() {
    let app = spawn_app_with_auth().await;
    let gear = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto(&format!("/gear/{}", gear.id)).await.unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(text.contains("Comandante"), "Should show make");
    assert!(text.contains("C40 MK4"), "Should show model");

    session.quit().await;
}

#[tokio::test]
async fn cafe_detail_shows_location() {
    let app = spawn_app_with_auth().await;
    let cafe = create_default_cafe(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/cafes/{}", cafe.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(text.contains("Blue Bottle"), "Should show cafe name");
    assert!(text.contains("San Francisco"), "Should show city");

    session.quit().await;
}
