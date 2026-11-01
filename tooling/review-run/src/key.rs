//! Each agent's answer key (ENG-28): a SHA-256 over what the review
//! declares its answer a function of, so an answer is reused only for
//! the same declared inputs, across commits and runs alike.
//!
//! A specialist's key is over, each field named and its length given:
//! the key's own version, [`SCHEMA`]; its name; its definition file's
//! bytes; the model, effort and turn budget its front matter names; the
//! `claude --version` line of this run; the whole command line the
//! runner gives it, its prompt (the runner's template filled in), tools,
//! refused tools, turn and cost caps among it; the split limit; the
//! pull request's agent instructions as text,
//! `review/pr-instructions.txt`; its own diff,
//! `review/diffs/<name>.diff`; the context the bundle wrote for it,
//! `review/context/<name>.md`; and the [`salt`]. A part of a split
//! diff is keyed by its specialist's key and its place. A generalist's
//! key is over the same fields but a diff, then each of its
//! specialists' keys, so a changed specialist changes its generalists'
//! keys, then, as the generalist starts, the SHA-256 of each report it
//! is told to read, so its answer is reused only beside the reports it
//! weighed.
//!
//! The key under-approximates what an agent reads: it may Read or Grep
//! any file of `pr-head/` or `pr.diff`, and a generalist the whole
//! `review/bundle.md`, which no key holds whole (a specialist reads only
//! its diff and its context, both in its key), so a change there that its own diff does not show is
//! not seen. The salt narrows that: it hashes the files of the pull
//! request most agents read whatever their diff, [`INVALIDATE_ON`], and
//! those main's checkout hands every session, [`SESSION`], so a change
//! to any of them changes every key. Main's agent definitions are not
//! in the salt: each is in its own agent's key.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::Duration;

use review_eval::component;
use review_eval::host::{Command, Host};
use review_eval::runner;
use review_eval::world::{INSTRUCTIONS_TEXT, PR_HEAD};
use sha2::{Digest, Sha256};

use crate::REVIEW;
use crate::plan::{Plan, report_file};
use crate::run::world_of;

/// The key's own version: a change to what a key holds changes it, so
/// no answer keyed the old way is read as keyed the new.
pub const SCHEMA: &str = "review-run answer key 2";

/// The files of the pull request's tree, under `pr-head/`, whose change
/// changes every key: the invariants, the agent instructions, the
/// domain model's index and every agent definition, a path ending in
/// `/` naming each file under it.
pub const INVALIDATE_ON: [&str; 5] = [
    "docs/srs/invariants.md",
    "CLAUDE.md",
    "AGENTS.md",
    "docs/domain-model/index.md",
    ".claude/agents/",
];

/// The files of main's checkout every agent's session reads, whose
/// change changes every key: the instructions and settings
/// `--setting-sources project` loads, the invariants and the domain
/// model's index.
pub const SESSION: [&str; 5] = [
    "CLAUDE.md",
    "AGENTS.md",
    ".claude/settings.json",
    "docs/srs/invariants.md",
    "docs/domain-model/index.md",
];

/// How long `claude --version` may take.
pub const VERSION_DEADLINE: Duration = Duration::from_secs(30);

/// Every dispatched agent's key in one review, and what a generalist's
/// key is completed with as it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    /// Each agent's key before any report, by name; none for one whose
    /// definition does not load.
    own: BTreeMap<String, Option<String>>,
    /// Each generalist's specialists, sorted.
    dispatch: BTreeMap<String, BTreeSet<String>>,
}

/// A hash fed named, length-prefixed fields, so no two field lists
/// hash alike.
struct Fields(Sha256);

impl Fields {
    fn new() -> Self {
        let mut f = Self(Sha256::new());
        f.add("schema", SCHEMA.as_bytes());
        f
    }

    fn add(&mut self, name: &str, bytes: &[u8]) {
        self.0
            .update(format!("{name}\0{}\0", bytes.len()).as_bytes());
        self.0.update(bytes);
    }

    /// Adds the file at `path`, or that it is absent.
    fn file(&mut self, name: &str, path: &Path) {
        match fs::read(path) {
            Ok(bytes) => self.add(name, &bytes),
            Err(e) => self.add(&format!("{name} absent"), e.kind().to_string().as_bytes()),
        }
    }

    fn finish(self) -> String {
        hex(&self.0.finalize())
    }
}

/// `bytes` in lowercase hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether `text` reads as a key: 64 lowercase hex digits.
#[must_use]
pub fn is_key(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The salt every key holds: the files [`INVALIDATE_ON`] names in the
/// pull request's tree under `workdir`, and those [`SESSION`] names in
/// main's, each file there hashed and each absent one marked.
#[must_use]
pub fn salt(workdir: &Path) -> String {
    let mut f = Fields::new();
    for (tree, list) in [
        (workdir.join(PR_HEAD), &INVALIDATE_ON),
        (workdir.to_path_buf(), &SESSION),
    ] {
        for rel in list {
            for (name, path) in files_under(&tree, rel) {
                f.file(&name, &path);
            }
        }
    }
    f.finish()
}

/// The files `rel` names under `tree`, by their path relative to it, in
/// path order: `rel` itself, or, for one ending in `/`, every file under
/// it, `rel` alone when it is no directory that lists.
fn files_under(tree: &Path, rel: &str) -> Vec<(String, std::path::PathBuf)> {
    let Some(dir) = rel.strip_suffix('/') else {
        return vec![(rel.to_owned(), tree.join(rel))];
    };
    let mut out = Vec::new();
    let mut stack = vec![dir.to_owned()];
    while let Some(at) = stack.pop() {
        let Ok(entries) = fs::read_dir(tree.join(&at)) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = format!("{at}/{}", entry.file_name().to_string_lossy());
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(name);
            } else {
                out.push((name.clone(), tree.join(name)));
            }
        }
    }
    out.sort();
    if out.is_empty() {
        out.push((rel.to_owned(), tree.join(rel)));
    }
    out
}

impl Keys {
    /// Every key of `plan`'s agents in `workdir`, run on the `claude`
    /// that printed `claude` as its version, with the split limit
    /// `split`.
    #[must_use]
    pub fn compute(workdir: &Path, plan: &Plan, claude: &str, split: usize) -> Self {
        let salt = salt(workdir);
        let mut own = BTreeMap::new();
        for name in plan.specialists() {
            let key = agent(workdir, plan, &name, claude, split, &salt).map(|mut f| {
                f.file("diff", &workdir.join(format!("{REVIEW}/diffs/{name}.diff")));
                f.file(
                    "context",
                    &workdir.join(format!("{REVIEW}/context/{name}.md")),
                );
                f.finish()
            });
            own.insert(name, key);
        }
        let mut dispatch = BTreeMap::new();
        for name in plan.generalists() {
            let specialists = plan.dispatch.get(&name).cloned().unwrap_or_default();
            let key = agent(workdir, plan, &name, claude, split, &salt).map(|mut f| {
                for s in &specialists {
                    let key = own.get(s).cloned().flatten().unwrap_or_default();
                    f.add("specialist", s.as_bytes());
                    f.add("specialist key", key.as_bytes());
                }
                f.finish()
            });
            own.insert(name.clone(), key);
            dispatch.insert(name, specialists);
        }

        Self { own, dispatch }
    }

    /// Agent `name`'s own key, before any report: a specialist's whole
    /// key, a generalist's before the reports it weighs; none when it
    /// has none.
    #[must_use]
    pub fn own(&self, name: &str) -> Option<&str> {
        self.own.get(name).and_then(Option::as_deref)
    }

    /// The key of part `index` of `count` of specialist `name`'s split
    /// diff; none when the specialist has none.
    #[must_use]
    pub fn part(&self, name: &str, index: usize, count: usize) -> Option<String> {
        let own = self.own(name)?;
        let mut f = Fields::new();
        f.add("specialist key", own.as_bytes());
        f.add("part", format!("{index} of {count}").as_bytes());
        Some(f.finish())
    }

    /// Generalist `name`'s key beside its specialists' reports, each
    /// read by `report`, none when there is none; none when it has no
    /// key of its own.
    #[must_use]
    pub fn generalist(
        &self,
        name: &str,
        report: &dyn Fn(&str) -> Option<Vec<u8>>,
    ) -> Option<String> {
        let own = self.own.get(name).and_then(Option::as_deref)?;
        let mut f = Fields::new();
        f.add("generalist key", own.as_bytes());
        for s in self.dispatch.get(name).into_iter().flatten() {
            f.add("report of", s.as_bytes());
            match report(s) {
                Some(bytes) => f.add("report", hex(&Sha256::digest(&bytes)).as_bytes()),
                None => f.add("no report", b""),
            }
        }
        Some(f.finish())
    }

    /// Generalist `name`'s key beside the reports in `workdir`.
    #[must_use]
    pub fn generalist_in(&self, workdir: &Path, name: &str) -> Option<String> {
        self.generalist(name, &|s| fs::read(workdir.join(report_file(s))).ok())
    }
}

/// The fields every key of agent `name` holds; none when its
/// definition does not load.
fn agent(
    workdir: &Path,
    plan: &Plan,
    name: &str,
    claude: &str,
    split: usize,
    salt: &str,
) -> Option<Fields> {
    let definition = component::file_of(name);
    let bytes = fs::read(workdir.join(&definition)).ok()?;
    let c = component::load(workdir, name).ok()?;
    let world = world_of(workdir, plan, name);
    let command = runner::command(&c, &world, None, Path::new(""), Duration::ZERO);
    let mut f = Fields::new();
    f.add("name", name.as_bytes());
    f.add("kind", c.kind.word().as_bytes());
    f.add("definition", &bytes);
    f.add("model", c.model.as_bytes());
    f.add("effort", c.effort.as_bytes());
    f.add("turns", c.max_turns.to_string().as_bytes());
    f.add("claude", claude.as_bytes());
    f.add("command", command.args.join("\0").as_bytes());
    f.add("split", split.to_string().as_bytes());
    f.file("instructions", &workdir.join(INSTRUCTIONS_TEXT));
    f.add("salt", salt.as_bytes());
    Some(f)
}

/// The version line `claude --version` prints, asked of `host` in
/// `workdir` once, within [`VERSION_DEADLINE`].
///
/// # Errors
///
/// Fails, naming why, when it cannot start, fails, or prints nothing.
pub fn claude_version(host: &dyn Host, workdir: &Path) -> Result<String, String> {
    let command = Command::new(runner::PROGRAM, &["--version"], workdir).within(VERSION_DEADLINE);
    let out = host.run(&command)?;
    if !out.ok() {
        return Err(format!("claude --version failed: {}", out.stderr.trim()));
    }
    let line = out.stdout.trim();
    if line.is_empty() {
        return Err("claude --version printed nothing".to_owned());
    }
    Ok(line.to_owned())
}

#[cfg(test)]
mod tests;
