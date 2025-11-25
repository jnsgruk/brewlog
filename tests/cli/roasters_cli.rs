use crate::helpers::{TestServer, output_contains, run_brewlog, setup};

#[test]
fn test_add_roaster_requires_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(
        &[
            "add-roaster",
            "--name",
            "Test Roasters",
            "--country",
            "UK",
            "--server",
            &server.address,
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "add-roaster without auth should fail"
    );
}

#[test]
fn test_add_roaster_with_authentication() {
    let server = TestServer::start();
    let token = server.create_token("test-token");

    let output = run_brewlog(
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

    assert!(
        output.status.success(),
        "add-roaster with auth should succeed"
    );
}

#[test]
fn test_list_roasters_works_without_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(&["list-roasters", "--server", &server.address], &[]);

    assert!(
        output.status.success(),
        "list-roasters should work without auth"
    );
}

#[test]
fn test_list_roasters_shows_added_roaster() {
    let server = TestServer::start();
    let token = server.create_token("test-token");

    // Add a roaster
    let add_output = run_brewlog(
        &[
            "add-roaster",
            "--name",
            "Example Roasters",
            "--country",
            "USA",
            "--server",
            &server.address,
        ],
        &[("BREWLOG_TOKEN", &token)],
    );

    assert!(add_output.status.success());

    // List roasters
    let list_output = run_brewlog(&["list-roasters", "--server", &server.address], &[]);

    assert!(list_output.status.success());
    assert!(
        output_contains(&list_output, "Example Roasters"),
        "Should list the added roaster"
    );
}

#[test]
fn test_delete_roaster_requires_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(
        &[
            "delete-roaster",
            "--id",
            "some-id",
            "--server",
            &server.address,
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "delete-roaster without auth should fail"
    );
}

#[test]
fn test_update_roaster_requires_authentication() {
    let server = TestServer::start();

    let output = run_brewlog(
        &[
            "update-roaster",
            "--id",
            "some-id",
            "--name",
            "Updated Name",
            "--server",
            &server.address,
        ],
        &[],
    );

    assert!(
        !output.status.success(),
        "update-roaster without auth should fail"
    );
}
