//! The `review-bundle` executable as the review workflow runs it: two
//! trees, a diff and the stakeholder's comments in, the review bundle
//! and findings written, the review class on stdout and an exit code
//! out. Each run has a temporary `HOME` and `PARINN_HOME` and nothing
//! else of the environment but a coverage run's profile path.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use testkit::TempDir;

/// Runs the executable in `home` with `args`, after `--consent` naming
/// a file that holds no comment.
fn review_bundle(home: &Path, args: &[&Path]) -> Output {
    let consent = home.join("consent.json");
    fs::write(&consent, "[]").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_review-bundle"));
    command
        .env_clear()
        .env("HOME", home)
        .env("PARINN_HOME", home)
        .args(args)
        .arg("--consent")
        .arg(&consent);
    // Coverage of the executable needs its profile path, the one
    // variable the run keeps.
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }

    command.output().unwrap()
}

/// The flags for `base`, `head`, `diff` and `out`.
fn flags<'a>(base: &'a Path, head: &'a Path, diff: &'a Path, out: &'a Path) -> Vec<&'a Path> {
    vec![
        Path::new("--base"),
        base,
        Path::new("--head"),
        head,
        Path::new("--diff"),
        diff,
        Path::new("--out"),
        out,
    ]
}

#[test]
fn the_executable_bundles_this_repository_against_itself() {
    let out = TempDir::new().unwrap();
    let diff = out.path().join("pr.diff");
    fs::write(&diff, "diff --git a/docs/srs/10-engineering-quality.md b/docs/srs/10-engineering-quality.md\n+ENG-28\n").unwrap();
    let root = testkit::repo_root();

    let run = review_bundle(out.path(), &flags(&root, &root, &diff, out.path()));

    assert_eq!(
        run.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert!(stdout.contains("class=deep"), "{stdout}");
    assert!(out.path().join("bundle.md").is_file());
    assert!(out.path().join("checked.json").is_file());
}

#[test]
fn the_executable_exits_2_on_a_wrong_command_line() {
    let home = TempDir::new().unwrap();

    let run = review_bundle(home.path(), &[Path::new("--nope")]);

    assert_eq!(run.status.code(), Some(2));
    assert!(!run.stderr.is_empty());
}

#[test]
fn the_executable_exits_1_on_a_tree_it_cannot_read() {
    let home = TempDir::new().unwrap();
    let diff = home.path().join("pr.diff");
    fs::write(&diff, "").unwrap();
    let missing = home.path().join("no-such-tree");

    let run = review_bundle(
        home.path(),
        &flags(&missing, &testkit::repo_root(), &diff, home.path()),
    );

    assert_eq!(run.status.code(), Some(1));
    assert!(run.stdout.is_empty());
    assert!(!run.stderr.is_empty());
}

/// Copies the file or directory `rel` from `from` to `to`.
fn copy(from: &Path, to: &Path, rel: &str) {
    let (src, dst) = (from.join(rel), to.join(rel));
    if src.is_file() {
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(&src, &dst).unwrap();
        return;
    }
    for entry in fs::read_dir(&src).unwrap() {
        let name = entry.unwrap().file_name();
        copy(from, to, &format!("{rel}/{}", name.to_string_lossy()));
    }
}

#[test]
fn the_executable_refuses_a_live_scenario_put_back_on_pending() {
    let root = testkit::repo_root();
    let home = TempDir::new().unwrap();
    let head = home.path().join("head");
    for rel in ["docs/srs", "features", "docs/adr", ".github/CODEOWNERS"] {
        copy(&root, &head, rel);
    }
    let feature = head.join("features/engineering.feature");
    let body = fs::read_to_string(&feature).unwrap();
    assert!(body.contains("  @ENG-28 @P0\n"), "ENG-28 is no longer live");
    fs::write(
        &feature,
        body.replace("  @ENG-28 @P0\n", "  @ENG-28 @P0 @pending\n"),
    )
    .unwrap();
    let diff = home.path().join("pr.diff");
    fs::write(
        &diff,
        "diff --git a/features/engineering.feature b/features/engineering.feature\n",
    )
    .unwrap();

    let run = review_bundle(home.path(), &flags(&root, &head, &diff, home.path()));

    assert_eq!(
        run.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8(run.stdout).unwrap();
    assert!(stdout.contains("findings=1"), "{stdout}");
    let checked: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(home.path().join("checked.json")).unwrap())
            .unwrap();
    assert_eq!(checked[0]["severity"], "blocking", "{checked}");
    assert!(
        checked[0]["body"].as_str().unwrap().contains("ENG-28"),
        "{checked}"
    );
}
