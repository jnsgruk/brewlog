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
        
        // Wait for server to be ready
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        Self {
            address,
            admin_password: admin_password.to_string(),
            db_url,
            _temp_dir: temp_dir,
            _process: process,
        }
    }
    
    pub fn create_token(&self, name: &str) -> String {
        let output = Command::new(brewlog_bin())
            .args(&["create-token", "--name", name, "--server", &self.address])
            .env("BREWLOG_ADMIN_PASSWORD", &self.admin_password)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to create token");
        
        assert!(output.status.success(), "Failed to create token: {}", String::from_utf8_lossy(&output.stderr));
        
        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 in token output")
            .trim()
            .to_string()
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
