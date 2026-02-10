use brewlog::domain::cups::Cup;
use brewlog::domain::roasters::Roaster;

use crate::helpers::{
    assert_datastar_headers, assert_html_fragment, create_default_cafe, create_default_roast,
    create_default_roaster, spawn_app_with_auth,
};

/// Generate a minimal valid 1x1 red PNG as a base64 data URL.
fn tiny_png_data_url() -> String {
    use base64::Engine;
    use image::{ImageBuffer, Rgba};

    let img = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 0, 0, 255]));
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(encoder, img.as_raw(), 1, 1, image::ColorType::Rgba8.into())
        .expect("failed to encode test PNG");

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    format!("data:image/png;base64,{b64}")
}

fn image_url(entity_type: &str, id: impl std::fmt::Display) -> String {
    format!("/{entity_type}/{id}/image")
}

fn thumbnail_url(entity_type: &str, id: impl std::fmt::Display) -> String {
    format!("/{entity_type}/{id}/thumbnail")
}

async fn upload_image(
    client: &reqwest::Client,
    app: &crate::helpers::TestApp,
    entity_type: &str,
    id: impl std::fmt::Display,
) -> reqwest::Response {
    client
        .put(app.api_url(&image_url(entity_type, &id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({ "image": tiny_png_data_url() }))
        .send()
        .await
        .expect("failed to upload image")
}

// ===========================================================================
// Upload & retrieval
// ===========================================================================

#[tokio::test]
async fn upload_image_returns_204() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    let response = upload_image(&client, &app, "roaster", roaster.id).await;
    assert_eq!(response.status(), 204);
}

#[tokio::test]
async fn get_image_returns_uploaded_data() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    let response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("image/jpeg")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=604800")
    );
    let body = response.bytes().await.expect("failed to read body");
    assert!(!body.is_empty(), "image body should not be empty");
}

#[tokio::test]
async fn get_thumbnail_returns_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    let response = client
        .get(app.api_url(&thumbnail_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get thumbnail");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("image/jpeg")
    );
    let body = response.bytes().await.expect("failed to read body");
    assert!(!body.is_empty(), "thumbnail body should not be empty");
}

#[tokio::test]
async fn upload_image_upserts() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    // Upload twice
    upload_image(&client, &app, "roaster", roaster.id).await;
    upload_image(&client, &app, "roaster", roaster.id).await;

    // GET should still work (upsert, not duplicate)
    let response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");
    assert_eq!(response.status(), 200);
}

// ===========================================================================
// Deletion
// ===========================================================================

#[tokio::test]
async fn delete_image_returns_204() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    let response = client
        .delete(app.api_url(&image_url("roaster", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("failed to delete image");

    assert_eq!(response.status(), 204);
}

#[tokio::test]
async fn delete_image_makes_get_return_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    client
        .delete(app.api_url(&image_url("roaster", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("failed to delete image");

    let response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn deleting_entity_deletes_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    // Delete the roaster entity
    client
        .delete(app.api_url(&format!("/roasters/{}", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .send()
        .await
        .expect("failed to delete roaster");

    // Image should be gone too
    let response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(response.status(), 404);
}

// ===========================================================================
// Auth
// ===========================================================================

#[tokio::test]
async fn upload_image_requires_auth() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    let response = client
        .put(app.api_url(&image_url("roaster", roaster.id)))
        .json(&serde_json::json!({ "image": tiny_png_data_url() }))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn delete_image_requires_auth() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    let response = client
        .delete(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn get_image_does_not_require_auth() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    // GET without auth token
    let response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn get_thumbnail_does_not_require_auth() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    let response = client
        .get(app.api_url(&thumbnail_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get thumbnail");

    assert_eq!(response.status(), 200);
}

// ===========================================================================
// Validation
// ===========================================================================

#[tokio::test]
async fn upload_image_invalid_entity_type_returns_400() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let response = client
        .put(app.api_url("/invalid/1/image"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({ "image": tiny_png_data_url() }))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn upload_image_nonexistent_entity_returns_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let response = client
        .put(app.api_url("/roaster/999999/image"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&serde_json::json!({ "image": tiny_png_data_url() }))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn get_image_nonexistent_returns_404() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    let response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn get_image_invalid_entity_type_returns_400() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let response = client
        .get(app.api_url("/invalid/1/image"))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(response.status(), 400);
}

// ===========================================================================
// Datastar
// ===========================================================================

#[tokio::test]
async fn upload_image_with_datastar_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    let response = client
        .put(app.api_url(&image_url("roaster", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .json(&serde_json::json!({ "image": tiny_png_data_url() }))
        .send()
        .await
        .expect("failed to upload image");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#entity-image");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

#[tokio::test]
async fn delete_image_with_datastar_returns_fragment() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();
    let roaster = create_default_roaster(&app).await;

    upload_image(&client, &app, "roaster", roaster.id).await;

    let response = client
        .delete(app.api_url(&image_url("roaster", roaster.id)))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .send()
        .await
        .expect("failed to delete image");

    assert_eq!(response.status(), 200);
    assert_datastar_headers(&response, "#entity-image");

    let body = response.text().await.expect("failed to read body");
    assert_html_fragment(&body);
}

// ===========================================================================
// Deferred image via create forms
// ===========================================================================

#[tokio::test]
async fn create_roaster_with_image_saves_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "name": "Image Roasters",
        "country": "UK",
        "image": tiny_png_data_url(),
    });

    let response = client
        .post(app.api_url("/roasters"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to create roaster");

    assert_eq!(response.status(), 201);
    let roaster: Roaster = response.json().await.expect("failed to parse roaster");

    // Image should be retrievable
    let img_response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(img_response.status(), 200);
}

#[tokio::test]
async fn create_roaster_without_image_field_works() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "name": "No Image Roasters",
        "country": "UK",
    });

    let response = client
        .post(app.api_url("/roasters"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to create roaster");

    assert_eq!(response.status(), 201);
    let roaster: Roaster = response.json().await.expect("failed to parse roaster");

    // No image
    let img_response = client
        .get(app.api_url(&image_url("roaster", roaster.id)))
        .send()
        .await
        .expect("failed to get image");

    assert_eq!(img_response.status(), 404);
}

// ===========================================================================
// Deferred image via checkin
// ===========================================================================

#[tokio::test]
async fn checkin_with_cafe_image_saves_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    let payload = serde_json::json!({
        "roast_id": roast.id.to_string(),
        "cafe_name": "Imaged Cafe",
        "cafe_city": "London",
        "cafe_country": "UK",
        "cafe_lat": 51.5074,
        "cafe_lng": -0.1278,
        "cafe_image": tiny_png_data_url(),
    });

    let response = client
        .post(app.api_url("/check-in"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to check in");

    assert_eq!(response.status(), 201);
    let cup: Cup = response.json().await.expect("failed to parse cup");

    // Cafe image should be retrievable
    let img_response = client
        .get(app.api_url(&image_url("cafe", cup.cafe_id)))
        .send()
        .await
        .expect("failed to get cafe image");

    assert_eq!(img_response.status(), 200);
}

#[tokio::test]
async fn checkin_with_cup_image_saves_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let cafe = create_default_cafe(&app).await;

    let payload = serde_json::json!({
        "roast_id": roast.id.to_string(),
        "cafe_id": cafe.id.to_string(),
        "cup_image": tiny_png_data_url(),
    });

    let response = client
        .post(app.api_url("/check-in"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to check in");

    assert_eq!(response.status(), 201);
    let cup: Cup = response.json().await.expect("failed to parse cup");

    // Cup image should be retrievable
    let img_response = client
        .get(app.api_url(&image_url("cup", cup.id)))
        .send()
        .await
        .expect("failed to get cup image");

    assert_eq!(img_response.status(), 200);
}

// ===========================================================================
// Deferred image via scan
// ===========================================================================

#[tokio::test]
async fn scan_with_image_saves_roast_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "roaster_name": "Scan Img Roasters",
        "roaster_country": "UK",
        "roast_name": "Scan Img Roast",
        "origin": "Ethiopia",
        "region": "Yirgacheffe",
        "producer": "Coop",
        "process": "Washed",
        "tasting_notes": "Blueberry",
        "scan_image": tiny_png_data_url(),
    });

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to submit scan");

    assert_eq!(response.status(), 201);

    let result: serde_json::Value = response.json().await.expect("failed to parse scan result");
    let roast_id = result["roast_id"].as_i64().expect("missing roast_id");

    // Roast image should be retrievable
    let img_response = client
        .get(app.api_url(&image_url("roast", roast_id)))
        .send()
        .await
        .expect("failed to get roast image");

    assert_eq!(img_response.status(), 200);
}

#[tokio::test]
async fn scan_matched_roast_saves_image_if_none_exists() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    // Scan with matched_roast_id and scan_image
    let payload = serde_json::json!({
        "matched_roast_id": roast.id.into_inner().to_string(),
        "scan_image": tiny_png_data_url(),
    });

    let response = client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to submit scan");

    assert_eq!(response.status(), 201);

    // Roast should now have an image
    let img_response = client
        .get(app.api_url(&image_url("roast", roast.id)))
        .send()
        .await
        .expect("failed to get roast image");

    assert_eq!(img_response.status(), 200);
}

#[tokio::test]
async fn scan_matched_roast_does_not_overwrite_existing_image() {
    let app = spawn_app_with_auth().await;
    let client = reqwest::Client::new();

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    // Upload an image for the roast first
    upload_image(&client, &app, "roast", roast.id).await;

    // Get the existing image to compare later
    let original = client
        .get(app.api_url(&image_url("roast", roast.id)))
        .send()
        .await
        .expect("failed to get image")
        .bytes()
        .await
        .expect("failed to read body");

    // Scan with matched_roast_id and scan_image — should NOT overwrite
    let payload = serde_json::json!({
        "matched_roast_id": roast.id.into_inner().to_string(),
        "scan_image": tiny_png_data_url(),
    });

    client
        .post(app.api_url("/scan"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .json(&payload)
        .send()
        .await
        .expect("failed to submit scan");

    // Image should still be the original
    let after = client
        .get(app.api_url(&image_url("roast", roast.id)))
        .send()
        .await
        .expect("failed to get image")
        .bytes()
        .await
        .expect("failed to read body");

    assert_eq!(original, after, "existing image should not be overwritten");
}
