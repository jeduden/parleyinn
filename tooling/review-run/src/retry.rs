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
use review_gate::report::{Problem, assemble, report_check};

use crate::LOGS;
use crate::backoff::{self, Class, MAX_ATTEMPTS};
use crate::controls::Controls;
use crate::live::Running;
use crate::parts::add_usage;
use crate::run::{AgentRun, Review, Unit, attempt, describe, keep_log, not_started, write_report};

/// The forms, restated to a specialist on its second attempt, in two
/// lines: the finding lines, then the final answer.
pub const FORMS: &str = "Each finding is one line, `docs/x.md:12 — if not addressed: \
<consequence> — <ID>[, <ID>...]` or `docs/x.md:12 — request: add|change|remove <requirement> — \
<motivation>`; `revised: <one full finding line>` replaces the one at its place, nothing else \
beside it, and `withdrawn: docs/x.md:12 — <reason>` removes one. A line is only for a problem: what is \
sound gets none, never `no finding`. A finding line ends with its ids, bare, in no backticks and with \
no remark after them: state one only once you have checked it.\nYour final answer is only the \
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
        | Problem::Undashed { line }
        | Problem::AfterIds { line }
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
             state each finding line the moment you have checked it, read each file once, search no \
             whole tree, and send your outcome line before your turns or time run out. {FORMS}"
        ),
    }
}

/// Runs `unit`, and once more when its run did not answer, or, for a
/// specialist, when the gate would refuse its assembled report, holding
/// its ids to the known ones. A definition that does not read, a run a
/// cost cap stopped or kept from starting, and a run whose credential
/// was refused are not run again, and progress is told why. A run the
/// API turned away as overloaded first waits, as `backoff` says, and an
/// attempt so turned away is followed by another while waits are left.
/// The first failure or refusal is told to progress before the second
/// attempt, which a specialist runs told [`note`], and the first log is
/// kept as `<stem>.first.jsonl` beside the second's, a second turned
/// away as `<stem>.second.jsonl`. No attempt starts once the finished
/// runs reached the review's cost cap. A part's assembled report comes
/// back with its run, for the merge.
pub(crate) fn run_one(r: &Review, c: &Controls, unit: &Unit) -> (AgentRun, Option<String>) {
    let started = r.clock.now();
    let (first, report) = tried(r, c, unit, None);
    let refused = match (&report, first.answered()) {
        (Some(text), true) => refusal(text, &first.log.answer, r.known),
        _ => None,
    };
    // A run whose definition did not read keeps its count of none.
    if first.attempts == 0 || (first.answered() && refused.is_none()) {
        return (first, report);
    }
    let class = match &refused {
        Some(_) => Class::Again,
        None => class_of(r.workdir, unit, &first),
    };
    if let Class::Never(why) = class {
        (r.progress)(&format!("{}; not run again: {why}", describe(&first)));
        return (first, report);
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
    if class == Class::Again {
        (r.progress)(&format!("{why}; running it once more"));
    }
    keep(r, unit, "first");
    let note = (unit.kind == Kind::Specialist).then(|| note(&told));
    let first = Attempt {
        run: &first,
        report,
        class,
        started,
    };
    again(r, c, unit, first, note.as_deref())
}

/// The first attempt, as the attempts after it need it.
struct Attempt<'a> {
    run: &'a AgentRun,
    report: Option<String>,
    /// What running it again would do.
    class: Class,
    /// When it started.
    started: Duration,
}

/// The attempts after `first`, each told `note`: one, and another after
/// each turned away as overloaded while waits are left, each such
/// waiting first; the last judged alone.
fn again(
    r: &Review,
    c: &Controls,
    unit: &Unit,
    first: Attempt,
    note: Option<&str>,
) -> (AgentRun, Option<String>) {
    let mut last = (first.run.clone(), first.report);
    let mut class = first.class;
    let mut waited = 0;
    loop {
        if class == Class::Overloaded {
            pause(r, &last.0, waited, first.started);
            waited += 1;
        }
        if let Some(why) = c.budget.refusal() {
            let (mut run, report) = last;
            let before = run.failure.unwrap_or_default();
            run.failure = Some(format!("{why} again; before that {before}"));
            return (run, report);
        }
        let (mut next, report) = tried(r, c, unit, note);
        add_usage(&mut next.log, &last.0.log);
        next.attempts = last.0.attempts + 1;
        class = class_of(r.workdir, unit, &next);
        if next.answered() || next.attempts >= MAX_ATTEMPTS || class != Class::Overloaded {
            let report = salvage(r.workdir, unit, first.run, &mut next, report);
            return (next, report);
        }
        keep(r, unit, "second");
        last = (next, report);
    }
}

/// Runs `unit` once, with `note`, unless the review's finished runs
/// reached its cost cap: noted as running while it runs, and its cost
/// added once it ends.
fn tried(r: &Review, c: &Controls, unit: &Unit, note: Option<&str>) -> (AgentRun, Option<String>) {
    if let Some(why) = c.budget.refusal() {
        return not_started(r.workdir, unit, why);
    }
    let label = unit.label();
    c.live.start(
        &label,
        Running {
            kind: unit.kind,
            log: r.workdir.join(LOGS).join(format!("{}.jsonl", unit.stem())),
            since: r.clock.now(),
        },
    );
    let done = attempt(r.host, r.workdir, r.plan, unit, r.options.deadline, note);
    c.live.end(&label);
    c.budget.charge(done.0.log.cost);

    done
}

/// What running `unit` again would do after `run` failed, from its
/// failure and the errors in the `.stderr` beside its log.
fn class_of(workdir: &Path, unit: &Unit, run: &AgentRun) -> Class {
    let errors = workdir.join(LOGS).join(format!("{}.stderr", unit.stem()));
    let stderr = std::fs::read_to_string(errors).unwrap_or_default();
    backoff::class(run.failure.as_deref().unwrap_or_default(), &stderr)
}

/// Waits before a run again after `run` was turned away as overloaded,
/// `waited` waits so far, the agent started at `started`: not at all
/// once its deadline is spent.
fn pause(r: &Review, run: &AgentRun, waited: u32, started: Duration) {
    let ran = r.clock.now().saturating_sub(started);
    let left = r.options.deadline.saturating_sub(ran);
    let Some(wait) = backoff::wait(waited, r.clock.jitter(), left) else {
        (r.progress)(&format!(
            "{}; the API was overloaded and its deadline is spent, so it runs once more now",
            describe(run)
        ));
        return;
    };
    (r.progress)(&format!(
        "{}; the API was overloaded, so it waits {:.1} s and runs once more",
        describe(run),
        wait.as_secs_f64()
    ));
    r.clock.sleep(wait);
}

/// Keeps `unit`'s log as `which`, telling progress when it cannot.
fn keep(r: &Review, unit: &Unit, which: &str) {
    if let Err(e) = keep_log(r.workdir, &unit.stem(), which) {
        (r.progress)(&format!(
            "{} ({}): its {which} log was not kept: {e}",
            unit.label(),
            unit.kind.word()
        ));
    }
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
