//! The `review-report` executable as the workflows run it: a process
//! with work directories in, files and step outputs out, and an exit
//! code.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::process::Command;

use testkit::TempDir;

fn review(root: &TempDir, name: &str, verdict: &str) -> String {
    let dir = root.path().join(name);
    fs::create_dir_all(dir.join("review")).unwrap();
    fs::write(
        dir.join("review/outcome.json"),
        format!(r#"{{"verdict":"{verdict}","findings":[]}}"#),
    )
    .unwrap();
    dir.display().to_string()
}

#[test]
fn the_executable_writes_a_run_list_and_measures_stability() {
    let root = TempDir::new().unwrap();
    let one = review(&root, "one", "approve");
    let two = review(&root, "two", "approve");
    let three = review(&root, "three", "approve");
    let out = root.path().join("stability.json");

    let runs = Command::new(env!("CARGO_BIN_EXE_review-report"))
        .args(["write", "--workdir", &one])
        .output()
        .unwrap();
    let stability = Command::new(env!("CARGO_BIN_EXE_review-report"))
        .args([
            "stability",
            "--out",
            out.to_str().unwrap(),
            &one,
            &two,
            &three,
        ])
        .output()
        .unwrap();
    let usage = Command::new(env!("CARGO_BIN_EXE_review-report"))
        .output()
        .unwrap();

    assert!(runs.status.success());
    assert!(
        fs::read_to_string(format!("{one}/review/report.html"))
            .unwrap()
            .contains("approve")
    );
    assert!(stability.status.success());
    assert!(
        String::from_utf8(stability.stdout)
            .unwrap()
            .contains("stable=true")
    );
    assert!(out.is_file());
    assert_eq!(usage.status.code(), Some(2));
}
