use std::path::PathBuf;

use super::*;

fn linux() -> IdentitySource {
    IdentitySource::Files(vec![
        PathBuf::from("/etc/machine-id"),
        PathBuf::from("/var/lib/dbus/machine-id"),
    ])
}

#[test]
fn each_source_says_what_the_node_identity_is_checked_against() {
    assert_eq!(
        node_identity_line(&linux()),
        "node identity: checked (Linux machine id)"
    );
    assert_eq!(
        node_identity_line(&IdentitySource::HardwareUuid),
        "node identity: checked (macOS hardware UUID)"
    );
    assert_eq!(
        node_identity_line(&IdentitySource::MachineGuid),
        "node identity: checked (Windows MachineGuid)"
    );
    assert_eq!(
        node_identity_line(&IdentitySource::Unsupported),
        "node identity: not checked on this OS"
    );
}

#[test]
fn the_fork_detection_line_says_what_is_local_and_what_waits_for_the_peer_network() {
    assert_eq!(
        FORK_DETECTION_LINE,
        "fork detection: local scan by parinn verify; peer detection with the peer network"
    );
}

#[test]
fn the_json_carries_the_same_facts() {
    assert_eq!(
        json(&IdentitySource::HardwareUuid),
        serde_json::json!({"checked": true, "against": "macOS hardware UUID"})
    );
    assert_eq!(
        json(&IdentitySource::Unsupported),
        serde_json::json!({"checked": false, "against": null})
    );
    assert_eq!(
        fork_detection_json(),
        serde_json::json!({
            "local": "parinn verify",
            "peer": "with the peer network",
        })
    );
}
