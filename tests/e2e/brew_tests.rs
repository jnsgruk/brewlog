use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::forms::{
    fill_input, fill_textarea, select_option, select_searchable, submit_visible_form,
};
use crate::helpers::server_helpers::{
    create_default_bag, create_default_gear, create_default_roast, create_default_roaster,
    spawn_app_with_auth,
};
use crate::helpers::wait::{wait_for_url_contains, wait_for_visible};

#[tokio::test]
async fn create_brew_with_api_prerequisites() {
    let app = spawn_app_with_auth().await;

    // Create prerequisites via API (fast, no browser needed)
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let _bag = create_default_bag(&app, roast.id).await;
    let _grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let _brewer = create_default_gear(&app, "brewer", "Hario", "V60").await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Navigate to add page with brew tab
    session.goto("/add?type=brew").await.unwrap();

    // Wait for the brew form's bag select to appear
    wait_for_visible(&session.driver, "searchable-select[name='bag_id']")
        .await
        .unwrap();

    // Select the bag (displays the roast name "Test Roast")
    select_searchable(&session.driver, "bag_id", "Test Roast")
        .await
        .unwrap();

    // The grinder and brewer are pre-selected (only one option each).
    // Default values for weight, temp, volume, time are pre-filled via signals.
    // Submit the form.
    submit_visible_form(&session.driver).await.unwrap();

    // Should redirect to the brew detail page
    wait_for_url_contains(&session.driver, "/brews/")
        .await
        .unwrap();

    // Verify the brew detail page loaded
    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("Test Roast"),
        "Brew detail should show the roast name"
    );

    session.quit().await;
}

#[tokio::test]
async fn create_full_chain_via_browser() {
    let app = spawn_app_with_auth().await;
    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Step 1: Create roaster
    session.goto("/add?type=roaster").await.unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();
    fill_input(&session.driver, "name", "Chain Roasters")
        .await
        .unwrap();
    fill_input(&session.driver, "country", "Ethiopia")
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/roasters/")
        .await
        .unwrap();

    // Step 2: Create grinder
    session.goto("/add?type=gear").await.unwrap();
    wait_for_visible(&session.driver, "select[name='category']")
        .await
        .unwrap();
    select_option(&session.driver, "category", "grinder")
        .await
        .unwrap();
    fill_input(&session.driver, "make", "Comandante")
        .await
        .unwrap();
    fill_input(&session.driver, "model", "C40").await.unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/gear/")
        .await
        .unwrap();

    // Step 3: Create brewer
    session.goto("/add?type=gear").await.unwrap();
    wait_for_visible(&session.driver, "select[name='category']")
        .await
        .unwrap();
    select_option(&session.driver, "category", "brewer")
        .await
        .unwrap();
    fill_input(&session.driver, "make", "Hario").await.unwrap();
    fill_input(&session.driver, "model", "V60").await.unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/gear/")
        .await
        .unwrap();

    // Step 4: Create roast
    session.goto("/add?type=roast").await.unwrap();
    wait_for_visible(&session.driver, "searchable-select[name='roaster_id']")
        .await
        .unwrap();
    select_searchable(&session.driver, "roaster_id", "Chain")
        .await
        .unwrap();
    fill_input(&session.driver, "name", "Chain Blend")
        .await
        .unwrap();
    fill_input(&session.driver, "origin", "Ethiopia")
        .await
        .unwrap();
    fill_input(&session.driver, "region", "Guji").await.unwrap();
    fill_input(&session.driver, "producer", "Coop")
        .await
        .unwrap();
    fill_input(&session.driver, "process", "Washed")
        .await
        .unwrap();
    fill_textarea(&session.driver, "tasting_notes", "Blueberry, Jasmine")
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    // Roast creation redirects to the roaster detail page
    wait_for_url_contains(&session.driver, "/roasters/")
        .await
        .unwrap();

    // Step 5: Create bag
    session.goto("/add?type=bag").await.unwrap();
    wait_for_visible(&session.driver, "searchable-select[name='roast_id']")
        .await
        .unwrap();
    select_searchable(&session.driver, "roast_id", "Chain Blend")
        .await
        .unwrap();
    fill_input(&session.driver, "amount", "250").await.unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/bags/")
        .await
        .unwrap();

    // Step 6: Create brew
    session.goto("/add?type=brew").await.unwrap();
    wait_for_visible(&session.driver, "searchable-select[name='bag_id']")
        .await
        .unwrap();
    select_searchable(&session.driver, "bag_id", "Chain Blend")
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_contains(&session.driver, "/brews/")
        .await
        .unwrap();

    // Verify the brew detail page shows the roast name
    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("Chain Blend"),
        "Brew detail should show the roast name"
    );

    session.quit().await;
}

#[tokio::test]
async fn edit_brew_can_clear_all_quick_notes_and_optional_fields() {
    use crate::helpers::server_helpers::create_default_brew;
    use crate::helpers::wait::wait_for_url_not_contains;
    use brewlog::domain::brews::{Brew, QuickNote};

    let app = spawn_app_with_auth().await;
    let brew = create_default_brew(&app).await;
    let filter = create_default_gear(&app, "filter_paper", "Hario", "V60").await;
    let client = reqwest::Client::new();
    let url = app.api_url(&format!("/brews/{}", brew.id));
    let response = client
        .put(&url)
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({"quick_notes": ["good", "too-fast"], "filter_paper_id": filter.id, "brew_time": 150}))
        .send().await.unwrap();
    assert_eq!(response.status(), 200);

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();
    session
        .goto(&format!("/brews/{}/edit", brew.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "select[name='filter_paper_id']")
        .await
        .unwrap();
    for signal in ["_qnGood", "_qnTooFast"] {
        session
            .driver
            .find(By::Css(format!(
                "button[data-on\\:click='${signal} = !${signal}']"
            )))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
    }
    select_option(&session.driver, "filter_paper_id", "")
        .await
        .unwrap();
    session
        .driver
        .find(By::Css("button[aria-label='Clear brew time']"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    assert_eq!(
        session
            .driver
            .find(By::Css("input[name='brew_time']"))
            .await
            .unwrap()
            .prop("value")
            .await
            .unwrap(),
        Some(String::new()),
        "Clearing time must submit an empty hidden field"
    );
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();
    let stored: Brew = client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(stored.quick_notes, Vec::<QuickNote>::new());
    assert_eq!(stored.filter_paper_id, None);
    assert_eq!(stored.brew_time, None);

    // A subsequent edit must preserve absent time rather than replace it with zero.
    session
        .goto(&format!("/brews/{}/edit", brew.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "select[name='filter_paper_id']")
        .await
        .unwrap();
    submit_visible_form(&session.driver).await.unwrap();
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();
    let stored: Brew = client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(stored.brew_time, None);
    session.quit().await;
}
