use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::{MAX_PACKED_REFS, MAX_SYMBOLIC, default_branch_tip, resolve};
use crate::repo::Repo;
use crate::{CommitId, Error};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";

fn id(text: &str) -> Option<CommitId> {
    CommitId::parse(text)
}

fn write(git: &Path, name: &str, text: &str) {
    let path = git.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn repo(dir: &TempDir) -> Repo {
    Repo {
        head_dir: dir.path().to_path_buf(),
        common_dir: dir.path().to_path_buf(),
    }
}

fn tip(dir: &TempDir) -> Result<Option<CommitId>, Error> {
    default_branch_tip(&repo(dir), &|| false)
}

fn named(dir: &TempDir, name: &str) -> Result<Option<CommitId>, Error> {
    resolve(&repo(dir), name, &|| false, MAX_PACKED_REFS)
}

fn corrupt<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

#[test]
fn a_detached_head_is_its_commit() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{A}\n"));
    assert_eq!(tip(&dir).unwrap(), id(A));
}

#[test]
fn head_follows_its_branch() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    write(dir.path(), "refs/heads/main", &format!("{A}\n"));
    assert_eq!(tip(&dir).unwrap(), id(A));
}

#[test]
fn head_follows_its_branch_into_packed_refs() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    let packed = format!(
        "# pack-refs with: peeled fully-peeled sorted \n{B} refs/heads/main2\n{A} refs/heads/main\n^{C}\n{C} refs/tags/v1\n"
    );
    write(dir.path(), "packed-refs", &packed);
    assert_eq!(tip(&dir).unwrap(), id(A));
}

#[test]
fn a_loose_reference_overrides_a_packed_one() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    write(dir.path(), "refs/heads/main", &format!("{B}\n"));
    write(dir.path(), "packed-refs", &format!("{A} refs/heads/main\n"));
    assert_eq!(tip(&dir).unwrap(), id(B));
}

#[test]
fn a_branch_with_no_commit_has_no_tip() {
    let dir = TempDir::new().unwrap();
    assert_eq!(tip(&dir).unwrap(), None);
    write(dir.path(), "HEAD", "ref: refs/heads/main\n");
    assert_eq!(tip(&dir).unwrap(), None);
    write(
        dir.path(),
        "packed-refs",
        &format!("{A} refs/heads/other\n"),
    );
    assert_eq!(tip(&dir).unwrap(), None);
}

#[test]
fn the_remotes_head_is_preferred_over_head() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/topic\n");
    write(dir.path(), "refs/heads/topic", &format!("{A}\n"));
    write(
        dir.path(),
        "refs/remotes/origin/HEAD",
        "ref: refs/remotes/origin/main\n",
    );
    write(dir.path(), "refs/remotes/origin/main", &format!("{B}\n"));
    assert_eq!(tip(&dir).unwrap(), id(B));
}

#[test]
fn the_remotes_head_may_point_into_packed_refs() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{A}\n"));
    write(
        dir.path(),
        "refs/remotes/origin/HEAD",
        "ref: refs/remotes/origin/main\n",
    );
    write(
        dir.path(),
        "packed-refs",
        &format!("{B} refs/remotes/origin/main\n"),
    );
    assert_eq!(tip(&dir).unwrap(), id(B));
}

#[test]
fn a_remotes_head_that_points_nowhere_falls_back_to_head() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", &format!("{A}\n"));
    write(
        dir.path(),
        "refs/remotes/origin/HEAD",
        "ref: refs/remotes/origin/gone\n",
    );
    assert_eq!(tip(&dir).unwrap(), id(A));
}

#[test]
fn a_chain_of_symbolic_references_is_followed_up_to_its_limit() {
    let dir = TempDir::new().unwrap();
    for n in 0..MAX_SYMBOLIC {
        write(
            dir.path(),
            &format!("refs/s{n}"),
            &format!("ref: refs/s{}\n", n + 1),
        );
    }
    write(
        dir.path(),
        &format!("refs/s{MAX_SYMBOLIC}"),
        &format!("{A}\n"),
    );
    assert_eq!(named(&dir, "refs/s1").unwrap(), id(A));
    assert!(corrupt(named(&dir, "refs/s0")));
}

#[test]
fn a_loop_of_symbolic_references_is_corrupt() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads/a\n");
    write(dir.path(), "refs/heads/a", "ref: refs/heads/b\n");
    write(dir.path(), "refs/heads/b", "ref: refs/heads/a\n");
    assert!(corrupt(tip(&dir)));
}

#[test]
fn a_reference_name_that_leaves_the_references_is_corrupt() {
    for target in [
        "../x",
        "/etc/passwd",
        "refs/../../x",
        "refs/./x",
        "refs//x",
        "refs/heads/",
        "refs/heads\\x",
        "other/x",
        "",
        "refs",
    ] {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "HEAD", &format!("ref: {target}\n"));
        assert!(corrupt(tip(&dir)), "{target:?}");
    }
}

#[test]
fn a_reference_that_is_not_a_commit_id_is_corrupt() {
    for text in [
        "garbage\n",
        "",
        &format!("{}\n", &A[1..]),
        &format!("{}\n", A.to_uppercase()),
        &format!("{A}{A}\n"),
    ] {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "HEAD", text);
        assert!(corrupt(tip(&dir)), "{text:?}");
    }
}

#[test]
fn whitespace_around_a_reference_is_ignored() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref:refs/heads/main \r\n");
    write(dir.path(), "refs/heads/main", &format!("  {A}\r\n"));
    assert_eq!(tip(&dir).unwrap(), id(A));
}

#[test]
fn a_directory_in_the_place_of_a_reference_is_refused() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "HEAD", "ref: refs/heads\n");
    fs::create_dir_all(dir.path().join("refs/heads")).unwrap();
    assert!(matches!(tip(&dir), Err(Error::Unsupported(_))));
}

#[test]
fn packed_refs_that_are_not_valid_are_corrupt() {
    for text in [
        "nonsense\n",
        &format!("{} refs/heads/main\n", &A[1..]),
        &format!("{A}refs/heads/main\n"),
        &format!("{A} refs/heads/other\nnonsense\n"),
    ] {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "packed-refs", text);
        assert!(corrupt(named(&dir, "refs/heads/main")), "{text:?}");
    }
}

#[test]
fn packed_refs_with_windows_line_ends_and_blank_lines_are_read() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "packed-refs",
        &format!("# header\r\n\r\n{B} refs/heads/x\r\n{A} refs/heads/main\r\n"),
    );
    assert_eq!(named(&dir, "refs/heads/main").unwrap(), id(A));
    assert_eq!(named(&dir, "refs/heads/y").unwrap(), None);
}

#[test]
fn packed_refs_past_the_limit_are_too_large() {
    let dir = TempDir::new().unwrap();
    let line = format!("{A} refs/heads/other\n");
    write(dir.path(), "packed-refs", &line.repeat(10));
    let limit = 5 * line.len() as u64;
    let result = resolve(&repo(&dir), "refs/heads/main", &|| false, limit);
    assert!(matches!(result, Err(Error::TooLarge(_))));
    let exact = 10 * line.len() as u64;
    assert_eq!(
        resolve(&repo(&dir), "refs/heads/main", &|| false, exact).unwrap(),
        None
    );
}

#[test]
fn a_deadline_that_has_passed_stops_the_scan() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "packed-refs", &format!("{A} refs/heads/main\n"));
    let result = resolve(&repo(&dir), "refs/heads/main", &|| true, MAX_PACKED_REFS);
    assert!(matches!(result, Err(Error::Deadline)));
}

#[test]
fn head_of_a_linked_worktree_is_read_beside_the_shared_references() {
    let dir = TempDir::new().unwrap();
    let common = dir.path().join("common");
    let linked = dir.path().join("linked");
    write(&linked, "HEAD", "ref: refs/heads/topic\n");
    write(&common, "refs/heads/topic", &format!("{A}\n"));
    let repo = Repo {
        head_dir: linked,
        common_dir: common,
    };
    assert_eq!(default_branch_tip(&repo, &|| false).unwrap(), id(A));
}

#[test]
fn packed_refs_that_are_a_directory_are_refused() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("packed-refs")).unwrap();
    assert!(matches!(
        named(&dir, "refs/heads/main"),
        Err(Error::Unsupported(_))
    ));
}

#[cfg(unix)]
#[test]
fn packed_refs_that_are_a_link_are_refused() {
    let dir = TempDir::new().unwrap();
    let packed = dir.path().join("packed-refs");
    std::os::unix::fs::symlink(&packed, &packed).unwrap();
    assert!(matches!(
        named(&dir, "refs/heads/main"),
        Err(Error::Unsupported(_))
    ));
}
