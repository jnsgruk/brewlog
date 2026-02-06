use crate::helpers::{create_token, run_brewlog};
use crate::test_macros::{define_cli_auth_test, define_cli_list_test};
use serde_json::Value;

define_cli_auth_test!(
    test_add_gear_requires_authentication,
    &[
        "gear",
        "add",
        "--category",
        "grinder",
        "--make",
        "Baratza",
        "--model",
        "Encore"
    ]
);
define_cli_auth_test!(
    test_update_gear_requires_authentication,
    &["gear", "update", "--id", "123", "--make", "Updated"]
);
define_cli_auth_test!(
    test_delete_gear_requires_authentication,
    &["gear", "delete", "--id", "123"]
);
define_cli_list_test!(
    test_list_gear_works_without_authentication,
    &["gear", "list"]
);

#[test]
fn test_add_gear_with_authentication() {
    let token = create_token("test-add-gear");

    let output = run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "grinder",
            "--make",
            "Baratza",
            "--model",
            "Encore",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let gear: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(gear["make"], "Baratza");
    assert_eq!(gear["model"], "Encore");
    assert_eq!(gear["category"], "grinder");
    assert!(gear["id"].is_i64());
}

#[test]
fn test_list_gear_shows_added_gear() {
    let token = create_token("test-list-gear");

    // Add gear
    let add_output = run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "grinder",
            "--make",
            "Timemore",
            "--model",
            "C2",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );
    assert!(add_output.status.success());

    // List gear
    let list_output = run_brewlog(&["gear", "list"], &[]);
    assert!(list_output.status.success());

    let gear_list: Value = serde_json::from_slice(&list_output.stdout).unwrap();
    let gear_array = gear_list.as_array().unwrap();

    assert!(
        gear_array
            .iter()
            .any(|g| g["make"] == "Timemore" && g["model"] == "C2"),
        "Added gear should appear in list"
    );
}

#[test]
fn test_list_gear_filtered_by_category() {
    let token = create_token("test-list-gear-filter");

    // Add grinder
    run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "grinder",
            "--make",
            "Fellow",
            "--model",
            "Ode",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    // Add brewer
    run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "brewer",
            "--make",
            "Kalita",
            "--model",
            "Wave",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    // List only grinders
    let output = run_brewlog(&["gear", "list", "--category", "grinder"], &[]);
    assert!(output.status.success());

    let gear_list: Value = serde_json::from_slice(&output.stdout).unwrap();
    let gear_array = gear_list.as_array().unwrap();

    // All items should be grinders
    for gear in gear_array {
        assert_eq!(gear["category"], "grinder");
    }

    // Should contain our Fellow Ode
    assert!(gear_array.iter().any(|g| g["make"] == "Fellow"));
}

#[test]
fn test_get_gear_by_id() {
    let token = create_token("test-get-gear");

    // Add gear
    let add_output = run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "grinder",
            "--make",
            "Weber",
            "--model",
            "EG-1",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );
    let gear: Value = serde_json::from_slice(&add_output.stdout).unwrap();
    let gear_id = gear["id"].as_i64().unwrap().to_string();

    // Get gear by ID
    let output = run_brewlog(&["gear", "get", "--id", &gear_id], &[]);
    assert!(output.status.success());

    let retrieved_gear: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(retrieved_gear["make"], "Weber");
    assert_eq!(retrieved_gear["model"], "EG-1");
}

#[test]
fn test_update_gear_with_authentication() {
    let token = create_token("test-update-gear");

    // Add gear
    let add_output = run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "grinder",
            "--make",
            "Porlex",
            "--model",
            "Mini",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );
    let gear: Value = serde_json::from_slice(&add_output.stdout).unwrap();
    let gear_id = gear["id"].as_i64().unwrap().to_string();

    // Update gear
    let output = run_brewlog(
        &["gear", "update", "--id", &gear_id, "--model", "Mini II"],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(output.status.success());
    let updated_gear: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(updated_gear["make"], "Porlex");
    assert_eq!(updated_gear["model"], "Mini II");
}

#[test]
fn test_delete_gear_with_authentication() {
    let token = create_token("test-delete-gear");

    // Add gear
    let add_output = run_brewlog(
        &[
            "gear",
            "add",
            "--category",
            "grinder",
            "--make",
            "Niche",
            "--model",
            "Zero",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );
    let gear: Value = serde_json::from_slice(&add_output.stdout).unwrap();
    let gear_id = gear["id"].as_i64().unwrap().to_string();

    // Delete gear
    let delete_output = run_brewlog(
        &["gear", "delete", "--id", &gear_id],
        &[("BREWLOG_TOKEN", &token)],
    );
    assert!(delete_output.status.success());

    // Verify deletion - get should fail
    let get_output = run_brewlog(&["gear", "get", "--id", &gear_id], &[]);
    assert!(!get_output.status.success());
}
