//! The repository's own finding ledger.

use std::fs;

/// Every finding theme the domain-model rounds closed still has its
/// closing sentences in the model and the SRS.
#[test]
fn finding_ledger_is_carried() {
    let root = testkit::repo_root();
    let read = |path: &str| fs::read_to_string(root.join(path));

    let ledger = finding_ledger::load(
        read,
        "plan/2610012322_parinn-for-agent-fleets/finding-ledger.json",
    )
    .unwrap();

    if let Err(err) = finding_ledger::check(&ledger, read) {
        panic!("{err}");
    }
}
