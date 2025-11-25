use crate::helpers::{create_token, run_brewlog, server_info};
use serde_json::Value;

#[test]
fn test_add_roast_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(
        &[
            "add-roast",
            "--roaster-id",
            "some-id",
            "--name",
            "Test Roast",
            "--origin",
            "Ethiopia",
            "--region",
            "Yirgacheffe",
            "--producer",
            "Local Coop",
            "--process",
            "Washed",
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "add-roast without auth should fail"
    );
}

#[test]
fn test_add_roast_with_authentication() {
    let token = create_token("test-add-roast");

    // First create a roaster
    let roaster_output = run_brewlog(
        &["add-roaster", "--name", "Test Roasters", "--country", "UK"],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(roaster_output.status.success());

    let roaster_stdout = String::from_utf8_lossy(&roaster_output.stdout);
    let roaster: Value = serde_json::from_str(&roaster_stdout).expect("Should output valid JSON");
    let roaster_id = roaster["id"].as_str().unwrap();

    // Now add a roast
    let output = run_brewlog(
        &[
            "add-roast",
            "--roaster-id",
            roaster_id,
            "--name",
            "Ethiopian Yirgacheffe",
            "--origin",
            "Ethiopia",
            "--region",
            "Yirgacheffe",
            "--producer",
            "Local Coop",
            "--process",
            "Washed",
            "--tasting-notes",
            "Blueberry,Chocolate",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "add-roast with auth should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roast: Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    assert_eq!(roast["name"], "Ethiopian Yirgacheffe");
    assert_eq!(roast["roaster_id"], roaster_id);
    assert!(roast["id"].is_string(), "Should have an ID");
}

#[test]
fn test_list_roasts_works_without_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["list-roasts"], &[]);

    assert!(
        output.status.success(),
        "list-roasts should work without auth"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roasts: Value = serde_json::from_str(&stdout).expect("Should output valid JSON array");

    assert!(roasts.is_array(), "Should return an array");
}

#[test]
fn test_list_roasts_shows_added_roast() {
    let token = create_token("test-list-roasts");

    // First create a roaster
    let roaster_output = run_brewlog(
        &["add-roaster", "--name", "Test Roasters", "--country", "UK"],
        &[("BREWLOG_TOKEN", &token)],
    );

    let roaster_stdout = String::from_utf8_lossy(&roaster_output.stdout);
    let roaster: Value = serde_json::from_str(&roaster_stdout).unwrap();
    let roaster_id = roaster["id"].as_str().unwrap();

    // Add a roast
    let add_output = run_brewlog(
        &[
            "add-roast",
            "--roaster-id",
            roaster_id,
            "--name",
            "Colombian Supremo",
            "--origin",
            "Colombia",
            "--region",
            "Huila",
            "--producer",
            "Farm Co-op",
            "--process",
            "Natural",
            "--tasting-notes",
            "Caramel,Nuts",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(add_output.status.success());

    let stdout = String::from_utf8_lossy(&add_output.stdout);
    let added_roast: Value = serde_json::from_str(&stdout).unwrap();
    let roast_id = added_roast["id"].as_str().unwrap();

    // List roasts
    let list_output = run_brewlog(&["list-roasts"], &[]);

    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let roasts: Value = serde_json::from_str(&list_stdout).unwrap();

    assert!(roasts.is_array());
    let roasts_array = roasts.as_array().unwrap();

    // Find our roast in the list (note: list returns RoastWithRoaster which has nested structure)
    let found = roasts_array
        .iter()
        .any(|item| item["roast"]["id"] == roast_id);
    assert!(
        found,
        "Should find the added roast in the list. Looking for id={}, found {} roasts",
        roast_id,
        roasts_array.len()
    );
}

#[test]
fn test_delete_roast_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["delete-roast", "--id", "some-id"], &[]);

    assert!(
        !output.status.success(),
        "delete-roast without auth should fail"
    );
}
