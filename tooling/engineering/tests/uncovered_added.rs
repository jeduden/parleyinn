//! The local coverage gate names the lines a diff adds that the unit
//! layer leaves uncovered (ENG-27's local runner): the script reads a
//! zero-context diff and an lcov report and prints `path:line` for
//! each added line with no unit hits.

use std::fs;
use std::path::Path;
use std::process::Command;

use testkit::TempDir;

const DIFF: &str = "diff --git a/crates/a/src/x.rs b/crates/a/src/x.rs\n--- a/crates/a/src/x.rs\n+++ b/crates/a/src/x.rs\n@@ -3,0 +4,2 @@ fn f\n+a\n+b\n@@ -9 +11 @@ fn g\n-c\n+d\n@@ -20,2 +22,0 @@\n-e\n-f\ndiff --git a/old b/old\n--- a/old\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-x\n-y\n";

const LCOV: &str = "TN:\nSF:/work/crates/a/src/x.rs\nDA:4,0\nDA:5,3\nDA:6,0\nDA:11,0\nend_of_record\nSF:/work/crates/a/src/other.rs\nDA:4,0\nend_of_record\n";

fn script() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/uncovered-added.sh")
}

fn run(diff: &str, lcov: &str) -> (bool, String) {
    let dir = TempDir::new().expect("temp dir");
    fs::write(dir.path().join("d"), diff).expect("diff");
    fs::write(dir.path().join("l"), lcov).expect("lcov");
    let out = Command::new("bash")
        .arg(script())
        .arg(dir.path().join("d"))
        .arg(dir.path().join("l"))
        .arg("/work")
        .output()
        .expect("script runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn an_added_line_with_no_unit_hits_is_named_not_an_unchanged_one() {
    let (ok, out) = run(DIFF, LCOV);

    assert!(ok);
    assert_eq!(out, "crates/a/src/x.rs:4\ncrates/a/src/x.rs:11\n");
}

#[test]
fn a_covered_line_a_deleted_line_and_an_untouched_file_are_not_named() {
    let (ok, out) = run(
        DIFF,
        "SF:/work/crates/a/src/x.rs\nDA:4,2\nDA:5,1\nDA:6,9\nDA:11,1\nDA:22,0\nend_of_record\n",
    );

    assert!(ok);
    assert_eq!(out, "");
}

#[test]
fn a_missing_report_is_an_error() {
    let dir = TempDir::new().expect("temp dir");
    let out = Command::new("bash")
        .arg(script())
        .arg(dir.path().join("d"))
        .arg(dir.path().join("l"))
        .output()
        .expect("script runs");

    assert!(!out.status.success());
}
