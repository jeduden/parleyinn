use std::fs;

use testkit::TempDir;

use super::*;

fn srs(dir: &Path, body: &str) {
    fs::create_dir_all(dir.join(SRS)).unwrap();
    fs::write(dir.join(SRS).join("10-x.md"), body).unwrap();
}

/// One requirement, written as a block.
fn block(id: &str) -> String {
    format!("## {id}\n\n- Pri: P0\n- Ver: T\n- Traces: I2\n\nGate.\n")
}

#[test]
fn the_ids_are_main_s_and_the_pull_request_s_and_the_invariants() {
    let dir = TempDir::new().unwrap();
    srs(dir.path(), &block("ENG-28"));
    srs(&dir.path().join(PR_HEAD), &block("ENG-33"));

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
        "| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n| ENG-28 | P0 | Gate. | T | I2 |\n",
    );

    let err = ids(dir.path()).unwrap_err();

    assert!(err.contains("a requirement table"), "{err}");
}
