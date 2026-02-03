use crate::helpers::{create_cafe, create_token, run_brewlog, server_info};
use serde_json::Value;

#[test]
fn test_add_cafe_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(
        &[
            "cafe",
            "add",
            "--name",
            "Test Cafe",
            "--city",
            "London",
            "--country",
            "UK",
            "--latitude",
            "51.5074",
            "--longitude",
            "-0.1278",
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "cafe add without auth should fail"
    );
}

#[test]
fn test_add_cafe_with_authentication() {
    let token = create_token("test-add-cafe");

    let output = run_brewlog(
        &[
            "cafe",
            "add",
            "--name",
            "Test Cafe",
            "--city",
            "London",
            "--country",
            "UK",
            "--latitude",
            "51.5074",
            "--longitude",
            "-0.1278",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "cafe add with auth should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cafe: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| panic!("Should output valid JSON, got: {}", stdout));

    assert_eq!(cafe["name"], "Test Cafe");
    assert_eq!(cafe["city"], "London");
    assert_eq!(cafe["country"], "UK");
    assert!(cafe["id"].is_i64(), "Should have an ID");
}

#[test]
fn test_list_cafes_works_without_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["cafe", "list"], &[]);

    assert!(
        output.status.success(),
        "cafe list should work without auth"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cafes: Value = serde_json::from_str(&stdout).expect("Should output valid JSON array");

    assert!(cafes.is_array(), "Should return an array");
}

#[test]
fn test_list_cafes_shows_added_cafe() {
    let token = create_token("test-list-cafes");

    let cafe_id = create_cafe(
        "List Test Cafe",
        "Bristol",
        "UK",
        "51.4545",
        "-2.5879",
        &token,
    );

    let list_output = run_brewlog(&["cafe", "list"], &[]);

    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let cafes: Value = serde_json::from_str(&list_stdout).expect("Should output valid JSON array");

    assert!(cafes.is_array(), "Should return an array");
    let cafes_array = cafes.as_array().unwrap();

    let found = cafes_array
        .iter()
        .any(|c| c["id"].as_i64().unwrap().to_string() == cafe_id);
    assert!(found, "Should find the added cafe in the list");
}

#[test]
fn test_delete_cafe_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["cafe", "delete", "--id", "some-id"], &[]);

    assert!(
        !output.status.success(),
        "cafe delete without auth should fail"
    );
}

#[test]
fn test_update_cafe_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(
        &[
            "cafe",
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
        "cafe update without auth should fail"
    );
}
