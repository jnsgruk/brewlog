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
    // Cargo sets this for integration tests - works for both debug and release
    env!("CARGO_BIN_EXE_brewlog").to_string()
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
            .env("BREWLOG_ADMIN_USERNAME", "admin")
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

/// Create a token for testing using the CLI
pub fn create_token(name: &str) -> String {
    let (_, password) = ensure_server_started().expect("Failed to start test server");

    let output = run_brewlog(
        &[
            "token",
            "create",
            "--name",
            name,
            "--username",
            "admin",
            "--password",
            &password,
        ],
        &[],
    );

    if !output.status.success() {
        panic!(
            "Failed to create token: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(token) = line.trim().strip_prefix("export BREWLOG_TOKEN=") {
            return token.to_string();
        }
    }

    panic!("Could not find token in output: {}", stdout);
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
