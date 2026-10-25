//! A hostile clone cannot stall or redirect the reader (I2, I9): a
//! FIFO, device or symbolic link where a regular file or a directory
//! belongs is refused before it is opened, so a FIFO named `HEAD` never
//! blocks the core.
//!
//! The tests make FIFOs with `mkfifo`; the crate under test starts no
//! program. Every read runs on a thread under a deadline, so a reader
//! that blocks fails its test instead of hanging the run.

#![expect(
    clippy::disallowed_types,
    reason = "a test makes its FIFOs with the mkfifo binary; the crate under test starts no program"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use repository::{CommitId, Error, git_dir, has_commit, root_commit};
use testkit::TempDir;

/// How long a read of a hostile clone may take before it counts as
/// stalled.
const PROMPT: Duration = Duration::from_secs(5);

const ID: &str = "0123456789abcdef0123456789abcdef01234567";

fn mkfifo(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut child = Command::new("mkfifo").arg(path).spawn().unwrap();
    for _ in 0..2000 {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = child.kill();
    unreachable!("mkfifo did not finish");
}

fn write(git: &Path, name: &str, content: &str) {
    let path = git.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// The result of `read`, or a failure when it does not come back soon.
fn prompt<T: Send + 'static>(read: impl FnOnce() -> T + Send + 'static) -> T {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = send.send(read());
    });
    receive
        .recv_timeout(PROMPT)
        .expect("the reader stalled on a hostile file")
}

fn root(git: &Path) -> Result<Option<CommitId>, Error> {
    let git: PathBuf = git.to_path_buf();
    prompt(move || root_commit(&git, &|| false))
}

fn refused(result: Result<Option<CommitId>, Error>) {
    assert!(matches!(result, Err(Error::Unsupported(_))), "{result:?}");
}

fn loose(git: &Path) -> PathBuf {
    git.join("objects/01/23456789abcdef0123456789abcdef01234567")
}

#[test]
fn a_fifo_named_head_is_refused() {
    let dir = TempDir::new().unwrap();
    mkfifo(&dir.path().join("HEAD"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_a_loose_reference_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    mkfifo(&dir.path().join("refs/heads/main"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_the_remote_head_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    mkfifo(&dir.path().join("refs/remotes/origin/HEAD"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_packed_refs_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    mkfifo(&dir.path().join("packed-refs"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_the_configuration_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    mkfifo(&dir.path().join("config"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_the_shallow_list_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    mkfifo(&dir.path().join("shallow"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_commondir_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    mkfifo(&dir.path().join("commondir"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_the_alternates_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    mkfifo(&dir.path().join("objects/info/alternates"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_a_loose_object_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    mkfifo(&loose(dir.path()));
    refused(root(dir.path()));
    let git = dir.path().to_path_buf();
    let id = CommitId::parse(ID).unwrap();
    let held = prompt(move || has_commit(&git, &id));
    assert!(matches!(held, Err(Error::Unsupported(_))), "{held:?}");
}

#[test]
fn a_fifo_for_a_pack_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    write(dir.path(), "objects/pack/pack-a.idx", "");
    mkfifo(&dir.path().join("objects/pack/pack-a.pack"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_a_pack_index_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    write(
        dir.path(),
        "objects/pack/pack-a.pack",
        "PACK\0\0\0\x02\0\0\0\0",
    );
    mkfifo(&dir.path().join("objects/pack/pack-a.idx"));
    refused(root(dir.path()));
}

#[test]
fn a_fifo_for_the_dot_git_file_does_not_stall() {
    let dir = TempDir::new().unwrap();
    mkfifo(&dir.path().join(".git"));
    let at = dir.path().to_path_buf();
    let found = prompt(move || git_dir(&at, None));
    assert!(matches!(found, Ok(None) | Err(_)));
}

#[test]
fn a_symlinked_head_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "real", &format!("{ID}\n"));
    symlink(dir.path().join("real"), dir.path().join("HEAD")).unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_reference_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    write(dir.path(), "real", &format!("{ID}\n"));
    fs::create_dir_all(dir.path().join("refs/heads")).unwrap();
    symlink(dir.path().join("real"), dir.path().join("refs/heads/main")).unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_reference_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    write(dir.path(), "elsewhere/main", &format!("{ID}\n"));
    fs::create_dir_all(dir.path().join("refs")).unwrap();
    symlink(dir.path().join("elsewhere"), dir.path().join("refs/heads")).unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_loose_object_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    write(dir.path(), "real", "");
    fs::create_dir_all(loose(dir.path()).parent().unwrap()).unwrap();
    symlink(dir.path().join("real"), loose(dir.path())).unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_fan_out_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    write(dir.path(), "elsewhere/x", "");
    fs::create_dir_all(dir.path().join("objects")).unwrap();
    symlink(dir.path().join("elsewhere"), dir.path().join("objects/01")).unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_objects_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    fs::create_dir_all(dir.path().join("elsewhere")).unwrap();
    symlink(dir.path().join("elsewhere"), dir.path().join("objects")).unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_pack_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    fs::create_dir_all(dir.path().join("elsewhere")).unwrap();
    fs::create_dir_all(dir.path().join("objects")).unwrap();
    symlink(
        dir.path().join("elsewhere"),
        dir.path().join("objects/pack"),
    )
    .unwrap();
    refused(root(dir.path()));
}

#[test]
fn a_symlinked_pack_file_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{ID}\n"));
    write(dir.path(), "real", "PACK\0\0\0\x02\0\0\0\0");
    write(dir.path(), "objects/pack/pack-a.idx", "");
    symlink(
        dir.path().join("real"),
        dir.path().join("objects/pack/pack-a.pack"),
    )
    .unwrap();
    refused(root(dir.path()));
}
