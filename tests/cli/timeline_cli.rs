use crate::helpers::{create_roast, create_roaster, create_token, run_brewlog, server_info};

#[test]
fn test_editing_roaster_updates_timeline_events_after_rebuild() {
    let token = create_token("test-timeline-rebuild");
    let (address, _) = server_info();

    // Create a roaster and roast
    let original_name = "Timeline CLI Roasters";
    let roaster_id = create_roaster(original_name, &token);
    let _roast_id = create_roast(&roaster_id, "Timeline CLI Roast", &token);

    // Allow timeline events to be created
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Verify original roaster name appears in timeline HTML
    let client = reqwest::blocking::Client::new();
    let body = client
        .get(format!("{address}/timeline"))
        .send()
        .expect("failed to fetch timeline")
        .text()
        .expect("failed to read body");
    assert!(
        body.contains(original_name),
        "Expected original roaster name in timeline before edit"
    );

    // Update the roaster name via CLI
    let updated_name = "Renamed CLI Roasters";
    let output = run_brewlog(
        &[
            "roaster",
            "update",
            "--id",
            &roaster_id,
            "--name",
            updated_name,
        ],
        &[("BREWLOG_TOKEN", &token)],
    );
    assert!(
        output.status.success(),
        "roaster update should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Trigger timeline rebuild via CLI
    let output = run_brewlog(&["timeline", "rebuild"], &[("BREWLOG_TOKEN", &token)]);
    assert!(
        output.status.success(),
        "timeline rebuild should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Wait for background rebuild task to process
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify both timeline events now show the updated roaster name
    let body = client
        .get(format!("{address}/timeline"))
        .send()
        .expect("failed to fetch timeline after rebuild")
        .text()
        .expect("failed to read body");

    assert!(
        body.contains(updated_name),
        "Expected updated roaster name '{updated_name}' in timeline after rebuild, got: {body}"
    );
    assert!(
        !body.contains(original_name),
        "Expected original roaster name '{original_name}' to no longer appear in timeline, got: {body}"
    );
}
