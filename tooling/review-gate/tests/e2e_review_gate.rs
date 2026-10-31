//! The `review-gate` executable as the review workflow runs it: a
//! process with files in, the review on stdout and an exit code out.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::process::Command;

use testkit::TempDir;

const HEAD: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";

/// Runs the executable in `dir`, beside a `checked.json` of no
/// deterministic findings and a dispatch the review followed.
fn review_gate(dir: &TempDir, args: &[&str]) -> std::process::Output {
    fs::write(dir.path().join("checked.json"), "[]").unwrap();
    fs::write(
        dir.path().join("dispatch.json"),
        r#"{"review-code":["lang-rust"]}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("consulted.json"),
        r#"["review-code","lang-rust"]"#,
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("reports")).unwrap();
    if !dir.path().join("reports/lang-rust.md").exists() {
        fs::write(dir.path().join("reports/lang-rust.md"), "Outcome: sound.\n").unwrap();
    }
    fs::write(
        dir.path().join("answers.json"),
        r#"{"lang-rust":"Outcome: sound."}"#,
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_review-gate"))
        .current_dir(dir.path())
        .args(args)
        .args([
            "--dispatch",
            "dispatch.json",
            "--consulted",
            "consulted.json",
            "--reports",
            "reports",
            "--answers",
            "answers.json",
        ])
        .output()
        .unwrap()
}

#[test]
fn the_executable_prints_the_review_for_a_green_head() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("outcome.json"),
        r#"{"verdict":"approve","summary":"Fine.","findings":[]}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("runs.jsonl"),
        r#"{"name":"CI","status":"completed","conclusion":"success"}"#,
    )
    .unwrap();

    let out = review_gate(
        &dir,
        &[
            "--outcome",
            "outcome.json",
            "--check-runs",
            "runs.jsonl",
            "--reviewed",
            HEAD,
            "--head",
            HEAD,
            "--checked",
            "checked.json",
        ],
    );

    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let review: review_gate::Review = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(review.event, review_gate::APPROVE);
}

#[test]
fn the_executable_exits_2_on_a_wrong_command_line() {
    let dir = TempDir::new().unwrap();

    let out = review_gate(&dir, &["--approve"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn the_executable_exits_1_without_a_review_outcome() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("runs.jsonl"),
        r#"{"name":"CI","status":"completed","conclusion":"success"}"#,
    )
    .unwrap();

    let out = review_gate(
        &dir,
        &[
            "--outcome",
            "outcome.json",
            "--check-runs",
            "runs.jsonl",
            "--reviewed",
            HEAD,
            "--head",
            HEAD,
            "--checked",
            "checked.json",
        ],
    );

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("read outcome.json"));
}

#[test]
fn the_executable_prints_nothing_for_a_head_that_moved() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("outcome.json"),
        r#"{"verdict":"approve","summary":"Fine.","findings":[]}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("runs.jsonl"),
        r#"{"name":"CI","status":"completed","conclusion":"success"}"#,
    )
    .unwrap();

    let out = review_gate(
        &dir,
        &[
            "--outcome",
            "outcome.json",
            "--check-runs",
            "runs.jsonl",
            "--reviewed",
            HEAD,
            "--head",
            "490efa7",
            "--checked",
            "checked.json",
        ],
    );

    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("moved past"));
}

#[test]
fn the_executable_exits_2_without_the_deterministic_findings() {
    let dir = TempDir::new().unwrap();

    let out = review_gate(
        &dir,
        &[
            "--outcome",
            "outcome.json",
            "--check-runs",
            "runs.jsonl",
            "--reviewed",
            HEAD,
            "--head",
            HEAD,
        ],
    );

    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--checked is required"));
}

#[test]
fn the_executable_exits_1_when_a_report_s_outcome_is_not_what_its_agent_answered() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("outcome.json"),
        r#"{"verdict":"approve","summary":"s","findings":[]}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("check-runs.jsonl"),
        "{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":\"success\"}\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("reports")).unwrap();
    fs::write(
        dir.path().join("reports/lang-rust.md"),
        "Outcome: blocking.\n",
    )
    .unwrap();

    let out = review_gate(
        &dir,
        &[
            "--outcome",
            "outcome.json",
            "--check-runs",
            "check-runs.jsonl",
            "--reviewed",
            HEAD,
            "--head",
            HEAD,
            "--checked",
            "checked.json",
        ],
    );

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("lang-rust has a report whose outcome differs"),
        "{err}"
    );
}
