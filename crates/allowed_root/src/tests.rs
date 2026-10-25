use std::fs;
use std::path::{Path, PathBuf};

use testkit::TempDir;

use super::{
    Access, AllowedRoots, PathOutsideAllowedRoots, expand_home, git_directory, repository,
};

/// A home as SEC-18's scenario lays it out: transcript roots, a
/// repository with a worktree, and a secret outside them all.
struct Layout {
    dir: TempDir,
}

impl Layout {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let p = dir.path();
        for d in [
            ".claude/projects/p",
            "src/app/pkg",
            "src/app/docs",
            "src/app/.git",
            ".ssh",
            ".parinn",
        ] {
            fs::create_dir_all(p.join(d)).unwrap();
        }
        fs::write(p.join(".claude/projects/p/t.jsonl"), "{}\n").unwrap();
        fs::write(p.join("src/app/.git/HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(p.join("src/app/pkg/main.rs"), "").unwrap();
        fs::write(p.join(".ssh/id_ed25519"), "secret").unwrap();
        Self { dir }
    }

    fn at(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    fn roots(&self) -> AllowedRoots {
        AllowedRoots::default()
            .with(&self.at(".claude/projects"), Access::Read)
            .with(&self.at(".parinn"), Access::ReadWrite)
            .with_worktree(&self.at("src/app/pkg"))
    }
}

#[cfg(unix)]
fn link(target: &Path, at: &Path) {
    std::os::unix::fs::symlink(target, at).unwrap();
}

#[test]
fn a_path_within_a_root_resolves_to_its_real_path() {
    let l = Layout::new();
    let roots = l.roots();
    let found = roots
        .resolve(&l.at(".claude/projects/p/t.jsonl"), Access::Read)
        .unwrap();
    assert_eq!(
        found.path(),
        fs::canonicalize(l.at(".claude/projects/p/t.jsonl")).unwrap()
    );
    assert!(roots.resolve(&l.at(".parinn"), Access::ReadWrite).is_ok());
}

#[test]
fn a_path_outside_every_root_or_climbing_out_is_refused() {
    let l = Layout::new();
    let roots = l.roots();
    for path in [
        PathBuf::from("/etc/passwd"),
        l.at(".claude/projects/../../.ssh/id_ed25519"),
        l.at(".ssh/id_ed25519"),
        l.at(".ssh/missing"),
        l.at(".claude/projects/nowhere/missing.jsonl"),
        PathBuf::from("/"),
    ] {
        assert_eq!(
            roots.resolve(&path, Access::Read),
            Err(PathOutsideAllowedRoots),
            "{}",
            path.display()
        );
    }
    assert_eq!(
        PathOutsideAllowedRoots.to_string(),
        "path outside allowed roots"
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_that_leaves_its_root_is_refused() {
    let l = Layout::new();
    link(
        Path::new("/"),
        &l.at(".claude/projects/p/link-to-root.jsonl"),
    );
    link(&l.at(".ssh/id_ed25519"), &l.at("src/app/docs/key"));
    let roots = l.roots();
    assert!(
        roots
            .resolve(&l.at(".claude/projects/p/link-to-root.jsonl"), Access::Read)
            .is_err()
    );
    assert!(
        roots
            .resolve(&l.at("src/app/docs/key"), Access::Read)
            .is_err()
    );
}

#[test]
fn the_worktree_and_its_git_directory_are_for_reading_only() {
    let l = Layout::new();
    let roots = l.roots();
    assert!(
        roots
            .resolve(&l.at("src/app/.git/HEAD"), Access::Read)
            .is_ok()
    );
    assert!(
        roots
            .resolve(&l.at("src/app/pkg/main.rs"), Access::Read)
            .is_ok()
    );
    assert_eq!(
        roots.resolve(&l.at("src/app/.git/HEAD"), Access::ReadWrite),
        Err(PathOutsideAllowedRoots)
    );
    assert!(
        roots
            .resolve(&l.at(".claude/projects/p/t.jsonl"), Access::ReadWrite)
            .is_err()
    );
}

#[test]
fn a_root_that_does_not_resolve_allows_nothing() {
    let l = Layout::new();
    let roots = AllowedRoots::default().with(&l.at("nowhere"), Access::ReadWrite);
    assert_eq!(roots, AllowedRoots::default());
    let none = AllowedRoots::default().with_worktree(&l.at(".ssh"));
    assert_eq!(none, AllowedRoots::default());
}

#[test]
fn repository_follows_a_git_file_to_the_directory_it_names() {
    let l = Layout::new();
    fs::create_dir_all(l.at("src/wt/sub")).unwrap();
    fs::create_dir_all(l.at("src/app/.git/worktrees/wt")).unwrap();
    fs::write(
        l.at("src/app/.git/worktrees/wt/HEAD"),
        "ref: refs/heads/wt\n",
    )
    .unwrap();
    fs::write(l.at("src/wt/.git"), "gitdir: ../app/.git/worktrees/wt\n").unwrap();
    let (top, git_dir) = repository(&l.at("src/wt/sub")).unwrap();
    assert_eq!(top, fs::canonicalize(l.at("src/wt")).unwrap());
    assert_eq!(
        git_dir,
        fs::canonicalize(l.at("src/app/.git/worktrees/wt")).unwrap()
    );
    fs::write(l.at("src/wt/.git"), "no gitdir here\n").unwrap();
    assert_eq!(repository(&l.at("src/wt/sub")), None);
    assert_eq!(repository(&l.at("nowhere")), None);
    let (top, git_dir) = repository(&l.at("src/app/pkg")).unwrap();
    assert_eq!(git_dir, top.join(".git"));
}

#[test]
fn expand_home_takes_a_leading_tilde_as_the_home() {
    let home = Path::new("/h");
    assert_eq!(
        expand_home("~/.claude/projects", Some(home)),
        PathBuf::from("/h/.claude/projects")
    );
    assert_eq!(expand_home("~", Some(home)), PathBuf::from("/h"));
    assert_eq!(expand_home("~x/y", Some(home)), PathBuf::from("~x/y"));
    assert_eq!(expand_home("~/y", None), PathBuf::from("~/y"));
    assert_eq!(expand_home("/abs", Some(home)), PathBuf::from("/abs"));
}

#[test]
fn a_missing_file_within_a_root_resolves_for_its_read_to_fail() {
    let l = Layout::new();
    let found = l
        .roots()
        .resolve(&l.at(".claude/projects/p/missing.jsonl"), Access::Read)
        .unwrap();
    assert_eq!(
        found.path(),
        fs::canonicalize(l.at(".claude/projects/p"))
            .unwrap()
            .join("missing.jsonl")
    );
}

#[cfg(unix)]
#[test]
fn a_path_that_cannot_be_resolved_for_another_reason_is_refused() {
    let l = Layout::new();
    let file = l.at(".claude/projects/p/t.jsonl");
    assert!(
        l.roots()
            .resolve(&file.join("below-a-file"), Access::Read)
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn a_dangling_link_within_a_root_is_refused() {
    let l = Layout::new();
    link(
        &l.at(".ssh/not-yet"),
        &l.at(".claude/projects/p/dangling.jsonl"),
    );
    assert!(
        l.roots()
            .resolve(&l.at(".claude/projects/p/dangling.jsonl"), Access::Read)
            .is_err()
    );
}

/// A hostile worktree's `.git` file whose `gitdir:` reads like a git
/// directory but resolves to the home, or an ancestor of the worktree,
/// opens no root over the home's secrets (SEC-18).
#[cfg(unix)]
#[test]
fn a_git_file_whose_git_directory_resolves_elsewhere_adds_no_root() {
    let l = Layout::new();
    fs::create_dir_all(l.at("src/evil/sub.git")).unwrap();
    fs::write(l.at("HEAD"), "ref: refs/heads/main\n").unwrap();
    link(l.dir.path(), &l.at("src/evil/x.git"));
    for named in ["x.git", "sub.git/../../..", "sub.git/../../../"] {
        fs::write(l.at("src/evil/.git"), format!("gitdir: {named}\n")).unwrap();
        let roots = AllowedRoots::default().with_worktree(&l.at("src/evil"));
        assert!(
            roots
                .resolve(&l.at(".ssh/id_ed25519"), Access::Read)
                .is_err(),
            "gitdir: {named}"
        );
    }
}

#[test]
fn a_git_file_naming_no_git_directory_adds_no_root() {
    let l = Layout::new();
    fs::create_dir_all(l.at("src/evil")).unwrap();
    fs::write(l.at("src/evil/.git"), "gitdir: /\n").unwrap();
    let roots = AllowedRoots::default().with_worktree(&l.at("src/evil"));
    assert!(
        roots
            .resolve(&l.at(".ssh/id_ed25519"), Access::Read)
            .is_err()
    );
    assert!(roots.resolve(&l.at("src/evil/.git"), Access::Read).is_ok());
}

#[test]
fn git_directory_wants_a_resolved_git_directory_holding_head_outside_the_worktree() {
    let l = Layout::new();
    for d in [
        "bare.git",
        "nohead.git",
        "anc.git/proj",
        "src/app/.git/modules/m",
    ] {
        fs::create_dir_all(l.at(d)).unwrap();
    }
    for d in ["bare.git", "anc.git", "src/app/.git/modules/m"] {
        fs::write(l.at(d).join("HEAD"), "ref: refs/heads/main\n").unwrap();
    }
    let top = fs::canonicalize(l.at("src")).unwrap();
    for named in ["src/app/.git", "bare.git", "src/app/.git/modules/m"] {
        let found = git_directory(&top, &l.at(named));
        assert_eq!(
            found,
            Some(fs::canonicalize(l.at(named)).unwrap()),
            "{named}"
        );
    }
    for named in ["src/app", "nohead.git", "missing.git"] {
        assert_eq!(git_directory(&top, &l.at(named)), None, "{named}");
    }
    let inside = fs::canonicalize(l.at("anc.git/proj")).unwrap();
    assert_eq!(git_directory(&inside, &l.at("anc.git")), None);
}
