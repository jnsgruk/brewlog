use once_cell::sync::Lazy;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tempfile::TempDir;

/// Shared test server state
/// Fields prefixed with `_` are kept alive to prevent cleanup:
/// - `_temp_dir`: keeps temporary database file from being deleted
/// - `_process`: keeps server process running
/// - `_db_url`: retained for consistency
struct SharedServer {
    address: String,
    admin_password: String,
    _db_url: String,
    _temp_dir: TempDir,
    _process: std::process::Child,
}

/// Single shared test server for all CLI tests
static TEST_SERVER: Lazy<Mutex<Option<SharedServer>>> = Lazy::new(|| Mutex::new(None));

/// Get path to the brewlog binary
pub fn brewlog_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/brewlog", manifest_dir)
}

/// Get or start the shared test server - handles mutex poisoning gracefully
fn ensure_server_started() -> Result<(String, String), String> {
    let mut server = match TEST_SERVER.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if server.is_none() {
        eprintln!("Starting test server...");

        // Create temporary database
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        let admin_password = "test_admin_password";

        // Start server on a random port
        let port = portpicker::pick_unused_port().ok_or("No ports available")?;
        let address = format!("http://127.0.0.1:{}", port);

        eprintln!("Starting server on {}", address);

        let bind_address = format!("127.0.0.1:{}", port);
        let process = Command::new(brewlog_bin())
            .args(&[
                "serve",
                "--bind-address",
                &bind_address,
                "--database-url",
                &db_url,
            ])
            .env("BREWLOG_ADMIN_PASSWORD", admin_password)
            .env("RUST_LOG", "error")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start brewlog server: {}", e))?;

        // Wait for server to be ready
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let health_url = format!("{}/api/v1/roasters", address);

        let mut server_ready = false;
        for attempt in 0..100 {
            match client.get(&health_url).send() {
                Ok(_) => {
                    eprintln!("Server ready after {} attempts", attempt + 1);
                    server_ready = true;
                    break;
                }
                Err(_) => {
                    if attempt == 99 {
                        return Err(
                            "Server failed to start after 100 attempts (10 seconds)".to_string()
                        );
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }

        if !server_ready {
            return Err("Server never became ready".to_string());
        }

        *server = Some(SharedServer {
            address: address.clone(),
            admin_password: admin_password.to_string(),
            _db_url: db_url,
            _temp_dir: temp_dir,
            _process: process,
        });
    }

    let srv = server.as_ref().unwrap();
    Ok((srv.address.clone(), srv.admin_password.clone()))
}

/// Get the shared server address and admin password
pub fn server_info() -> (String, String) {
    ensure_server_started().expect("Failed to start test server")
}

/// Create a token for testing using the API directly
pub fn create_token(name: &str) -> String {
    let (address, password) = ensure_server_started().expect("Failed to start test server");

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");

    let response = client
        .post(format!("{}/api/v1/tokens", address))
        .json(&serde_json::json!({
            "username": "admin",
            "password": password,
            "name": name
        }))
        .send()
        .expect("Failed to send token creation request");

    if !response.status().is_success() {
        panic!(
            "Failed to create token: status={} body={}",
            response.status(),
            response.text().unwrap_or_default()
        );
    }

    let token_response: serde_json::Value =
        response.json().expect("Failed to parse token response");

    token_response["token"]
        .as_str()
        .expect("Token not found in response")
        .to_string()
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
