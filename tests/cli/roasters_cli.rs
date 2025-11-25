use crate::helpers::{create_token, run_brewlog, server_info};
use serde_json::Value;

#[test]
fn test_add_roaster_requires_authentication() {
    let _ = server_info(); // Ensure server is started

    let output = run_brewlog(
        &["add-roaster", "--name", "Test Roasters", "--country", "UK"],
        &[],
    );

    assert!(
        !output.status.success(),
        "add-roaster without auth should fail"
    );
}

#[test]
fn test_add_roaster_with_authentication() {
    let token = create_token("test-add-roaster");

    let output = run_brewlog(
        &["add-roaster", "--name", "Test Roasters", "--country", "UK"],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "add-roaster with auth should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roaster: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| panic!("Should output valid JSON, got: {}", stdout));

    assert_eq!(roaster["name"], "Test Roasters");
    assert_eq!(roaster["country"], "UK");
    assert!(roaster["id"].is_string(), "Should have an ID");
}

#[test]
fn test_list_roasters_works_without_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["list-roasters"], &[]);

    assert!(
        output.status.success(),
        "list-roasters should work without auth"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roasters: Value = serde_json::from_str(&stdout).expect("Should output valid JSON array");

    assert!(roasters.is_array(), "Should return an array");
}

#[test]
fn test_list_roasters_shows_added_roaster() {
    let token = create_token("test-list-roasters");

    // Add a roaster
    let add_output = run_brewlog(
        &[
            "add-roaster",
            "--name",
            "Example Roasters",
            "--country",
            "USA",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        add_output.status.success(),
        "Failed to add roaster: {}",
        String::from_utf8_lossy(&add_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&add_output.stdout);
    let added_roaster: Value = serde_json::from_str(&stdout).expect("Should output valid JSON");
    let roaster_id = added_roaster["id"].as_str().unwrap();

    // List roasters
    let list_output = run_brewlog(&["list-roasters"], &[]);

    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let roasters: Value =
        serde_json::from_str(&list_stdout).expect("Should output valid JSON array");

    assert!(roasters.is_array(), "Should return an array");
    let roasters_array = roasters.as_array().unwrap();

    // Find our roaster in the list
    let found = roasters_array.iter().any(|r| r["id"] == roaster_id);
    assert!(found, "Should find the added roaster in the list");
}

#[test]
fn test_delete_roaster_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["delete-roaster", "--id", "some-id"], &[]);

    assert!(
        !output.status.success(),
        "delete-roaster without auth should fail"
    );
}

#[test]
fn test_update_roaster_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(
        &[
            "update-roaster",
            "--id",
            "some-id",
            "--name",
            "Updated Name",
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "update-roaster without auth should fail"
    );
}
