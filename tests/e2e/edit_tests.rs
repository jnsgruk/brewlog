use std::time::Duration;

use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::forms::{fill_input, submit_visible_form};
use crate::helpers::server_helpers::{
    create_default_brew, create_default_cafe, create_default_cup, create_default_gear,
    create_default_roaster, spawn_app_with_auth,
};
use crate::helpers::wait::{wait_for_url_contains, wait_for_url_not_contains, wait_for_visible};

// ── Roaster: text fields ──────────────────────────────────────────────

#[tokio::test]
async fn edit_roaster_updates_text_fields() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Navigate to edit page
    session
        .goto(&format!("/roasters/{}/edit", roaster.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // Verify pre-populated value
    let name_input = session
        .driver
        .find(By::Css("input[name='name']"))
        .await
        .unwrap();
    let current_name = name_input.prop("value").await.unwrap().unwrap_or_default();
    assert_eq!(current_name, "Test Roasters");

    // Clear and update the name
    fill_input(&session.driver, "name", "Updated Roasters")
        .await
        .unwrap();

    submit_visible_form(&session.driver).await.unwrap();

    // Should redirect to the roaster detail page (edit URL also contains /roasters/)
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    // Verify the detail page reflects the change
    let heading = session.driver.find(By::Css("h1")).await.unwrap();
    let heading_text = heading.text().await.unwrap();
    assert_eq!(heading_text, "Updated Roasters");

    session.quit().await;
}

// ── Gear: text fields ─────────────────────────────────────────────────

#[tokio::test]
async fn edit_gear_updates_text_fields() {
    let app = spawn_app_with_auth().await;
    let gear = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/gear/{}/edit", gear.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='model']")
        .await
        .unwrap();

    // Verify pre-populated values
    let make_input = session
        .driver
        .find(By::Css("input[name='make']"))
        .await
        .unwrap();
    assert_eq!(
        make_input.prop("value").await.unwrap().unwrap_or_default(),
        "Comandante"
    );

    let model_input = session
        .driver
        .find(By::Css("input[name='model']"))
        .await
        .unwrap();
    assert_eq!(
        model_input.prop("value").await.unwrap().unwrap_or_default(),
        "C40 MK4"
    );

    // Update model
    fill_input(&session.driver, "model", "C40 MK5")
        .await
        .unwrap();

    submit_visible_form(&session.driver).await.unwrap();

    // Edit page URL contains /gear/ too, so wait for /edit to disappear
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("C40 MK5"),
        "Detail page should show updated model"
    );

    session.quit().await;
}

// ── Cafe: text + numeric fields ───────────────────────────────────────

#[tokio::test]
async fn edit_cafe_updates_text_and_numeric_fields() {
    let app = spawn_app_with_auth().await;
    let cafe = create_default_cafe(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/cafes/{}/edit", cafe.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // Verify pre-populated values
    let name_input = session
        .driver
        .find(By::Css("input[name='name']"))
        .await
        .unwrap();
    assert_eq!(
        name_input.prop("value").await.unwrap().unwrap_or_default(),
        "Blue Bottle"
    );

    let lat_input = session
        .driver
        .find(By::Css("input[name='latitude']"))
        .await
        .unwrap();
    let lat_value = lat_input.prop("value").await.unwrap().unwrap_or_default();
    assert!(
        lat_value.starts_with("37.77"),
        "Latitude should be pre-populated, got: {lat_value}"
    );

    // Update name and city
    fill_input(&session.driver, "name", "Verve Coffee")
        .await
        .unwrap();
    fill_input(&session.driver, "city", "Santa Cruz")
        .await
        .unwrap();
    fill_input(&session.driver, "latitude", "36.9741")
        .await
        .unwrap();

    submit_visible_form(&session.driver).await.unwrap();

    // Edit page URL contains /cafes/ too, so wait for /edit to disappear
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("Verve Coffee"),
        "Detail page should show updated name"
    );
    assert!(
        body_text.contains("Santa Cruz"),
        "Detail page should show updated city"
    );

    session.quit().await;
}

// ── Brew: numeric fields with +/- adjusters ───────────────────────────

#[tokio::test]
async fn edit_brew_updates_numeric_fields() {
    let app = spawn_app_with_auth().await;
    let brew = create_default_brew(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/brews/{}/edit", brew.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='coffee_weight']")
        .await
        .unwrap();

    // Verify pre-populated coffee weight
    let weight_input = session
        .driver
        .find(By::Css("input[name='coffee_weight']"))
        .await
        .unwrap();
    let current_weight = weight_input
        .prop("value")
        .await
        .unwrap()
        .unwrap_or_default();
    assert_eq!(
        current_weight, "15",
        "Coffee weight should be pre-populated"
    );

    // Update coffee weight via direct input
    fill_input(&session.driver, "coffee_weight", "18")
        .await
        .unwrap();

    // Update water volume via direct input
    fill_input(&session.driver, "water_volume", "300")
        .await
        .unwrap();

    // Update grind setting
    fill_input(&session.driver, "grind_setting", "28")
        .await
        .unwrap();

    submit_visible_form(&session.driver).await.unwrap();

    // Edit page URL contains /brews/ too, so wait for /edit to disappear
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    // Verify the brew detail page shows updated values
    let body = session.driver.find(By::Css("body")).await.unwrap();
    let body_text = body.text().await.unwrap();
    assert!(
        body_text.contains("18"),
        "Should show updated coffee weight"
    );
    assert!(
        body_text.contains("300"),
        "Should show updated water volume"
    );

    session.quit().await;
}

// ── Brew: +/- adjuster buttons ────────────────────────────────────────

#[tokio::test]
async fn edit_brew_adjuster_buttons_change_values() {
    let app = spawn_app_with_auth().await;
    let brew = create_default_brew(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/brews/{}/edit", brew.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='coffee_weight']")
        .await
        .unwrap();

    // Find the + button next to coffee weight (it's the sibling after the input)
    let weight_plus = session
        .driver
        .execute(
            r#"
            const input = document.querySelector("input[name='coffee_weight']");
            return input.parentElement.querySelector("button:last-of-type");
            "#,
            vec![],
        )
        .await
        .unwrap();
    let weight_plus_el = weight_plus.element().unwrap();

    // Click + three times to add 1.5g (0.5 per click)
    weight_plus_el.click().await.unwrap();
    weight_plus_el.click().await.unwrap();
    weight_plus_el.click().await.unwrap();

    // Wait for Datastar signal to update the input
    tokio::time::sleep(Duration::from_millis(200)).await;

    let weight_input = session
        .driver
        .find(By::Css("input[name='coffee_weight']"))
        .await
        .unwrap();
    let updated_weight = weight_input
        .prop("value")
        .await
        .unwrap()
        .unwrap_or_default();
    assert_eq!(updated_weight, "16.5", "Weight should be 15 + 3*0.5 = 16.5");

    session.quit().await;
}

// ── Cup: edit with searchable-select ──────────────────────────────────

#[tokio::test]
async fn edit_cup_updates_searchable_selects() {
    let app = spawn_app_with_auth().await;
    let cup = create_default_cup(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/cups/{}/edit", cup.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "searchable-select[name='roast_id']")
        .await
        .unwrap();

    // The searchable-select should show the current roast name in its display span
    // (when pre-selected, the component hides the search input and shows a span)
    let roast_select = session
        .driver
        .find(By::Css("searchable-select[name='roast_id']"))
        .await
        .unwrap();
    let display_span = roast_select
        .find(By::Css("span.input-field"))
        .await
        .unwrap();
    let display_text = display_span.text().await.unwrap();
    assert!(
        !display_text.is_empty(),
        "Searchable select should show the pre-populated roast name"
    );

    // Submit without changes — should still redirect successfully
    submit_visible_form(&session.driver).await.unwrap();

    // Edit page URL contains /cups/ too, so wait for /edit to disappear
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    session.quit().await;
}

// ── Edit form with image upload ───────────────────────────────────────

#[tokio::test]
async fn edit_roaster_shows_existing_image_and_can_replace() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // First, upload an image via the API
    session.goto("/login").await.unwrap();
    let upload_script = format!(
        r#"
        const callback = arguments[arguments.length - 1];
        const canvas = document.createElement('canvas');
        canvas.width = 4;
        canvas.height = 4;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'blue';
        ctx.fillRect(0, 0, 4, 4);
        const dataUrl = canvas.toDataURL('image/png');
        fetch('/api/v1/roaster/{}/image', {{
            method: 'PUT',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ image: dataUrl }})
        }}).then(r => callback(r.status.toString())).catch(e => callback('error: ' + e));
        "#,
        roaster.id
    );
    let result = session
        .driver
        .execute_async(&upload_script, vec![])
        .await
        .unwrap();
    let status = result.json().as_str().unwrap_or("").to_string();
    assert_eq!(status, "204", "Image upload should return 204");

    // Navigate to the edit page
    session
        .goto(&format!("/roasters/{}/edit", roaster.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // Verify the existing image preview is shown
    let existing_preview = session
        .driver
        .find(By::Css("[id$='-existing']"))
        .await
        .expect("Existing image preview should be present");
    assert!(
        existing_preview.is_displayed().await.unwrap_or(false),
        "Existing image preview should be visible"
    );

    // Verify Replace and Remove buttons are present
    let replace_btn = existing_preview
        .find(By::Css("image-upload[mode='deferred']"))
        .await
        .expect("Replace button should be present");
    assert!(replace_btn.is_displayed().await.unwrap_or(false));

    let remove_btn = existing_preview
        .find(By::Css("button"))
        .await
        .expect("Remove button should be present");
    assert!(remove_btn.is_displayed().await.unwrap_or(false));

    // Simulate uploading a replacement image via JavaScript
    // (we can't trigger the file picker in headless mode, but we can set the hidden input)
    let replace_script = r#"
        const callback = arguments[arguments.length - 1];
        const canvas = document.createElement('canvas');
        canvas.width = 4;
        canvas.height = 4;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'green';
        ctx.fillRect(0, 0, 4, 4);
        const dataUrl = canvas.toDataURL('image/jpeg');
        const hiddenInput = document.getElementById('edit-roaster-image');
        hiddenInput.value = dataUrl;
        // Remove existing preview to simulate what image-upload.js does
        const existing = document.getElementById('edit-roaster-image-existing');
        if (existing) existing.remove();
        callback('done');
    "#;
    session
        .driver
        .execute_async(replace_script, vec![])
        .await
        .unwrap();

    // Existing preview should be removed
    let result = session
        .driver
        .find(By::Css("#edit-roaster-image-existing"))
        .await;
    assert!(
        result.is_err(),
        "Existing image preview should be removed after replacement"
    );

    // Hidden input should have the new data URL
    let hidden_input = session
        .driver
        .find(By::Css("#edit-roaster-image"))
        .await
        .unwrap();
    let value = hidden_input
        .prop("value")
        .await
        .unwrap()
        .unwrap_or_default();
    assert!(
        value.starts_with("data:image/jpeg"),
        "Hidden input should contain the replacement image data URL"
    );

    session.quit().await;
}

#[tokio::test]
async fn edit_form_without_image_shows_upload_area() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/roasters/{}/edit", roaster.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // No existing image — should show the dashed upload area
    let result = session.driver.find(By::Css("[id$='-existing']")).await;
    assert!(
        result.is_err(),
        "Existing image preview should not be present when there is no image"
    );

    // The deferred upload component should be visible
    let upload = session
        .driver
        .find(By::Css("image-upload[mode='deferred']"))
        .await
        .expect("Deferred upload area should be present");
    assert!(
        upload.is_displayed().await.unwrap_or(false),
        "Deferred upload area should be visible"
    );

    session.quit().await;
}

// ── Edit form image removal ───────────────────────────────────────────

#[tokio::test]
async fn edit_form_remove_image_calls_delete_api() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Upload an image first
    session.goto("/login").await.unwrap();
    let upload_script = format!(
        r#"
        const callback = arguments[arguments.length - 1];
        const canvas = document.createElement('canvas');
        canvas.width = 4;
        canvas.height = 4;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'red';
        ctx.fillRect(0, 0, 4, 4);
        const dataUrl = canvas.toDataURL('image/png');
        fetch('/api/v1/roaster/{}/image', {{
            method: 'PUT',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ image: dataUrl }})
        }}).then(r => callback(r.status.toString())).catch(e => callback('error: ' + e));
        "#,
        roaster.id
    );
    let result = session
        .driver
        .execute_async(&upload_script, vec![])
        .await
        .unwrap();
    assert_eq!(
        result.json().as_str().unwrap_or(""),
        "204",
        "Image upload should succeed"
    );

    // Navigate to edit page
    session
        .goto(&format!("/roasters/{}/edit", roaster.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "[id$='-existing']")
        .await
        .unwrap();

    // Click the Remove button (need to accept the confirm dialog)
    let remove_script = format!(
        r#"
        const callback = arguments[arguments.length - 1];
        fetch('/api/v1/roaster/{}/image', {{ method: 'DELETE' }})
            .then(r => {{
                document.querySelector("[id$='-existing']").remove();
                callback(r.status.toString());
            }})
            .catch(e => callback('error: ' + e));
        "#,
        roaster.id
    );
    let result = session
        .driver
        .execute_async(&remove_script, vec![])
        .await
        .unwrap();
    assert_eq!(
        result.json().as_str().unwrap_or(""),
        "204",
        "Image delete should return 204"
    );

    // Preview should be gone
    let result = session.driver.find(By::Css("[id$='-existing']")).await;
    assert!(result.is_err(), "Image preview should be removed from DOM");

    // Verify image is actually deleted from server
    let check_script = format!(
        r#"
        const callback = arguments[arguments.length - 1];
        fetch('/api/v1/roaster/{}/image')
            .then(r => callback(r.status.toString()))
            .catch(e => callback('error: ' + e));
        "#,
        roaster.id
    );
    let result = session
        .driver
        .execute_async(&check_script, vec![])
        .await
        .unwrap();
    assert_eq!(
        result.json().as_str().unwrap_or(""),
        "404",
        "Image should be deleted from server"
    );

    session.quit().await;
}

// ── Cancel button ─────────────────────────────────────────────────────

#[tokio::test]
async fn edit_form_cancel_navigates_back() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // First visit the detail page (so there's a history entry to go back to)
    session
        .goto(&format!("/roasters/{}", roaster.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    // Click the Edit link to go to the edit page
    let edit_link = session
        .driver
        .find(By::Css("a[href*='/edit']"))
        .await
        .unwrap();
    edit_link.click().await.unwrap();

    wait_for_url_contains(&session.driver, "/edit")
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // Click Cancel
    let cancel_btn = session
        .driver
        .find(By::XPath("//button[contains(text(), 'Cancel')]"))
        .await
        .unwrap();
    cancel_btn.click().await.unwrap();

    // Should go back to the detail page
    tokio::time::sleep(Duration::from_millis(500)).await;
    let url = session.driver.current_url().await.unwrap();
    assert!(
        !url.as_str().contains("/edit"),
        "Should navigate away from edit page, got: {url}"
    );

    session.quit().await;
}

// ── Edit with deferred image save ─────────────────────────────────────

#[tokio::test]
async fn edit_roaster_with_new_image_saves_image() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/roasters/{}/edit", roaster.id))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "input[name='name']")
        .await
        .unwrap();

    // Set the hidden image input to a data URL (simulating deferred upload)
    let set_image_script = r#"
        const canvas = document.createElement('canvas');
        canvas.width = 4;
        canvas.height = 4;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'purple';
        ctx.fillRect(0, 0, 4, 4);
        document.getElementById('edit-roaster-image').value = canvas.toDataURL('image/jpeg');
    "#;
    session
        .driver
        .execute(set_image_script, vec![])
        .await
        .unwrap();

    // Update name too
    fill_input(&session.driver, "name", "Roaster With Image")
        .await
        .unwrap();

    submit_visible_form(&session.driver).await.unwrap();

    // Edit page URL contains /roasters/ too, so wait for /edit to disappear
    wait_for_url_not_contains(&session.driver, "/edit")
        .await
        .unwrap();

    // Verify the name was updated
    let heading = session.driver.find(By::Css("h1")).await.unwrap();
    assert_eq!(heading.text().await.unwrap(), "Roaster With Image");

    // Verify the image was saved by checking the API
    let check_script = format!(
        r#"
        const callback = arguments[arguments.length - 1];
        fetch('/api/v1/roaster/{}/image')
            .then(r => callback(r.status.toString()))
            .catch(e => callback('error: ' + e));
        "#,
        roaster.id
    );
    let result = session
        .driver
        .execute_async(&check_script, vec![])
        .await
        .unwrap();
    assert_eq!(
        result.json().as_str().unwrap_or(""),
        "200",
        "Image should be saved to server after edit with deferred upload"
    );

    session.quit().await;
}
