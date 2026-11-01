use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::*;
use crate::plan::parse_dispatch;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn plan() -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(r#"{"review-code": ["lang-rust"]}"#).unwrap(),
    }
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn sha256_is_the_full_digest_in_hex() {
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn a_run_id_hashes_each_definition_each_specialist_s_diff_and_the_inputs() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), ".claude/agents/lang-rust.md", "abc");
    write(dir.path(), "review/diffs/lang-rust.diff", "abc");
    write(dir.path(), "review/context/lang-rust.md", "ctx");

    let id = compute(dir.path(), &plan(), Some(HEAD.to_owned()), 9);

    assert_eq!(id.head.as_deref(), Some(HEAD));
    assert_eq!(id.split, 9);
    assert_eq!(id.resumed_from, None);
    assert_eq!(
        id.agents["lang-rust"],
        AgentId {
            definition: Some(sha256(b"abc")),
            diff: Some(sha256(b"abc")),
            context: Some(sha256(b"ctx")),
        }
    );
    // A generalist has no diff of its own, and a definition that does
    // not read has no hash.
    assert_eq!(
        id.agents["review-code"],
        AgentId {
            definition: None,
            diff: None,
            context: None,
        }
    );
    let text = serde_json::to_string(&id).unwrap();
    assert_eq!(parse(&text).unwrap(), id);
    assert!(!text.contains("resumed_from"), "{text}");
}

#[test]
fn changing_only_a_specialist_s_context_changes_its_id() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "review/diffs/lang-rust.diff", "abc");
    write(dir.path(), "review/context/lang-rust.md", "one");
    let before = compute(dir.path(), &plan(), None, 9);
    write(dir.path(), "review/context/lang-rust.md", "two");
    let after = compute(dir.path(), &plan(), None, 9);
    assert_ne!(before.agents["lang-rust"], after.agents["lang-rust"]);
    assert_eq!(
        before.agents["lang-rust"].diff,
        after.agents["lang-rust"].diff
    );
    assert_eq!(before.inputs, after.inputs);
}

#[test]
fn every_input_changes_the_inputs_hash_and_nothing_else_does() {
    let dir = TempDir::new().unwrap();
    let before = compute(dir.path(), &plan(), None, 9).inputs;
    write(dir.path(), "review/checked.json", "[]");
    assert_eq!(compute(dir.path(), &plan(), None, 9).inputs, before);
    let mut seen = vec![before];
    for rel in INPUTS {
        write(dir.path(), rel, "x");
        let now = compute(dir.path(), &plan(), None, 9).inputs;
        assert!(!seen.contains(&now), "{rel}");
        seen.push(now);
    }
    // An empty file is not a missing one.
    write(dir.path(), "pr.diff", "");
    assert!(!seen.contains(&compute(dir.path(), &plan(), None, 9).inputs));
}

#[test]
fn a_run_id_in_another_shape_is_refused() {
    let err = parse(r#"{"head": null}"#).unwrap_err();
    assert!(err.starts_with("malformed run id: "), "{err}");
    let err = parse(r#"{"head": null, "inputs": "x", "split": 1, "agents": {}, "other": 1}"#)
        .unwrap_err();
    assert!(err.contains("unknown field"), "{err}");
}

#[test]
fn a_detached_head_reads_from_the_git_directory() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), ".git/HEAD", &format!("{HEAD}\n"));

    assert_eq!(git_head(dir.path()).as_deref(), Some(HEAD));
}

#[test]
fn a_worktree_s_head_reads_through_its_gitdir_file_and_common_directory() {
    let dir = TempDir::new().unwrap();
    let tree = dir.path().join("tree");
    write(&tree, ".git", "gitdir: ../repo/.git/worktrees/tree\n");
    write(
        dir.path(),
        "repo/.git/worktrees/tree/HEAD",
        "ref: refs/heads/topic\n",
    );
    write(dir.path(), "repo/.git/worktrees/tree/commondir", "../..\n");
    write(
        dir.path(),
        "repo/.git/refs/heads/topic",
        &format!("{OTHER}\n"),
    );

    assert_eq!(git_head(&tree).as_deref(), Some(OTHER));
}

#[test]
fn a_branch_reads_loose_then_packed() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), ".git/HEAD", "ref: refs/heads/main\n");
    write(
        dir.path(),
        ".git/packed-refs",
        &format!("# pack-refs with: peeled\n{OTHER} refs/heads/other\n{HEAD} refs/heads/main\n"),
    );
    assert_eq!(git_head(dir.path()).as_deref(), Some(HEAD));

    write(dir.path(), ".git/refs/heads/main", "not a commit\n");
    assert_eq!(git_head(dir.path()).as_deref(), Some(HEAD));
    write(dir.path(), ".git/refs/heads/main", &format!("{OTHER}\n"));
    assert_eq!(git_head(dir.path()).as_deref(), Some(OTHER));
}

#[test]
fn a_head_that_cannot_be_read_is_none() {
    let dir = TempDir::new().unwrap();
    assert_eq!(git_head(dir.path()), None);
    write(dir.path(), ".git", "not a gitdir line\n");
    assert_eq!(git_head(dir.path()), None);
    fs::remove_file(dir.path().join(".git")).unwrap();
    write(dir.path(), ".git/HEAD", "ABC\n");
    assert_eq!(git_head(dir.path()), None);
    write(dir.path(), ".git/HEAD", "ref: refs/heads/gone\n");
    assert_eq!(git_head(dir.path()), None);
    write(dir.path(), ".git/packed-refs", "nospace\n");
    assert_eq!(git_head(dir.path()), None);
}

#[test]
fn a_commit_id_is_forty_or_sixty_four_lowercase_hex_digits() {
    assert!(is_commit(HEAD));
    assert!(is_commit(&"a".repeat(64)));
    assert!(!is_commit(&"a".repeat(41)));
    assert!(!is_commit(&HEAD.to_uppercase()));
    assert!(!is_commit(&format!("{}g", &HEAD[1..])));
}
