use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use testkit::TempDir;

use super::{
    CommitId, Error, RepositoryIdentity, commit_object_id, git_dir, has_commit, identity,
    root_commit, roots,
};
use crate::fixture::{put_commit, put_object};
use crate::loose::rel_of;

/// An id no object is stored under.
fn id(n: u8) -> CommitId {
    CommitId::from_bytes(&[n; 20])
}

fn write(git: &Path, name: &str, content: &str) {
    let path = git.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// Stores a commit named by `message`, with `parents`, under its real
/// id in the git directory `git`.
fn commit(git: &Path, message: &str, parents: &[&CommitId]) -> CommitId {
    put_commit(&git.join("objects"), parents, message)
}

/// Files the object `id` again under `wrong`.
fn misfile(git: &Path, id: &CommitId, wrong: &CommitId) {
    let objects = git.join("objects");
    let to = objects.join(rel_of(wrong));
    fs::create_dir_all(to.parent().unwrap()).unwrap();
    fs::copy(objects.join(rel_of(id)), to).unwrap();
}

fn checked_out(git: &Path, tip: &CommitId) {
    write(git, "HEAD", &format!("{}\n", tip.as_str()));
}

fn root(git: &Path) -> Result<Option<CommitId>, Error> {
    root_commit(git, &|| false)
}

/// Every file under `dir` with its bytes and modification time, and
/// every directory.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Option<(Vec<u8>, SystemTime)>> {
    let mut seen = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(path) = pending.pop() {
        let meta = fs::symlink_metadata(&path).unwrap();
        if meta.is_dir() {
            pending.extend(fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
            seen.insert(path, None);
        } else {
            seen.insert(
                path.clone(),
                Some((fs::read(&path).unwrap(), meta.modified().unwrap())),
            );
        }
    }
    seen
}

#[test]
fn the_root_of_a_linear_history_names_the_repository() {
    let dir = TempDir::new().unwrap();
    let first = commit(dir.path(), "first", &[]);
    let second = commit(dir.path(), "second", &[&first]);
    checked_out(dir.path(), &second);
    assert_eq!(root(dir.path()).unwrap(), Some(first.clone()));
    assert_eq!(
        identity(dir.path(), &|| false).unwrap(),
        RepositoryIdentity::Bound(first)
    );
}

#[test]
fn the_lowest_of_two_roots_names_the_repository() {
    let dir = TempDir::new().unwrap();
    let one = commit(dir.path(), "one", &[]);
    let two = commit(dir.path(), "two", &[]);
    let merge = commit(dir.path(), "merge", &[&one, &two]);
    checked_out(dir.path(), &merge);
    assert_eq!(root(dir.path()).unwrap(), Some(one.min(two)));
}

#[test]
fn a_repository_with_no_commit_is_provisional() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    assert_eq!(root(dir.path()).unwrap(), None);
    assert_eq!(
        identity(dir.path(), &|| false).unwrap(),
        RepositoryIdentity::Provisional
    );
}

#[test]
fn a_shallow_clone_with_no_root_is_provisional() {
    let dir = TempDir::new().unwrap();
    let tip = commit(dir.path(), "tip", &[&id(1)]);
    checked_out(dir.path(), &tip);
    write(dir.path(), "shallow", &format!("{}\n", tip.as_str()));
    assert_eq!(root(dir.path()).unwrap(), None);
    assert_eq!(
        identity(dir.path(), &|| false).unwrap(),
        RepositoryIdentity::Provisional
    );
}

#[test]
fn the_remotes_default_branch_is_the_one_walked() {
    let dir = TempDir::new().unwrap();
    let remote = commit(dir.path(), "remote", &[]);
    let local = commit(dir.path(), "local", &[]);
    checked_out(dir.path(), &local);
    write(
        dir.path(),
        "refs/remotes/origin/HEAD",
        "ref: refs/remotes/origin/main\n",
    );
    write(
        dir.path(),
        "refs/remotes/origin/main",
        &format!("{}\n", remote.as_str()),
    );
    assert_eq!(root(dir.path()).unwrap(), Some(remote));
}

#[test]
fn a_linked_worktree_names_the_repository_it_belongs_to() {
    let dir = TempDir::new().unwrap();
    let common = dir.path().join("main.git");
    let linked = common.join("worktrees/w");
    let first = commit(&common, "first", &[]);
    let second = commit(&common, "second", &[&first]);
    write(&linked, "commondir", "../..\n");
    checked_out(&linked, &second);
    assert_eq!(root(&linked).unwrap(), Some(first));
    assert!(has_commit(&linked, &second).unwrap());
}

#[test]
fn a_commit_is_named_by_the_hash_of_its_header_and_body() {
    let body = crate::fixture::commit_body(&[], "message");
    let id = commit_object_id(body.as_bytes());
    let dir = TempDir::new().unwrap();
    assert_eq!(put_commit(&dir.path().join("objects"), &[], "message"), id);
    assert_eq!(
        commit_object_id(b"").as_str(),
        "dcf5b16e76cce7425d0beaef62d79a7d10fce1f5"
    );
}

#[test]
fn a_hostile_common_directory_is_not_followed() {
    let dir = TempDir::new().unwrap();
    let elsewhere = dir.path().join("elsewhere");
    let tip = commit(&elsewhere, "tip", &[]);
    let linked = dir.path().join("clone/.git");
    write(&linked, "commondir", &format!("{}\n", elsewhere.display()));
    checked_out(&linked, &tip);
    assert!(matches!(root(&linked), Err(Error::Unsupported(_))));
    assert!(matches!(
        has_commit(&linked, &tip),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn a_sha256_repository_is_not_read() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "config",
        "[extensions]\n\tobjectformat = sha256\n",
    );
    checked_out(dir.path(), &id(1));
    assert!(matches!(
        root(dir.path()),
        Err(Error::UnsupportedObjectFormat)
    ));
    assert!(matches!(
        has_commit(dir.path(), &id(1)),
        Err(Error::UnsupportedObjectFormat)
    ));
    assert!(matches!(
        identity(dir.path(), &|| false),
        Err(Error::UnsupportedObjectFormat)
    ));
}

#[test]
fn a_tip_the_clone_lacks_is_missing() {
    let dir = TempDir::new().unwrap();
    checked_out(dir.path(), &id(1));
    assert!(matches!(root(dir.path()), Err(Error::Missing(_))));
}

#[test]
fn a_directory_that_is_not_a_git_directory_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("none");
    assert!(matches!(root(&missing), Err(Error::Io { .. })));
    assert!(matches!(
        has_commit(&missing, &id(1)),
        Err(Error::Io { .. })
    ));
}

#[test]
fn a_directory_with_no_head_has_no_commit() {
    let dir = TempDir::new().unwrap();
    assert_eq!(root(dir.path()).unwrap(), None);
}

#[test]
fn the_deadline_is_passed_down() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    write(
        dir.path(),
        "packed-refs",
        &format!("{} refs/heads/main\n", id(1).as_str()),
    );
    let result = root_commit(dir.path(), &|| true);
    assert!(matches!(result, Err(Error::Deadline)));
    let held = commit(dir.path(), "held", &[]);
    write(
        dir.path(),
        "packed-refs",
        &format!("{} refs/heads/main\n", held.as_str()),
    );
    let result = root_commit(dir.path(), &|| true);
    assert!(matches!(result, Err(Error::Deadline)));
}

#[test]
fn has_commit_tells_a_held_commit_from_an_absent_one() {
    let dir = TempDir::new().unwrap();
    let held = commit(dir.path(), "held", &[]);
    assert!(has_commit(dir.path(), &held).unwrap());
    assert!(!has_commit(dir.path(), &id(2)).unwrap());
}

#[test]
fn has_commit_refuses_an_object_filed_under_the_wrong_id() {
    let dir = TempDir::new().unwrap();
    let held = commit(dir.path(), "held", &[]);
    misfile(dir.path(), &held, &id(1));
    assert!(matches!(
        has_commit(dir.path(), &id(1)),
        Err(Error::Corrupt(_))
    ));
}

#[test]
fn has_commit_is_never_true_for_an_empty_file() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), &format!("objects/01/{}", "01".repeat(19)), "");
    let held = has_commit(dir.path(), &id(1));
    assert!(matches!(held, Ok(false) | Err(_)), "{held:?}");
}

#[test]
fn has_commit_is_false_for_an_object_that_is_not_a_commit() {
    let dir = TempDir::new().unwrap();
    let blob = put_object(&dir.path().join("objects"), "blob", b"abc");
    assert!(!has_commit(dir.path(), &blob).unwrap());
}

#[test]
fn the_walk_refuses_a_commit_filed_under_the_wrong_id() {
    let dir = TempDir::new().unwrap();
    let held = commit(dir.path(), "held", &[]);
    misfile(dir.path(), &held, &id(1));
    checked_out(dir.path(), &id(1));
    assert!(matches!(root(dir.path()), Err(Error::Corrupt(_))));
}

#[test]
fn the_walk_refuses_a_parent_filed_under_the_wrong_id() {
    let dir = TempDir::new().unwrap();
    let base = commit(dir.path(), "base", &[]);
    let swapped = commit(dir.path(), "swapped", &[]);
    misfile(dir.path(), &swapped, &base);
    let tip = commit(dir.path(), "tip", &[&base]);
    checked_out(dir.path(), &tip);
    assert!(matches!(root(dir.path()), Err(Error::Corrupt(_))));
}

#[test]
fn has_commit_cannot_tell_when_alternates_could_hold_it() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "objects/info/alternates",
        "/elsewhere/objects\n",
    );
    assert!(matches!(
        has_commit(dir.path(), &id(1)),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn reading_a_repository_changes_nothing_in_it() {
    let dir = TempDir::new().unwrap();
    let git = dir.path().join("work/.git");
    let first = commit(&git, "first", &[]);
    let second = commit(&git, "second", &[&first]);
    checked_out(&git, &second);
    write(
        &git,
        "packed-refs",
        &format!("{} refs/heads/x\n", second.as_str()),
    );
    write(&git, "shallow", "");
    write(&git, "config", "[core]\n\tbare = false\n");
    let work = dir.path().join("work/sub");
    fs::create_dir_all(&work).unwrap();
    let before = snapshot(dir.path());
    let found = git_dir(&work, None).unwrap().unwrap();
    assert_eq!(found, git);
    root(&found).unwrap();
    has_commit(&found, &first).unwrap();
    has_commit(&found, &id(7)).unwrap();
    identity(&found, &|| false).unwrap();
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn every_root_reachable_from_the_default_branch_is_named_lowest_first_and_no_other() {
    let dir = TempDir::new().unwrap();
    let one = commit(dir.path(), "one", &[]);
    let two = commit(dir.path(), "two", &[]);
    let merge = commit(dir.path(), "merge", &[&one, &two]);
    // A root the branch does not reach, stored in the clone all the same.
    let stray = commit(dir.path(), "stray", &[]);
    checked_out(dir.path(), &merge);

    let named = roots(dir.path(), &|| false).unwrap();

    let mut both = vec![one, two];
    both.sort();
    assert_eq!(named, both);
    assert!(!named.contains(&stray));
}

#[test]
fn a_repository_with_no_commit_has_no_roots_and_a_deadline_stops_the_walk() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    assert_eq!(roots(dir.path(), &|| false).unwrap(), vec![]);
    let tip = commit(dir.path(), "tip", &[]);
    checked_out(dir.path(), &tip);

    assert!(matches!(roots(dir.path(), &|| true), Err(Error::Deadline)));
}
