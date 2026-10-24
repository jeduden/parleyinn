//! ADM-13's M1 part (plan 2609292012, phase 11): the core writes to no
//! git repository. Every component of the built core, confined to its
//! boundary (ENG-12), runs in an isolated home over a repository with a
//! worktree, refs, notes, git configuration and git hooks, a hook event
//! and a transcript naming it among them. Afterwards every file of the
//! repository, its `.git` included, holds the same bytes and was last
//! modified when it was, and no file or directory is new or gone.
//! `parinn install` and the commit hook are later plans'.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a start the confinement refuses fails the test with its message"
)]
mod git_fixture;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::json;
use testkit::TempDir;
use testkit::boundary::{Component, Confinement};

/// Every file and directory under `dir`: its bytes and modification time.
fn snapshot(dir: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let modified = fs::symlink_metadata(&path).unwrap().modified().unwrap();
        if path.is_dir() {
            out.insert(path.clone(), (Vec::new(), modified));
            snapshot(&path, out);
        } else {
            out.insert(path.clone(), (fs::read(&path).unwrap(), modified));
        }
    }
}

#[track_caller]
fn parinn(home: &Path, args: &[&str], input: Option<&[u8]>) {
    let out = Confinement::of(Component::Core)
        .unwrap_or_else(|e| panic!("{e}"))
        .run_in(
            Path::new(env!("CARGO_BIN_EXE_parinn")),
            args,
            home,
            &[],
            input,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        out.status.success(),
        "parinn {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn no_component_of_the_core_writes_to_a_repository() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let repo = home.join("work").join("app");
    fs::create_dir_all(&repo).unwrap();
    git_fixture::git(home, &repo, &["init", "-q"]);
    fs::write(repo.join("a.txt"), "a").unwrap();
    git_fixture::git(home, &repo, &["add", "."]);
    git_fixture::git(home, &repo, &["commit", "-q", "-m", "root"]);
    git_fixture::git(home, &repo, &["notes", "add", "-m", "a note"]);
    git_fixture::git(home, &repo, &["config", "demo.key", "value"]);
    fs::write(repo.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 0\n").unwrap();
    fs::write(repo.join("untracked.txt"), "u").unwrap();
    let transcripts = home.join(".claude/projects/-work-app");
    fs::create_dir_all(&transcripts).unwrap();
    let transcript = transcripts.join("s1.jsonl");
    fs::write(
        &transcript,
        "{\"type\":\"atis-latch\",\"atis\":\"\",\"sessionId\":\"s1\"}\n",
    )
    .unwrap();
    let mut before = BTreeMap::new();
    snapshot(&repo, &mut before);

    for hook in ["SessionStart", "UserPromptSubmit", "PermissionRequest"] {
        let input = json!({
            "session_id": "s1",
            "transcript_path": transcript,
            "cwd": repo,
            "hook_event_name": hook,
            "source": "startup",
            "prompt_id": "p1",
            "prompt": "hello",
            "permission_mode": "default",
        });
        parinn(home, &["hook", hook], Some(input.to_string().as_bytes()));
    }
    for args in [
        &["ingest", "--all"][..],
        &["status"][..],
        &["verify"][..],
        &["rebuild"][..],
    ] {
        parinn(home, args, None);
    }
    let repo_arg = repo.display().to_string();
    parinn(home, &["repository", "show", &repo_arg], None);
    let initialize = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}});
    parinn(home, &["mcp"], Some(format!("{initialize}\n").as_bytes()));

    let mut after = BTreeMap::new();
    snapshot(&repo, &mut after);
    assert_eq!(
        after.keys().collect::<Vec<_>>(),
        before.keys().collect::<Vec<_>>()
    );
    for (path, was) in &before {
        assert!(after[path] == *was, "{} changed", path.display());
    }
}
