use crate::helpers::{create_roast, create_roaster, create_token, run_brewlog};
use crate::test_macros::{define_cli_auth_test, define_cli_list_test};
use serde_json::Value;

define_cli_auth_test!(
    test_add_roast_requires_authentication,
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
        "Washed"
    ]
);
define_cli_auth_test!(
    test_update_roast_requires_authentication,
    &["roast", "update", "--id", "123", "--name", "Updated"]
);
define_cli_auth_test!(
    test_delete_roast_requires_authentication,
    &["roast", "delete", "--id", "some-id"]
);
define_cli_list_test!(
    test_list_roasts_works_without_authentication,
    &["roast", "list"]
);

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
fn test_update_roast_with_authentication() {
    let token = create_token("test-update-roast");

    // Setup: create roaster and roast
    let roaster_id = create_roaster("Update Roast Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Original Name", &token);

    // Update the roast
    let output = run_brewlog(
        &[
            "roast",
            "update",
            "--id",
            &roast_id,
            "--name",
            "Updated Name",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let updated_roast: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(updated_roast["name"], "Updated Name");
}
