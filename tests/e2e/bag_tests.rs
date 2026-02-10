use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::server_helpers::{
    create_default_bag, create_default_roast, create_default_roaster, spawn_app_with_auth,
};
use crate::helpers::wait::wait_for_visible;

#[tokio::test]
async fn close_bag_from_detail_page() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session.goto(&format!("/bags/{}", bag.id)).await.unwrap();

    // Find and click the "Close Bag" button
    wait_for_visible(&session.driver, "button").await.unwrap();
    let buttons = session.driver.find_all(By::Css("button")).await.unwrap();
    for button in buttons {
        let text = button.text().await.unwrap_or_default();
        if text.contains("Close Bag") {
            button.click().await.unwrap();
            break;
        }
    }

    // Accept the confirm() dialog
    session.driver.accept_alert().await.unwrap();

    // After closing, the "Close Bag" button should disappear.
    // Wait for the page to update — the @put returns a redirect or refreshes.
    // The button text "Close Bag" should no longer be present.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(
        !text.contains("Close Bag"),
        "Close Bag button should be gone after closing"
    );

    // Bag should still show on the page (just marked as closed)
    assert!(text.contains("Test Roast"), "Should still show roast name");

    session.quit().await;
}
