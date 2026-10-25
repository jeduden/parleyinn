//! The drift suite (ENG-27). It copies the checkout into one work
//! directory, proves the unedited copy passes every registered check,
//! then injects each case into a fresh copy at the same path and
//! requires its check to fail with the case's message. File times are
//! kept, and every check builds into one target directory beside the
//! copy, so cargo compiles once and reuses the build for every case.
//!
//! It needs mdsmith and taplo on PATH, so its tests are ignored by
//! default; CI's drift job runs them with `--ignored`.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use drift::{Check, apply, cases};
use testkit::TempDir;

/// Bounds one check; a run of one test target takes seconds once the
/// first build is done.
const RUN_TIMEOUT: Duration = Duration::from_secs(600);

/// How often a running check is polled.
const POLL: Duration = Duration::from_millis(100);

/// Proves the unedited copy passes every registered check, then
/// injects each case and requires its check to fail with its message.
/// One test, so every check builds into the one target directory in
/// turn rather than two copies compiling side by side.
#[test]
#[ignore = "needs mdsmith and taplo on PATH; CI's drift job runs it"]
fn every_drift_is_caught() {
    let tmp = TempDir::new().unwrap();
    let (work, target) = (tmp.path().join("work"), tmp.path().join("target"));

    sync_copy(&testkit::repo_root(), &work);
    let checks: BTreeSet<Check> = cases().into_iter().map(|c| c.check).collect();
    for check in checks {
        let (ok, out) = run(&work, &target, &check);
        assert!(ok, "{check} fails on the unedited copy:\n{out}");
    }

    let mut problems = Vec::new();
    for c in cases() {
        sync_copy(&testkit::repo_root(), &work);
        apply(&work, &c.injection).unwrap();

        let (ok, out) = run(&work, &target, &c.check);
        match (c.known_gap, ok) {
            (true, true) => println!("known gap, not yet caught: {} ({})", c.name, c.guards),
            (true, false) => problems.push(format!(
                "known gap {:?} is caught now: drop known_gap from the case",
                c.name
            )),
            (false, true) => {
                problems.push(format!("drift {:?} went uncaught by {}", c.name, c.guards))
            }
            (false, false) if !out.contains(c.want) => problems.push(format!(
                "{} failed on {:?}, but not with {:?}:\n{out}",
                c.guards, c.name, c.want
            )),
            (false, false) => println!("caught: {} ({})", c.name, c.guards),
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

/// Runs `check` in `dir` with a deadline, building into `target`, and
/// returns whether it passed and its combined output.
fn run(dir: &Path, target: &Path, check: &Check) -> (bool, String) {
    let log = dir.with_extension("log");
    let out = File::create(&log).unwrap();
    let mut child = Command::new(&check.tool)
        .args(&check.args)
        .current_dir(dir)
        .env("CARGO_TARGET_DIR", target)
        // HOME stays, so rustup and cargo find their toolchains; ParleyInn's
        // own home is the copy's (ENG-14).
        .env("PARINN_HOME", dir.join(".parinn"))
        // The `[env]` of .cargo/config.toml sets this in the suite's own
        // environment; the copy's config alone decides it for the check.
        .env_remove("LIBSQLITE3_FLAGS")
        .stdin(Stdio::null())
        .stdout(out.try_clone().unwrap())
        .stderr(out)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + RUN_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            break None;
        }
        thread::sleep(POLL);
    };
    let output = fs::read_to_string(&log).unwrap();

    match status {
        Some(status) => (status.success(), output),
        None => (
            false,
            format!("{check} timed out after {RUN_TIMEOUT:?}\n{output}"),
        ),
    }
}

/// Replaces `work`'s contents with a copy of the checkout at `root`,
/// leaving out `.git` and `target`. Each file keeps its modification
/// time, so cargo sees an unchanged source and does not rebuild.
fn sync_copy(root: &Path, work: &Path) {
    if work.exists() {
        fs::remove_dir_all(work).unwrap();
    }
    copy_tree(root, work);
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(&name));
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&src, &dst);
            continue;
        }
        fs::copy(&src, &dst).unwrap();
        let modified = entry.metadata().unwrap().modified().unwrap();
        File::options()
            .write(true)
            .open(&dst)
            .unwrap()
            .set_modified(modified)
            .unwrap();
    }
}
