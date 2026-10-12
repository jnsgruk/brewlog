use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::helpers::brewlog_bin;

struct TestServer(Child);

impl Drop for TestServer {
    fn drop(&mut self) {
        self.0.kill().expect("Failed to stop test server");
        self.0.wait().expect("Failed to reap test server");
    }
}

#[test]
fn serve_starts_with_optional_foursquare_configuration() {
    for api_key in [None, Some(""), Some("   "), Some("test-api-key")] {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let port = portpicker::pick_unused_port().expect("No ports available");
        let mut command = Command::new(brewlog_bin());
        command
            .env_clear()
            .current_dir(temp_dir.path())
            .args([
                "serve",
                "--bind-address",
                &format!("127.0.0.1:{port}"),
                "--database-url",
                "sqlite://test.db",
                "--openrouter-api-key",
                "test-api-key",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(api_key) = api_key {
            command.env("BREWLOG_FOURSQUARE_API_KEY", api_key);
        }
        let mut server = TestServer(command.spawn().expect("Failed to start test server"));
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(200))
            .build()
            .expect("Failed to build HTTP client");
        let deadline = Instant::now() + Duration::from_secs(10);

        loop {
            assert!(
                server
                    .0
                    .try_wait()
                    .expect("Failed to check test server")
                    .is_none(),
                "Server exited before becoming healthy with Foursquare key {api_key:?}"
            );
            if let Ok(response) = client.get(format!("http://127.0.0.1:{port}/health")).send()
                && response.status().is_success()
            {
                break;
            }
            assert!(Instant::now() < deadline, "Server did not become healthy");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
