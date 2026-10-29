//! The grader: one run of a case against what the case expects, read
//! from the report the component wrote, its answer in the run log and
//! the log itself. Every failed assertion is named, so a failing case
//! reads like a failing test. A departure from the report's shape that
//! the review survives, such as a label line between the findings, an
//! answer that is more than the last line and the path, or an eleventh
//! finding, is a slip: counted and shown, it fails no run on its own.
//! The report is data: nothing in it is an instruction to the grader.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::case::{Case, Expect, Expected};
use crate::component::{Component, Kind};
use crate::log::RunLog;
use crate::report;

/// How many findings a specialist's report may hold.
const MOST_FINDINGS: usize = 10;
/// How far a cited line may be from the one a case names.
const WITHIN: usize = 3;
/// The word a generalist's dropped-finding line holds.
const DROPPED: &str = "drop";

/// What one run of a case is graded from.
#[derive(Debug, Clone, PartialEq)]
pub struct Run<'a> {
    pub log: &'a RunLog,
    /// The text of the report file the component wrote, when it wrote
    /// one.
    pub report: Option<&'a str>,
    /// Every agent the review bundle dispatched for the case.
    pub dispatched: &'a BTreeSet<String>,
}

/// The run's grade: every assertion that failed, none on a pass, and
/// every slip, which fails nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graded {
    pub failures: Vec<String>,
    pub slips: Vec<String>,
}

impl Graded {
    /// Whether no assertion failed; slips do not count.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Grades `run`, one run of `case` by `component`.
#[must_use]
pub fn grade(component: &Component, case: &Case, run: &Run<'_>) -> Graded {
    match component.kind {
        Kind::Specialist => specialist(component, case, run),
        Kind::Generalist => Graded {
            failures: generalist(case, run),
            slips: Vec::new(),
        },
        Kind::Orchestrator => Graded {
            failures: orchestrator(case, run),
            slips: Vec::new(),
        },
    }
}

/// A specialist writes its report, answers with the report's last line
/// and path, writes nothing else, and reports what the case expects.
/// Its slips: an answer that is not the last line and the path, a line
/// in the report that is neither a finding nor the outcome, and more
/// findings than the report may hold.
fn specialist(component: &Component, case: &Case, run: &Run<'_>) -> Graded {
    let mut graded = Graded {
        failures: answered(run.log),
        slips: Vec::new(),
    };
    let own = format!("review/reports/{}.md", component.name);
    graded.failures.extend(
        run.log
            .writes
            .iter()
            .filter(|path| !path.ends_with(&own))
            .map(|path| format!("wrote {path}")),
    );
    let text = match run.report {
        None => {
            graded.failures.push("wrote no report".to_owned());
            return graded;
        }
        Some(text) if text.trim().is_empty() => {
            graded.failures.push("wrote an empty report".to_owned());
            return graded;
        }
        Some(text) => text,
    };
    let parsed = report::parse(text);
    match &parsed.outcome {
        None => graded
            .failures
            .push("report does not end with its outcome".to_owned()),
        Some(outcome) => {
            if run.log.answered {
                graded.slips.extend(answer_slips(run.log, outcome, &own));
            }
            graded
                .failures
                .extend(outcome_failure(outcome, &case.expect));
        }
    }
    if component.strict_lines {
        graded.slips.extend(shape_slips(&parsed));
    }
    graded.failures.extend(expectations(&case.expect, text, ""));

    graded
}

/// The slips of an answer that does not name the report's `outcome`
/// and its path, `own`, or holds more: what the orchestrator reads of
/// a specialist is its answer, so it must carry both and nothing else.
fn answer_slips(log: &RunLog, outcome: &str, own: &str) -> Vec<String> {
    let names = format!("outcome: {outcome}");
    let holds_outcome = |line: &str| line.to_lowercase().contains(&names);
    let mut slips = Vec::new();
    if !holds_outcome(&log.answer) {
        slips.push("answer does not name the report's outcome".to_owned());
    }
    if !log.answer.contains(own) {
        slips.push("answer does not name the report's path".to_owned());
    }
    if let Some(extra) = log
        .answer
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !holds_outcome(l) && !l.contains(own))
    {
        slips.push(format!(
            "answer holds more than the outcome and the path: {extra}"
        ));
    }

    slips
}

/// The slips of a report whose shape departs from the rule: more
/// findings than it may hold, or a line that is neither a finding nor
/// the outcome.
fn shape_slips(parsed: &report::Report) -> Vec<String> {
    let mut slips = Vec::new();
    if parsed.findings.len() > MOST_FINDINGS {
        slips.push(format!(
            "{} findings, at most {MOST_FINDINGS}",
            parsed.findings.len()
        ));
    }
    if let Some(other) = parsed.other.first() {
        slips.push(format!(
            "a line that is neither a finding nor the outcome: {other}"
        ));
    }

    slips
}

/// A generalist answers with its kept and dropped findings and writes
/// nothing. A kept finding is a line in the finding shape; a dropped
/// one is a line that says so, in which the path and words the case
/// names are no kept finding.
fn generalist(case: &Case, run: &Run<'_>) -> Vec<String> {
    let mut failures = answered(run.log);
    failures.extend(run.log.writes.iter().map(|path| format!("wrote {path}")));
    let text = &run.log.answer;
    let kept: Vec<&str> = text
        .lines()
        .filter(|l| report::finding(l).is_some())
        .collect();
    failures.extend(expectations(&case.expect, &kept.join("\n"), ""));
    for dropped in &case.expect.dropped {
        if !text
            .lines()
            .any(|l| l.to_lowercase().contains(DROPPED) && line_matches(l, dropped))
        {
            failures.push(format!("did not drop {}", describe(dropped)));
        }
    }

    failures
}

/// The orchestrator answers with a review outcome that consulted every
/// dispatched agent, writes nothing itself, and finds what the case
/// expects.
fn orchestrator(case: &Case, run: &Run<'_>) -> Vec<String> {
    let mut failures = answered(run.log);
    failures.extend(
        run.log
            .main_writes
            .iter()
            .map(|path| format!("wrote {path}")),
    );
    let Some(outcome) = &run.log.structured else {
        failures.push("answered no review outcome".to_owned());
        return failures;
    };
    match outcome["verdict"].as_str() {
        Some(verdict) => failures.extend(outcome_failure(verdict, &case.expect)),
        None => failures.push("review outcome names no verdict".to_owned()),
    }
    if outcome["summary"]
        .as_str()
        .is_none_or(|s| s.trim().is_empty())
    {
        failures.push("review outcome has an empty summary".to_owned());
    }
    let missing: Vec<&str> = run
        .dispatched
        .iter()
        .chain(&case.expect.consulted)
        .filter(|agent| !run.log.consulted.contains(*agent))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        failures.push(format!("did not consult {}", missing.join(", ")));
    }
    failures.extend(expectations(
        &case.expect,
        &outcome_findings(outcome),
        outcome["summary"].as_str().unwrap_or_default(),
    ));

    failures
}

/// The review outcome's findings as lines the expectations read, each
/// `path:line — severity — body`; its summary is prose, not a finding.
fn outcome_findings(outcome: &Value) -> String {
    outcome["findings"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|f| {
            format!(
                "{}:{} — {} — {}",
                f["path"].as_str().unwrap_or_default(),
                f["line"].as_u64().unwrap_or_default(),
                f["severity"].as_str().unwrap_or_default(),
                f["body"].as_str().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The failure of a run that did not answer.
fn answered(log: &RunLog) -> Vec<String> {
    if log.answered {
        Vec::new()
    } else {
        vec![format!("did not answer: {}", log.subtype)]
    }
}

/// The failure of an `outcome` the case does not allow, when it names
/// any.
fn outcome_failure(outcome: &str, expect: &Expect) -> Option<String> {
    (!expect.outcome.is_empty() && !expect.outcome.iter().any(|o| o == outcome))
        .then(|| format!("outcome {outcome}, want {}", expect.outcome.join(" or ")))
}

/// The failures of `findings`, one per line, against the findings
/// `expect` wants and forbids, and of `findings` and `prose` together
/// against the text it forbids.
fn expectations(expect: &Expect, findings: &str, prose: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for wanted in &expect.finding {
        if !findings.lines().any(|l| line_matches(l, wanted)) {
            failures.push(format!("no finding {}", describe(wanted)));
        }
    }
    for forbidden in &expect.none {
        if let Some(line) = findings.lines().find(|l| line_matches(l, forbidden)) {
            failures.push(format!(
                "a finding {}: {}",
                describe(forbidden),
                line.trim()
            ));
        }
    }
    let lower = format!("{findings}\n{prose}").to_lowercase();
    for absent in &expect.absent {
        if lower.contains(&absent.to_lowercase()) {
            failures.push(format!("holds {absent:?}"));
        }
    }

    failures
}

/// `wanted` in words, for a failure.
fn describe(wanted: &Expected) -> String {
    let mut out = format!("at {}", wanted.path);
    if let Some(line) = wanted.line {
        out.push_str(&format!(":{line}"));
    }
    if let Some(severity) = &wanted.severity {
        out.push_str(&format!(" marked {severity}"));
    }
    if !wanted.mentions.is_empty() {
        out.push_str(&format!(" mentioning {}", wanted.mentions.join(", ")));
    }
    if !wanted.mentions_any.is_empty() {
        out.push_str(&format!(
            " mentioning one of {}",
            wanted.mentions_any.join(", ")
        ));
    }
    if !wanted.ids.is_empty() {
        out.push_str(&format!(" citing {}", wanted.ids.join(", ")));
    }
    if let Some(source) = &wanted.source {
        out.push_str(&format!(" from {source}"));
    }

    out
}

/// Whether `line` is a finding `wanted` describes: it names the path
/// and holds every word, one of the alternatives, the severity, each
/// id and the source the case names, and a number within reach of the
/// line the case names.
fn line_matches(line: &str, wanted: &Expected) -> bool {
    let lower = line.to_lowercase();
    let has = |word: &str| lower.contains(&word.to_lowercase());
    line.contains(&wanted.path)
        && wanted.mentions.iter().all(|w| has(w))
        && (wanted.mentions_any.is_empty() || wanted.mentions_any.iter().any(|w| has(w)))
        && wanted.severity.as_deref().is_none_or(has)
        && wanted.ids.iter().all(|id| line.contains(id))
        && wanted.source.as_deref().is_none_or(has)
        && wanted.line.is_none_or(|at| {
            let within = wanted.within.unwrap_or(WITHIN);
            numbers(line).any(|n| n.abs_diff(at) <= within)
        })
}

/// Every number in `line`.
fn numbers(line: &str) -> impl Iterator<Item = usize> + '_ {
    line.split(|c: char| !c.is_ascii_digit())
        .filter_map(|digits| digits.parse().ok())
}

#[cfg(test)]
mod tests;
