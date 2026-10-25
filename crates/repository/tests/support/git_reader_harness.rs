//! ENG-07, LANE-02: the checks of the `git_reader` fuzz target. The same
//! file is the fuzz target's body and the replay test's, so what the
//! nightly fuzzer proves and what the committed corpus proves in a normal
//! test run cannot drift.
//!
//! The input is split on [`SEPARATOR`] into up to five parts, each written
//! where git keeps it in a clone made under the directory given: the
//! branch tip, a loose object, a pack, its index and the target of a
//! linked worktree's `gitdir:` file. The reader then runs on the clone
//! under a deadline of a fixed number of polls, so a run takes no clock.
//! It must neither panic nor stall, and what it answers must agree with
//! itself.

#![allow(
    dead_code,
    reason = "the fuzz target uses less of it than the replay test"
)]

use std::cell::Cell;
use std::fs;
use std::path::Path;

use repository::{CommitId, git_dir, has_commit, identity, locate, root_commit, roots};

/// What splits the input into the files of the clone.
pub const SEPARATOR: &[u8] = b"\xff\xfe\xfd";

/// How many times the reader may ask the deadline before it expires.
const POLLS: u32 = 5000;

/// The longest input run, so a run stays fast.
pub const MAX_INPUT: usize = 64 * 1024;

/// The five parts of `data`: the last holds whatever follows the fourth
/// separator, separators included.
fn parts(data: &[u8]) -> Vec<&[u8]> {
    let mut parts = Vec::new();
    let mut rest = data;
    while parts.len() < 4 {
        match rest.windows(SEPARATOR.len()).position(|w| w == SEPARATOR) {
            Some(at) => {
                parts.push(&rest[..at]);
                rest = &rest[at + SEPARATOR.len()..];
            }
            None => break,
        }
    }
    parts.push(rest);

    parts
}

/// The commit id the branch tip names, else a fixed one the loose object
/// is still filed under.
fn named_id(tip: &[u8]) -> String {
    let text = String::from_utf8_lossy(tip);
    let head: String = text.chars().take(40).collect();
    if CommitId::parse(&head).is_some() {
        head
    } else {
        "a".repeat(40)
    }
}

/// Writes `bytes` at `path`, making its directory; a write that fails
/// leaves the clone as it is.
fn put(path: &Path, bytes: &[u8]) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(path, bytes);
}

/// Makes the clone `data` describes below `root`, runs the reader on it
/// and checks what it answers. Panics on a broken invariant, which is
/// what a fuzz crash is.
pub fn check(root: &Path, data: &[u8]) {
    let data = &data[..data.len().min(MAX_INPUT)];
    let _ = fs::remove_dir_all(root);
    let git = root.join("repo/.git");
    let parts = parts(data);
    let part = |i: usize| parts.get(i).copied().unwrap_or_default();
    let id = named_id(part(0));
    put(&git.join("HEAD"), b"ref: refs/heads/main\n");
    put(&git.join("refs/heads/main"), part(0));
    put(&git.join("objects").join(&id[..2]).join(&id[2..]), part(1));
    put(&git.join("objects/pack/pack-f.pack"), part(2));
    put(&git.join("objects/pack/pack-f.idx"), part(3));
    let mut gitdir = b"gitdir: ".to_vec();
    gitdir.extend_from_slice(part(4));
    put(&root.join("repo/wt/.git"), &gitdir);

    let polls = Cell::new(0_u32);
    let deadline = || {
        polls.set(polls.get() + 1);
        polls.get() > POLLS
    };
    let all = roots(&git, &deadline);
    let lowest = root_commit(&git, &deadline);
    if let (Ok(all), Ok(lowest)) = (&all, &lowest) {
        assert!(
            all.windows(2).all(|w| w[0] < w[1]),
            "roots unsorted: {all:?}"
        );
        assert_eq!(all.first(), lowest.as_ref(), "the lowest root differs");
    }
    let _ = identity(&git, &deadline);
    if let Ok(Some(lowest)) = &lowest {
        let _ = has_commit(&git, lowest);
    }
    if let Some(commit) = CommitId::parse(&id) {
        let _ = has_commit(&git, &commit);
    }
    let wt = root.join("repo/wt");
    if let Ok(Some(found)) = locate(&wt, Some(root)) {
        assert!(
            found.worktree.starts_with(root),
            "{found:?} leaves the clone"
        );
    }
    let _ = git_dir(&wt.join("sub/dir"), Some(root));
}
