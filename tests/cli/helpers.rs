use once_cell::sync::Lazy;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tempfile::TempDir;

/// Shared test server state
struct SharedServer {
    address: String,
    admin_password: String,
    #[allow(dead_code)]
    db_url: String,
    #[allow(dead_code)]
    temp_dir: TempDir,
    #[allow(dead_code)]
    process: std::process::Child,
}

/// Single shared test server for all CLI tests
static TEST_SERVER: Lazy<Mutex<Option<SharedServer>>> = Lazy::new(|| {
    // Build the binary first
    let status = Command::new("cargo")
        .args(&["build", "--bin", "brewlog"])
        .status()
        .expect("Failed to build brewlog binary");
    assert!(status.success(), "Failed to compile brewlog");

    Mutex::new(None)
});

/// Get path to the brewlog binary
pub fn brewlog_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/brewlog", manifest_dir)
}

/// Get or start the shared test server
fn get_or_start_server() -> (String, String) {
    let mut server = TEST_SERVER.lock().unwrap();

    if server.is_none() {
        // Create temporary database
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        let admin_password = "test_admin_password";

        // Start server on a random port
        let port = portpicker::pick_unused_port().expect("No ports available");
        let address = format!("http://127.0.0.1:{}", port);

        let process = Command::new(brewlog_bin())
            .args(&["serve", "--port", &port.to_string(), "--database", &db_url])
            .env("BREWLOG_ADMIN_PASSWORD", admin_password)
            .env("RUST_LOG", "error")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start brewlog server");

        // Wait for server to be ready
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();
        let health_url = format!("{}/api/v1/roasters", address);

        for _ in 0..50 {
            if client.get(&health_url).send().is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Give it a bit more time to stabilize
        std::thread::sleep(std::time::Duration::from_millis(200));

        *server = Some(SharedServer {
            address: address.clone(),
            admin_password: admin_password.to_string(),
            db_url,
            temp_dir,
            process,
        });
    }

    let srv = server.as_ref().unwrap();
    (srv.address.clone(), srv.admin_password.clone())
}

/// Get the shared server address and admin password
pub fn server_info() -> (String, String) {
    get_or_start_server()
}

/// Create a token for testing using the API directly (avoids interactive CLI)
pub fn create_token(name: &str) -> String {
    let (address, password) = server_info();

    // Use the API directly to create a token
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("{}/api/v1/tokens", address))
        .json(&serde_json::json!({
            "username": "admin",
            "password": password,
            "name": name
        }))
        .send()
        .expect("Failed to create token via API");

    assert!(
        response.status().is_success(),
        "Failed to create token: status={} body={}",
        response.status(),
        response.text().unwrap_or_default()
    );

    let token_response: serde_json::Value =
        response.json().expect("Failed to parse token response");

    token_response["token"]
        .as_str()
        .expect("Token not found in response")
        .to_string()
}

/// Run a brewlog CLI command and return the output
pub fn run_brewlog(args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let (address, _) = server_info();

    let mut cmd = Command::new(brewlog_bin());
    cmd.args(args);
    cmd.env("BREWLOG_SERVER", &address);

    for (key, value) in env {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to run brewlog command")
}
