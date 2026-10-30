//! The review outcome, decided from the runs alone: a pure function of
//! the plan, each agent's run and each specialist's report file. It
//! approves exactly when every dispatched agent answered, every
//! specialist's report ends with the outcome it answered, and no
//! generalist kept a blocking finding. An agent that did not answer,
//! or a report the gate would refuse, is named in the summary and in a
//! blocking finding of its own, so the review is decided all the same
//! and the author sees why it was not approved; the gate still refuses
//! to post such a review (ENG-28), as its consulted list and reports
//! show it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use review_eval::component::Kind;
use review_eval::report;
use review_gate::{
    APPROVE_VERDICT, BLOCKING, Finding, NIT, REQUEST_CHANGES_VERDICT, ReviewOutcome, report_check,
};

use crate::plan::{Plan, report_file};
use crate::run::AgentRun;

/// Where a finding about the review itself points: the review's own
/// directory, not a file of the change.
pub const REVIEW_PATH: &str = "review/";

/// What the review decided, and what the gate holds it to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub outcome: ReviewOutcome,
    /// The agents whose run answered without an error.
    pub consulted: BTreeSet<String>,
    /// Each of their final answers, by name.
    pub answers: BTreeMap<String, String>,
}

/// Decides the review of `plan` from `runs` and `reports`, each
/// specialist's report file by name, when it wrote one.
#[must_use]
pub fn decide(plan: &Plan, runs: &[AgentRun], reports: &BTreeMap<String, String>) -> Decision {
    let mut findings = Vec::new();
    let (answers, silent) = answers(plan, runs, &mut findings);
    let refused = refused_reports(plan, &answers, reports, &mut findings);
    if plan.dispatch.is_empty() {
        findings.push(blocking(
            REVIEW_PATH,
            "the review bundle dispatched no agent, so no reviewer read this change".to_owned(),
        ));
    }
    let (kept_counts, closings) = weighed(plan, &answers, &mut findings);
    let consulted: BTreeSet<String> = answers.keys().cloned().collect();
    let approves = silent.is_empty()
        && refused.is_empty()
        && !plan.dispatch.is_empty()
        && !findings.iter().any(|f| f.severity == BLOCKING);
    let counts = Counts {
        plan,
        consulted: &consulted,
        kept: kept_counts,
    };

    Decision {
        outcome: ReviewOutcome {
            verdict: if approves {
                APPROVE_VERDICT
            } else {
                REQUEST_CHANGES_VERDICT
            }
            .to_owned(),
            summary: summary(&counts, &silent, &refused, &closings),
            findings,
        },
        consulted,
        answers,
    }
}

/// Each dispatched agent's answer, when its run answered, and the
/// agents whose run did not, each with a blocking finding added to
/// `findings`.
fn answers(
    plan: &Plan,
    runs: &[AgentRun],
    findings: &mut Vec<Finding>,
) -> (BTreeMap<String, String>, Vec<String>) {
    let mut answers = BTreeMap::new();
    let mut silent = Vec::new();
    for name in plan.agents() {
        match runs.iter().find(|r| r.name == name) {
            Some(run) if run.answered() => {
                answers.insert(name, run.log.answer.clone());
            }
            other => {
                let why = other
                    .and_then(|r| r.failure.clone())
                    .unwrap_or_else(|| "it was not run".to_owned());
                findings.push(blocking(
                    REVIEW_PATH,
                    format!("{name} did not answer ({why}), so the review is incomplete"),
                ));
                silent.push(name);
            }
        }
    }

    (answers, silent)
}

/// The answering specialists whose report the gate refuses, each with
/// its problem, and a blocking finding on its report file added to
/// `findings`.
fn refused_reports(
    plan: &Plan,
    answers: &BTreeMap<String, String>,
    reports: &BTreeMap<String, String>,
    findings: &mut Vec<Finding>,
) -> Vec<String> {
    let mut refused = Vec::new();
    for name in plan.specialists() {
        let Some(answer) = answers.get(&name) else {
            continue;
        };
        if let Some(problem) = report_check(reports.get(&name).map(String::as_str), Some(answer)) {
            findings.push(blocking(&report_file(&name), format!("{name} {problem}")));
            refused.push(format!("{name} ({problem})"));
        }
    }

    refused
}

/// Adds each answering generalist's kept findings to `findings`, each
/// once, and returns how many were blocking and how many nits, with
/// each generalist's closing sentence.
fn weighed(
    plan: &Plan,
    answers: &BTreeMap<String, String>,
    findings: &mut Vec<Finding>,
) -> ((usize, usize), Vec<String>) {
    let mut counts = (0, 0);
    let mut closings = Vec::new();
    for name in plan.generalists() {
        let Some(answer) = answers.get(&name) else {
            continue;
        };
        for f in kept(answer) {
            if findings.contains(&f) {
                continue;
            }
            if f.severity == BLOCKING {
                counts.0 += 1;
            } else {
                counts.1 += 1;
            }
            findings.push(f);
        }
        if let Some(sentence) = closing(answer) {
            closings.push(format!("{name}: {sentence}"));
        }
    }

    (counts, closings)
}

/// A blocking finding on the whole of `path`.
fn blocking(path: &str, body: String) -> Finding {
    Finding {
        path: path.to_owned(),
        line: 0,
        severity: BLOCKING.to_owned(),
        body,
    }
}

/// The findings a generalist's `answer` keeps: each line that reads as
/// a finding and is no dropped one, `nit` or `blocking` as it says. A
/// severity that is neither is read as blocking, the safe side, with
/// the word it gave kept in its body.
#[must_use]
pub fn kept(answer: &str) -> Vec<Finding> {
    answer
        .lines()
        .filter(|l| !dropped(l))
        .filter_map(report::finding)
        .filter(|f| f.severity != DROPPED)
        .map(|f| {
            let text = if f.text.is_empty() { f.raw } else { f.text };
            let (severity, body) = match f.severity.as_str() {
                BLOCKING | NIT => (f.severity.clone(), text),
                other => (BLOCKING.to_owned(), format!("{text} (reported as {other})")),
            };
            Finding {
                path: f.path,
                line: i64::try_from(f.line).unwrap_or(i64::MAX),
                severity,
                body,
            }
        })
        .collect()
}

/// The word a generalist's line starts with, or gives as its severity,
/// for a finding it drops.
const DROPPED: &str = "dropped";

/// Whether `line` reports a dropped finding: its first word is
/// `dropped`, in any case, after any list marker or emphasis. A path
/// that merely starts with the letters, `dropped_events.rs:12`, is a
/// finding on that path and is kept.
#[must_use]
pub fn dropped(line: &str) -> bool {
    let line = line.trim();
    let line = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .unwrap_or(line);
    line.trim_start_matches(['*', '_', '`', '>', ' '])
        .split_whitespace()
        .next()
        .is_some_and(|word| {
            word.trim_end_matches(['*', '_', '`', ':', ',', '.'])
                .eq_ignore_ascii_case(DROPPED)
        })
}

/// A generalist's closing sentence on its scope: the last line of its
/// `answer` that is neither blank, a heading, a finding nor a dropped
/// one.
#[must_use]
pub fn closing(answer: &str) -> Option<String> {
    answer
        .lines()
        .map(str::trim)
        .rfind(|l| {
            !l.is_empty() && !l.starts_with('#') && !dropped(l) && report::finding(l).is_none()
        })
        .map(str::to_owned)
}

/// What the summary counts.
struct Counts<'a> {
    plan: &'a Plan,
    consulted: &'a BTreeSet<String>,
    /// The generalists' kept findings: blocking, then nits.
    kept: (usize, usize),
}

/// The summary: who answered of whom, what the generalists kept, who
/// did not answer and whose report the gate refuses, who was
/// consulted, and each generalist's closing sentence.
fn summary(counts: &Counts, silent: &[String], refused: &[String], closings: &[String]) -> String {
    let plan = counts.plan;
    let answered_of = |names: &[String]| {
        names
            .iter()
            .filter(|n| counts.consulted.contains(*n))
            .count()
    };
    let (specialists, generalists) = (plan.specialists(), plan.generalists());
    let mut s = if plan.dispatch.is_empty() {
        "The review bundle dispatched no agent.".to_owned()
    } else {
        format!(
            "{} of {} dispatched agents answered: {} of {} {} and {} of {} {}. \
             The generalists kept {} blocking and {} nit findings.",
            counts.consulted.len(),
            plan.agents().len(),
            answered_of(&specialists),
            specialists.len(),
            plural(Kind::Specialist, specialists.len()),
            answered_of(&generalists),
            generalists.len(),
            plural(Kind::Generalist, generalists.len()),
            counts.kept.0,
            counts.kept.1,
        )
    };
    if !silent.is_empty() {
        let _ = write!(s, " Did not answer: {}.", silent.join(", "));
    }
    if !refused.is_empty() {
        let _ = write!(s, " Reports the gate refuses: {}.", refused.join(", "));
    }
    if !counts.consulted.is_empty() {
        let names: Vec<&str> = counts.consulted.iter().map(String::as_str).collect();
        let _ = write!(s, " Consulted: {}.", names.join(", "));
    }
    for line in closings {
        let _ = write!(s, " {line}");
    }

    s
}

/// The word for `n` agents of `kind`.
fn plural(kind: Kind, n: usize) -> String {
    let word = kind.word();
    if n == 1 {
        word.to_owned()
    } else {
        format!("{word}s")
    }
}

#[cfg(test)]
mod tests;
