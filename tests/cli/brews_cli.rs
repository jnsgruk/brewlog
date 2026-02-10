use crate::helpers::{
    create_bag, create_brew, create_gear, create_roast, create_roaster, create_token, run_brewlog,
};
use crate::test_macros::define_cli_auth_test;

#[test]
fn brew_add_creates_brew_via_api() {
    let token = create_token("brew-add-test");
    let roaster_id = create_roaster("Brew Test Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Brew Test Roast", &token);
    let bag_id = create_bag(&roast_id, &token);
    let grinder_id = create_gear("grinder", "Comandante", "C40", &token);
    let brewer_id = create_gear("brewer", "Hario", "V60", &token);

    let output = run_brewlog(
        &[
            "brew",
            "add",
            "--bag-id",
            &bag_id,
            "--coffee-weight",
            "15.0",
            "--grinder-id",
            &grinder_id,
            "--grind-setting",
            "24.0",
            "--brewer-id",
            &brewer_id,
            "--water-volume",
            "250",
            "--water-temp",
            "92.0",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "brew add should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brew: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    assert_eq!(brew["coffee_weight"], 15.0);
    assert_eq!(brew["grind_setting"], 24.0);
    assert_eq!(brew["water_volume"], 250);
    assert_eq!(brew["water_temp"], 92.0);
}

#[test]
fn brew_add_with_defaults_uses_default_values() {
    let token = create_token("brew-defaults-test");
    let roaster_id = create_roaster("Brew Defaults Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Brew Defaults Roast", &token);
    let bag_id = create_bag(&roast_id, &token);
    let grinder_id = create_gear("grinder", "1Zpresso", "JMax", &token);
    let brewer_id = create_gear("brewer", "AeroPress", "Original", &token);

    // Only provide required args, let defaults apply
    let output = run_brewlog(
        &[
            "brew",
            "add",
            "--bag-id",
            &bag_id,
            "--grinder-id",
            &grinder_id,
            "--brewer-id",
            &brewer_id,
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "brew add should succeed with defaults: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brew: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    // Check defaults: 15.0g coffee, 6.0 grind, 250ml water, 91.0°C
    assert_eq!(brew["coffee_weight"], 15.0);
    assert_eq!(brew["grind_setting"], 6.0);
    assert_eq!(brew["water_volume"], 250);
    assert_eq!(brew["water_temp"], 91.0);
}

#[test]
fn brew_list_returns_json_array() {
    let token = create_token("brew-list-test");
    let roaster_id = create_roaster("Brew List Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Brew List Roast", &token);
    let bag_id = create_bag(&roast_id, &token);
    let grinder_id = create_gear("grinder", "Fellow", "Ode", &token);
    let brewer_id = create_gear("brewer", "Chemex", "Classic", &token);

    // Create a brew first
    run_brewlog(
        &[
            "brew",
            "add",
            "--bag-id",
            &bag_id,
            "--grinder-id",
            &grinder_id,
            "--brewer-id",
            &brewer_id,
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    // List brews
    let output = run_brewlog(&["brew", "list"], &[("BREWLOG_TOKEN", &token)]);

    assert!(
        output.status.success(),
        "brew list should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brews: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    assert!(brews.is_array(), "brew list should return an array");
    assert!(
        !brews.as_array().unwrap().is_empty(),
        "should have at least one brew"
    );
}

#[test]
fn brew_get_returns_brew_details() {
    let token = create_token("brew-get-test");
    let roaster_id = create_roaster("Brew Get Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Brew Get Roast", &token);
    let bag_id = create_bag(&roast_id, &token);
    let grinder_id = create_gear("grinder", "Baratza", "Encore", &token);
    let brewer_id = create_gear("brewer", "Kalita", "Wave", &token);

    // Create a brew
    let create_output = run_brewlog(
        &[
            "brew",
            "add",
            "--bag-id",
            &bag_id,
            "--grinder-id",
            &grinder_id,
            "--brewer-id",
            &brewer_id,
            "--coffee-weight",
            "18.5",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    let create_stdout = String::from_utf8_lossy(&create_output.stdout);
    let created_brew: serde_json::Value =
        serde_json::from_str(&create_stdout).expect("Should output valid JSON");
    let brew_id = created_brew["id"].as_i64().unwrap().to_string();

    // Get the brew
    let output = run_brewlog(
        &["brew", "get", "--id", &brew_id],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "brew get should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brew: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    assert_eq!(brew["coffee_weight"], 18.5);
    assert!(
        brew["roast_name"].is_string(),
        "should include enriched roast_name"
    );
}

#[test]
fn brew_delete_removes_brew() {
    let token = create_token("brew-delete-test");
    let roaster_id = create_roaster("Brew Delete Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Brew Delete Roast", &token);
    let bag_id = create_bag(&roast_id, &token);
    let grinder_id = create_gear("grinder", "Timemore", "C2", &token);
    let brewer_id = create_gear("brewer", "Melitta", "Pour Over", &token);

    // Create a brew
    let create_output = run_brewlog(
        &[
            "brew",
            "add",
            "--bag-id",
            &bag_id,
            "--grinder-id",
            &grinder_id,
            "--brewer-id",
            &brewer_id,
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    let create_stdout = String::from_utf8_lossy(&create_output.stdout);
    let created_brew: serde_json::Value =
        serde_json::from_str(&create_stdout).expect("Should output valid JSON");
    let brew_id = created_brew["id"].as_i64().unwrap().to_string();

    // Delete the brew
    let output = run_brewlog(
        &["brew", "delete", "--id", &brew_id],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "brew delete should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify it's gone
    let get_output = run_brewlog(
        &["brew", "get", "--id", &brew_id],
        &[("BREWLOG_TOKEN", &token)],
    );
    assert!(
        !get_output.status.success(),
        "brew get should fail after deletion"
    );
}

define_cli_auth_test!(
    test_update_brew_requires_authentication,
    &["brew", "update", "--id", "123", "--coffee-weight", "16.0"]
);

#[test]
fn test_update_brew_with_authentication() {
    let token = create_token("test-update-brew");
    let roaster_id = create_roaster("Update Brew Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Update Brew Roast", &token);
    let bag_id = create_bag(&roast_id, &token);
    let grinder_id = create_gear("grinder", "Weber", "Key", &token);
    let brewer_id = create_gear("brewer", "Origami", "Air", &token);
    let brew_id = create_brew(&bag_id, &grinder_id, &brewer_id, &token);

    let output = run_brewlog(
        &[
            "brew",
            "update",
            "--id",
            &brew_id,
            "--coffee-weight",
            "16.0",
            "--water-temp",
            "94.0",
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "brew update should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brew: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");

    assert_eq!(brew["coffee_weight"], 16.0);
    assert_eq!(brew["water_temp"], 94.0);
}
