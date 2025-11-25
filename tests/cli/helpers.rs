use std::process::{Command, Stdio};
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Initialize test environment (compile the binary, etc.)
pub fn setup() {
    INIT.call_once(|| {
        // Build the project before running CLI tests
        let status = Command::new("cargo")
            .args(&["build", "--bin", "brewlog"])
            .status()
            .expect("Failed to build brewlog binary");

        assert!(status.success(), "Failed to compile brewlog");
    });
}

/// Get path to the brewlog binary
pub fn brewlog_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/brewlog", manifest_dir)
}

/// Create a temporary directory for test database
pub fn create_test_db() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite:{}", db_path.display());
    (temp_dir, db_url)
}

/// Start a brewlog server in the background for testing
pub struct TestServer {
    pub address: String,
    pub admin_password: String,
    pub db_url: String,
    _temp_dir: TempDir,
    _process: std::process::Child,
}

impl TestServer {
    pub fn start() -> Self {
        setup();

        let (temp_dir, db_url) = create_test_db();
        let admin_password = "test_admin_password";

        // Start server on a random port
        let port = portpicker::pick_unused_port().expect("No ports available");
        let address = format!("http://127.0.0.1:{}", port);

        let process = Command::new(brewlog_bin())
            .args(&["serve", "--port", &port.to_string(), "--database", &db_url])
            .env("BREWLOG_ADMIN_PASSWORD", admin_password)
            .env("RUST_LOG", "info")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start brewlog server");

        // Wait for server to be ready with health check
        let client = reqwest::blocking::Client::new();
        let health_url = format!("{}/api/v1/roasters", address);
        let max_attempts = 30; // 30 seconds total
        let mut attempts = 0;

        while attempts < max_attempts {
            if client.get(&health_url).send().is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            attempts += 1;
        }

        if attempts >= max_attempts {
            panic!("Server failed to start within 30 seconds");
        }

        Self {
            address,
            admin_password: admin_password.to_string(),
            db_url,
            _temp_dir: temp_dir,
            _process: process,
        }
    }

    pub fn create_token(&self, name: &str) -> String {
        use std::io::Write;

        let mut child = Command::new(brewlog_bin())
            .args(&["create-token", "--name", name])
            .env("BREWLOG_SERVER", &self.address)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn create-token command");

        // Write username and password to stdin
        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            writeln!(stdin, "admin").expect("Failed to write username");
            writeln!(stdin, "{}", self.admin_password).expect("Failed to write password");
        }

        let output = child
            .wait_with_output()
            .expect("Failed to wait for command");

        assert!(
            output.status.success(),
            "Failed to create token: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Parse the output to extract the token
        // The token is on the line after "Save this token securely"
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in token output");

        for line in stdout.lines() {
            let trimmed = line.trim();
            // The token line starts with a base64-looking string (long alphanumeric with possible +/=)
            if trimmed.len() > 40 && !trimmed.contains(':') && !trimmed.contains("export") {
                return trimmed.to_string();
            }
        }

        panic!("Could not find token in output:\n{}", stdout);
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Server process will be killed when _process is dropped
    }
}

/// Run a brewlog CLI command and return the output
pub fn run_brewlog(args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(brewlog_bin());
    cmd.args(args);

    for (key, value) in env {
        cmd.env(key, value);
    }

    cmd.output().expect("Failed to run brewlog command")
}

/// Check if output contains expected text
pub fn output_contains(output: &std::process::Output, text: &str) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout.contains(text) || stderr.contains(text)
}
