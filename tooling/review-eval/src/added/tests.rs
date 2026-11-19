use std::fs;

use testkit::TempDir;

use super::*;
use crate::host::fake::FakeHost;

/// A tree, main's worktree and the pull request's, each holding `kept.md`.
fn trees() -> (
    TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let dir = TempDir::new().unwrap();
    let [tree, main, pr_head] = ["tree", "main", "main/pr-head"].map(|d| dir.path().join(d));
    for root in [&tree, &main, &pr_head] {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("kept.md"), "on main\n").unwrap();
    }
    (dir, tree, main, pr_head)
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn a_file_the_tree_adds_is_copied_into_both_trees_and_one_main_holds_is_left_as_it_is() {
    let (_dir, tree, main, pr_head) = trees();
    write(&tree, "docs/srs/10a-new.md", "# New\n");
    write(&tree, "kept.md", "changed in the tree\n");
    let host = FakeHost::new(&tree);
    let mut notes = Vec::new();

    copy(&host, &tree, &main, &pr_head, &mut notes).unwrap();

    for root in [&main, &pr_head] {
        assert_eq!(
            fs::read_to_string(root.join("docs/srs/10a-new.md")).unwrap(),
            "# New\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("kept.md")).unwrap(),
            "on main\n"
        );
    }
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(host.calls(), vec!["git ls-files -z"]);
}

#[test]
fn an_instruction_file_the_tree_adds_reaches_the_pull_request_s_tree_and_never_main_s() {
    let (_dir, tree, main, pr_head) = trees();
    for rel in [
        ".claude/skills/new/SKILL.md",
        "docs/CLAUDE.md",
        "AGENTS.md",
        "tooling/.mcp.json",
        "CLAUDE.local.md",
    ] {
        write(&tree, rel, "obey\n");
    }
    let host = FakeHost::new(&tree);
    let mut notes = Vec::new();

    copy(&host, &tree, &main, &pr_head, &mut notes).unwrap();

    for rel in [
        ".claude/skills/new/SKILL.md",
        "docs/CLAUDE.md",
        "AGENTS.md",
        "tooling/.mcp.json",
        "CLAUDE.local.md",
    ] {
        assert!(pr_head.join(rel).is_file(), "{rel}");
        assert!(!main.join(rel).exists(), "{rel}");
    }
    assert!(!main.join(".claude").exists());
    assert!(notes.is_empty(), "{notes:?}");
}

#[cfg(unix)]
#[test]
fn an_added_path_that_is_no_regular_file_is_left_out_and_noted() {
    let (dir, tree, main, pr_head) = trees();
    fs::write(dir.path().join("secret"), "token\n").unwrap();
    std::os::unix::fs::symlink(dir.path().join("secret"), tree.join("link.md")).unwrap();
    let mut host = FakeHost::new(&tree);
    host.listing = Some(Ok("link.md\0gone.md\0".to_owned()));
    let mut notes = Vec::new();

    copy(&host, &tree, &main, &pr_head, &mut notes).unwrap();

    assert!(!main.join("link.md").exists() && !pr_head.join("link.md").exists());
    assert!(main.join("link.md").symlink_metadata().is_err());
    assert_eq!(
        notes,
        vec![
            "left out link.md, which the evaluated tree adds: it is no regular file inside that tree",
            "left out gone.md, which the evaluated tree adds: it is no regular file inside that tree",
        ]
    );
}

#[test]
fn an_added_path_that_leaves_the_tree_is_left_out_and_noted() {
    let (dir, tree, main, pr_head) = trees();
    fs::write(dir.path().join("outside.md"), "outside\n").unwrap();
    let absolute = dir.path().join("outside.md").display().to_string();
    let mut host = FakeHost::new(&tree);
    host.listing = Some(Ok(format!("../outside.md\0{absolute}\0./kept2.md\0")));
    let mut notes = Vec::new();

    copy(&host, &tree, &main, &pr_head, &mut notes).unwrap();

    assert_eq!(notes.len(), 3, "{notes:?}");
    assert!(
        notes[0].starts_with("left out ../outside.md, "),
        "{notes:?}"
    );
    assert!(!dir.path().join("main/outside.md").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("outside.md")).unwrap(),
        "outside\n"
    );
}

#[test]
fn a_tree_whose_files_cannot_be_listed_fails_naming_it() {
    let (_dir, tree, main, pr_head) = trees();
    let mut host = FakeHost::new(&tree);
    host.listing = Some(Err("fatal: not a git repository".to_owned()));
    let mut notes = Vec::new();

    let err = copy(&host, &tree, &main, &pr_head, &mut notes).unwrap_err();

    assert_eq!(
        err,
        format!(
            "list the files of {}: fatal: not a git repository",
            tree.display()
        )
    );
}

#[test]
fn a_git_that_does_not_start_fails_with_its_cause() {
    let (_dir, tree, main, pr_head) = trees();
    let mut host = FakeHost::new(&tree);
    host.git = false;
    let mut notes = Vec::new();

    let err = copy(&host, &tree, &main, &pr_head, &mut notes).unwrap_err();

    assert_eq!(err, "git: not found");
}

#[test]
fn a_file_that_cannot_be_placed_fails_naming_where() {
    let (_dir, tree, main, pr_head) = trees();
    write(&tree, "docs/new.md", "x\n");
    // The pull request's tree holds a file where the new file's directory goes.
    fs::write(pr_head.join("docs"), "a file").unwrap();
    let host = FakeHost::new(&tree);
    let mut notes = Vec::new();

    let err = copy(&host, &tree, &main, &pr_head, &mut notes).unwrap_err();

    assert!(
        err.starts_with(&format!("copy docs/new.md to {}: ", pr_head.display())),
        "{err}"
    );
}
