use crate::helpers::{create_roast, create_roaster, create_token, run_brewlog};
use crate::test_macros::{define_cli_auth_test, define_cli_list_test};
use serde_json::Value;

define_cli_auth_test!(
    test_add_bag_requires_authentication,
    &["bag", "add", "--roast-id", "123", "--amount", "250.0"]
);
define_cli_auth_test!(
    test_update_bag_requires_authentication,
    &["bag", "update", "--id", "123", "--remaining", "100.0"]
);
define_cli_auth_test!(
    test_delete_bag_requires_authentication,
    &["bag", "delete", "--id", "123"]
);
define_cli_list_test!(
    test_list_bags_works_without_authentication,
    &["bag", "list"]
);

#[test]
fn test_add_bag_with_authentication() {
    let token = create_token("test-add-bag");

    // Setup: Create Roaster & Roast
    let roaster_id = create_roaster("Bag Add Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag Add Roast", &token);

    // Test: Add Bag
    let output = run_brewlog(
        &[
            "bag",
            "add",
            "--roast-id",
            &roast_id,
            "--amount",
            "250.0",
            "--roast-date",
            "2023-01-01",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let bag: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(bag["amount"].as_f64(), Some(250.0));
    assert_eq!(bag["roast_date"], "2023-01-01");
    assert!(bag["id"].is_i64());
}

#[test]
fn test_update_bag_with_authentication() {
    let token = create_token("test-update-bag");

    // Setup: Create Roaster & Roast & Bag
    let roaster_id = create_roaster("Bag Update Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag Update Roast", &token);

    let bag_output = run_brewlog(
        &["bag", "add", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap().to_string();

    // Test: Update Bag
    let output = run_brewlog(
        &[
            "bag",
            "update",
            "--id",
            &bag_id,
            "--remaining",
            "150.0",
            "--closed",
            "true",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let updated_bag: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(updated_bag["remaining"].as_f64(), Some(150.0));
    assert_eq!(updated_bag["closed"].as_bool(), Some(true));
}

#[test]
fn test_list_bags_shows_added_bag() {
    let token = create_token("test-list-bags");

    // Setup
    let roaster_id = create_roaster("Bag List Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag List Roast", &token);

    let bag_output = run_brewlog(
        &["bag", "add", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap();

    // Test: List Bags (Authenticated)
    let output = run_brewlog(
        &["bag", "list", "--roast-id", &roast_id],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let bags: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(bags.is_array());
    let bags_array = bags.as_array().unwrap();
    assert!(bags_array.iter().any(|b| b["id"].as_i64() == Some(bag_id)));
}

#[test]
fn test_list_bags_without_roast_id_shows_all_bags() {
    let token = create_token("test-list-all-bags");

    // Setup
    let roaster_id = create_roaster("Bag List All Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag List All Roast", &token);

    let bag_output = run_brewlog(
        &["bag", "add", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap();

    // Test: List Bags (Authenticated)
    let output = run_brewlog(&["bag", "list"], &[("BREWLOG_TOKEN", &token)]);

    assert!(output.status.success());
    let bags: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(bags.is_array());
    let bags_array = bags.as_array().unwrap();
    assert!(bags_array.iter().any(|b| b["id"].as_i64() == Some(bag_id)));
}

#[test]
fn test_list_bags_shows_open_and_closed_bags() {
    let token = create_token("test-list-mixed-bags");

    // Setup
    let roaster_id = create_roaster("Bag List Mixed Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag List Mixed Roast", &token);

    // Create Open Bag
    let open_bag_output = run_brewlog(
        &["bag", "add", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let open_bag: Value = serde_json::from_slice(&open_bag_output.stdout).unwrap();
    let open_bag_id = open_bag["id"].as_i64().unwrap();

    // Create Closed Bag
    let closed_bag_output = run_brewlog(
        &["bag", "add", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let closed_bag: Value = serde_json::from_slice(&closed_bag_output.stdout).unwrap();
    let closed_bag_id = closed_bag["id"].as_i64().unwrap();

    // Close the second bag
    let _ = run_brewlog(
        &[
            "bag",
            "update",
            "--id",
            &closed_bag_id.to_string(),
            "--closed",
            "true",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    // Test: List Bags (Authenticated)
    let output = run_brewlog(&["bag", "list"], &[("BREWLOG_TOKEN", &token)]);

    assert!(output.status.success());
    let bags: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(bags.is_array());
    let bags_array = bags.as_array().unwrap();

    assert!(
        bags_array
            .iter()
            .any(|b| b["id"].as_i64() == Some(open_bag_id)),
        "Open bag should be listed"
    );
    assert!(
        bags_array
            .iter()
            .any(|b| b["id"].as_i64() == Some(closed_bag_id)),
        "Closed bag should be listed"
    );
}

#[test]
fn test_delete_bag_with_authentication() {
    let token = create_token("test-delete-bag");

    // Setup
    let roaster_id = create_roaster("Bag Delete Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag Delete Roast", &token);

    let bag_output = run_brewlog(
        &["bag", "add", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap().to_string();

    // Test: Delete Bag
    let output = run_brewlog(
        &["bag", "delete", "--id", &bag_id],
        &[("BREWLOG_TOKEN", &token)],
    );
    assert!(output.status.success());

    // Verify deletion
    let get_output = run_brewlog(&["bag", "get", "--id", &bag_id], &[]);
    assert!(!get_output.status.success());
}
