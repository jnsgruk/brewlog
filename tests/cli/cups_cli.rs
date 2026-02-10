use crate::helpers::{
    create_cafe, create_cup, create_roast, create_roaster, create_token, run_brewlog,
};
use crate::test_macros::define_cli_auth_test;

define_cli_auth_test!(
    test_update_cup_requires_authentication,
    &["cup", "update", "--id", "123", "--roast-id", "1"]
);

#[test]
fn test_update_cup_with_authentication() {
    let token = create_token("test-update-cup");
    let roaster_id = create_roaster("Update Cup Roaster", &token);
    let roast_id = create_roast(&roaster_id, "Update Cup Roast", &token);
    let cafe_id = create_cafe("Update Cup Cafe", "London", "UK", "51.5", "-0.1", &token);
    let cup_id = create_cup(&roast_id, &cafe_id, &token);

    // Create a second roast to update the cup to
    let roast_id_2 = create_roast(&roaster_id, "Update Cup Roast 2", &token);

    let output = run_brewlog(
        &["cup", "update", "--id", &cup_id, "--roast-id", &roast_id_2],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(
        output.status.success(),
        "cup update should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cup: serde_json::Value = serde_json::from_str(&stdout).expect("Should output valid JSON");
    assert!(cup["id"].is_i64(), "cup should have an id");
}
