use std::fs;

use testkit::TempDir;

use super::*;

const TABLE: &str = "| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n";

fn srs(dir: &Path, rows: &str) {
    fs::create_dir_all(dir.join(SRS)).unwrap();
    fs::write(dir.join(SRS).join("10-x.md"), format!("{TABLE}{rows}")).unwrap();
}

#[test]
fn the_ids_are_main_s_and_the_pull_request_s_and_the_invariants() {
    let dir = TempDir::new().unwrap();
    srs(dir.path(), "| ENG-28 | P0 | Gate. | T | I2 |\n");
    srs(
        &dir.path().join(PR_HEAD),
        "| ENG-33 | P0 | New. | T | I2 |\n",
    );

    let got = ids(dir.path()).unwrap();

    assert!(got.contains("ENG-28") && got.contains("ENG-33"));
    assert!(got.contains("I1") && got.contains("I10"));
    assert_eq!(got.len(), 12);
}

#[test]
fn a_tree_without_an_srs_adds_no_id() {
    let dir = TempDir::new().unwrap();

    assert_eq!(ids(dir.path()).unwrap(), review_gate::report::invariants());
}

#[test]
fn an_srs_that_does_not_read_fails() {
    let dir = TempDir::new().unwrap();
    srs(
        &dir.path().join(PR_HEAD),
        "| eng-1 | P0 | Bad. | T | I2 |\n",
    );

    let err = ids(dir.path()).unwrap_err();

    assert!(err.contains("malformed requirement id"), "{err}");
}
