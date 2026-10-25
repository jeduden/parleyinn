//! A repository's fixtures built with the real `git`, as a person's
//! would be, in an isolated home.

#![expect(
    clippy::disallowed_types,
    reason = "the test builds its repository with the real git, as a person's would be"
)]
#![expect(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![expect(
    clippy::panic,
    reason = "a git that does not finish fails the test with its message"
)]

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn git(home: &Path, dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    testkit::isolate(&mut command, home)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=a",
            "-c",
            "user.email=a@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command.spawn().unwrap();
    for _ in 0..2_000 {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "git {args:?}");
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = child.kill();
    panic!("git {args:?} did not finish");
}
