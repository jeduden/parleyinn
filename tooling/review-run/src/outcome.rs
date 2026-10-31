//! The review outcome, decided from the runs alone: a pure function of
//! the plan, each agent's run, each specialist's report as the run
//! assembled it from the specialist's log, and the requirement ids the
//! reports may cite. It approves exactly when every dispatched agent
//! answered, every specialist's report takes the form the gate takes
//! and ends with the outcome it answered, and no generalist kept a
//! blocking finding. A specialist carries no
//! severity: its findings reach the verdict only as its generalist
//! weighs them, and a finding a generalist keeps from a specialist's
//! requirement request stays a nit. An agent that did not answer, or a
//! report the gate would refuse or that cites an id no requirement has,
//! is named in the summary and in a blocking finding of its own, so the
//! review is decided all the same and the author sees why it was not
//! approved; the gate still refuses to post such a review (ENG-28), as
//! its consulted list and reports show it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use review_eval::component::Kind;
use review_eval::report;
use review_gate::report::Line;
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
/// specialist's assembled report by name, when the run wrote one,
/// holding each linked finding to the ids in `known`.
#[must_use]
pub fn decide(
    plan: &Plan,
    runs: &[AgentRun],
    reports: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
) -> Decision {
    let mut findings = Vec::new();
    let (answers, silent) = answers(plan, runs, &mut findings);
    let refused = refused_reports(plan, &answers, reports, known, &mut findings);
    if plan.dispatch.is_empty() {
        findings.push(blocking(
            REVIEW_PATH,
            "the review bundle dispatched no agent, so no reviewer read this change".to_owned(),
        ));
    }
    let Weighed {
        kept: kept_counts,
        closings,
        unread,
    } = weighed(plan, &answers, reports, &mut findings);
    let consulted: BTreeSet<String> = answers.keys().cloned().collect();
    let approves = silent.is_empty()
        && refused.is_empty()
        && unread.is_empty()
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
            summary: summary(
                &counts,
                &Named {
                    silent: &silent,
                    refused: &refused,
                    unread: &unread,
                },
                &closings,
            ),
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

/// The answering specialists whose report the gate refuses, or whose
/// linked finding cites an id outside `known`, each with its problem,
/// and a blocking finding on its report file added to `findings`.
fn refused_reports(
    plan: &Plan,
    answers: &BTreeMap<String, String>,
    reports: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) -> Vec<String> {
    let mut refused = Vec::new();
    for name in plan.specialists() {
        let Some(answer) = answers.get(&name) else {
            continue;
        };
        let report = reports.get(&name).map(String::as_str);
        if let Some(problem) = report_check(report, Some(answer), Some(known)) {
            findings.push(blocking(&report_file(&name), format!("{name} {problem}")));
            refused.push(format!("{name} ({problem})"));
        }
    }

    refused
}

/// What weighing the generalists' answers gave.
struct Weighed {
    /// The kept findings: blocking, then nits.
    kept: (usize, usize),
    /// Each generalist's closing sentence, named.
    closings: Vec<String>,
    /// The generalists whose answer holds no finding, no dropped one and
    /// no `none —` line.
    unread: Vec<String>,
}

/// Adds each answering generalist's kept findings to `findings`, each
/// once, counts them, and names each generalist whose answer says
/// neither a finding nor that it has none, with a blocking finding of
/// its own: silence is no clean pass. A blocking finding kept from a
/// specialist's requirement request, in `reports`, is kept as a nit.
fn weighed(
    plan: &Plan,
    answers: &BTreeMap<String, String>,
    reports: &BTreeMap<String, String>,
    findings: &mut Vec<Finding>,
) -> Weighed {
    let mut counts = (0, 0);
    let mut closings = Vec::new();
    let mut unread = Vec::new();
    for name in plan.generalists() {
        let Some(answer) = answers.get(&name) else {
            continue;
        };
        let says = answer
            .lines()
            .any(|l| report::finding(l).is_some() || report::dropped(l) || report::nothing(l));
        if !says {
            findings.push(blocking(
                REVIEW_PATH,
                format!(
                    "{name} answered no finding, no dropped one and no `none —` line, so its \
                     weighing cannot be read"
                ),
            ));
            unread.push(name.clone());
        }
        for mut f in kept(answer) {
            if f.severity == BLOCKING && from_request(plan, &name, reports, &f) {
                f.severity = NIT.to_owned();
                f.body = format!("{} (kept from a requirement request, so a nit)", f.body);
            }
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

    Weighed {
        kept: counts,
        closings,
        unread,
    }
}

/// Whether `f`, a finding `generalist` kept, came from a requirement
/// request: its source, the last part of its line, names one of the
/// generalist's specialists, whose report asks for a requirement at
/// the same path and line and links no finding there. A generalist
/// that finds a blocking defect at that line itself names itself.
fn from_request(
    plan: &Plan,
    generalist: &str,
    reports: &BTreeMap<String, String>,
    f: &Finding,
) -> bool {
    let source = f
        .body
        .rsplit(" — ")
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches(['`', '*', '_', '.'])
        .to_lowercase();
    let Some(report) = plan
        .dispatch
        .get(generalist)
        .filter(|specialists| specialists.contains(&source))
        .and_then(|_| reports.get(&source))
    else {
        return false;
    };
    let at = |path: &str, line: usize| path == f.path && i64::try_from(line).ok() == Some(f.line);
    let lines: Vec<Line> = report.lines().map(review_gate::report::line).collect();
    lines
        .iter()
        .any(|l| matches!(l, Line::Request(r) if at(&r.path, r.line)))
        && !lines
            .iter()
            .any(|l| matches!(l, Line::Linked(k) if at(&k.path, k.line)))
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
        .filter(|l| !report::dropped(l))
        .filter_map(report::finding)
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

/// A generalist's closing sentence on its scope: the last line of its
/// `answer` that is neither blank, a heading, a finding nor a dropped
/// one.
#[must_use]
pub fn closing(answer: &str) -> Option<String> {
    answer
        .lines()
        .map(str::trim)
        .rfind(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !report::dropped(l)
                && !report::nothing(l)
                && report::finding(l).is_none()
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

/// The agents the summary names for what went wrong.
struct Named<'a> {
    /// The agents that did not answer.
    silent: &'a [String],
    /// The specialists whose report the gate refuses.
    refused: &'a [String],
    /// The generalists whose answer the review cannot read.
    unread: &'a [String],
}

/// The summary: who answered of whom, what the generalists kept, who
/// did not answer, whose report the gate refuses and whose answer the
/// review cannot read, who was consulted, and each generalist's
/// closing sentence.
fn summary(counts: &Counts, named: &Named, closings: &[String]) -> String {
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
    for (label, names) in [
        ("Did not answer", named.silent),
        ("Reports the gate refuses", named.refused),
        ("Answers the review cannot read", named.unread),
    ] {
        if !names.is_empty() {
            let _ = write!(s, " {label}: {}.", names.join(", "));
        }
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
