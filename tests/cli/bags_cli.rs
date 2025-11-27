use crate::helpers::{create_roast, create_roaster, create_token, run_brewlog, server_info};
use serde_json::Value;

#[test]
fn test_add_bag_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["add-bag", "--roast-id", "123", "--amount", "250.0"], &[]);

    assert!(!output.status.success(), "add-bag without auth should fail");
}

#[test]
fn test_add_bag_with_authentication() {
    let token = create_token("test-add-bag");

    // Setup: Create Roaster & Roast
    let roaster_id = create_roaster("Bag Add Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag Add Roast", &token);

    // Test: Add Bag
    let output = run_brewlog(
        &[
            "add-bag",
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
fn test_update_bag_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["update-bag", "--id", "123", "--remaining", "100.0"], &[]);

    assert!(
        !output.status.success(),
        "update-bag without auth should fail"
    );
}

#[test]
fn test_update_bag_with_authentication() {
    let token = create_token("test-update-bag");

    // Setup: Create Roaster & Roast & Bag
    let roaster_id = create_roaster("Bag Update Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag Update Roast", &token);

    let bag_output = run_brewlog(
        &["add-bag", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap().to_string();

    // Test: Update Bag
    let output = run_brewlog(
        &[
            "update-bag",
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
fn test_list_bags_works_without_authentication() {
    let _ = server_info();
    // Listing without roast_id might return all or empty, but should succeed (200 OK)
    let output = run_brewlog(&["list-bags"], &[]);
    assert!(output.status.success());
}

#[test]
fn test_list_bags_shows_added_bag() {
    let token = create_token("test-list-bags");

    // Setup
    let roaster_id = create_roaster("Bag List Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag List Roast", &token);

    let bag_output = run_brewlog(
        &["add-bag", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap();

    // Test: List Bags (Authenticated)
    let output = run_brewlog(
        &["list-bags", "--roast-id", &roast_id],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let bags: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(bags.is_array());
    let bags_array = bags.as_array().unwrap();
    assert!(bags_array.iter().any(|b| b["id"].as_i64() == Some(bag_id)));
}

#[test]
fn test_delete_bag_requires_authentication() {
    let _ = server_info();
    let output = run_brewlog(&["delete-bag", "--id", "123"], &[]);
    assert!(!output.status.success());
}

#[test]
fn test_delete_bag_with_authentication() {
    let token = create_token("test-delete-bag");

    // Setup
    let roaster_id = create_roaster("Bag Delete Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Bag Delete Roast", &token);

    let bag_output = run_brewlog(
        &["add-bag", "--roast-id", &roast_id, "--amount", "250.0"],
        &[("BREWLOG_TOKEN", &token)],
    );
    let bag: Value = serde_json::from_slice(&bag_output.stdout).unwrap();
    let bag_id = bag["id"].as_i64().unwrap().to_string();

    // Test: Delete Bag
    let output = run_brewlog(
        &["delete-bag", "--id", &bag_id],
        &[("BREWLOG_TOKEN", &token)],
    );
    assert!(output.status.success());

    // Verify deletion
    let get_output = run_brewlog(&["get-bag", "--id", &bag_id], &[]);
    assert!(!get_output.status.success());
}
