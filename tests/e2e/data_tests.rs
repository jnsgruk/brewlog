use std::time::Duration;

use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::server_helpers::{
    create_default_bag, create_default_gear, create_default_roast, create_default_roaster,
    create_roaster_with_name, spawn_app_with_auth,
};
use crate::helpers::wait::{wait_for_text, wait_for_visible};

#[tokio::test]
async fn tab_switching_loads_correct_entity_list() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let _bag = create_default_bag(&app, roast.id).await;
    let _grinder = create_default_gear(&app, "grinder", "Timemore", "C3").await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Default tab is brews — but we have no brews, so check empty state
    session.goto("/data").await.unwrap();
    wait_for_visible(&session.driver, "#data-content")
        .await
        .unwrap();

    // Click "Roasters" tab (desktop)
    let tabs = session
        .driver
        .find_all(By::Css("nav[role='tablist'] button[role='tab']"))
        .await
        .unwrap();
    for tab in &tabs {
        let text = tab.text().await.unwrap_or_default();
        if text.contains("Roasters") {
            tab.click().await.unwrap();
            break;
        }
    }

    // Wait for roaster list to load
    wait_for_text(&session.driver, "#data-content", "Test Roasters")
        .await
        .unwrap();

    // Click "Gear" tab
    let tabs = session
        .driver
        .find_all(By::Css("nav[role='tablist'] button[role='tab']"))
        .await
        .unwrap();
    for tab in &tabs {
        let text = tab.text().await.unwrap_or_default();
        if text.contains("Gear") {
            tab.click().await.unwrap();
            break;
        }
    }

    wait_for_text(&session.driver, "#data-content", "Timemore")
        .await
        .unwrap();

    session.quit().await;
}

#[tokio::test]
async fn search_filters_list() {
    let app = spawn_app_with_auth().await;
    create_roaster_with_name(&app, "Acme Coffee").await;
    create_roaster_with_name(&app, "Beta Beans").await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/data?type=roasters").await.unwrap();
    wait_for_visible(&session.driver, "#roaster-list")
        .await
        .unwrap();

    // Both roasters should be visible initially
    let body = session.driver.find(By::Css("#data-content")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(text.contains("Acme Coffee"), "Should show Acme Coffee");
    assert!(text.contains("Beta Beans"), "Should show Beta Beans");

    // Type in search field — the data page search is on the outer page (input[type='search'])
    let search = session
        .driver
        .find(By::Css("input[type='search']"))
        .await
        .unwrap();
    search.send_keys("Acme").await.unwrap();

    // Wait for debounce (300ms) + fragment load
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Wait for the list to update with filtered results
    wait_for_text(&session.driver, "#data-content", "Acme Coffee")
        .await
        .unwrap();

    // Beta Beans should be filtered out
    let text = session
        .driver
        .find(By::Css("#data-content"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        !text.contains("Beta Beans"),
        "Beta Beans should be filtered out"
    );

    session.quit().await;
}

#[tokio::test]
async fn pagination_next_and_prev() {
    let app = spawn_app_with_auth().await;

    // Create 11 roasters — default page size is 10
    for i in 0..11 {
        create_roaster_with_name(&app, &format!("Roaster {i:02}")).await;
    }

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/data?type=roasters").await.unwrap();
    wait_for_visible(&session.driver, ".pagination-controls")
        .await
        .unwrap();

    // Should show "Page 1 of 2"
    let pagination = session
        .driver
        .find(By::Css(".pagination-controls"))
        .await
        .unwrap();
    let pagination_text = pagination.text().await.unwrap();
    assert!(
        pagination_text.contains("Page 1 of 2"),
        "Should show page 1 of 2, got: {pagination_text}"
    );

    // Click Next
    let next_btn = pagination
        .find(By::XPath(".//button[contains(., 'Next')]"))
        .await
        .unwrap();
    next_btn.click().await.unwrap();

    // Wait for page 2
    tokio::time::sleep(Duration::from_millis(300)).await;
    let pagination = session
        .driver
        .find(By::Css(".pagination-controls"))
        .await
        .unwrap();
    wait_for_text(&session.driver, ".pagination-controls", "Page 2 of 2")
        .await
        .unwrap();

    // Click Prev
    let prev_btn = pagination
        .find(By::XPath(".//button[contains(., 'Prev')]"))
        .await
        .unwrap();
    prev_btn.click().await.unwrap();

    wait_for_text(&session.driver, ".pagination-controls", "Page 1 of 2")
        .await
        .unwrap();

    session.quit().await;
}

#[tokio::test]
async fn sort_by_column_reverses_order() {
    let app = spawn_app_with_auth().await;
    create_roaster_with_name(&app, "Alpha Roasters").await;
    create_roaster_with_name(&app, "Zulu Coffee").await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto("/data?type=roasters").await.unwrap();
    wait_for_visible(&session.driver, "#roaster-list")
        .await
        .unwrap();

    // Click "Name" sort header to sort ascending
    let name_header = session
        .driver
        .find(By::XPath("//thead//button[contains(., 'Name')]"))
        .await
        .unwrap();
    name_header.click().await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Get row order after first click (ascending)
    let rows = session
        .driver
        .find_all(By::Css("#roaster-list tbody tr"))
        .await
        .unwrap();
    let mut first_order = Vec::new();
    for row in &rows {
        first_order.push(row.text().await.unwrap_or_default());
    }

    // Click again to reverse
    let name_header = session
        .driver
        .find(By::XPath("//thead//button[contains(., 'Name')]"))
        .await
        .unwrap();
    name_header.click().await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    let rows = session
        .driver
        .find_all(By::Css("#roaster-list tbody tr"))
        .await
        .unwrap();
    let mut second_order = Vec::new();
    for row in &rows {
        second_order.push(row.text().await.unwrap_or_default());
    }

    // Orders should be different (reversed)
    assert_ne!(
        first_order, second_order,
        "Clicking sort header twice should reverse order"
    );

    session.quit().await;
}
