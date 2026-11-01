//! The run list of one review: a header and one entry per agent run, built
//! from the review's files and its run logs, and written as JSON lines
//! (the schema is in the crate's documentation).
//!
//! An entry is one run log, `review/logs/<name>[.<part>][.first].jsonl`:
//! a `.first` log is an attempt that was not accepted and ran once
//! more, its retry the log without it. Its timing is the log file's: it
//! ended when the file was last written and started its result's
//! duration before, or when the file was made when no result came. Its
//! tokens and turns are the log's; its cost is the `total_cost_usd` the
//! command line reported there, never recomputed from tokens, and
//! nothing for an agent reused from an earlier run. An agent the review
//! did not review and that left no log, or one reused that left none,
//! still has an entry, so no agent the review dispatched is missing.

use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use review_eval::component;
use review_eval::log::{self, RunLog};
use serde::Serialize;

use crate::inputs::{self, Gap, Outcome, RunId, Usage};

/// The verdict a review that could not run is listed with.
pub const FAILED: &str = "failed";
/// The verdict listed when the review wrote none.
pub const NO_VERDICT: &str = "none";

/// The run list's first line.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Header {
    /// The head commit reviewed, or `run-<inputs hash>` without one.
    pub id: String,
    /// The first agent run's start, in milliseconds since the Unix epoch.
    pub started: Option<u64>,
    /// The last agent run's end, in milliseconds since the Unix epoch.
    pub finished: Option<u64>,
    pub verdict: String,
    pub findings_blocking: usize,
    pub findings_nit: usize,
    /// How many agents, or parts, were not reviewed.
    pub gaps: usize,
    /// Why the review could not run, when it could not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
    /// Each review file that did not read.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// Whether a run's answer was taken.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Accepted,
    Gap,
}

/// A run's tokens, each bucket exclusive of the others.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Tokens {
    pub fresh: u64,
    pub cache_write: u64,
    pub cache_read: u64,
    pub output: u64,
}

/// One agent run.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AgentRun {
    pub name: String,
    /// `specialist`, `generalist` or `consultant`.
    pub kind: String,
    /// The part of a specialist's split diff, from one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part: Option<usize>,
    /// The attempt, from one; none ran when it is zero.
    pub attempt: u32,
    pub model: String,
    pub effort: String,
    pub started_ms: Option<u64>,
    pub ended_ms: Option<u64>,
    pub turns: usize,
    pub tokens: Tokens,
    /// The log's `total_cost_usd`; nothing for an agent reused.
    pub cost_usd: f64,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Whether the agent's answer was reused from an earlier run.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub reused: bool,
}

/// A review's run list.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentRuns {
    pub header: Header,
    pub agent_runs: Vec<AgentRun>,
}

impl AgentRuns {
    /// The run list as JSON lines: the header, then each entry.
    ///
    /// # Errors
    ///
    /// Fails when a line does not encode.
    pub fn jsonl(&self) -> Result<String, String> {
        let mut out = String::new();
        let header = serde_json::json!({ "review": self.header });
        out.push_str(&header.to_string());
        out.push('\n');
        for agent_run in &self.agent_runs {
            let line = serde_json::to_string(agent_run)
                .map_err(|e| format!("encode an agent run: {e}"))?;
            out.push_str(&line);
            out.push('\n');
        }

        Ok(out)
    }
}

/// One run log, read with its file times.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogFile {
    pub file: String,
    pub text: String,
    pub modified_ms: Option<u64>,
    pub created_ms: Option<u64>,
}

/// The agent, the part and whether it is a first attempt that a log
/// file named `file` holds; none for a file that is no run log.
#[must_use]
pub fn stem(file: &str) -> Option<(String, Option<usize>, bool)> {
    let stem = file.strip_suffix(".jsonl")?;
    let (stem, first) = match stem.strip_suffix(".first") {
        Some(stem) => (stem, true),
        None => (stem, false),
    };
    let (name, part) = match stem.rsplit_once('.') {
        Some((name, n)) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => {
            (name, n.parse().ok())
        }
        _ => (stem, None),
    };
    (!name.is_empty()).then(|| (name.to_owned(), part, first))
}

/// The run list of the review in `workdir`, `failure` saying why it could
/// not run, when it could not.
#[must_use]
pub fn build(workdir: &Path, failure: Option<&str>) -> AgentRuns {
    let mut notes = Vec::new();
    let outcome = inputs::outcome(workdir, &mut notes);
    let usage = inputs::usage(workdir, &mut notes);
    let gaps = inputs::gaps(workdir, &mut notes);
    let run_id = inputs::run_id(workdir, &mut notes);
    let logs = read_logs(&workdir.join(inputs::LOGS), &mut notes);
    let definition = |name: &str| {
        component::load(workdir, name).map_or_else(|_| Default::default(), |c| (c.model, c.effort))
    };
    let agent_runs = agent_runs(&logs, &gaps, &usage, &definition);
    let header = header(&run_id, &outcome, gaps.len(), &agent_runs, failure, notes);

    AgentRuns { header, agent_runs }
}

/// The run list's header, from what the review ran on, its outcome, how
/// many gaps it left and its entries.
#[must_use]
pub fn header(
    run_id: &RunId,
    outcome: &Outcome,
    gaps: usize,
    agent_runs: &[AgentRun],
    failure: Option<&str>,
    notes: Vec<String>,
) -> Header {
    let id = match (&run_id.head, run_id.inputs.get(..16)) {
        (Some(head), _) => head.clone(),
        (None, Some(inputs)) => format!("run-{inputs}"),
        (None, None) => "unknown".to_owned(),
    };
    let verdict = match failure {
        Some(_) => FAILED.to_owned(),
        None if outcome.verdict.is_empty() => NO_VERDICT.to_owned(),
        None => outcome.verdict.clone(),
    };
    let count = |s: &str| outcome.findings.iter().filter(|f| f.severity == s).count();

    Header {
        id,
        started: agent_runs.iter().filter_map(|s| s.started_ms).min(),
        finished: agent_runs.iter().filter_map(|s| s.ended_ms).max(),
        verdict,
        findings_blocking: count(review_gate::BLOCKING),
        findings_nit: count(review_gate::NIT),
        gaps,
        failure: failure.map(str::to_owned),
        notes,
    }
}

/// Every agent run: one per log in `logs`, then one for each gap and each
/// reused agent no log holds, ordered by agent, part and attempt.
/// `definition` gives an agent's model and effort.
#[must_use]
pub fn agent_runs(
    logs: &[LogFile],
    gaps: &[Gap],
    usage: &[Usage],
    definition: &dyn Fn(&str) -> (String, String),
) -> Vec<AgentRun> {
    let reused = |name: &str| usage.iter().any(|u| u.agent == name && u.reused);
    let stems: Vec<_> = logs
        .iter()
        .filter_map(|l| Some((stem(&l.file)?, l)))
        .collect();
    let mut out: Vec<AgentRun> = stems
        .iter()
        .map(|((name, part, first), file)| {
            let retried = !first
                && stems
                    .iter()
                    .any(|((n, p, f), _)| *f && n == name && p == part);
            let attempt = if retried { 2 } else { 1 };
            let gap = gaps.iter().find(|g| covers(g, name, *part));
            let agent_run = of_log(name, *part, *first, attempt, file, gap, definition);
            if reused(name) {
                AgentRun {
                    reused: true,
                    cost_usd: 0.0,
                    ..agent_run
                }
            } else {
                agent_run
            }
        })
        .collect();
    for gap in gaps {
        if !out.iter().any(|s| covers(gap, &s.name, s.part)) {
            let agent_run = without_log(&gap.name, gap.part, definition);
            out.push(AgentRun {
                status: Status::Gap,
                reason: Some(gap.reason.clone()),
                ..agent_run
            });
        }
    }
    for u in usage.iter().filter(|u| u.reused) {
        if !out.iter().any(|s| s.name == u.agent) {
            out.push(AgentRun {
                reused: true,
                ..without_log(&u.agent, None, definition)
            });
        }
    }
    out.sort_by(|a, b| (&a.name, a.part, a.attempt).cmp(&(&b.name, b.part, b.attempt)));

    out
}

/// Whether `gap` is about agent `name`'s `part`: its own, or the whole
/// agent's.
fn covers(gap: &Gap, name: &str, part: Option<usize>) -> bool {
    gap.name == name && (gap.part.is_none() || gap.part == part)
}

/// The agent run of agent `name`'s `part`, attempt `attempt`, from its log
/// `file`, a first attempt when `first`, not reviewed for `gap`.
fn of_log(
    name: &str,
    part: Option<usize>,
    first: bool,
    attempt: u32,
    file: &LogFile,
    gap: Option<&Gap>,
    definition: &dyn Fn(&str) -> (String, String),
) -> AgentRun {
    let log = log::parse(&file.text);
    let (status, reason) = if first {
        (Status::Gap, Some(retried(&log)))
    } else {
        match gap {
            Some(gap) => (Status::Gap, Some(gap.reason.clone())),
            None => (Status::Accepted, None),
        }
    };
    let (started_ms, ended_ms) = times(log.duration_ms, file.modified_ms, file.created_ms);
    let agent_run = without_log(name, part, definition);
    let model = if log.resolved_model.is_empty() {
        agent_run.model.clone()
    } else {
        log.resolved_model.clone()
    };

    AgentRun {
        attempt,
        model,
        started_ms,
        ended_ms,
        turns: log.turns,
        tokens: tokens(&log),
        cost_usd: log.cost,
        status,
        reason,
        ..agent_run
    }
}

/// The agent run of agent `name`'s `part` that left no log: no attempt, no
/// timing, nothing spent.
fn without_log(
    name: &str,
    part: Option<usize>,
    definition: &dyn Fn(&str) -> (String, String),
) -> AgentRun {
    let (model, effort) = definition(name);
    AgentRun {
        name: name.to_owned(),
        kind: component::kind_of(name).word().to_owned(),
        part,
        model,
        effort,
        ..AgentRun::default()
    }
}

/// Why a first attempt ran once more.
fn retried(log: &RunLog) -> String {
    if log.answered {
        "retried: its answer or report was not accepted".to_owned()
    } else {
        format!("retried: its run ended with {}", log.subtype)
    }
}

/// The tokens of `log`, summed over the models it ran on.
fn tokens(log: &RunLog) -> Tokens {
    log.usage.values().fold(Tokens::default(), |t, u| Tokens {
        fresh: t.fresh + u.fresh,
        cache_write: t.cache_write + u.cache_write,
        cache_read: t.cache_read + u.cache_read,
        output: t.output + u.output,
    })
}

/// A run's start and end: it ended when its log was last written and
/// started `duration_ms` before, or, with no duration, when its log
/// was made.
#[must_use]
pub fn times(
    duration_ms: u64,
    modified_ms: Option<u64>,
    created_ms: Option<u64>,
) -> (Option<u64>, Option<u64>) {
    let started = match (modified_ms, duration_ms) {
        (Some(end), 1..) => Some(end.saturating_sub(duration_ms)),
        _ => created_ms.or(modified_ms),
    };

    (started, modified_ms)
}

/// Every run log in `dir`, by file name; none when there is no `dir`,
/// and each that does not read named in `notes`.
pub fn read_logs(dir: &Path, notes: &mut Vec<String>) -> Vec<LogFile> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            notes.push(format!("{} does not read: {e}", dir.display()));
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for path in entries.filter_map(Result::ok).map(|e| e.path()) {
        let Some(file) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        if !file.ends_with(".jsonl") {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(text) => {
                let meta = fs::metadata(&path).ok();
                out.push(LogFile {
                    file: file.to_owned(),
                    text,
                    modified_ms: meta.as_ref().and_then(|m| ms(m.modified())),
                    created_ms: meta.as_ref().and_then(|m| ms(m.created())),
                });
            }
            Err(e) => notes.push(format!("{} does not read: {e}", path.display())),
        }
    }
    out.sort_by(|a, b| a.file.cmp(&b.file));

    out
}

/// `time` in milliseconds since the Unix epoch.
fn ms(time: io::Result<SystemTime>) -> Option<u64> {
    let since = time.ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some(u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests;
