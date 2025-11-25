use crate::helpers::{brewlog_bin, create_token, run_brewlog, server_info};

#[test]
fn test_create_token_without_server_fails() {
    use std::io::Write;
    use std::process::Stdio;

    // Don't start server, use invalid address
    let mut child = std::process::Command::new(brewlog_bin())
        .args(&["create-token", "--name", "test-token"])
        .env("BREWLOG_SERVER", "http://localhost:9999")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Provide dummy credentials
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = writeln!(stdin, "admin");
            let _ = writeln!(stdin, "password");
        }
    }

    let output = child.wait_with_output().expect("Failed to get output");
    assert!(
        !output.status.success(),
        "Should fail when server is unreachable"
    );
}

#[test]
fn test_create_token_with_valid_credentials() {
    let token = create_token("test-create-token");
    assert!(!token.is_empty(), "Should create a non-empty token");
    assert!(token.len() > 40, "Token should be reasonably long");
}

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

    let output = run_brewlog(&["revoke-token", "--id", "some-id"], &[]);

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
        let token_id = first_token["id"].as_str().expect("Token should have ID");

        let revoke_output = run_brewlog(
            &["revoke-token", "--id", token_id],
            &[("BREWLOG_TOKEN", &token)],
        );

        assert!(
            revoke_output.status.success(),
            "Should be able to revoke token"
        );
    }
}
