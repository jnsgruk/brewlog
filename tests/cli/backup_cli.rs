use super::helpers::{create_token, run_brewlog};

#[test]
fn backup_produces_valid_json() {
    let token = create_token("backup-test");

    let output = run_brewlog(&["backup"], &[("BREWLOG_TOKEN", &token)]);

    assert!(
        output.status.success(),
        "backup command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value =
        serde_json::from_str(&stdout).expect("backup output is not valid JSON");

    assert_eq!(data["version"], 1);
    assert!(data["roasters"].is_array());
    assert!(data["roasts"].is_array());
    assert!(data["bags"].is_array());
    assert!(data["gear"].is_array());
    assert!(data["brews"].is_array());
    assert!(data["cafes"].is_array());
    assert!(data["timeline_events"].is_array());
}

#[test]
fn backup_requires_auth() {
    let output = run_brewlog(&["backup"], &[]);

    assert!(
        !output.status.success(),
        "backup command should fail without auth"
    );
}
