use crate::helpers::{create_roaster, create_token, run_brewlog, server_info};
use serde_json::Value;

#[test]
fn test_add_roast_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(
        &[
            "roast",
            "add",
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
        "roast add without auth should fail"
    );
}

#[test]
fn test_add_roast_with_authentication() {
    let token = create_token("test-add-roast");

    // First create a roaster
    let roaster_id = create_roaster("Test Roasters Add", &token);

    // Now add a roast
    let output = run_brewlog(
        &[
            "roast",
            "add",
            "--roaster-id",
            &roaster_id,
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
        "roast add with auth should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roast: Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    assert_eq!(roast["name"], "Ethiopian Yirgacheffe");
    assert_eq!(
        roast["roaster_id"].as_i64().unwrap().to_string(),
        roaster_id
    );
    assert!(roast["id"].is_i64(), "Should have an ID");
}

#[test]
fn test_list_roasts_works_without_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["roast", "list"], &[]);

    assert!(
        output.status.success(),
        "roast list should work without auth"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let roasts: Value = serde_json::from_str(&stdout).expect("Should output valid JSON array");

    assert!(roasts.is_array(), "Should return an array");
}

#[test]
fn test_list_roasts_shows_added_roast() {
    let token = create_token("test-list-roasts");

    // First create a roaster
    let roaster_id = create_roaster("Test Roasters List", &token);

    // Add a roast
    let add_output = run_brewlog(
        &[
            "roast",
            "add",
            "--roaster-id",
            &roaster_id,
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
    let roast_id = added_roast["id"]
        .as_i64()
        .expect("roast id should be numeric");

    // List roasts
    let list_output = run_brewlog(&["roast", "list"], &[]);

    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let roasts: Value = serde_json::from_str(&list_stdout).unwrap();

    assert!(roasts.is_array());
    let roasts_array = roasts.as_array().unwrap();

    // Find our roast in the list
    let found = roasts_array
        .iter()
        .any(|item| item["id"].as_i64() == Some(roast_id));
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

    let output = run_brewlog(&["roast", "delete", "--id", "some-id"], &[]);

    assert!(
        !output.status.success(),
        "roast delete without auth should fail"
    );
}
