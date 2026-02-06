use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use once_cell::sync::Lazy;
use tempfile::TempDir;
use tokio::net::TcpListener;
use webauthn_rs::prelude::*;

use brewlog::application::routes::app_router;
use brewlog::application::state::{AppState, AppStateConfig};
use brewlog::infrastructure::database::Database;

/// Shared test server state.
/// `_temp_dir` is kept alive to prevent the temporary database file from being deleted.
/// The server itself runs on a background thread and dies when the test binary exits.
struct SharedServer {
    address: String,
    db_url: String,
    _temp_dir: TempDir,
}

/// Single shared test server for all CLI tests
static TEST_SERVER: Lazy<Mutex<Option<SharedServer>>> = Lazy::new(|| Mutex::new(None));

#[allow(clippy::expect_used)]
fn test_webauthn() -> Arc<Webauthn> {
    let rp_origin = url::Url::parse("http://localhost:0").expect("valid URL");
    Arc::new(
        WebauthnBuilder::new("localhost", &rp_origin)
            .expect("valid RP config")
            .rp_name("Brewlog Test")
            .build()
            .expect("valid WebAuthn"),
    )
}

/// Get path to the brewlog binary
pub fn brewlog_bin() -> String {
    // Cargo sets this for integration tests - works for both debug and release
    env!("CARGO_BIN_EXE_brewlog").to_string()
}

/// Get or start the shared test server - handles mutex poisoning gracefully.
///
/// Spawns the server in-process on a background thread (not as a child process),
/// so the server dies automatically when the test binary exits.
#[allow(clippy::too_many_lines)]
fn ensure_server_started() -> Result<(String, String), String> {
    let mut server = match TEST_SERVER.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if server.is_none() {
        eprintln!("Starting test server...");

        // Create temporary database (file-based so create_token() can connect independently)
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {e}"))?;
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        // Pick a random port
        let port = portpicker::pick_unused_port().ok_or("No ports available")?;
        let address = format!("http://127.0.0.1:{port}");
        let bind_address = format!("127.0.0.1:{port}");

        eprintln!("Starting server on {address}");

        let db_url_for_thread = db_url.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);

        // Start server on a background thread with its own tokio runtime.
        // The thread (and server) live until the test binary exits — no cleanup needed.
        std::thread::spawn(move || {
            #[allow(clippy::expect_used)]
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.block_on(async move {
                #[allow(clippy::expect_used)]
                let database = Database::connect(&db_url_for_thread)
                    .await
                    .expect("Failed to connect to test database");

                let state = AppState::from_database(
                    &database,
                    AppStateConfig {
                        webauthn: test_webauthn(),
                        foursquare_url: brewlog::infrastructure::foursquare::FOURSQUARE_SEARCH_URL
                            .to_string(),
                        foursquare_api_key: String::new(),
                        openrouter_url: brewlog::infrastructure::ai::OPENROUTER_URL.to_string(),
                        openrouter_api_key: String::new(),
                        openrouter_model: "openrouter/free".to_string(),
                    },
                );

                let app = app_router(state);

                #[allow(clippy::expect_used)]
                let listener = TcpListener::bind(&bind_address)
                    .await
                    .expect("Failed to bind to port");

                // Signal readiness to the main thread
                let _ = tx.send(());

                // Run server until the process exits
                #[allow(clippy::expect_used)]
                axum::serve(listener, app).await.expect("Server failed");
            });
        });

        // Wait for server to be ready
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|e| format!("Server failed to start within timeout: {e}"))?;

        eprintln!("Server ready on {address}");

        *server = Some(SharedServer {
            address: address.clone(),
            db_url: db_url.clone(),
            _temp_dir: temp_dir,
        });
    }

    #[allow(clippy::unwrap_used)]
    let srv = server.as_ref().unwrap();
    Ok((srv.address.clone(), srv.db_url.clone()))
}

/// Get the shared server address and database URL
pub fn server_info() -> (String, String) {
    ensure_server_started().expect("Failed to start test server")
}

/// Create a token for testing by directly inserting into the database
pub fn create_token(name: &str) -> String {
    let (_, db_url) = ensure_server_started().expect("Failed to start test server");

    #[allow(clippy::expect_used)]
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    rt.block_on(async {
        #[allow(clippy::expect_used)]
        let pool = sqlx::SqlitePool::connect(&db_url)
            .await
            .expect("Failed to connect to test database");

        // Ensure a user exists (INSERT OR IGNORE to handle concurrent test threads)
        let uuid = uuid::Uuid::new_v4().to_string();
        #[allow(clippy::expect_used)]
        sqlx::query(
            "INSERT OR IGNORE INTO users (username, uuid, created_at) VALUES (?, ?, datetime('now'))",
        )
        .bind("admin")
        .bind(&uuid)
        .execute(&pool)
        .await
        .expect("Failed to ensure test user");

        #[allow(clippy::expect_used)]
        let user_id: i64 =
            sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE username = ?")
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("Failed to query test user");

        // Generate and insert a bearer token
        #[allow(clippy::expect_used)]
        let token_value =
            brewlog::infrastructure::auth::generate_token().expect("Failed to generate token");
        let token_hash = brewlog::infrastructure::auth::hash_token(&token_value);

        #[allow(clippy::expect_used)]
        sqlx::query(
            "INSERT INTO tokens (user_id, token_hash, name, created_at) VALUES (?, ?, ?, datetime('now'))",
        )
        .bind(user_id)
        .bind(&token_hash)
        .bind(name)
        .execute(&pool)
        .await
        .expect("Failed to insert token");

        token_value
    })
}

/// Run a brewlog CLI command and return the output
pub fn run_brewlog(args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let (address, _) = ensure_server_started().expect("Failed to start test server");

    let mut cmd = Command::new(brewlog_bin());
    cmd.args(args);
    cmd.env("BREWLOG_URL", &address);

    for (key, value) in env {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to run brewlog command")
}

/// Helper to create a roaster and return its ID
pub fn create_roaster(name: &str, token: &str) -> String {
    let output = run_brewlog(
        &["roaster", "add", "--name", name, "--country", "UK"],
        &[("BREWLOG_TOKEN", token)],
    );

    if !output.status.success() {
        panic!(
            "Failed to create roaster: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roaster: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should output valid JSON");
    roaster["id"]
        .as_i64()
        .expect("roaster id should be numeric")
        .to_string()
}

/// Helper to create a roast and return its ID
pub fn create_roast(roaster_id: &str, name: &str, token: &str) -> String {
    let output = run_brewlog(
        &[
            "roast",
            "add",
            "--roaster-id",
            roaster_id,
            "--name",
            name,
            "--origin",
            "Kenya",
            "--region",
            "Nyeri",
            "--producer",
            "Coop",
            "--process",
            "Washed",
            "--tasting-notes",
            "Blackcurrant",
        ],
        &[("BREWLOG_TOKEN", token)],
    );

    if !output.status.success() {
        panic!(
            "Failed to create roast: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roast: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");
    roast["id"]
        .as_i64()
        .expect("roast id should be numeric")
        .to_string()
}

/// Helper to create a cafe and return its ID
pub fn create_cafe(
    name: &str,
    city: &str,
    country: &str,
    latitude: &str,
    longitude: &str,
    token: &str,
) -> String {
    let output = run_brewlog(
        &[
            "cafe",
            "add",
            "--name",
            name,
            "--city",
            city,
            "--country",
            country,
            "--latitude",
            latitude,
            "--longitude",
            longitude,
        ],
        &[("BREWLOG_TOKEN", token)],
    );

    if !output.status.success() {
        panic!(
            "Failed to create cafe: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cafe: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");
    cafe["id"]
        .as_i64()
        .expect("cafe id should be numeric")
        .to_string()
}
