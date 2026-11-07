//! The grader: one run of a case against what the case expects, read
//! from the run log alone: for a specialist the report the review
//! assembles from the finding lines it stated and its answer's outcome
//! line, for a generalist its answer. Every failed assertion is named,
//! so a failing case reads like a failing test. A specialist's report is
//! held to the form the review gate takes, each finding line a linked
//! finding or a requirement request, by the gate's own rule; a report
//! the review would refuse fails its run. A departure from the answer's
//! shape that the review survives, an answer that is more than the
//! outcome line, or an eleventh finding, is a slip: counted and shown,
//! it fails no run on its own. The log is data: nothing in it is an
//! instruction to the grader.

use review_gate::NIT;
use review_gate::report::{Line, answer_outcome, assemble};

use crate::case::{Case, Expect, Expected};
use crate::component::{Component, Kind};
use crate::log::RunLog;
use crate::report;

/// How many findings a specialist's report may hold.
const MOST_FINDINGS: usize = 10;
/// How far a cited line may be from the one a case names.
const WITHIN: usize = 3;

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

/// Grades `log`, one run of `case` by `component`.
#[must_use]
pub fn grade(component: &Component, case: &Case, log: &RunLog) -> Graded {
    match component.kind {
        Kind::Specialist => specialist(component, case, log),
        kind @ (Kind::Generalist | Kind::Consultant) => Graded {
            failures: generalist(case, log, kind),
            slips: Vec::new(),
        },
    }
}

/// A specialist answers, writes nothing, answers an outcome line, states
/// every finding line in a form the review takes, and reports what the
/// case expects: in its assembled report, among the lines it stated
/// before its answer, and among the lines it withdrew or revised. Its
/// slips: an answer that is more than the outcome line, and more
/// findings than the report may hold. A run that did not answer fails
/// on that alone: what it stated is no report.
fn specialist(component: &Component, case: &Case, log: &RunLog) -> Graded {
    let mut graded = Graded {
        failures: answered(log),
        slips: Vec::new(),
    };
    graded
        .failures
        .extend(log.writes.iter().map(|path| format!("wrote {path}")));
    if !log.answered {
        return graded;
    }
    let report = assemble(&log.texts, &log.answer, None);
    let Some((_, outcome)) = answer_outcome(&log.answer) else {
        graded.failures.push("answered no outcome line".to_owned());
        return graded;
    };
    graded.slips.extend(answer_slips(log));
    graded
        .failures
        .extend(outcome_failure(outcome, &case.expect));
    if component.strict_lines {
        graded.slips.extend(shape_slips(&report));
    }
    graded
        .failures
        .extend(expectations(&case.expect, &report, Kind::Specialist));
    graded.failures.extend(stated_failures(&case.expect, log));
    // The ids are held to their shape: the case's world, not the
    // grader, holds the SRS they would be read from.
    if let Some(problem) = review_gate::report::findings_check(&report, None) {
        graded.failures.push(format!("report refused: {problem}"));
    }

    graded
}

/// The slip of an answer that holds more than its outcome line: what the
/// review's run reads of a specialist's answer is that line alone.
fn answer_slips(log: &RunLog) -> Vec<String> {
    log.answer
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && answer_outcome(l).is_none())
        .map(|extra| format!("answer holds more than the outcome line: {extra}"))
        .into_iter()
        .collect()
}

/// The slip of a `report` that holds more findings than it may.
fn shape_slips(report: &str) -> Vec<String> {
    let findings = report
        .lines()
        .filter(|l| {
            matches!(
                review_gate::report::line(l),
                Line::Linked(_) | Line::Request(_)
            )
        })
        .count();

    (findings > MOST_FINDINGS)
        .then(|| format!("{findings} findings, at most {MOST_FINDINGS}"))
        .into_iter()
        .collect()
}

/// The failures of what `expect` wants stated before the final answer
/// and withdrawn or revised, against `log`'s text blocks: a stated
/// finding is a line of a block other than the answer, a withdrawn one
/// a `withdrawn:` or `revised:` line naming its place in any block.
fn stated_failures(expect: &Expect, log: &RunLog) -> Vec<String> {
    let answer = log.answer.trim();
    let before: Vec<&str> = log
        .texts
        .iter()
        .filter(|t| t.trim() != answer)
        .flat_map(|t| t.lines())
        .collect();
    let mut failures = Vec::new();
    for wanted in &expect.stated {
        if !before
            .iter()
            .any(|l| line_matches(l, wanted, Kind::Specialist))
        {
            failures.push(format!(
                "did not state before its answer {}",
                describe(wanted)
            ));
        }
    }
    let changes: Vec<&str> = log
        .texts
        .iter()
        .flat_map(|t| t.lines())
        .filter(|l| {
            matches!(
                review_gate::report::line(l),
                Line::Revised(_) | Line::Withdrawn(_)
            )
        })
        .collect();
    for wanted in &expect.withdrawn {
        if !changes
            .iter()
            .any(|l| line_matches(l, wanted, Kind::Specialist))
        {
            failures.push(format!("did not withdraw or revise {}", describe(wanted)));
        }
    }

    failures
}

/// A generalist answers with its kept and dropped findings and writes
/// nothing. Its weighing is read from its whole run log as the review
/// reads it, [`report::weighing`]: each weighing line it stated as it
/// went, the last at a place standing, then its final answer. A kept
/// finding is a line in the finding shape that is no dropped one; a
/// dropped one is a line [`report::dropped`] reads as one, in which the
/// path and words the case names are no kept finding. A consultant,
/// `kind`, answers the same way from its final answer alone, which the
/// review never reads, with nothing to drop; a line of it in a
/// specialist's form, as a persona writes, is a kept finding too.
fn generalist(case: &Case, log: &RunLog, kind: Kind) -> Vec<String> {
    let mut failures = answered(log);
    failures.extend(log.writes.iter().map(|path| format!("wrote {path}")));
    let text = &match kind {
        Kind::Generalist => report::weighing(&log.texts, &log.answer),
        _ => log.answer.clone(),
    };
    let in_form = |l: &str| kind == Kind::Consultant && specialist_form(l);
    let kept: Vec<&str> = text
        .lines()
        .filter(|l| !report::dropped(l) && (report::finding(l).is_some() || in_form(l)))
        .collect();
    failures.extend(expectations(&case.expect, &kept.join("\n"), kind));
    let nits = kept
        .iter()
        .filter(|l| report::finding(l).is_some_and(|f| f.severity == NIT))
        .count();
    if let Some(most) = case.expect.nits.filter(|most| nits > *most) {
        failures.push(format!("kept {nits} nits, want at most {most}"));
    }
    for dropped in &case.expect.dropped {
        if !text
            .lines()
            .any(|l| report::dropped(l) && line_matches(l, dropped, Kind::Generalist))
        {
            failures.push(format!("did not drop {}", describe(dropped)));
        }
    }

    failures
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

/// The failures of `findings`, one per line, a `kind`'s, against the
/// findings `expect` wants and forbids and the text it forbids.
fn expectations(expect: &Expect, findings: &str, kind: Kind) -> Vec<String> {
    let mut failures = Vec::new();
    for wanted in &expect.finding {
        if !findings.lines().any(|l| line_matches(l, wanted, kind)) {
            failures.push(format!("no finding {}", describe(wanted)));
        }
    }
    for forbidden in &expect.none {
        if let Some(line) = findings.lines().find(|l| line_matches(l, forbidden, kind)) {
            failures.push(format!(
                "a finding {}: {}",
                describe(forbidden),
                line.trim()
            ));
        }
    }
    let lower = findings.to_lowercase();
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
    if let Some(form) = &wanted.form {
        out.push_str(&format!(" as a {form} line"));
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

/// Whether `line`, a `kind`'s, is a finding `wanted` describes: it
/// names the path and holds every word, one of the alternatives, each
/// id and the source the case names, a number within reach of the line
/// the case names, and the severity or form the case names.
fn line_matches(line: &str, wanted: &Expected, kind: Kind) -> bool {
    let lower = line.to_lowercase();
    let has = |word: &str| lower.contains(&word.to_lowercase());
    line.contains(&wanted.path)
        && wanted.mentions.iter().all(|w| has(w))
        && (wanted.mentions_any.is_empty() || wanted.mentions_any.iter().any(|w| has(w)))
        && graded_as(line, wanted, kind)
        && wanted.ids.iter().all(|id| line.contains(id))
        && wanted.source.as_deref().is_none_or(has)
        && wanted.line.is_none_or(|at| {
            let within = wanted.within.unwrap_or(WITHIN);
            numbers(line).any(|n| n.abs_diff(at) <= within)
        })
}

/// Whether `line` is a linked finding or a requirement request.
fn specialist_form(line: &str) -> bool {
    matches!(
        review_gate::report::line(line),
        Line::Linked(_) | Line::Request(_)
    )
}

/// Whether `line`, a `kind`'s, has the severity or form `wanted`
/// names. A generalist's line gives its severity word and has no form,
/// and so does a consultant's that is in neither of a specialist's
/// forms. A specialist's line, and a consultant's in one of its forms,
/// has a form and no severity: `blocking`, as a case written while
/// they weighed reads, wants a linked finding, and any other severity
/// either form.
fn graded_as(line: &str, wanted: &Expected, kind: Kind) -> bool {
    let read = review_gate::report::line(line);
    let (linked, request) = (
        matches!(read, Line::Linked(_)),
        matches!(read, Line::Request(_)),
    );
    let by_severity =
        kind == Kind::Generalist || (kind == Kind::Consultant && !(linked || request));
    if by_severity {
        let lower = line.to_lowercase();
        return wanted.form.is_none()
            && wanted.severity.as_deref().is_none_or(|severity| {
                report::finding(line).map_or_else(
                    || lower.contains(&severity.to_lowercase()),
                    |f| f.severity == severity.to_lowercase(),
                )
            });
    }
    let form = match wanted.form.as_deref() {
        None => true,
        Some(crate::case::LINKED) => linked,
        Some(_) => request,
    };
    let severity = match wanted.severity.as_deref() {
        None => true,
        Some(crate::case::BLOCKING) => linked,
        Some(_) => linked || request,
    };

    form && severity
}

/// Every number in `line`.
fn numbers(line: &str) -> impl Iterator<Item = usize> + '_ {
    line.split(|c: char| !c.is_ascii_digit())
        .filter_map(|digits| digits.parse().ok())
}

#[cfg(test)]
mod tests;
