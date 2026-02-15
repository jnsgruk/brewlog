use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::forms::{fill_input, submit_visible_form};
use crate::helpers::server_helpers::{
    create_default_roast, create_default_roaster, spawn_app_with_timeline_sync,
};
use crate::helpers::wait::{wait_for_url_not_contains, wait_for_visible};

#[tokio::test]
async fn editing_roaster_updates_timeline_events_in_browser() {
    let app = spawn_app_with_timeline_sync().await;
    let roaster = create_default_roaster(&app).await;
    let _roast = create_default_roast(&app, roaster.id).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Verify original roaster name appears in the timeline
    session.goto("/timeline").await.unwrap();
    wait_for_visible(&session.driver, "#timeline-events")
        .await
        .unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();
    assert!(
        text.contains("Test Roasters"),
        "Expected original roaster name 'Test Roasters' in timeline, got: {text}"
    );

    // Navigate to the roaster edit page and change the name
    session
        .goto(&format!("/roasters/{}/edit", roaster.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    fill_input(&session.driver, "name", "E2E Renamed Roasters")
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();

    // Wait for redirect back to the detail page
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    // Wait for the background timeline rebuild task to process
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Go back to the timeline and verify both events reflect the new name
    session.goto("/timeline").await.unwrap();
    wait_for_visible(&session.driver, "#timeline-events")
        .await
        .unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let text = body.text().await.unwrap();

    assert!(
        text.contains("E2E Renamed Roasters"),
        "Expected updated roaster name 'E2E Renamed Roasters' in timeline after edit, got: {text}"
    );
    assert!(
        !text.contains("Test Roasters"),
        "Expected original name 'Test Roasters' to no longer appear in timeline, got: {text}"
    );

    session.quit().await;
}
