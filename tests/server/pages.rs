use reqwest::redirect::Policy;

use crate::helpers::{
    assert_full_page, create_default_bag, create_default_brew, create_default_roast,
    create_default_roaster, create_session, spawn_app, spawn_app_with_auth,
};

#[tokio::test]
async fn homepage_returns_200_with_empty_database() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(app.page_url("/"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn homepage_returns_200_with_data() {
    let app = spawn_app_with_auth().await;

    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let _bag = create_default_bag(&app, roast.id).await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.page_url("/"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
    assert!(
        body.contains("Test Roasters"),
        "Homepage should contain roaster name"
    );
}

#[tokio::test]
async fn homepage_shows_stats_counts() {
    let app = spawn_app_with_auth().await;

    let roaster1 = create_default_roaster(&app).await;
    let _roaster2 = crate::helpers::create_roaster_with_name(&app, "Second Roasters").await;
    let _roast = create_default_roast(&app, roaster1.id).await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.page_url("/"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    // Stats section should show counts
    assert!(body.contains(">2<"), "Should show 2 roasters in stats");
    assert!(body.contains(">1<"), "Should show 1 roast in stats");
}

#[tokio::test]
async fn login_page_returns_200() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(app.page_url("/login"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn checkin_page_redirects_unauthenticated_to_login() {
    let app = spawn_app().await;
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/check-in"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(
        response.status().is_redirection(),
        "Expected redirect, got {}",
        response.status()
    );
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok());
    assert_eq!(location, Some("/login"));
}

#[tokio::test]
async fn checkin_page_returns_200_when_authenticated() {
    let app = spawn_app_with_auth().await;
    let session_token = create_session(&app).await;

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/check-in"))
        .header("Cookie", format!("brewlog_session={session_token}"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn add_page_redirects_unauthenticated_to_login() {
    let app = spawn_app().await;
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/add"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(
        response.status().is_redirection(),
        "Expected redirect, got {}",
        response.status()
    );
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok());
    assert_eq!(location, Some("/login"));
}

#[tokio::test]
async fn add_page_returns_200_when_authenticated() {
    let app = spawn_app_with_auth().await;
    let session_token = create_session(&app).await;

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/add"))
        .header("Cookie", format!("brewlog_session={session_token}"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn admin_page_redirects_unauthenticated_to_login() {
    let app = spawn_app().await;
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/admin"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(
        response.status().is_redirection(),
        "Expected redirect, got {}",
        response.status()
    );
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok());
    assert_eq!(location, Some("/login"));
}

#[tokio::test]
async fn admin_page_returns_200_when_authenticated() {
    let app = spawn_app_with_auth().await;
    let session_token = create_session(&app).await;

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/admin"))
        .header("Cookie", format!("brewlog_session={session_token}"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert_full_page(&body);
}

#[tokio::test]
async fn logout_redirects_to_homepage() {
    let app = spawn_app_with_auth().await;
    let session_token = create_session(&app).await;

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .post(app.page_url("/logout"))
        .header("Cookie", format!("brewlog_session={session_token}"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(
        response.status().is_redirection(),
        "Expected redirect, got {}",
        response.status()
    );
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok());
    assert_eq!(location, Some("/"));
}

#[tokio::test]
async fn scan_redirect_returns_permanent_redirect() {
    let app = spawn_app().await;
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build client");

    let response = client
        .get(app.page_url("/scan"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 308);
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok());
    assert_eq!(location, Some("/"));
}

#[tokio::test]
async fn homepage_contains_chip_scroll_with_data() {
    let app = spawn_app_with_auth().await;

    let _brew = create_default_brew(&app).await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.page_url("/"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read body");
    assert!(
        body.contains("<chip-scroll"),
        "Homepage should contain chip-scroll component when data exists"
    );
}
