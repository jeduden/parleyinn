//! A specialist's diff too large for one run, run in parts: the diff
//! the review bundle wrote for it, past [`LIMIT`], split by file in
//! path order, each part written beside it as `<name>.<i>.diff`; each
//! part run as the same agent with the same prompt, told which part it
//! has and every part's files; and the parts' reports and runs merged
//! into the specialist's one report and run. A part that did not answer
//! leaves the specialist one that did not answer, its report headed
//! incomplete and naming that part, with every part's findings kept.

use std::fs;
use std::path::Path;

use review_bundle::cli::DIFFS;
use review_bundle::diff;
use review_eval::component::Kind;
use review_eval::log::{RunLog, SUCCESS};
use review_gate::report::{FINDINGS, INCOMPLETE, NONE, answer_outcome};

use crate::REVIEW;
use crate::run::AgentRun;

/// The size, in bytes, past which a specialist's diff runs in parts,
/// each at most this large unless one file is larger: one read of the
/// agent's file tool returns about 50 KB, and a run whose part took
/// three reads ran out of turns before it stated a finding.
pub const LIMIT: usize = 48 * 1024;

/// One part of a specialist's split diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// Its place, from one.
    pub index: usize,
    /// How many parts there are.
    pub count: usize,
    /// Its diff file, relative to the work directory, which its prompt
    /// tells it to read in place of its specialist's whole diff.
    pub diff: String,
    /// What its prompt adds: which part it has, its diff file, and
    /// every part's files.
    pub note: String,
}

impl Part {
    /// How progress names `name`'s run of this part.
    #[must_use]
    pub fn label(&self, name: &str) -> String {
        format!("{name} part {} of {}", self.index, self.count)
    }

    /// The stem of this part's log and diff file names.
    #[must_use]
    pub fn stem(&self, name: &str) -> String {
        format!("{name}.{}", self.index)
    }
}

/// `name`'s diff file, or its part `index`'s, relative to the work
/// directory.
fn diff_file(stem: &str) -> String {
    format!("{REVIEW}/{DIFFS}/{stem}.diff")
}

/// The parts specialist `name` runs in, under the work directory
/// `workdir`: none when its diff is missing or within `limit`, else
/// its diff split by file, each part written beside it.
///
/// # Errors
///
/// Fails when its diff is there but does not read, or a part's diff
/// cannot be written, so a failure is named, never run as no diff (I6).
pub fn prepare(workdir: &Path, name: &str, limit: usize) -> Result<Vec<Part>, String> {
    let path = workdir.join(diff_file(name));
    let text = match fs::read(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    if text.len() <= limit {
        return Ok(Vec::new());
    }
    let split = diff::split(&String::from_utf8_lossy(&text), limit);
    let count = split.len();
    let by_part: Vec<String> = split
        .iter()
        .enumerate()
        .map(|(at, p)| format!("part {}: {}", at + 1, p.files.join(", ")))
        .collect();
    let by_part = by_part.join("; ");
    let mut parts = Vec::with_capacity(count);
    for (at, p) in split.into_iter().enumerate() {
        let index = at + 1;
        let own = diff_file(&format!("{name}.{index}"));
        let path = workdir.join(&own);
        fs::write(&path, &p.text).map_err(|e| format!("write {}: {e}", path.display()))?;
        let note = format!(
            "Your diff is too large for one run, so the review runs you in {count} parts \
             and merges their reports: you are part {index} of {count}. Read only your \
             part's diff, `{own}`, not `{}`, and state findings on its files; you may \
             Read and Grep the rest of the tree. The full diff's files, by part: \
             {by_part}.",
            diff_file(name)
        );
        parts.push(Part {
            index,
            count,
            diff: own,
            note,
        });
    }

    Ok(parts)
}

/// Specialist `name`'s run and report merged from its parts' runs and
/// assembled reports, in part order: every part's finding lines, a line
/// identical to an earlier one dropped; then, when every part answered,
/// one outcome line, findings when any part's is, else none, saying
/// each part's, and that line as the answer, but no outcome line when a
/// part's report has none, so the report check refuses it. A part that
/// did not answer makes the run one that did not answer, its report
/// headed incomplete naming each such part. Costs, turns, usage and
/// attempts add up.
#[must_use]
pub fn merge(name: &str, parts: Vec<(AgentRun, String)>) -> (AgentRun, String) {
    let count = parts.len();
    let mut failures = Vec::new();
    let mut failed_parts = Vec::new();
    let mut outcomes = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut log = RunLog {
        answered: true,
        subtype: SUCCESS.to_owned(),
        ..RunLog::default()
    };
    let mut attempts = 0;
    for (at, (run, report)) in parts.into_iter().enumerate() {
        let place = format!("part {} of {count}", at + 1);
        if let Some(why) = &run.failure {
            failures.push(format!("{place}: {why}"));
            failed_parts.push((at + 1, why.clone()));
            if log.answered {
                log.subtype.clone_from(&run.log.subtype);
            }
            log.answered = false;
        }
        attempts += run.attempts;
        add(&mut log, run.log);
        let mut outcome = None;
        for line in report.lines() {
            if line.starts_with(INCOMPLETE) {
                continue;
            }
            if let Some((text, word)) = answer_outcome(line) {
                outcome = Some((text.to_owned(), word));
            } else if !lines.iter().any(|l| l == line) {
                lines.push(line.to_owned());
            }
        }
        outcomes.push((at + 1, outcome));
    }
    if count == 0 {
        failures.push("it ran in no part".to_owned());
        log.answered = false;
    }
    let mut report = String::new();
    let failure = (!failures.is_empty()).then(|| failures.join("; "));
    if let Some(why) = &failure {
        report.push_str(&format!("{INCOMPLETE} {why}\n"));
    }
    for line in &lines {
        report.push_str(line);
        report.push('\n');
    }
    log.answer = match (&failure, merged_outcome(count, &outcomes)) {
        (None, Some(outcome)) => {
            report.push_str(&outcome);
            report.push('\n');
            outcome
        }
        _ => String::new(),
    };

    (
        AgentRun {
            name: name.to_owned(),
            kind: Kind::Specialist,
            failure,
            log,
            attempts,
            parts: u32::try_from(count).unwrap_or(u32::MAX),
            failed_parts,
            reused: false,
            from_store: false,
            answer_key: None,
            store_failed: false,
        },
        report,
    )
}

/// The outcome line of `count` parts whose outcome lines and words are
/// `outcomes`, by part, or none when a part has none.
fn merged_outcome(count: usize, outcomes: &[(usize, Option<(String, &str)>)]) -> Option<String> {
    let mut word = NONE;
    let mut said = Vec::new();
    for (index, outcome) in outcomes {
        let (text, part_word) = outcome.as_ref()?;
        if *part_word == FINDINGS {
            word = FINDINGS;
        }
        let rest = text
            .split_once(':')
            .map_or(text.as_str(), |(_, r)| r)
            .trim();
        said.push(format!("part {index}, {rest}"));
    }

    Some(format!(
        "outcome: {word} — in {count} parts: {}",
        said.join("; ")
    ))
}

/// Adds `part`'s cost, turns, duration, usage, texts and writes to
/// `log`, which keeps the first model it resolved to.
fn add(log: &mut RunLog, part: RunLog) {
    add_usage(log, &part);
    if log.resolved_model.is_empty() {
        log.resolved_model = part.resolved_model;
    }
    log.texts.extend(part.texts);
    log.writes.extend(part.writes);
}

/// Adds `other`'s cost, turns, duration and usage per model to `log`,
/// what a run spent whether or not it is judged by its texts.
pub(crate) fn add_usage(log: &mut RunLog, other: &RunLog) {
    log.cost += other.cost;
    log.turns += other.turns;
    log.duration_ms += other.duration_ms;
    for (model, u) in &other.usage {
        let sum = log.usage.entry(model.clone()).or_default();
        sum.calls += u.calls;
        sum.fresh += u.fresh;
        sum.cache_write += u.cache_write;
        sum.cache_read += u.cache_read;
        sum.output += u.output;
        sum.cost += u.cost;
    }
}

#[cfg(test)]
mod tests;
