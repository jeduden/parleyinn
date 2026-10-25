//! The `parinn-eval` executable: a call it does not know prints its
//! usage and exits 2, in an isolated home (ENG-14).

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::process::Command;

use testkit::TempDir;

#[test]
fn an_unknown_call_prints_the_usage_and_exits_2() {
    let home = TempDir::new().unwrap();
    let out = testkit::isolate(
        &mut Command::new(env!("CARGO_BIN_EXE_parinn-eval")),
        home.path(),
    )
    .arg("other")
    .output()
    .unwrap();

    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        parinn_eval::cli::USAGE
    );
}
