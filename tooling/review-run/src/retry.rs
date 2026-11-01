//! The second attempt: an agent whose run did not answer, or a
//! specialist, whole or a part, whose assembled report the gate would
//! refuse, runs once more as the same agent. A specialist is told what
//! was wrong, the gate's problem with the line it names, and the forms
//! again; the second run is judged alone, its own report replacing the
//! first's. Only a second run that again did not answer shows what both
//! stated, under its `incomplete:` head, so nothing either found is lost
//! while the run stays one that did not answer. A second report the gate
//! refuses stays refused: the review names it as it would the first.

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use review_eval::component::Kind;
use review_eval::host::Host;
use review_gate::report::{Problem, assemble, report_check};

use crate::parts::add_usage;
use crate::plan::Plan;
use crate::run::{AgentRun, Unit, attempt, describe, keep_first_log, write_report};

/// The forms, restated to a specialist on its second attempt, in two
/// lines: the finding lines, then the final answer.
pub const FORMS: &str = "Each finding is one line, `docs/x.md:12 — if not addressed: \
<consequence> — <ID>[, <ID>...]` or `docs/x.md:12 — request: add|change|remove <requirement> — \
<motivation>`; `revised: <one full finding line>` replaces the one at its place, nothing else \
beside it, and `withdrawn: docs/x.md:12 — <reason>` removes one. A line is only for a problem: what is \
sound gets none, never `no finding`.\nYour final answer is only the \
outcome line: `outcome: findings — <sentence>` or `outcome: none — <sentence>`.";

/// Why a run is tried once more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Why<'a> {
    /// Its report was refused: the gate's problem, with the line.
    Refused(&'a str),
    /// It did not answer: why.
    Silent(&'a str),
}

/// What the gate refuses `report`, assembled beside `answer`, for,
/// holding its ids to `known`: its problem, and the line it names
/// quoted; none when the gate takes it.
#[must_use]
pub fn refusal(report: &str, answer: &str, known: &BTreeSet<String>) -> Option<String> {
    let problem = report_check(Some(report), Some(answer), Some(known))?;
    let line = match &problem {
        Problem::Malformed { line }
        | Problem::TemplatePlace { line }
        | Problem::Unlinked { line }
        | Problem::UnknownId { line, .. } => report.lines().nth(line - 1),
        _ => None,
    };

    Some(match line {
        Some(text) => format!("{problem}: `{text}`"),
        None => problem.to_string(),
    })
}

/// What a specialist's second attempt is told, after its prompt.
#[must_use]
pub fn note(why: &Why) -> String {
    match why {
        Why::Refused(problem) => format!(
            "The review refused your previous attempt's report: it {problem}. This attempt is \
             judged alone, so state every finding again. {FORMS}"
        ),
        Why::Silent(failure) => format!(
            "Your previous attempt did not answer: {failure}. This attempt is judged alone: \
             state each finding line the moment you find it, read each file once, search no \
             whole tree, and send your outcome line before your turns or time run out. {FORMS}"
        ),
    }
}

/// Runs `unit`, and once more when its run did not answer, or, for a
/// specialist, when the gate would refuse its assembled report, holding
/// its ids to `known`. A definition that does not read fails at once.
/// The first failure or refusal is told to `progress` before the second
/// attempt, which a specialist runs told [`note`], and the first log is
/// kept as `<stem>.first.jsonl` beside the second's. A part's
/// assembled report comes back with its run, for the merge.
pub(crate) fn run_one(
    host: &dyn Host,
    workdir: &Path,
    plan: &Plan,
    known: &BTreeSet<String>,
    unit: &Unit,
    deadline: Duration,
    progress: &(dyn Fn(&str) + Sync),
) -> (AgentRun, Option<String>) {
    let (first, report) = attempt(host, workdir, plan, unit, deadline, None);
    let refused = match (&report, first.answered()) {
        (Some(text), true) => refusal(text, &first.log.answer, known),
        _ => None,
    };
    if first.attempts == 0 || (first.answered() && refused.is_none()) {
        let attempts = first.attempts.max(1);
        return (AgentRun { attempts, ..first }, report);
    }
    let (why, told) = match (&refused, &first.failure) {
        (Some(problem), _) => (
            format!("{} ({}) {problem}", unit.label(), unit.kind.word()),
            Why::Refused(problem),
        ),
        (None, failure) => (
            describe(&first),
            Why::Silent(failure.as_deref().unwrap_or("")),
        ),
    };
    progress(&format!("{why}; running it once more"));
    if let Err(e) = keep_first_log(workdir, &unit.stem()) {
        progress(&format!(
            "{} ({}): its first log was not kept: {e}",
            unit.label(),
            unit.kind.word()
        ));
    }
    let note = (unit.kind == Kind::Specialist).then(|| note(&told));
    let (mut second, report) = attempt(host, workdir, plan, unit, deadline, note.as_deref());
    add_usage(&mut second.log, &first.log);
    second.attempts = 2;
    let report = salvage(workdir, unit, &first, &mut second, report);

    (second, report)
}

/// `report`, the second attempt's, or, when that attempt did not answer
/// either, what both attempts stated under its `incomplete:` head,
/// written as the specialist's report when `unit` is one run whole.
fn salvage(
    workdir: &Path,
    unit: &Unit,
    first: &AgentRun,
    second: &mut AgentRun,
    report: Option<String>,
) -> Option<String> {
    let (Some(_), Some(failure)) = (&report, second.failure.clone()) else {
        return report;
    };
    let texts: Vec<String> = first
        .log
        .texts
        .iter()
        .chain(&second.log.texts)
        .cloned()
        .collect();
    let text = assemble(&texts, &second.log.answer, Some(&failure));
    if report.as_ref() == Some(&text) {
        return report;
    }
    if unit.part.is_none()
        && let Err(e) = write_report(workdir, &unit.name, &text)
    {
        second.failure = Some(format!("{failure}; {e}"));
    }

    Some(text)
}

#[cfg(test)]
mod tests;
