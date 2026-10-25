//! The `license-check` executable as CI runs it: a checkout in, a
//! verdict on the terminal and an exit code out.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::process::{Command, Output};

use testkit::TempDir;

/// A workspace with one product binary and no dependencies.
fn fixture(deny: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let files = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/app\"]\nresolver = \"3\"\n",
        ),
        (
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.0.0\"\n",
        ),
        (
            "crates/app/Cargo.toml",
            "[package]\nname = \"app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        ),
        ("crates/app/src/main.rs", "fn main() {}\n"),
        (
            ".github/workflows/build.yml",
            "env:\n  LINUX_TARGETS: >-\n    x86_64-unknown-linux-musl:linux-amd64\n",
        ),
        ("deny.toml", deny),
    ];
    for (name, body) in files {
        let path = dir.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    dir
}

/// Runs `license-check --root DIR` with `args`.
fn license_check(dir: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_license-check"))
        .arg("--root")
        .arg(dir.path())
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn the_executable_writes_the_license_files_and_then_passes_a_checkout_that_matches_them() {
    let dir = fixture("[licenses]\nallow = [\"MIT\"]\n");

    let stale = license_check(&dir, &[]);
    assert_eq!(stale.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&stale.stderr).contains("run license-check --write"));

    assert!(license_check(&dir, &["--write"]).status.success());
    let file = dir
        .path()
        .join("licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES");
    assert!(fs::read_to_string(file).unwrap().contains("SQLite"));

    let out = license_check(&dir, &[]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("has a license text"));
}

#[test]
fn the_executable_fails_a_license_deny_toml_grants_without_a_text() {
    let dir = fixture("[licenses]\nallow = [\"WTFPL\"]\n");

    let out = license_check(&dir, &["--write"]);

    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("license-check: deny.toml grants WTFPL"));
}

#[test]
fn the_executable_refuses_an_unknown_flag() {
    let out = Command::new(env!("CARGO_BIN_EXE_license-check"))
        .arg("--bogus")
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
}
