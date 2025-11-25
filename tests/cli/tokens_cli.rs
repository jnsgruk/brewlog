use crate::helpers::{create_token, run_brewlog, server_info};

// Note: create-token CLI command tests are omitted due to stdin handling complexity.
// Token creation for testing is done via API in the create_token() helper.

#[test]
fn test_list_tokens_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["list-tokens"], &[]);

    assert!(
        !output.status.success(),
        "list-tokens without auth should fail"
    );
}

#[test]
fn test_list_tokens_with_authentication() {
    let token = create_token("test-list-tokens");

    let output = run_brewlog(&["list-tokens"], &[("BREWLOG_TOKEN", &token)]);

    assert!(
        output.status.success(),
        "list-tokens with auth should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test-list-tokens"),
        "Should list the created token"
    );
}

#[test]
fn test_revoke_token_requires_authentication() {
    let _ = server_info();

    let output = run_brewlog(&["revoke-token", "--id", "1"], &[]);

    assert!(
        !output.status.success(),
        "revoke-token without auth should fail"
    );
}

#[test]
fn test_revoke_token_with_authentication() {
    let token = create_token("test-revoke-token");

    // List tokens to get the ID
    let list_output = run_brewlog(&["list-tokens"], &[("BREWLOG_TOKEN", &token)]);
    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let tokens: serde_json::Value =
        serde_json::from_str(&list_stdout).expect("Should parse token list as JSON");

    // Find a token to revoke
    let tokens_array = tokens.as_array().expect("Should be an array");
    if let Some(first_token) = tokens_array.first() {
        let token_id = first_token["id"].as_i64().expect("Token should have ID");

        let revoke_output = run_brewlog(
            &["revoke-token", "--id", &token_id.to_string()],
            &[("BREWLOG_TOKEN", &token)],
        );

        assert!(
            revoke_output.status.success(),
            "Should be able to revoke token"
        );
    }
}

#[test]
fn test_revoked_token_cannot_be_used() {
    // Create a token that we will revoke
    let token_to_revoke = create_token("test-revoked-token");

    // Create a second token that we'll use to revoke the first and verify
    let admin_token = create_token("test-admin-token");

    // List tokens to get the ID of the token we want to revoke
    let list_output = run_brewlog(&["list-tokens"], &[("BREWLOG_TOKEN", &admin_token)]);
    assert!(list_output.status.success());

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let tokens: serde_json::Value =
        serde_json::from_str(&list_stdout).expect("Should parse token list as JSON");

    // Find the token to revoke by name
    let tokens_array = tokens.as_array().expect("Should be an array");
    let token_to_revoke_entry = tokens_array
        .iter()
        .find(|t| t["name"].as_str() == Some("test-revoked-token"))
        .expect("Should find token to revoke");

    let token_id = token_to_revoke_entry["id"]
        .as_i64()
        .expect("Token should have ID");

    // Revoke the token
    let revoke_output = run_brewlog(
        &["revoke-token", "--id", &token_id.to_string()],
        &[("BREWLOG_TOKEN", &admin_token)],
    );
    assert!(
        revoke_output.status.success(),
        "Should successfully revoke token"
    );

    // Try to use the revoked token - it should fail
    let list_with_revoked_output =
        run_brewlog(&["list-tokens"], &[("BREWLOG_TOKEN", &token_to_revoke)]);

    assert!(
        !list_with_revoked_output.status.success(),
        "Revoked token should not be able to authenticate"
    );

    let stderr = String::from_utf8_lossy(&list_with_revoked_output.stderr);
    assert!(
        stderr.contains("401") || stderr.contains("Unauthorized") || stderr.contains("failed"),
        "Error should indicate authentication failure, got: {}",
        stderr
    );
}
