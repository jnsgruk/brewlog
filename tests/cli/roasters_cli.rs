use crate::helpers::{create_roaster, create_token, run_brewlog, server_info};
use serde_json::Value;

#[test]
fn test_add_roaster_requires_authentication() {
    let _ = server_info(); // Ensure server is started

    let output = run_brewlog(
        &[
            "roaster",
            "add",
            "--name",
            "Test Roasters",
            "--country",
            "UK",
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "roaster add without auth should fail"
    );
}

#[test]
fn test_add_roaster_with_authentication() {
    let token = create_token("test-add-roaster");

    let output = run_brewlog(
        &[
            "roaster",
            "add",
            "--name",
            "Test Roasters",
            "--country",
            "UK",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "roaster add with auth should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roaster: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| panic!("Should output valid JSON, got: {}", stdout));

    assert_eq!(roaster["name"], "Test Roasters");
    assert_eq!(roaster["country"], "UK");
    assert!(roaster["id"].is_i64(), "Should have an ID");
}

#[test]
fn test_list_roasters_works_without_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["roaster", "list"], &[]);

    assert!(
        output.status.success(),
        "roaster list should work without auth"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roasters: Value = serde_json::from_str(&stdout).expect("Should output valid JSON array");

    assert!(roasters.is_array(), "Should return an array");
}

#[test]
fn test_list_roasters_shows_added_roaster() {
    let token = create_token("test-list-roasters");

    // Add a roaster
    let roaster_id = create_roaster("Example Roasters", &token);

    // List roasters
    let list_output = run_brewlog(&["roaster", "list"], &[]);

    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let roasters: Value =
        serde_json::from_str(&list_stdout).expect("Should output valid JSON array");

    assert!(roasters.is_array(), "Should return an array");
    let roasters_array = roasters.as_array().unwrap();

    // Find our roaster in the list
    let found = roasters_array
        .iter()
        .any(|r| r["id"].as_i64().unwrap().to_string() == roaster_id);
    assert!(found, "Should find the added roaster in the list");
}

#[test]
fn test_delete_roaster_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["roaster", "delete", "--id", "some-id"], &[]);

    assert!(
        !output.status.success(),
        "roaster delete without auth should fail"
    );
}

#[test]
fn test_update_roaster_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(
        &[
            "roaster",
            "update",
            "--id",
            "some-id",
            "--name",
            "Updated Name",
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "roaster update without auth should fail"
    );
}
