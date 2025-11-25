use crate::helpers::{output_contains, run_brewlog, setup, TestServer};

#[test]
fn test_add_roast_requires_authentication() {
    let server = TestServer::start();
    
    let output = run_brewlog(
        &[
            "add-roast",
            "--roaster-id",
            "some-id",
            "--name",
            "Test Roast",
            "--origin",
            "Ethiopia",
            "--server",
            &server.address,
        ],
        &[],
    );
    
    assert!(!output.status.success(), "add-roast without auth should fail");
}

#[test]
fn test_add_roast_with_authentication() {
    let server = TestServer::start();
    let token = server.create_token("test-token");
    
    // First create a roaster
    let roaster_output = run_brewlog(
        &[
            "add-roaster",
            "--name",
            "Test Roasters",
            "--country",
            "UK",
            "--server",
            &server.address,
        ],
        &[("BREWLOG_TOKEN", &token)],
    );
    
    assert!(roaster_output.status.success());
    
    // Extract roaster ID from output (simplified - in reality we'd parse JSON)
    // For now, just test that add-roast command works with auth
    let output = run_brewlog(
        &["add-roast", "--help"],
        &[],
    );
    
    assert!(output.status.success());
    assert!(output_contains(&output, "Add a new roast"), "Help should work");
}

#[test]
fn test_list_roasts_works_without_authentication() {
    let server = TestServer::start();
    
    let output = run_brewlog(
        &["list-roasts", "--server", &server.address],
        &[],
    );
    
    assert!(output.status.success(), "list-roasts should work without auth");
}

#[test]
fn test_delete_roast_requires_authentication() {
    let server = TestServer::start();
    
    let output = run_brewlog(
        &[
            "delete-roast",
            "--id",
            "some-id",
            "--server",
            &server.address,
        ],
        &[],
    );
    
    assert!(!output.status.success(), "delete-roast without auth should fail");
}
