//! The crate's public API on the repository's own records: main's tree
//! read twice, as an unchanged pull request would be.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeSet;

use review_bundle::{Class, Snapshot, analyse, diff, render};

fn tree() -> Snapshot {
    Snapshot::read(&testkit::repo_root()).unwrap()
}

/// The header a diff of the file at `path` starts with.
fn header(path: &str) -> String {
    format!("diff --git a/{path} b/{path}\n")
}

/// `base` as a pull request that edits the row `id` leaves it, and that
/// row's file.
fn with_row_edited(base: &Snapshot, id: &str) -> (Snapshot, String) {
    let mut head = base.clone();
    let row = head.requirements.iter_mut().find(|r| r.id == id).unwrap();
    row.text.push_str(" Edited.");
    let path = row.path.display().to_string();

    (head, path)
}

/// The rows ENG-29's row names by the rule ADR-2611152235 records:
/// every requirement I2's own text names, read from the invariants, and
/// SEC-01, which I4's boundaries rest on, since I4 names no requirement.
fn rows_i2_and_i4_rest_on(tree: &Snapshot) -> BTreeSet<String> {
    let (_, from_i2) = tree.invariants.split_once("\n## I2\n").unwrap();
    let (i2, _) = from_i2.split_once("\n## I3\n").unwrap();

    diff::ids_in(i2)
        .chain(["SEC-01"])
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_repository_reads_and_compares_equal_to_itself() {
    let t = tree();

    let got = analyse(
        &t,
        &t,
        "diff --git a/docs/testing.md b/docs/testing.md\n",
        &[],
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
        &[],
    );

    assert_eq!(got.class, Class::Deep);
    // ENG-29's alone: the SRS has no code owner, so ENG-21 asks nothing.
    assert!(
        got.findings.iter().any(|f| f.body.contains("ENG-29")),
        "{:?}",
        got.findings
    );
    assert!(
        !got.findings.iter().any(|f| f.body.contains("(ENG-21)")),
        "{:?}",
        got.findings
    );

    let owned = analyse(&t, &t, "diff --git a/CLAUDE.md b/CLAUDE.md\n", &[]);
    assert!(
        owned.findings.iter().any(|f| f.body.contains("(ENG-21)")),
        "{:?}",
        owned.findings
    );
}

#[test]
fn the_requirements_eng_29_marks_follow_its_row() {
    let base = tree();
    let (head, path) = with_row_edited(&base, "PRV-11");

    let got = analyse(&base, &head, &header(&path), &[]);

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

/// ENG-29 also guards the rows that spell out I2's closed paths and I4's
/// boundaries: a change to any of them is a finding until its record
/// and the stakeholder's comment are there. The rows are read from I2's
/// own text, so a requirement I2 comes to name fails here until ENG-29's
/// row names it too.
#[test]
fn the_rows_i2_and_i4_rest_on_are_guarded_by_eng_29() {
    let base = tree();
    let ids = rows_i2_and_i4_rest_on(&base);
    assert!(ids.len() > 1, "I2 names no requirement: {ids:?}");

    for id in &ids {
        let (head, path) = with_row_edited(&base, id);

        let got = analyse(&base, &head, &header(&path), &[]);

        assert!(
            got.findings
                .iter()
                .any(|f| f.body.contains(&format!("touches {id} (ENG-29)"))),
            "{id}: ENG-29's row must name it, since I2 names it or I4's boundaries rest on \
             it; an edit of its row raised {:?}",
            got.findings
        );
    }
}
