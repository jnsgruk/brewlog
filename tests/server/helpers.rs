use std::sync::Arc;

use brewlog::application::routes::app_router;
use brewlog::application::state::{AppState, AppStateConfig};
use brewlog::domain::cafes::{Cafe, NewCafe};
use brewlog::domain::repositories::{
    CafeRepository, RoastRepository, RoasterRepository, SessionRepository, TimelineEventRepository,
    TokenRepository, UserRepository,
};
use brewlog::domain::roasters::{NewRoaster, Roaster};
use brewlog::domain::users::NewUser;
use brewlog::infrastructure::database::Database;
use reqwest::Client;
use tokio::net::TcpListener;
use tokio::task::AbortHandle;
use webauthn_rs::prelude::*;

pub struct TestApp {
    pub address: String,
    pub roaster_repo: Arc<dyn RoasterRepository>,
    pub roast_repo: Arc<dyn RoastRepository>,
    #[allow(dead_code)]
    pub cafe_repo: Arc<dyn CafeRepository>,
    #[allow(dead_code)]
    pub timeline_repo: Arc<dyn TimelineEventRepository>,
    #[allow(dead_code)]
    pub user_repo: Option<Arc<dyn UserRepository>>,
    #[allow(dead_code)]
    pub token_repo: Option<Arc<dyn TokenRepository>>,
    pub session_repo: Option<Arc<dyn SessionRepository>>,
    pub auth_token: Option<String>,
    #[allow(dead_code)]
    pub mock_server: Option<wiremock::MockServer>,
    server_handle: AbortHandle,
}

impl TestApp {
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.address, path)
    }

    pub fn page_url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

fn test_webauthn() -> Arc<Webauthn> {
    #[allow(clippy::expect_used)]
    let rp_origin = url::Url::parse("http://localhost:0").expect("valid URL");
    #[allow(clippy::expect_used)]
    Arc::new(
        WebauthnBuilder::new("localhost", &rp_origin)
            .expect("valid RP config")
            .rp_name("Brewlog Test")
            .build()
            .expect("valid WebAuthn"),
    )
}

pub async fn spawn_app() -> TestApp {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    spawn_app_inner(database, test_state_config(), None).await
}

fn test_state_config() -> AppStateConfig {
    AppStateConfig {
        webauthn: test_webauthn(),
        foursquare_url: brewlog::infrastructure::foursquare::FOURSQUARE_SEARCH_URL.to_string(),
        foursquare_api_key: String::new(),
        openrouter_url: brewlog::infrastructure::ai::OPENROUTER_URL.to_string(),
        openrouter_api_key: String::new(),
        openrouter_model: "openrouter/free".to_string(),
    }
}

async fn spawn_app_inner(
    database: Database,
    config: AppStateConfig,
    mock_server: Option<wiremock::MockServer>,
) -> TestApp {
    let state = AppState::from_database(&database, config);

    // Clone repos we need for TestApp before consuming state in the router
    let roaster_repo = state.roaster_repo.clone();
    let roast_repo = state.roast_repo.clone();
    let cafe_repo = state.cafe_repo.clone();
    let timeline_repo = state.timeline_repo.clone();
    let user_repo = state.user_repo.clone();
    let token_repo = state.token_repo.clone();
    let session_repo = state.session_repo.clone();

    let app = app_router(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    let local_addr = listener.local_addr().expect("Failed to get local address");
    let address = format!("http://{}", local_addr);

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    })
    .abort_handle();

    TestApp {
        address,
        roaster_repo,
        roast_repo,
        cafe_repo,
        timeline_repo,
        user_repo: Some(user_repo),
        token_repo: Some(token_repo),
        session_repo: Some(session_repo),
        auth_token: None,
        mock_server,
        server_handle,
    }
}

pub async fn spawn_app_with_auth() -> TestApp {
    let app = spawn_app().await;
    add_auth_to_app(app).await
}

pub async fn spawn_app_with_foursquare_mock() -> TestApp {
    let mock_server = wiremock::MockServer::start().await;
    let foursquare_url = format!("{}/places/search", mock_server.uri());

    let database = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    let app = spawn_app_inner(
        database,
        AppStateConfig {
            foursquare_url,
            foursquare_api_key: "test-api-key".to_string(),
            ..test_state_config()
        },
        Some(mock_server),
    )
    .await;

    add_auth_to_app(app).await
}

async fn add_auth_to_app(mut app: TestApp) -> TestApp {
    // Create user with UUID (no password)
    let user_uuid = uuid::Uuid::new_v4().to_string();
    let admin_user = NewUser::new("admin".to_string(), user_uuid);

    let admin_user = app
        .user_repo
        .as_ref()
        .unwrap()
        .insert(admin_user)
        .await
        .expect("Failed to create admin user");

    // Create a token for testing via direct DB insert
    use brewlog::domain::tokens::NewToken;
    use brewlog::infrastructure::auth::{generate_token, hash_token};

    let token_value = generate_token().expect("Failed to generate token");
    let token_hash = hash_token(&token_value);
    let token = NewToken::new(admin_user.id, token_hash, "test-token".to_string());

    app.token_repo
        .as_ref()
        .unwrap()
        .insert(token)
        .await
        .expect("Failed to insert token");

    app.auth_token = Some(token_value);
    app
}

pub async fn create_roaster_with_payload(app: &TestApp, payload: NewRoaster) -> Roaster {
    let client = Client::new();
    let mut request = client.post(app.api_url("/roasters")).json(&payload);

    // Add auth token if available
    if let Some(token) = &app.auth_token {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .expect("failed to create roaster via API");

    response
        .json()
        .await
        .expect("failed to deserialize roaster from response")
}

pub async fn create_roast_with_payload(
    app: &TestApp,
    payload: brewlog::domain::roasts::NewRoast,
) -> brewlog::domain::roasts::Roast {
    let client = Client::new();
    let mut request = client.post(app.api_url("/roasts")).json(&payload);

    if let Some(token) = &app.auth_token {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .expect("failed to create roast via API");

    response
        .json()
        .await
        .expect("failed to deserialize roast from response")
}

pub async fn create_default_roaster(app: &TestApp) -> Roaster {
    create_roaster_with_name(app, "Test Roasters").await
}

pub async fn create_default_roast(
    app: &TestApp,
    roaster_id: brewlog::domain::ids::RoasterId,
) -> brewlog::domain::roasts::Roast {
    create_roast_with_payload(
        app,
        brewlog::domain::roasts::NewRoast {
            roaster_id,
            name: "Test Roast".to_string(),
            origin: "Ethiopia".to_string(),
            region: "Yirgacheffe".to_string(),
            producer: "Coop".to_string(),
            tasting_notes: vec!["Blueberry".to_string()],
            process: "Washed".to_string(),
        },
    )
    .await
}

pub async fn create_roaster_with_name(app: &TestApp, name: &str) -> Roaster {
    create_roaster_with_payload(
        app,
        NewRoaster {
            name: name.to_string(),
            country: "UK".to_string(),
            city: None,
            homepage: None,
        },
    )
    .await
}

pub async fn create_default_bag(
    app: &TestApp,
    roast_id: brewlog::domain::ids::RoastId,
) -> brewlog::domain::bags::Bag {
    let client = Client::new();
    let new_bag = brewlog::domain::bags::NewBag {
        roast_id,
        roast_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        amount: 250.0,
    };

    let mut request = client.post(app.api_url("/bags")).json(&new_bag);

    if let Some(token) = &app.auth_token {
        request = request.bearer_auth(token);
    }

    let response = request.send().await.expect("failed to create bag via API");

    response
        .json()
        .await
        .expect("failed to deserialize bag from response")
}

/// Asserts that the response has valid Datastar fragment headers
pub fn assert_datastar_headers(response: &reqwest::Response, expected_selector: &str) {
    assert_datastar_headers_with_mode(response, expected_selector, "replace");
}

pub fn assert_datastar_headers_with_mode(
    response: &reqwest::Response,
    expected_selector: &str,
    expected_mode: &str,
) {
    let selector = response
        .headers()
        .get("datastar-selector")
        .and_then(|v| v.to_str().ok());
    assert_eq!(
        selector,
        Some(expected_selector),
        "Expected datastar-selector header to be '{}', got {:?}",
        expected_selector,
        selector
    );

    let mode = response
        .headers()
        .get("datastar-mode")
        .and_then(|v| v.to_str().ok());
    assert_eq!(
        mode,
        Some(expected_mode),
        "Expected datastar-mode header to be '{}', got {:?}",
        expected_mode,
        mode
    );
}

/// Asserts that the response body is an HTML fragment (not a full page)
pub fn assert_html_fragment(body: &str) {
    assert!(
        !body.contains("<!DOCTYPE"),
        "Expected HTML fragment, but found DOCTYPE declaration"
    );
    assert!(
        !body.contains("<html"),
        "Expected HTML fragment, but found <html> tag"
    );
}

/// Asserts that the body contains full HTML page structure
pub fn assert_full_page(body: &str) {
    assert!(
        body.contains("<!DOCTYPE") || body.contains("<html"),
        "Expected full HTML page with DOCTYPE or <html> tag"
    );
}

pub async fn create_default_gear(
    app: &TestApp,
    category: &str,
    make: &str,
    model: &str,
) -> brewlog::domain::gear::Gear {
    let client = Client::new();
    let gear_category = match category {
        "grinder" => brewlog::domain::gear::GearCategory::Grinder,
        "brewer" => brewlog::domain::gear::GearCategory::Brewer,
        "filter_paper" => brewlog::domain::gear::GearCategory::FilterPaper,
        _ => panic!("Unknown gear category: {}", category),
    };
    let new_gear = brewlog::domain::gear::NewGear {
        category: gear_category,
        make: make.to_string(),
        model: model.to_string(),
    };

    let mut request = client.post(app.api_url("/gear")).json(&new_gear);

    if let Some(token) = &app.auth_token {
        request = request.bearer_auth(token);
    }

    let response = request.send().await.expect("failed to create gear via API");

    response
        .json()
        .await
        .expect("failed to deserialize gear from response")
}

pub async fn create_default_cafe(app: &TestApp) -> Cafe {
    create_cafe_with_payload(
        app,
        NewCafe {
            name: "Blue Bottle".to_string(),
            city: "San Francisco".to_string(),
            country: "US".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
            website: Some("https://bluebottlecoffee.com".to_string()),
        },
    )
    .await
}

pub async fn create_cafe_with_payload(app: &TestApp, payload: NewCafe) -> Cafe {
    let client = Client::new();
    let mut request = client.post(app.api_url("/cafes")).json(&payload);

    if let Some(token) = &app.auth_token {
        request = request.bearer_auth(token);
    }

    let response = request.send().await.expect("failed to create cafe via API");

    response
        .json()
        .await
        .expect("failed to deserialize cafe from response")
}

/// Creates a session for the authenticated user and returns the raw session token
/// to use as a `brewlog_session` cookie value.
pub async fn create_session(app: &TestApp) -> String {
    use brewlog::domain::sessions::NewSession;
    use brewlog::infrastructure::auth::{generate_session_token, hash_token};

    let session_token = generate_session_token();
    let session_hash = hash_token(&session_token);

    // Get the user ID from the auth token
    let token_hash = hash_token(app.auth_token.as_ref().expect("auth token required"));
    let token = app
        .token_repo
        .as_ref()
        .expect("token_repo required")
        .get_by_token_hash(&token_hash)
        .await
        .expect("failed to find token");

    let now = chrono::Utc::now();
    #[allow(clippy::expect_used)]
    let expires_at = now
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("timestamp overflow");

    let new_session = NewSession::new(token.user_id, session_hash, now, expires_at);

    app.session_repo
        .as_ref()
        .expect("session_repo required")
        .insert(new_session)
        .await
        .expect("failed to create session");

    session_token
}

pub async fn spawn_app_with_openrouter_mock() -> TestApp {
    let mock_server = wiremock::MockServer::start().await;
    let openrouter_url = format!("{}/api/v1/chat/completions", mock_server.uri());

    let database = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    let app = spawn_app_inner(
        database,
        AppStateConfig {
            openrouter_url,
            ..test_state_config()
        },
        Some(mock_server),
    )
    .await;

    add_auth_to_app(app).await
}
