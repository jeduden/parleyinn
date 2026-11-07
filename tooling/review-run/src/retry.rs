//! The attempts after the first. A run that was charged, by its cost or
//! by model calls that used tokens, ran, and is never run again, cut off
//! or not: its log is read as it stands, so no run is paid for twice.
//! Only a run that cost nothing, which did not run, runs again, as the
//! same agent: once, a specialist told why its first did not run and
//! the forms again, and judged by that run alone; a run the API turned
//! away as overloaded waits first, and runs a third time at most.

use std::collections::BTreeSet;
use std::time::Duration;

use review_eval::component::Kind;
use review_eval::log::RunLog;
use review_eval::runner;
use review_gate::report::{Problem, report_check};

use crate::LOGS;
use crate::backoff::{self, Class, MAX_ATTEMPTS};
use crate::controls::{Controls, Reservation};
use crate::live::Running;
use crate::parts::add_usage;
use crate::run::{AgentRun, Review, Unit, attempt, describe, keep_log, keep_spend, not_started};

/// Why an overloaded run gets no wait and so no further run.
const SPENT: &str = "the API was overloaded and its backoff deadline is spent";

/// The forms, restated to a specialist on its second attempt, in two
/// lines: the finding lines, then the final answer.
pub const FORMS: &str = "Each finding is one line, `docs/x.md:12 — if not addressed: \
<consequence> — <ID>[, <ID>...]` or `docs/x.md:12 — request: add|change|remove <requirement> — \
<motivation>`; `revised: <one full finding line>` replaces the one at its place, nothing else \
beside it, and `withdrawn: docs/x.md:12 — <reason>` removes one. A line is only for a problem: what is \
sound gets none, never `no finding`. A finding line ends with its ids, bare, in no backticks and with \
no remark after them: state one only once you have checked it.\nYour final answer is only the \
outcome line: `outcome: findings — <sentence>` or `outcome: none — <sentence>`.";

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

/// What a specialist's second attempt is told, after its prompt, when
/// its first run did not run for `failure` and cost nothing.
#[must_use]
pub fn note(failure: &str) -> String {
    format!(
        "Your previous attempt did not run: {failure}. This attempt is judged alone: \
         state each finding line the moment you have checked it, read each file once, search no \
         whole tree, and send your outcome line before your turns or time run out. {FORMS}"
    )
}

/// Runs `unit`, and once more when it did not run, its log holding no
/// model call. A definition that does not read, a run a cost cap kept
/// from starting, a run whose credential was refused and a run that was
/// charged, cut off or not, are not run again, and progress is told why:
/// a cut run by its class, a cost cap's, or by its charge. A run the API turned away as
/// overloaded first waits, as `backoff` says, and an attempt so turned
/// away is followed by another while waits are left and it was not
/// charged. The first failure is told to progress before the second
/// attempt, which a specialist runs told [`note`], and the first log is
/// kept as `<stem>.first.jsonl` beside the second's, a second turned
/// away as `<stem>.second.jsonl`. No attempt starts until its tier's
/// cap fits in the pull request's cost cap. A part's assembled report comes
/// back with its run, for the merge.
pub(crate) fn run_one(r: &Review, c: &Controls, unit: &Unit) -> (AgentRun, Option<String>) {
    let started = r.clock.now();
    let (first, report) = tried(r, c, unit, None);
    // A run whose definition did not read keeps its count of none.
    if first.attempts == 0 || (first.failure.is_none() && first.cut.is_none()) {
        return (first, report);
    }
    let class = class_of(r, unit, &first);
    if let Class::Never(why) = class {
        not_again(r, &first, why);
        return (first, report);
    }
    // A run that was paid for, cut off or with a report the review could
    // not keep, ran: it is not paid for again.
    if let Some(why) = charged(&first.log) {
        not_again(r, &first, &why);
        return (first, report);
    }
    if class == Class::Again {
        (r.progress)(&format!("{}; running it once more", describe(&first)));
    }
    keep(r, unit, "first");
    let note =
        (unit.kind == Kind::Specialist).then(|| note(first.failure.as_deref().unwrap_or("")));
    let first = Attempt {
        run: &first,
        report,
        class,
        started,
    };
    again(r, c, unit, first, note.as_deref())
}

/// Tells progress that `run` of `unit`, cut off, is not run again: a
/// cut run ran, so it was charged. Why is its class's when it may never
/// run again, else its charge.
fn tell_cut(r: &Review, unit: &Unit, run: &AgentRun) {
    let why = match class_of(r, unit, run) {
        Class::Never(why) => why.to_owned(),
        _ => charged(&run.log).unwrap_or_default(),
    };
    not_again(r, run, &why);
}

/// Tells progress that `run` is not run again for `why`, and, for one
/// cut off, that its log is read as it stands.
fn not_again(r: &Review, run: &AgentRun, why: &str) {
    let read = if run.cut.is_some() {
        ", so its log is read as it stands"
    } else {
        ""
    };
    (r.progress)(&format!("{}; not run again: {why}{read}", describe(run)));
}

/// Why the run `log` reads was charged: it reported a cost, or, with
/// no cost in it, as a run killed at its deadline or crashed before its
/// result has none, it made model calls that used tokens; none when it
/// cost nothing. The turns a result counts are no charge: a call the
/// API refused counts one and costs nothing.
pub(crate) fn charged(log: &RunLog) -> Option<String> {
    if log.cost > 0.0 {
        return Some(format!("it cost {:.4} USD", log.cost));
    }
    let calls: u64 = log
        .usage
        .values()
        .filter(|u| u.fresh + u.cache_write + u.cache_read + u.output > 0)
        .map(|u| u.calls)
        .sum();
    match calls {
        0 => None,
        1 => Some("it made 1 model call".to_owned()),
        n => Some(format!("it made {n} model calls")),
    }
}

/// Whether the run `log` reads ran: it ended with an answer, or was
/// charged, so its log holds a model call.
pub(crate) fn did_run(log: &RunLog) -> bool {
    (log.answered && !log.answer.trim().is_empty()) || charged(log).is_some()
}

/// What the review charges the run `log` reads, of a tier whose cap is
/// `most` US dollars: the cost its result reported, none or 0 being
/// nothing, as a run billed to a subscription or answered from the
/// cache reports; or, for a run [`charged`] that left no result at all,
/// killed at its deadline, crashed or cancelled in flight, the whole of
/// its tier's cap, the most `--max-budget-usd` let it spend, so a run
/// that was paid for is never counted free and no price table is
/// needed; nothing for a run that cost nothing.
#[must_use]
pub fn billed(log: &RunLog, most: f64) -> f64 {
    if log.cost > 0.0 {
        log.cost
    } else if !log.resulted && charged(log).is_some() {
        most
    } else {
        0.0
    }
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
            if !pause(r, &last.0, waited, first.started) {
                let (mut run, report) = last;
                let before = run.failure.unwrap_or_default();
                run.failure = Some(format!("{before}; {SPENT}, so it was not run again"));
                return (run, report);
            }
            waited += 1;
        }
        let held = match c
            .budget
            .reserve(runner::budget_of(unit.kind), r.options.deadline)
        {
            Ok(held) => held,
            Err(why) => {
                let (mut run, report) = last;
                let before = run.failure.unwrap_or_default();
                run.failure = Some(format!("{why} again; before that {before}"));
                return (run, report);
            }
        };
        let (mut next, report) = ran(r, c, unit, note, held);
        // Only a run charged nothing is run again, so the last
        // attempt's charge is the run's; a charged one ran.
        add_usage(&mut next.log, &last.0.log);
        next.attempts = last.0.attempts + 1;
        class = class_of(r, unit, &next);
        if next.cut.is_some() {
            tell_cut(r, unit, &next);
        }
        if next.answered() || next.attempts >= MAX_ATTEMPTS || class != Class::Overloaded {
            return (next, report);
        }
        keep(r, unit, "second");
        last = (next, report);
    }
}

/// Runs `unit` once, with `note`, once its tier's cap is reserved in
/// the pull request's cost cap, and not at all when it can never be.
fn tried(r: &Review, c: &Controls, unit: &Unit, note: Option<&str>) -> (AgentRun, Option<String>) {
    match c
        .budget
        .reserve(runner::budget_of(unit.kind), r.options.deadline)
    {
        Ok(held) => ran(r, c, unit, note, held),
        Err(why) => not_started(r.workdir, unit, why),
    }
}

/// Runs `unit` once, with `note`, its tier's cap `held`: noted as
/// running while it runs, and charged once it ends what [`billed`]
/// says, its reservation released.
fn ran(
    r: &Review,
    c: &Controls,
    unit: &Unit,
    note: Option<&str>,
    held: Reservation,
) -> (AgentRun, Option<String>) {
    let label = unit.label();
    c.live.start(
        &label,
        Running {
            kind: unit.kind,
            log: r.workdir.join(LOGS).join(format!("{}.jsonl", unit.stem())),
            since: r.clock.now(),
        },
    );
    let mut done = attempt(
        (r.host, r.workdir, r.plan),
        unit,
        (r.options.deadline, r.known),
        note,
    );
    c.live.end(&label);
    done.0.charged = billed(&done.0.log, runner::budget_of(unit.kind));
    held.charge(done.0.charged, &|spent| keep_spend(r, spent));

    done
}

/// What running `unit` again would do after `run` did not run or was
/// cut off, from why and the errors in the `.stderr` beside its log;
/// errors that are there but do not read are told to progress, and
/// count as none.
fn class_of(r: &Review, unit: &Unit, run: &AgentRun) -> Class {
    let errors = r.workdir.join(LOGS).join(format!("{}.stderr", unit.stem()));
    let stderr = std::fs::read_to_string(&errors).unwrap_or_else(|e| {
        if e.kind() != std::io::ErrorKind::NotFound {
            (r.progress)(&format!(
                "{}: read {}: {e}; its errors are taken as none",
                unit.label(),
                errors.display()
            ));
        }
        String::new()
    });
    let why = run.failure.as_deref().or(run.cut.as_deref());
    backoff::class(why.unwrap_or_default(), &stderr)
}

/// Waits before a run again after `run` was turned away as overloaded,
/// `waited` waits so far, the agent started at `started`; whether it
/// waited. Once its deadline is spent there is no wait to take, and
/// progress says it is not run again.
fn pause(r: &Review, run: &AgentRun, waited: u32, started: Duration) -> bool {
    let ran = r.clock.now().saturating_sub(started);
    let left = r.options.deadline.saturating_sub(ran);
    let Some(wait) = backoff::wait(waited, r.clock.jitter(), left) else {
        (r.progress)(&format!(
            "{}; {SPENT}, so it is not run again",
            describe(run)
        ));
        return false;
    };
    (r.progress)(&format!(
        "{}; the API was overloaded, so it waits {:.1} s and runs once more",
        describe(run),
        wait.as_secs_f64()
    ));
    r.clock.sleep(wait);
    true
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

#[cfg(test)]
mod tests;
