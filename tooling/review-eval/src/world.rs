//! The world of one run: the review's own input, built for a case as
//! the review workflow builds it for a pull request. A worktree of
//! main is the review's checkout, with the component under test
//! copied in from the tree being evaluated; a second worktree inside
//! it, `pr-head/`, carries the case's edits, with its instruction
//! files moved out as text; `pr.diff` is their diff; and `review/`
//! holds what `review-bundle` writes, the reports a case seeds, and
//! the files the workflow's steps add.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::case::Case;
use crate::component::Component;
use crate::diff;
use crate::host::{Command, Host};

/// The pull request's tree, inside the checkout, as the workflow
/// places it.
pub const PR_HEAD: &str = "pr-head";
/// The review's directory and what it holds.
pub const REVIEW: &str = "review";
pub const REPORTS: &str = "review/reports";
pub const DIFF: &str = "pr.diff";
pub const CONSENT: &str = "stakeholder-comments.json";
pub const INSTRUCTIONS_TEXT: &str = "review/pr-instructions.txt";
pub const PATTERNS: &str = "review/mdsmith-patterns.json";
/// The instruction files the workflow keeps out of the agent's load
/// path, at any depth of the pull request's tree.
const INSTRUCTION_NAMES: [&str; 5] = [
    ".claude",
    "CLAUDE.md",
    "CLAUDE.local.md",
    "AGENTS.md",
    ".mcp.json",
];

/// A built world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    /// The review's checkout, the working directory of the run.
    pub main: PathBuf,
    /// The review class the bundle set.
    pub class: String,
    /// How many ids the change touches, as the bundle counts them.
    pub touched: usize,
    /// The reports the case seeded, relative to the checkout.
    pub seeded: Vec<String>,
    /// What the world could not build as the workflow does.
    pub notes: Vec<String>,
}

impl World {
    /// The text of `component`'s report file, when it wrote one.
    #[must_use]
    pub fn report(&self, component: &str) -> Option<String> {
        fs::read_to_string(self.main.join(REPORTS).join(format!("{component}.md"))).ok()
    }
}

/// Builds the world of `case` for `component`, read from `tree`, under
/// `dir`, from the checkout at `checkout`. Its worktrees are added under
/// `worktrees`, one world at a time: git names a worktree's record after
/// its path's last component, `main` or `pr-head` for every world, and
/// two adds at once race for the same free name.
///
/// # Errors
///
/// Fails when a worktree cannot be added, the component's file cannot
/// be copied, the case does not apply, a file cannot be written, or
/// the review bundle cannot be built.
pub fn build(
    host: &dyn Host,
    checkout: &Path,
    tree: &Path,
    component: &Component,
    case: &Case,
    dir: &Path,
    worktrees: &Mutex<()>,
) -> Result<World, String> {
    let main = dir.join("main");
    let pr_head = main.join(PR_HEAD);
    {
        let _one_at_a_time = worktrees.lock().unwrap_or_else(PoisonError::into_inner);
        worktree(host, checkout, &main)?;
        worktree(host, checkout, &pr_head)?;
    }
    let under_test = main.join(&component.file);
    under_test
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::copy(tree.join(&component.file), &under_test))
        .map_err(|e| format!("copy {}: {e}", component.file))?;
    let changes = case.apply(&pr_head)?;
    write(&main.join(DIFF), &diff::all(&changes))?;
    fs::create_dir_all(main.join(REPORTS)).map_err(|e| format!("{REPORTS}: {e}"))?;
    let mut notes = Vec::new();
    write(&main.join(PATTERNS), &patterns(host, &main, &mut notes))?;
    write(&main.join(CONSENT), &consent(case.consent.as_deref()))?;
    // The workflow builds the bundle from the whole tree, then moves its
    // instruction files out, so a specialist's diff is built alike.
    let (class, touched) = bundle(&main, &pr_head)?;
    move_instructions(&pr_head, &main.join(INSTRUCTIONS_TEXT))?;
    let mut seeded = Vec::new();
    for seed in &case.seeds {
        let rel = format!("{REPORTS}/{}.md", seed.agent);
        write(&main.join(&rel), &seed.content)?;
        seeded.push(rel);
    }

    Ok(World {
        main,
        class,
        touched,
        seeded,
        notes,
    })
}

/// Removes the worktrees a world under `dir` holds and prunes the
/// checkout's record of them, under `worktrees`, the lock [`build`]
/// adds them under.
///
/// # Errors
///
/// Fails naming every worktree that could not be removed.
pub fn teardown(
    host: &dyn Host,
    checkout: &Path,
    dir: &Path,
    worktrees: &Mutex<()>,
) -> Result<(), String> {
    let main = dir.join("main");
    let _one_at_a_time = worktrees.lock().unwrap_or_else(PoisonError::into_inner);
    let mut problems = Vec::new();
    for path in [main.join(PR_HEAD), main] {
        if !path.exists() {
            continue;
        }
        let path = path.display().to_string();
        match host.run(&Command::new(
            "git",
            &["worktree", "remove", "--force", &path],
            checkout,
        )) {
            Ok(out) if out.ok() => {}
            Ok(out) => problems.push(format!("remove worktree {path}: {}", out.stderr.trim())),
            Err(e) => problems.push(format!("remove worktree {path}: {e}")),
        }
    }
    let _ = host.run(&Command::new("git", &["worktree", "prune"], checkout));
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

/// Adds a worktree of `checkout`'s HEAD at `path`.
fn worktree(host: &dyn Host, checkout: &Path, path: &Path) -> Result<(), String> {
    let at = path.display().to_string();
    let out = host.run(&Command::new(
        "git",
        &["worktree", "add", "--detach", &at, "HEAD"],
        checkout,
    ))?;
    if out.ok() {
        Ok(())
    } else {
        Err(format!("add worktree {at}: {}", out.stderr.trim()))
    }
}

/// Writes `text` to `path`, making its directories.
fn write(path: &Path, text: &str) -> Result<(), String> {
    path.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(path, text))
        .map_err(|e| format!("write {}: {e}", path.display()))
}

/// mdsmith's pattern catalog as the workflow writes it, or an empty
/// catalog with a note when mdsmith is not at hand.
fn patterns(host: &dyn Host, main: &Path, notes: &mut Vec<String>) -> String {
    match host.run(&Command::new(
        "mdsmith",
        &["help", "patterns", "-f", "json"],
        main,
    )) {
        Ok(out) if out.ok() => out.stdout,
        Ok(out) => {
            notes.push(format!(
                "mdsmith help patterns failed, so the pattern catalog is empty: {}",
                out.stderr.trim()
            ));
            "[]\n".to_owned()
        }
        Err(e) => {
            notes.push(format!("no mdsmith, so the pattern catalog is empty: {e}"));
            "[]\n".to_owned()
        }
    }
}

/// The stakeholder's comments: one consenting comment, or none.
fn consent(body: Option<&str>) -> String {
    match body {
        Some(body) => serde_json::json!([{
            "author": "stakeholder",
            "at": "2026-01-01T00:00:00Z",
            "body": body,
        }])
        .to_string(),
        None => "[]".to_owned(),
    }
}

/// Moves every instruction file under `root` into one text file at
/// `into`, each under its path, so none loads beside the agent.
fn move_instructions(root: &Path, into: &Path) -> Result<(), String> {
    let mut found = Vec::new();
    find_instructions(root, root, &mut found)?;
    found.sort();
    let mut text = String::new();
    for path in &found {
        let mut files = Vec::new();
        collect_files(path, &mut files)?;
        files.sort();
        for file in files {
            let rel = file.strip_prefix(root).unwrap_or(&file).display();
            let body = fs::read(&file).map_err(|e| format!("read {}: {e}", file.display()))?;
            text.push_str(&format!(
                "\n===== {rel} =====\n{}",
                String::from_utf8_lossy(&body)
            ));
        }
        let removed = if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        removed.map_err(|e| format!("remove {}: {e}", path.display()))?;
    }

    write(into, &text)
}

/// Every instruction file or directory under `dir`, `.git` left out,
/// without descending into one found.
fn find_instructions(root: &Path, dir: &Path, found: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        let path = entry.path();
        let name = entry.file_name();
        if name == ".git" && path.parent() == Some(root) {
            continue;
        }
        if INSTRUCTION_NAMES.iter().any(|n| *n == name) {
            found.push(path);
        } else if path.is_dir() {
            find_instructions(root, &path, found)?;
        }
    }

    Ok(())
}

/// Every file under `path`, or `path` itself when it is a file.
fn collect_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_dir() {
        let entries = fs::read_dir(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read {}: {e}", path.display()))?;
            collect_files(&entry.path(), files)?;
        }
    } else {
        files.push(path.to_path_buf());
    }

    Ok(())
}

/// Runs `review-bundle` over `main` and `pr_head` as the workflow does,
/// and reads the class and the touched ids it prints.
fn bundle(main: &Path, pr_head: &Path) -> Result<(String, usize), String> {
    let args = [
        "--base",
        &main.display().to_string(),
        "--head",
        &pr_head.display().to_string(),
        "--diff",
        &main.join(DIFF).display().to_string(),
        "--consent",
        &main.join(CONSENT).display().to_string(),
        "--out",
        &main.join(REVIEW).display().to_string(),
    ]
    .map(str::to_owned);
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = review_bundle::cli::run(args, &mut stdout, &mut stderr);
    if code != 0 {
        return Err(format!(
            "review-bundle exited {code}: {}",
            String::from_utf8_lossy(&stderr).trim()
        ));
    }
    outputs(&String::from_utf8_lossy(&stdout))
}

/// The class and the touched ids `review-bundle` printed as `stdout`.
fn outputs(stdout: &str) -> Result<(String, usize), String> {
    let value = |key: &str| {
        stdout
            .lines()
            .find_map(|l| l.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
            .map(str::to_owned)
    };
    let class = value("class").ok_or("review-bundle printed no class")?;
    let touched = value("touched")
        .and_then(|c| c.parse().ok())
        .ok_or("review-bundle printed no touched count")?;

    Ok((class, touched))
}

#[cfg(test)]
pub(crate) mod tests;
