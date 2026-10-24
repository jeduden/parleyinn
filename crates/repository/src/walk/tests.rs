use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::{MAX_COMMITS, check_commit, lowest_root, parents, read_shallow};
use crate::fixture::{commit_body, put_commit, put_object};
use crate::objects::Objects;
use crate::repo::Repo;
use crate::{CommitId, Error};

/// An id no object is stored under: named in a parent header or a
/// shallow file, never read.
fn id(n: u8) -> CommitId {
    CommitId::from_bytes(&[n; 20])
}

/// Stores a commit named by `message`, with `parents`, under its real
/// id.
fn commit(objects: &Path, message: &str, parents: &[&CommitId]) -> CommitId {
    put_commit(objects, parents, message)
}

/// A history of `count` commits, each the child of the one before, the
/// first a root.
fn linear(objects: &Path, count: usize) -> Vec<CommitId> {
    let mut ids: Vec<CommitId> = Vec::new();
    for n in 0..count {
        let parents: Vec<&CommitId> = ids.last().into_iter().collect();
        let next = commit(objects, &format!("step {n}"), &parents);
        ids.push(next);
    }
    ids
}

fn root(dir: &TempDir, tip: &CommitId, shallow: &[&CommitId]) -> Result<Option<CommitId>, Error> {
    let objects = Objects::open(dir.path()).unwrap();
    let shallow: BTreeSet<CommitId> = shallow.iter().map(|id| (*id).clone()).collect();
    lowest_root(&objects, &shallow, tip.clone(), &|| false, MAX_COMMITS)
}

fn corrupt<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

#[test]
fn a_commit_without_parents_has_none() {
    let data = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\nauthor A 1 +0000\n\nmessage\n";
    assert!(parents(data).unwrap().is_empty());
}

#[test]
fn each_parent_header_names_a_parent() {
    let one = format!("tree t\nparent {}\nauthor A\n\nm", id(1).as_str());
    assert_eq!(parents(one.as_bytes()).unwrap(), [id(1)]);
    let octopus = format!(
        "tree t\nparent {}\nparent {}\nparent {}\nauthor A\n\nm",
        id(1).as_str(),
        id(2).as_str(),
        id(3).as_str()
    );
    assert_eq!(parents(octopus.as_bytes()).unwrap(), [id(1), id(2), id(3)]);
}

#[test]
fn a_parent_line_in_the_message_is_not_a_parent() {
    let text = format!("tree t\nauthor A\n\nparent {}\n", id(1).as_str());
    assert!(parents(text.as_bytes()).unwrap().is_empty());
    let signed = format!(
        "tree t\nauthor A\ngpgsig -----BEGIN\n parent {}\n -----END\n\nm",
        id(1).as_str()
    );
    assert!(parents(signed.as_bytes()).unwrap().is_empty());
}

#[test]
fn a_parent_header_that_is_not_an_id_is_corrupt() {
    for text in [
        "parent \n\n",
        "parent abc\n\n",
        "parent zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\n\n",
        "parent 1111111111111111111111111111111111111111 \n\n",
    ] {
        assert!(corrupt(parents(text.as_bytes())), "{text:?}");
    }
}

#[test]
fn a_commit_without_a_blank_line_still_gives_its_parents() {
    let text = format!("tree t\nparent {}", id(1).as_str());
    assert_eq!(parents(text.as_bytes()).unwrap(), [id(1)]);
}

#[test]
fn a_linear_history_ends_at_its_one_root() {
    let dir = TempDir::new().unwrap();
    let ids = linear(dir.path(), 3);
    assert_eq!(root(&dir, &ids[2], &[]).unwrap(), Some(ids[0].clone()));
    assert_eq!(root(&dir, &ids[0], &[]).unwrap(), Some(ids[0].clone()));
}

#[test]
fn the_lowest_of_several_roots_wins_whatever_the_order() {
    for (first, second) in [(0, 1), (1, 0)] {
        let dir = TempDir::new().unwrap();
        let roots = [
            commit(dir.path(), "root one", &[]),
            commit(dir.path(), "root two", &[]),
        ];
        let lowest = roots.iter().min().unwrap().clone();
        let left = commit(dir.path(), "left", &[&roots[first]]);
        let right = commit(dir.path(), "right", &[&roots[second]]);
        let merge = commit(dir.path(), "merge", &[&left, &right]);
        assert_eq!(root(&dir, &merge, &[]).unwrap(), Some(lowest.clone()));
        let again = commit(dir.path(), "again", &[&right, &left]);
        assert_eq!(root(&dir, &again, &[]).unwrap(), Some(lowest));
    }
}

#[test]
fn a_shared_ancestor_is_visited_once() {
    let dir = TempDir::new().unwrap();
    let base = commit(dir.path(), "base", &[]);
    let left = commit(dir.path(), "left", &[&base]);
    let right = commit(dir.path(), "right", &[&base]);
    let merge = commit(dir.path(), "merge", &[&left, &right]);
    assert_eq!(root(&dir, &merge, &[]).unwrap(), Some(base));
}

#[test]
fn a_shallow_boundary_is_not_a_root() {
    let dir = TempDir::new().unwrap();
    let cut = commit(dir.path(), "cut", &[&id(1)]);
    let tip = commit(dir.path(), "tip", &[&cut]);
    assert_eq!(root(&dir, &tip, &[&cut]).unwrap(), None);
    assert_eq!(root(&dir, &cut, &[&cut]).unwrap(), None);
}

#[test]
fn a_boundary_that_holds_its_parentless_commit_is_still_not_a_root() {
    let dir = TempDir::new().unwrap();
    let only = commit(dir.path(), "only", &[]);
    assert_eq!(root(&dir, &only, &[&only]).unwrap(), None);
}

#[test]
fn a_root_behind_one_side_of_a_shallow_history_is_still_found() {
    let dir = TempDir::new().unwrap();
    let base = commit(dir.path(), "base", &[]);
    let left = commit(dir.path(), "left", &[&base]);
    let cut = commit(dir.path(), "cut", &[&id(4)]);
    let merge = commit(dir.path(), "merge", &[&left, &cut]);
    assert_eq!(root(&dir, &merge, &[&cut]).unwrap(), Some(base));
}

#[test]
fn a_parent_the_clone_lacks_is_missing() {
    let dir = TempDir::new().unwrap();
    let child = commit(dir.path(), "child", &[&id(1)]);
    let result = root(&dir, &child, &[]);
    assert!(matches!(result, Err(Error::Missing(missing)) if missing == id(1)));
    let result = root(&dir, &id(7), &[]);
    assert!(matches!(result, Err(Error::Missing(missing)) if missing == id(7)));
}

#[test]
fn a_tip_that_is_not_a_commit_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let blob = put_object(dir.path(), "blob", b"abc");
    assert!(corrupt(root(&dir, &blob, &[])));
}

#[test]
fn a_commit_with_a_bad_parent_header_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let body = format!("tree {}\nparent nonsense\n\n", "0".repeat(40));
    let bad = put_object(dir.path(), "commit", body.as_bytes());
    assert!(corrupt(root(&dir, &bad, &[])));
}

#[test]
fn a_commit_object_that_does_not_parse_as_a_commit_is_corrupt() {
    let dir = TempDir::new().unwrap();
    for (n, body) in [
        "".to_owned(),
        "author A\n\nmessage\n".to_owned(),
        "tree \n\n".to_owned(),
        "tree nonsense\n\n".to_owned(),
        format!("parent {}\ntree {}\n\n", id(1).as_str(), "0".repeat(40)),
    ]
    .into_iter()
    .enumerate()
    {
        let stored = put_object(dir.path(), "commit", body.as_bytes());
        assert!(corrupt(root(&dir, &stored, &[])), "{n}: {body:?}");
    }
}

#[test]
fn a_commit_opens_with_a_tree_header_that_names_an_id() {
    assert!(check_commit(commit_body(&[], "m").as_bytes()).is_ok());
    assert!(check_commit(format!("tree {}", "a".repeat(40)).as_bytes()).is_ok());
    for text in ["", "tree", "tree ", "tree abc\n", "TREE 0\n", "x\ntree 0\n"] {
        assert!(corrupt(check_commit(text.as_bytes())), "{text:?}");
    }
    let bytes = [b"tree ".as_slice(), &[0xff; 40], b"\n"].concat();
    assert!(corrupt(check_commit(&bytes)));
}

#[test]
fn the_walk_stops_when_the_deadline_passes() {
    let dir = TempDir::new().unwrap();
    let ids = linear(dir.path(), 20);
    let objects = Objects::open(dir.path()).unwrap();
    let asked = Cell::new(0);
    let deadline = || {
        asked.set(asked.get() + 1);
        asked.get() > 5
    };
    let tip = ids[19].clone();
    let result = lowest_root(&objects, &BTreeSet::new(), tip, &deadline, MAX_COMMITS);
    assert!(matches!(result, Err(Error::Deadline)));
    assert_eq!(asked.get(), 6);
}

#[test]
fn a_walk_past_the_commit_bound_is_too_large() {
    let dir = TempDir::new().unwrap();
    let ids = linear(dir.path(), 10);
    let objects = Objects::open(dir.path()).unwrap();
    let shallow = BTreeSet::new();
    let tip = ids[9].clone();
    let result = lowest_root(&objects, &shallow, tip.clone(), &|| false, 5);
    assert!(matches!(result, Err(Error::TooLarge(_))));
    let result = lowest_root(&objects, &shallow, tip, &|| false, 10);
    assert_eq!(result.unwrap(), Some(ids[0].clone()));
}

fn repo(dir: &TempDir) -> Repo {
    Repo {
        head_dir: dir.path().to_path_buf(),
        common_dir: dir.path().to_path_buf(),
    }
}

#[test]
fn a_clone_that_is_not_shallow_lists_no_boundary() {
    let dir = TempDir::new().unwrap();
    assert!(read_shallow(&repo(&dir), 1000).unwrap().is_empty());
}

#[test]
fn the_shallow_file_lists_the_boundary_commits() {
    let dir = TempDir::new().unwrap();
    let text = format!("{}\n\n{}\r\n", id(1).as_str(), id(2).as_str());
    fs::write(dir.path().join("shallow"), text).unwrap();
    let expected: BTreeSet<CommitId> = [id(1), id(2)].into();
    assert_eq!(read_shallow(&repo(&dir), 1000).unwrap(), expected);
}

#[test]
fn a_shallow_file_that_lists_something_else_is_corrupt() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("shallow"), "not an id\n").unwrap();
    assert!(corrupt(read_shallow(&repo(&dir), 1000)));
}

#[test]
fn a_shallow_file_past_the_limit_is_too_large() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("shallow"), format!("{}\n", id(1).as_str())).unwrap();
    assert!(matches!(
        read_shallow(&repo(&dir), 10),
        Err(Error::TooLarge(_))
    ));
}

#[test]
fn a_shallow_file_in_a_linked_worktree_is_the_repositorys() {
    let dir = TempDir::new().unwrap();
    let common = dir.path().join("common");
    fs::create_dir_all(&common).unwrap();
    fs::write(common.join("shallow"), format!("{}\n", id(3).as_str())).unwrap();
    let repo = Repo {
        head_dir: dir.path().join("linked"),
        common_dir: common,
    };
    assert_eq!(read_shallow(&repo, 1000).unwrap(), [id(3)].into());
}
