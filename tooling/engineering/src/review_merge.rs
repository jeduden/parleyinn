//! ENG-21: the review reads a pull request as landing it would leave
//! main. `scripts/review-merge-head.sh` moves the review's `pr-head` to
//! the pull request's head merged into the commit main's checkout holds,
//! and writes `pr.diff` from main to that merge; a head that holds main
//! is its own merge, and one that does not merge with main fails, naming
//! the files that conflict. Read alone, a head that merged main and
//! another pull request apart has two merge bases with main once main
//! merged that one, and `git diff main...head` diffs from the first,
//! reading that pull request's change as its own; and a head behind main
//! differs from main's tree in each change main made since, which the
//! evaluation plan reads as one the head reverts.
//!
//! [`merge_problems`] runs the checkout's script on four histories, and
//! [`wiring_problems`] finds the review job, the evaluation's job and the
//! local review running it once they hold the head and before they read
//! it.

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::Checkout;
use crate::isolation_guard::{drain, wait_within};
use crate::review_workflow::parse;

/// The script that merges the head into main.
pub const SCRIPT: &str = "scripts/review-merge-head.sh";
/// The workflow whose review and evaluation jobs read the head.
pub const WORKFLOW: &str = ".github/workflows/review.yml";
/// The local review.
pub const LOCAL: &str = "scripts/review-local.sh";
/// How long one git command, or the script, may run.
const LIMIT: Duration = Duration::from_secs(60);

/// What is wrong with how the checkout's review reads the head. The
/// review job and the evaluation's job check it out with its history
/// and merge it into main before the review bundle, the evaluation plan
/// or the cases read it, and diff from no merge base; so does the local
/// review once it has added its two worktrees; and the review job and
/// the local review tell review-run the head, not the merge.
///
/// # Errors
///
/// Fails when the workflow or the local review cannot be read.
pub fn wiring_problems(checkout: &Checkout) -> Result<Vec<String>, String> {
    let flow = parse(&checkout.read(WORKFLOW)?);
    let job = |name: &str| {
        flow.jobs
            .iter()
            .find(|j| j.name == name)
            .map_or("", |j| j.text.as_str())
    };
    let local = checkout.read(LOCAL)?;
    let run = format!("run: {SCRIPT}");
    let mut problems = Vec::new();
    for (name, reads) in [
        (
            "review",
            &[
                "run: scripts/review-bundle.sh",
                "run: scripts/review-plan.sh",
            ][..],
        ),
        ("evaluate", &["run: scripts/review-eval-run-cases.sh"][..]),
    ] {
        let what = format!("{WORKFLOW}'s job {name}");
        problems.extend(order_problems(&what, job(name), "path: pr-head", &[&run]));
        problems.extend(order_problems(&what, job(name), &run, reads));
        problems.extend(merge_base_problems(&what, job(name)));
        problems.extend(history_problems(&what, job(name)));
    }
    problems.extend(head_problems(
        &format!("{WORKFLOW}'s job review"),
        job("review"),
    ));
    problems.extend(order_problems(
        LOCAL,
        &local,
        "\"$work/pr-head\" HEAD",
        &[SCRIPT, "tool review-bundle"],
    ));
    problems.extend(order_problems(
        LOCAL,
        &local,
        SCRIPT,
        &["tool review-bundle"],
    ));
    problems.extend(merge_base_problems(LOCAL, &local));
    problems.extend(head_problems(LOCAL, &local));
    // A missing script is named once, however many orders want it.
    let mut named = BTreeSet::new();
    problems.retain(|p| named.insert(p.clone()));

    Ok(problems)
}

/// What is wrong with the order of `text`, as `what` holds it: `first`
/// must be in it, and before each of `later`.
#[must_use]
pub fn order_problems(what: &str, text: &str, first: &str, later: &[&str]) -> Vec<String> {
    let Some(at) = text.find(first) else {
        return vec![format!("{what} does not hold {first}")];
    };

    later
        .iter()
        .filter_map(|l| match text.find(l) {
            None => Some(format!("{what} does not hold {l}")),
            Some(i) if i < at => Some(format!("{what} holds {l} before {first}")),
            Some(_) => None,
        })
        .collect()
}

/// What is wrong with `text`, as `what` holds it: no line may diff from
/// a merge base, which reads another pull request's change as the head's
/// own once there are two.
#[must_use]
pub fn merge_base_problems(what: &str, text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| l.contains(" diff ") && l.contains("..."))
        .map(|l| format!("{what} diffs from a merge base: {}", l.trim()))
        .collect()
}

/// What is wrong with the job `text`, as `what` holds it: its head must
/// be checked out with the history a merge with main needs.
#[must_use]
pub fn history_problems(what: &str, text: &str) -> Vec<String> {
    if text.contains("fetch-depth: 0") {
        Vec::new()
    } else {
        vec![format!(
            "{what} checks the head out without its history, so it cannot merge it into main"
        )]
    }
}

/// What is wrong with `text`, as `what` holds it: its review run must be
/// told the head, since `pr-head` holds the merge, which the run would
/// otherwise name as the head a later review weighs its verdict against.
#[must_use]
pub fn head_problems(what: &str, text: &str) -> Vec<String> {
    if text.contains("--head-sha") {
        Vec::new()
    } else {
        vec![format!(
            "{what} runs review-run without --head-sha, so its run names the merge in pr-head as the head"
        )]
    }
}

/// One history the script is tried on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tried {
    /// What the history is.
    pub name: &'static str,
    /// Main's commit.
    pub main: String,
    /// The pull request's head.
    pub head: String,
    /// The files the review must read changed, or, for a head that does
    /// not merge, `None`, with the file that conflicts in `conflict`.
    pub changes: Option<Vec<&'static str>>,
    /// The file a head that does not merge conflicts in.
    pub conflict: &'static str,
    /// Whether the head holds main, so its merge is the head itself.
    pub holds_main: bool,
}

/// What the script did with one history: whether it succeeded, what it
/// printed, the files `pr.diff` names, the files `pr-head` differs from
/// main in and the commit `pr-head` was left at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Merged {
    pub success: bool,
    pub output: String,
    pub diffed: BTreeSet<String>,
    pub differs: BTreeSet<String>,
    pub at: String,
}

/// What is wrong with how the checkout's script reads a head merged into
/// main, tried in `work` on four histories: a branch that merged main
/// and another pull request apart, once main merged that one; a branch
/// behind main; a head that holds main; and a head that does not merge.
#[must_use]
pub fn merge_problems(checkout: &Checkout, work: &Path) -> Vec<String> {
    let script = checkout.root.join(SCRIPT);
    if !script.is_file() {
        return vec![format!("{SCRIPT} is missing")];
    }
    let run = || -> Result<Vec<String>, String> {
        let git = Git::new(work)?;
        let mut problems = Vec::new();
        for tried in histories(&git, &work.join("repo"))? {
            let merged = merge(&git, &script, &work.join("repo"), work, &tried)?;
            problems.extend(tried_problems(&tried, &merged));
        }
        Ok(problems)
    };

    run().unwrap_or_else(|e| vec![format!("{SCRIPT} could not be tried: {e}")])
}

/// What is wrong with what the script did with `tried`.
#[must_use]
pub fn tried_problems(tried: &Tried, merged: &Merged) -> Vec<String> {
    let name = tried.name;
    let Some(changes) = &tried.changes else {
        if merged.success {
            return vec![format!("{SCRIPT} reviews {name}")];
        }
        if !merged.output.contains(tried.conflict) {
            return vec![format!(
                "{SCRIPT} refuses {name} without naming {}",
                tried.conflict
            )];
        }
        return Vec::new();
    };
    if !merged.success {
        return vec![format!(
            "{SCRIPT} fails on {name}: {}",
            merged.output.trim()
        )];
    }
    let want: BTreeSet<String> = changes.iter().map(|c| (*c).to_owned()).collect();
    let mut problems = Vec::new();
    if merged.diffed != want {
        problems.push(format!(
            "{SCRIPT}'s pr.diff for {name} names {}, not {}",
            listed(&merged.diffed),
            listed(&want)
        ));
    }
    if merged.differs != want {
        problems.push(format!(
            "{SCRIPT}'s pr-head for {name} differs from main in {}, not {}",
            listed(&merged.differs),
            listed(&want)
        ));
    }
    if tried.holds_main && merged.at != tried.head {
        problems.push(format!(
            "{SCRIPT} leaves pr-head for {name} at {}, not the head",
            merged.at
        ));
    }

    problems
}

/// `files`, comma-separated, or "nothing".
fn listed(files: &BTreeSet<String>) -> String {
    if files.is_empty() {
        "nothing".to_owned()
    } else {
        files.iter().cloned().collect::<Vec<_>>().join(", ")
    }
}

/// The files a diff's text names, from its `diff --git` lines.
#[must_use]
pub fn diffed_files(diff: &str) -> BTreeSet<String> {
    diff.lines()
        .filter_map(|l| l.strip_prefix("diff --git a/"))
        .filter_map(|l| l.split_once(" b/").map(|(a, _)| a.to_owned()))
        .collect()
}

/// Builds in `repo` the commits the four histories are read from and
/// returns them. Main and a pull request start apart from one commit;
/// a branch merges that pull request, then main, then adds its own file,
/// and main merges the pull request. A second branch starts from that
/// main, which then changes the base file and adds another. One head
/// adds a file to that main, and another changes the base file from the
/// first commit.
fn histories(git: &Git, repo: &Path) -> Result<Vec<Tried>, String> {
    fs::create_dir_all(repo).map_err(|e| format!("make {}: {e}", repo.display()))?;
    git.run(repo, &["init", "-q", "-b", "main"])?;
    let first = git.commit(repo, "base.txt", "base\n")?;
    git.run(repo, &["checkout", "-q", "-b", "parent"])?;
    let parent = git.commit(repo, "parent.txt", "parent\n")?;
    git.run(repo, &["checkout", "-q", "main"])?;
    let moved = git.commit(repo, "main.txt", "main\n")?;
    git.run(repo, &["checkout", "-q", "-b", "stacked", &parent])?;
    git.run(repo, &["merge", "-q", "--no-edit", &moved])?;
    let stacked = git.commit(repo, "own.txt", "own\n")?;
    git.run(repo, &["checkout", "-q", "main"])?;
    git.run(repo, &["merge", "-q", "--no-edit", &parent])?;
    let merged = git.run(repo, &["rev-parse", "HEAD"])?;
    git.run(repo, &["checkout", "-q", "-b", "behind"])?;
    let behind = git.commit(repo, "behind.txt", "behind\n")?;
    git.run(repo, &["checkout", "-q", "main"])?;
    git.commit(repo, "later.txt", "later\n")?;
    let later = git.commit(repo, "base.txt", "main\n")?;
    let holds = git.commit(repo, "held.txt", "held\n")?;
    git.run(repo, &["checkout", "-q", "-b", "conflict", &first])?;
    let conflict = git.commit(repo, "base.txt", "head\n")?;
    let tried = |name, main: &str, head: &str, changes| Tried {
        name,
        main: main.to_owned(),
        head: head.to_owned(),
        changes,
        conflict: "base.txt",
        holds_main: false,
    };

    Ok(vec![
        tried(
            "a branch that merged main and another pull request apart, once main merged that one",
            &merged,
            &stacked,
            Some(vec!["own.txt"]),
        ),
        tried(
            "a branch behind main",
            &later,
            &behind,
            Some(vec!["behind.txt"]),
        ),
        Tried {
            holds_main: true,
            ..tried(
                "a head that holds main",
                &later,
                &holds,
                Some(vec!["held.txt"]),
            )
        },
        tried(
            "a head that does not merge with main",
            &later,
            &conflict,
            None,
        ),
    ])
}

/// Runs `script` on `tried`, in a work directory under `work` holding
/// main's commit with the head's worktree in `pr-head`, as the review
/// does, and reads what it left.
fn merge(
    git: &Git,
    script: &Path,
    repo: &Path,
    work: &Path,
    tried: &Tried,
) -> Result<Merged, String> {
    let dir = work.join(format!("try-{}", tried.head));
    let head = dir.join("pr-head");
    let (dir_s, head_s) = (dir.display().to_string(), head.display().to_string());
    git.run(
        repo,
        &["worktree", "add", "-q", "--detach", &dir_s, &tried.main],
    )?;
    git.run(
        repo,
        &["worktree", "add", "-q", "--detach", &head_s, &tried.head],
    )?;
    let mut command = git.command("bash", &dir);
    // Run as a user who signs every commit, with a signer that fails: the
    // merge is the review's, never signed as theirs.
    command
        .arg(script)
        .env("GIT_CONFIG_COUNT", "2")
        .env("GIT_CONFIG_KEY_0", "commit.gpgSign")
        .env("GIT_CONFIG_VALUE_0", "true")
        .env("GIT_CONFIG_KEY_1", "gpg.program")
        .env("GIT_CONFIG_VALUE_1", "false");
    let ran = git.output(command)?;
    if !ran.0 {
        return Ok(Merged {
            output: format!("{}{}", ran.1, ran.2),
            ..Merged::default()
        });
    }
    let written = dir.join("pr.diff");
    let diff =
        fs::read_to_string(&written).map_err(|e| format!("read {}: {e}", written.display()))?;
    let at = git.run(&head, &["rev-parse", "HEAD"])?;
    let differs = git.run(&head, &["diff", "--name-only", &tried.main, "HEAD"])?;

    Ok(Merged {
        success: true,
        output: format!("{}{}", ran.1, ran.2),
        diffed: diffed_files(&diff),
        differs: differs.lines().map(str::to_owned).collect(),
        at,
    })
}

/// Runs git, and the script, with a home and configuration of their own
/// under the work directory, so none of the user's reaches them (ENG-14),
/// each within [`LIMIT`] (ENG-03).
struct Git {
    home: PathBuf,
    path: OsString,
}

impl Git {
    /// A runner whose home is made under `work`.
    fn new(work: &Path) -> Result<Self, String> {
        let home = work.join("home");
        fs::create_dir_all(&home).map_err(|e| format!("make {}: {e}", home.display()))?;

        Ok(Self {
            home,
            path: env::var_os("PATH").unwrap_or_default(),
        })
    }

    /// `program` to run in `dir`, its environment the runner's alone.
    fn command(&self, program: &str, dir: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(dir)
            .env_clear()
            .env("PATH", &self.path)
            .env("HOME", &self.home)
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.home.join("gitconfig"))
            .env("GIT_AUTHOR_NAME", "drift")
            .env("GIT_AUTHOR_EMAIL", "drift@localhost")
            .env("GIT_COMMITTER_NAME", "drift")
            .env("GIT_COMMITTER_EMAIL", "drift@localhost")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        command
    }

    /// Whether `command` succeeded, and what it printed to its two
    /// streams.
    fn output(&self, mut command: Command) -> Result<(bool, String, String), String> {
        let mut child = command
            .spawn()
            .map_err(|e| format!("start {command:?}: {e}"))?;
        let status = wait_within(&mut child, LIMIT).map_err(|e| format!("{command:?}: {e}"))?;

        Ok((
            status.success(),
            child.stdout.take().map(drain).unwrap_or_default(),
            child.stderr.take().map(drain).unwrap_or_default(),
        ))
    }

    /// git `args` in `dir`, what it printed, trimmed.
    fn run(&self, dir: &Path, args: &[&str]) -> Result<String, String> {
        let mut command = self.command("git", dir);
        command.args(args);
        let (success, out, err) = self.output(command)?;
        if !success {
            return Err(format!("git {}: {}", args.join(" "), err.trim()));
        }

        Ok(out.trim().to_owned())
    }

    /// Writes `text` to `file` in `repo`, commits it and returns the
    /// commit.
    fn commit(&self, repo: &Path, file: &str, text: &str) -> Result<String, String> {
        fs::write(repo.join(file), text).map_err(|e| format!("write {file}: {e}"))?;
        self.run(repo, &["add", file])?;
        self.run(repo, &["commit", "-q", "-m", file])?;

        self.run(repo, &["rev-parse", "HEAD"])
    }
}

#[cfg(test)]
mod tests;
