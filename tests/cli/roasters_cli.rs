use crate::helpers::{TestServer, run_brewlog};
use serde_json::Value;

#[test]
fn test_add_roaster_requires_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(
        &["add-roaster", "--name", "Test Roasters", "--country", "UK"],
        &[("BREWLOG_SERVER", &server.address)],
    );

    assert!(
        !output.status.success(),
        "add-roaster without auth should fail"
    );
}

#[test]
fn test_add_roaster_with_authentication() {
    let server = TestServer::start();
    let token = server.create_token("test-token");

    let output = run_brewlog(
        &["add-roaster", "--name", "Test Roasters", "--country", "UK"],
        &[
            ("BREWLOG_TOKEN", &token),
            ("BREWLOG_SERVER", &server.address),
        ],
    );

    assert!(
        output.status.success(),
        "add-roaster with auth should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse the JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let roaster: Value =
        serde_json::from_str(&stdout).expect(&format!("Should output valid JSON, got: {}", stdout));

    assert_eq!(roaster["name"], "Test Roasters");
    assert_eq!(roaster["country"], "UK");
    assert!(roaster["id"].is_string(), "Should have an ID");
}

#[test]
fn test_list_roasters_works_without_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(&["list-roasters"], &[("BREWLOG_SERVER", &server.address)]);

    assert!(
        output.status.success(),
        "list-roasters should work without auth"
    );

    // Parse the JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let roasters: Value = serde_json::from_str(&stdout).expect("Should output valid JSON array");

    assert!(roasters.is_array(), "Should return an array");
}

#[test]
fn test_list_roasters_shows_added_roaster() {
    let server = TestServer::start();
    let token = server.create_token("test-token");

    // Add a roaster
    let add_output = run_brewlog(
        &[
            "add-roaster",
            "--name",
            "Example Roasters",
            "--country",
            "USA",
        ],
        &[
            ("BREWLOG_TOKEN", &token),
            ("BREWLOG_SERVER", &server.address),
        ],
    );

    assert!(
        add_output.status.success(),
        "Failed to add roaster: {}",
        String::from_utf8_lossy(&add_output.stderr)
    );

    // Parse the added roaster
    let stdout = String::from_utf8_lossy(&add_output.stdout);
    let added_roaster: Value = serde_json::from_str(&stdout).expect("Should output valid JSON");
    let roaster_id = added_roaster["id"].as_str().unwrap();

    // List roasters
    let list_output = run_brewlog(&["list-roasters"], &[("BREWLOG_SERVER", &server.address)]);

    assert!(list_output.status.success());

    // Parse and verify the list
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let roasters: Value =
        serde_json::from_str(&list_stdout).expect("Should output valid JSON array");

    assert!(roasters.is_array(), "Should return an array");
    let roasters_array = roasters.as_array().unwrap();
    assert_eq!(roasters_array.len(), 1, "Should have exactly one roaster");

    let listed_roaster = &roasters_array[0];
    assert_eq!(listed_roaster["id"], roaster_id);
    assert_eq!(listed_roaster["name"], "Example Roasters");
    assert_eq!(listed_roaster["country"], "USA");
}

#[test]
fn test_delete_roaster_requires_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(
        &["delete-roaster", "--id", "some-id"],
        &[("BREWLOG_SERVER", &server.address)],
    );

    assert!(
        !output.status.success(),
        "delete-roaster without auth should fail"
    );
}

#[test]
fn test_update_roaster_requires_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(
        &[
            "update-roaster",
            "--id",
            "some-id",
            "--name",
            "Updated Name",
        ],
        &[("BREWLOG_SERVER", &server.address)],
    );

    assert!(
        !output.status.success(),
        "update-roaster without auth should fail"
    );
}
