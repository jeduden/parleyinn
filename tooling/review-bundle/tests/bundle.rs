//! The crate's public API on the repository's own records: main's tree
//! read twice, as an unchanged pull request would be.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use review_bundle::{Class, Snapshot, analyse, render};

fn tree() -> Snapshot {
    Snapshot::read(&testkit::repo_root()).unwrap()
}

#[test]
fn the_repository_reads_and_compares_equal_to_itself() {
    let t = tree();

    let got = analyse(
        &t,
        &t,
        "diff --git a/docs/testing.md b/docs/testing.md\n",
        false,
    );

    assert_eq!(got.class, Class::Light);
    assert!(got.requirements.is_empty() && got.scenarios.is_empty());
    assert!(got.findings.is_empty(), "{:?}", got.findings);
}

#[test]
fn an_invariant_edit_without_its_record_is_a_blocking_finding() {
    let t = tree();

    let got = analyse(
        &t,
        &t,
        "diff --git a/docs/srs/invariants.md b/docs/srs/invariants.md\n",
        false,
    );

    assert_eq!(got.class, Class::Deep);
    // ENG-29's, and ENG-21's: the SRS has a code owner.
    assert_eq!(got.findings.len(), 2, "{:?}", got.findings);
    assert!(got.findings.iter().any(|f| f.body.contains("ENG-29")));
}

#[test]
fn the_requirements_eng_29_marks_follow_its_row() {
    let base = tree();
    let mut head = tree();
    let row = head
        .requirements
        .iter_mut()
        .find(|r| r.id == "PRV-11")
        .unwrap();
    row.text.push_str(" Edited.");
    let path = row.path.display().to_string();

    let got = analyse(
        &base,
        &head,
        &format!("diff --git a/{path} b/{path}\n"),
        false,
    );

    assert!(
        got.findings
            .iter()
            .any(|f| f.body.contains("touches PRV-11")),
        "{:?}",
        got.findings
    );
    let bundle = render::bundle(&got, &base, &head);
    assert!(
        bundle.contains(&format!("This pull request (`{path}:")),
        "{bundle}"
    );
}
