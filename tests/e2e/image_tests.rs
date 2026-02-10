use thirtyfour::prelude::*;

use crate::helpers::auth::authenticate_browser;
use crate::helpers::browser::BrowserSession;
use crate::helpers::server_helpers::{create_default_roaster, spawn_app_with_auth};
use crate::helpers::wait::{wait_for_element, wait_for_visible};

#[tokio::test]
async fn detail_page_shows_upload_placeholder_when_no_image() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    session
        .goto(&format!("/roasters/{}", roaster.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    // Authenticated users should see the image-upload component
    let upload = session
        .driver
        .find(By::Css("image-upload"))
        .await
        .expect("image-upload component should be present for authenticated user");

    assert!(
        upload.is_displayed().await.unwrap_or(false),
        "image-upload should be visible"
    );

    session.quit().await;
}

#[tokio::test]
async fn detail_page_shows_image_after_upload() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    authenticate_browser(&session, &app).await.unwrap();

    // Upload an image via the API using JavaScript fetch
    session
        .goto(&format!("/roasters/{}", roaster.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    // Use fetch() to upload a tiny PNG via the API
    let upload_script = format!(
        r#"
        const callback = arguments[arguments.length - 1];
        const canvas = document.createElement('canvas');
        canvas.width = 1;
        canvas.height = 1;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'red';
        ctx.fillRect(0, 0, 1, 1);
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
    assert_eq!(status, "204", "Upload should return 204, got {status}");

    // Reload the page to see the image
    session
        .goto(&format!("/roasters/{}", roaster.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    // Should now have an <img> with the thumbnail URL
    let img = wait_for_element(
        &session.driver,
        &format!("img[src='/api/v1/roaster/{}/image']", roaster.id),
    )
    .await
    .expect("image element should be present after upload");

    assert!(
        img.is_displayed().await.unwrap_or(false),
        "uploaded image should be visible"
    );

    session.quit().await;
}

#[tokio::test]
async fn detail_page_hides_upload_when_unauthenticated() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let session = BrowserSession::new(&app.address).await.unwrap();
    // Don't authenticate

    session
        .goto(&format!("/roasters/{}", roaster.slug))
        .await
        .unwrap();
    wait_for_visible(&session.driver, "h1").await.unwrap();

    // image-upload should NOT be present
    let result = session.driver.find(By::Css("image-upload")).await;
    assert!(
        result.is_err(),
        "image-upload component should not be present for unauthenticated user"
    );

    session.quit().await;
}
