use std::sync::Arc;

use brewlog::application::routes::app_router;
use brewlog::application::server::AppState;
use brewlog::domain::cafes::{Cafe, NewCafe};
use brewlog::domain::repositories::{
    CafeRepository, PasskeyCredentialRepository, RegistrationTokenRepository, RoastRepository,
    RoasterRepository, SessionRepository, TimelineEventRepository, TokenRepository, UserRepository,
};
use brewlog::domain::roasters::{NewRoaster, Roaster};
use brewlog::domain::users::NewUser;
use brewlog::infrastructure::backup::BackupService;
use brewlog::infrastructure::database::Database;
use brewlog::infrastructure::repositories::bags::SqlBagRepository;
use brewlog::infrastructure::repositories::brews::SqlBrewRepository;
use brewlog::infrastructure::repositories::cafes::SqlCafeRepository;
use brewlog::infrastructure::repositories::cups::SqlCupRepository;
use brewlog::infrastructure::repositories::gear::SqlGearRepository;
use brewlog::infrastructure::repositories::passkey_credentials::SqlPasskeyCredentialRepository;
use brewlog::infrastructure::repositories::registration_tokens::SqlRegistrationTokenRepository;
use brewlog::infrastructure::repositories::roasters::SqlRoasterRepository;
use brewlog::infrastructure::repositories::roasts::SqlRoastRepository;
use brewlog::infrastructure::repositories::sessions::SqlSessionRepository;
use brewlog::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;
use brewlog::infrastructure::repositories::tokens::SqlTokenRepository;
use brewlog::infrastructure::repositories::users::SqlUserRepository;
use brewlog::infrastructure::webauthn::ChallengeStore;
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
    // Use in-memory SQLite database for testing
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    // Run migrations
    database
        .migrate()
        .await
        .expect("Failed to migrate database");

    // Create repositories
    let roaster_repo = Arc::new(SqlRoasterRepository::new(database.clone_pool()));
    let roast_repo = Arc::new(SqlRoastRepository::new(database.clone_pool()));
    let bag_repo = Arc::new(SqlBagRepository::new(database.clone_pool()));
    let gear_repo = Arc::new(SqlGearRepository::new(database.clone_pool()));
    let brew_repo = Arc::new(SqlBrewRepository::new(database.clone_pool()));
    let cafe_repo = Arc::new(SqlCafeRepository::new(database.clone_pool()));
    let cup_repo = Arc::new(SqlCupRepository::new(database.clone_pool()));
    let timeline_repo = Arc::new(SqlTimelineEventRepository::new(database.clone_pool()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(SqlUserRepository::new(database.clone_pool()));
    let token_repo: Arc<dyn TokenRepository> =
        Arc::new(SqlTokenRepository::new(database.clone_pool()));
    let session_repo: Arc<dyn SessionRepository> =
        Arc::new(SqlSessionRepository::new(database.clone_pool()));
    let passkey_repo: Arc<dyn PasskeyCredentialRepository> =
        Arc::new(SqlPasskeyCredentialRepository::new(database.clone_pool()));
    let registration_token_repo: Arc<dyn RegistrationTokenRepository> =
        Arc::new(SqlRegistrationTokenRepository::new(database.clone_pool()));

    spawn_app_inner(
        database,
        roaster_repo,
        roast_repo,
        bag_repo,
        gear_repo,
        brew_repo,
        cafe_repo,
        cup_repo,
        timeline_repo,
        user_repo,
        token_repo,
        session_repo,
        passkey_repo,
        registration_token_repo,
        brewlog::infrastructure::foursquare::FOURSQUARE_SEARCH_URL.to_string(),
        String::new(),
        brewlog::infrastructure::ai::OPENROUTER_URL.to_string(),
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn spawn_app_inner(
    _database: Database,
    roaster_repo: Arc<SqlRoasterRepository>,
    roast_repo: Arc<SqlRoastRepository>,
    bag_repo: Arc<SqlBagRepository>,
    gear_repo: Arc<SqlGearRepository>,
    brew_repo: Arc<SqlBrewRepository>,
    cafe_repo: Arc<SqlCafeRepository>,
    cup_repo: Arc<SqlCupRepository>,
    timeline_repo: Arc<SqlTimelineEventRepository>,
    user_repo: Arc<dyn UserRepository>,
    token_repo: Arc<dyn TokenRepository>,
    session_repo: Arc<dyn SessionRepository>,
    passkey_repo: Arc<dyn PasskeyCredentialRepository>,
    registration_token_repo: Arc<dyn RegistrationTokenRepository>,
    foursquare_url: String,
    foursquare_api_key: String,
    openrouter_url: String,
    mock_server: Option<wiremock::MockServer>,
) -> TestApp {
    let backup_service = Arc::new(BackupService::new(_database.clone_pool()));

    let session_repo_clone: Arc<dyn SessionRepository> = session_repo.clone();

    // Create application state
    let state = AppState {
        roaster_repo: roaster_repo.clone(),
        roast_repo: roast_repo.clone(),
        bag_repo: bag_repo.clone(),
        gear_repo: gear_repo.clone(),
        brew_repo: brew_repo.clone(),
        cafe_repo: cafe_repo.clone(),
        cup_repo: cup_repo.clone(),
        timeline_repo: timeline_repo.clone(),
        user_repo: user_repo.clone(),
        token_repo: token_repo.clone(),
        session_repo,
        passkey_repo,
        registration_token_repo,
        ai_usage_repo: Arc::new(
            brewlog::infrastructure::repositories::ai_usage::SqlAiUsageRepository::new(
                _database.clone_pool(),
            ),
        ),
        webauthn: test_webauthn(),
        challenge_store: Arc::new(ChallengeStore::new()),
        http_client: reqwest::Client::new(),
        foursquare_url,
        foursquare_api_key,
        openrouter_url,
        openrouter_api_key: String::new(),
        openrouter_model: "openrouter/free".to_string(),
        backup_service,
    };

    // Create router
    let app = app_router(state);

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    let local_addr = listener.local_addr().expect("Failed to get local address");
    let address = format!("http://{}", local_addr);

    // Spawn the server in a background task
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
        session_repo: Some(session_repo_clone),
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

    database
        .migrate()
        .await
        .expect("Failed to migrate database");

    let roaster_repo = Arc::new(SqlRoasterRepository::new(database.clone_pool()));
    let roast_repo = Arc::new(SqlRoastRepository::new(database.clone_pool()));
    let bag_repo = Arc::new(SqlBagRepository::new(database.clone_pool()));
    let gear_repo = Arc::new(SqlGearRepository::new(database.clone_pool()));
    let brew_repo = Arc::new(SqlBrewRepository::new(database.clone_pool()));
    let cafe_repo = Arc::new(SqlCafeRepository::new(database.clone_pool()));
    let cup_repo = Arc::new(SqlCupRepository::new(database.clone_pool()));
    let timeline_repo = Arc::new(SqlTimelineEventRepository::new(database.clone_pool()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(SqlUserRepository::new(database.clone_pool()));
    let token_repo: Arc<dyn TokenRepository> =
        Arc::new(SqlTokenRepository::new(database.clone_pool()));
    let session_repo: Arc<dyn SessionRepository> =
        Arc::new(SqlSessionRepository::new(database.clone_pool()));
    let passkey_repo: Arc<dyn PasskeyCredentialRepository> =
        Arc::new(SqlPasskeyCredentialRepository::new(database.clone_pool()));
    let registration_token_repo: Arc<dyn RegistrationTokenRepository> =
        Arc::new(SqlRegistrationTokenRepository::new(database.clone_pool()));

    let app = spawn_app_inner(
        database,
        roaster_repo,
        roast_repo,
        bag_repo,
        gear_repo,
        brew_repo,
        cafe_repo,
        cup_repo,
        timeline_repo,
        user_repo,
        token_repo,
        session_repo,
        passkey_repo,
        registration_token_repo,
        foursquare_url,
        "test-api-key".to_string(),
        brewlog::infrastructure::ai::OPENROUTER_URL.to_string(),
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

    database
        .migrate()
        .await
        .expect("Failed to migrate database");

    let roaster_repo = Arc::new(SqlRoasterRepository::new(database.clone_pool()));
    let roast_repo = Arc::new(SqlRoastRepository::new(database.clone_pool()));
    let bag_repo = Arc::new(SqlBagRepository::new(database.clone_pool()));
    let gear_repo = Arc::new(SqlGearRepository::new(database.clone_pool()));
    let brew_repo = Arc::new(SqlBrewRepository::new(database.clone_pool()));
    let cafe_repo = Arc::new(SqlCafeRepository::new(database.clone_pool()));
    let cup_repo = Arc::new(SqlCupRepository::new(database.clone_pool()));
    let timeline_repo = Arc::new(SqlTimelineEventRepository::new(database.clone_pool()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(SqlUserRepository::new(database.clone_pool()));
    let token_repo: Arc<dyn TokenRepository> =
        Arc::new(SqlTokenRepository::new(database.clone_pool()));
    let session_repo: Arc<dyn SessionRepository> =
        Arc::new(SqlSessionRepository::new(database.clone_pool()));
    let passkey_repo: Arc<dyn PasskeyCredentialRepository> =
        Arc::new(SqlPasskeyCredentialRepository::new(database.clone_pool()));
    let registration_token_repo: Arc<dyn RegistrationTokenRepository> =
        Arc::new(SqlRegistrationTokenRepository::new(database.clone_pool()));

    let app = spawn_app_inner(
        database,
        roaster_repo,
        roast_repo,
        bag_repo,
        gear_repo,
        brew_repo,
        cafe_repo,
        cup_repo,
        timeline_repo,
        user_repo,
        token_repo,
        session_repo,
        passkey_repo,
        registration_token_repo,
        brewlog::infrastructure::foursquare::FOURSQUARE_SEARCH_URL.to_string(),
        String::new(),
        openrouter_url,
        Some(mock_server),
    )
    .await;

    add_auth_to_app(app).await
}
