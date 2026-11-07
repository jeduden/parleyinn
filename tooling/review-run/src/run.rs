//! The runs: every dispatched specialist, in parallel up to a bound,
//! and every dispatched generalist the same way once its own
//! specialists have ended, each as `claude --agent` through the
//! evaluation's runner, with the review's reading tools, turn budget and
//! cost cap, and a deadline of its own. A run whose log holds a model
//! call ran, and its log is read whatever result it ended with: one cut
//! off by its turn budget, its deadline or its cost cap is no failure,
//! only noted as cut. Only a run whose log holds no model call, one
//! that could not start or that the API turned away, did not run; the
//! review goes on without it. A specialist's report is read from its
//! run log as `reading` says and written to the report file its
//! generalist reads, and whether it read its whole diff is decided from
//! the reads its log shows. A specialist whose diff is past the split
//! limit runs in parts, each a run of its own under the same bound, and
//! its report and run are merged from theirs; a part that did not run
//! leaves the specialist one that did not run. A run is tried again as
//! `retry` says. What starts when is `tasks`'s; what holds the runs
//! back, `controls`'s.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use review_eval::component::{self, Kind};
use review_eval::host::Host;
use review_eval::log::{RunLog, SUCCESS};
use review_eval::runner;
use review_eval::world::World;
use review_gate::report::assemble;

use crate::reading::{self, ReportLine};

use crate::LOGS;
use crate::controls::{
    BREAKER_FRACTION, Breaker, Budget, Clock, Controls, Fraction, PULL_REQUEST_COST_USD,
    SystemClock,
};
use crate::key::Keys;
use crate::live::{self, EVERY, Live, Watch};
use crate::parts::{self, Part};
use crate::plan::{Plan, report_file};
use crate::resume::Resume;
use crate::spend::SPEND;
use crate::store::Store;
use crate::tasks;

/// How many agents run at once unless the run says otherwise.
pub const PARALLEL: usize = 8;
/// How long one agent may run before it is killed; what its log shows
/// by then is read.
pub const DEADLINE: Duration = Duration::from_secs(review_eval::DEADLINE_SECONDS);

/// How the runs go.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// How many agents run at once, at least one.
    pub parallel: usize,
    /// Each run's deadline.
    pub deadline: Duration,
    /// The size, in bytes, past which a specialist's diff runs in parts.
    pub split: usize,
    /// What the pull request's reviews may cost, in US dollars, before
    /// no run starts.
    pub max_cost_usd: f64,
    /// What the pull request's earlier reviews spent, in US dollars.
    pub spent_before: f64,
    /// The fraction of the dispatched specialists that may end in a gap
    /// before no generalist starts.
    pub breaker: Fraction,
    /// How often what is running is told.
    pub every: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            parallel: PARALLEL,
            deadline: DEADLINE,
            split: parts::LIMIT,
            max_cost_usd: PULL_REQUEST_COST_USD,
            spent_before: 0.0,
            breaker: BREAKER_FRACTION,
            every: EVERY,
        }
    }
}

/// One agent's run.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentRun {
    pub name: String,
    /// A specialist or a generalist.
    pub kind: Kind,
    /// Why it did not run, its log holding no model call, or why the
    /// review could not keep what it ran; none when its log was read.
    pub failure: Option<String>,
    /// Why a run whose log was read ended without a successful result:
    /// its turn budget, deadline or cost cap cut it off, or it answered
    /// nothing; none when it ended with one.
    pub cut: Option<String>,
    /// Its log, read; empty when it logged nothing. After a second
    /// attempt, the last one's, with both costs. A specialist's answer
    /// is the outcome line its report ends with.
    pub log: RunLog,
    /// How many times it ran: two when the first did not run; for a
    /// specialist run in parts, every part's attempts.
    pub attempts: u32,
    /// How many parts it ran in: one unless its diff was split.
    pub parts: u32,
    /// For a specialist run in parts, each part that did not run, by
    /// its place from one, with why.
    pub failed_parts: Vec<(usize, String)>,
    /// For a specialist, each run of it, whole or a part by its place
    /// from one, whose log shows no read of some lines of its diff, with
    /// which.
    pub partial: Vec<(Option<usize>, String)>,
    /// The lines of its report set aside as no finding.
    pub ignored: Vec<ReportLine>,
    /// The finding-shaped lines of its report that do not read.
    pub unreadable: Vec<ReportLine>,
    /// Whether its answer was taken from an earlier run of the same
    /// inputs rather than run again.
    pub reused: bool,
    /// Whether that answer came from the answer store.
    pub from_store: bool,
    /// Its answer key; none when it has none.
    pub answer_key: Option<String>,
    /// Whether its accepted answer could not be stored.
    pub store_failed: bool,
    /// What the review's budget charged its runs, in US dollars: each
    /// run's reported cost, or its tier's cap for a run that was charged
    /// and reported none ([`crate::retry::billed`]).
    pub charged: f64,
}

impl AgentRun {
    /// The run of `name`, of `kind`, in one part, run `attempts` times,
    /// failing for `failure`, its log `log`; not reused.
    pub(crate) fn of(
        name: String,
        kind: Kind,
        failure: Option<String>,
        log: RunLog,
        attempts: u32,
    ) -> Self {
        Self {
            name,
            kind,
            failure,
            log,
            cut: None,
            attempts,
            parts: 1,
            failed_parts: Vec::new(),
            partial: Vec::new(),
            ignored: Vec::new(),
            unreadable: Vec::new(),
            reused: false,
            from_store: false,
            answer_key: None,
            store_failed: false,
            charged: 0.0,
        }
    }

    /// Whether it ran and its log was read: its answer is consulted,
    /// whatever result it ended with.
    #[must_use]
    pub fn answered(&self) -> bool {
        self.failure.is_none()
    }

    /// Whether the review of it is whole: it ran, read every line of its
    /// diff and stated no line that does not read.
    #[must_use]
    pub fn whole(&self) -> bool {
        self.answered() && self.partial.is_empty() && self.unreadable.is_empty()
    }
}

/// One review's runs: what starts them, the clock they keep, the work
/// directory, the plan, the ids a specialist's report may cite, how
/// they go, what a resumed review keeps, the answer keys and store, and
/// what is told of each.
pub struct Review<'a> {
    pub host: &'a dyn Host,
    pub clock: &'a dyn Clock,
    pub workdir: &'a Path,
    pub plan: &'a Plan,
    pub known: &'a BTreeSet<String>,
    pub options: Options,
    pub resume: Option<&'a Resume>,
    /// Each agent's answer key; none when the keys are unknown.
    pub keys: Option<&'a Keys>,
    /// The answer store answers are taken from and kept in; none when
    /// the review uses none.
    pub store: Option<&'a Store>,
    /// The scrubber of the Claude token; an answer it finds the token
    /// in is never stored (ENG-28). None without a token to scrub.
    pub scrubber: Option<&'a review_eval::secret::Scrubber>,
    pub progress: &'a (dyn Fn(&str) + Sync),
}

/// Runs every agent `plan` dispatches in the work directory `workdir`,
/// on the machine's clock, as [`review_on`] does.
///
/// # Errors
///
/// Fails when the logs' or the reports' directory cannot be made.
pub fn review(
    host: &dyn Host,
    workdir: &Path,
    plan: &Plan,
    known: &BTreeSet<String>,
    options: Options,
    resume: Option<&Resume>,
    progress: &(dyn Fn(&str) + Sync),
) -> Result<Vec<AgentRun>, String> {
    review_on(&Review {
        host,
        clock: &SystemClock::default(),
        workdir,
        plan,
        known,
        options,
        resume,
        keys: None,
        store: None,
        scrubber: None,
        progress,
    })
}

/// Runs every agent the review's plan dispatches: each specialist, and
/// each generalist once its own specialists have ended, telling
/// progress of each, and every so often of what is running; a
/// specialist's report is held to the known ids before it is taken.
/// What the resume plan keeps is not run again; no run starts once the
/// pull request's earlier reviews and this one's finished runs cost its
/// cap, and no generalist once the breaker tripped. What they spent is
/// kept in [`SPEND`] as the review starts and as each run ends.
///
/// # Errors
///
/// Fails when the logs' or the reports' directory cannot be made.
pub fn review_on(r: &Review) -> Result<Vec<AgentRun>, String> {
    for dir in [LOGS, crate::REPORTS] {
        let path = r.workdir.join(dir);
        fs::create_dir_all(&path).map_err(|e| format!("make {}: {e}", path.display()))?;
    }
    keep_spend(r, 0.0);
    let c = Controls {
        budget: Budget::new(r.options.max_cost_usd, r.options.spent_before),
        breaker: Breaker::new(r.options.breaker, r.plan.specialists().len()),
        live: Live::default(),
    };
    let watch = Watch {
        clock: r.clock,
        every: r.options.every,
        total: r.plan.agents().len(),
        cost: &|| c.budget.total(),
        cap: r.options.max_cost_usd,
        progress: r.progress,
    };

    Ok(live::watching(&c.live, &watch, || tasks::run_all(r, &c)))
}

/// Keeps `spent`, what this review cost so far, in [`SPEND`]; one that
/// cannot be kept is told to progress (I6).
pub(crate) fn keep_spend(r: &Review, spent: f64) {
    let path = r.workdir.join(SPEND);
    if let Err(e) = crate::spend::write(&path, &crate::spend::text(spent)) {
        (r.progress)(&format!(
            "what the pull request's reviews spent was not kept: {e}"
        ));
    }
}

/// The run of `unit`, not started for `why`: it costs nothing and has
/// no attempt, so it is never tried again; a specialist's report says
/// why, headed incomplete, and is written when it runs whole: it did not
/// run.
pub(crate) fn not_started(workdir: &Path, unit: &Unit, why: String) -> (AgentRun, Option<String>) {
    let report = (unit.kind == Kind::Specialist).then(|| assemble(&[], "", Some(&why)));
    let mut failure = why;
    if let (Some(text), None) = (&report, &unit.part)
        && let Err(e) = write_report(workdir, &unit.name, text)
    {
        failure = format!("{failure}; {e}");
    }

    (
        AgentRun::of(unit.label(), unit.kind, Some(failure), RunLog::default(), 0),
        report,
    )
}

/// The run of agent or part `name`, of `kind`, kept from an earlier
/// run in which it answered `answer`, having stated `texts`: it costs
/// nothing and was not started.
pub(crate) fn reused(name: String, kind: Kind, answer: &str, texts: Vec<String>) -> AgentRun {
    let log = RunLog {
        answered: true,
        subtype: SUCCESS.to_owned(),
        answer: answer.to_owned(),
        texts,
        ..RunLog::default()
    };
    AgentRun {
        reused: true,
        ..AgentRun::of(name, kind, None, log, 0)
    }
}

/// One run the review starts: an agent whole, or one part of a
/// specialist's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Unit {
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) part: Option<Part>,
}

impl Unit {
    /// Agent `name` run whole, its kind by its name.
    pub(crate) fn whole(name: String) -> Self {
        Self {
            kind: review_eval::component::kind_of(&name),
            name,
            part: None,
        }
    }

    /// The stem of its log's file name.
    pub(crate) fn stem(&self) -> String {
        self.part
            .as_ref()
            .map_or_else(|| self.name.clone(), |p| p.stem(&self.name))
    }

    /// How progress names it.
    pub(crate) fn label(&self) -> String {
        self.part
            .as_ref()
            .map_or_else(|| self.name.clone(), |p| p.label(&self.name))
    }
}

/// The runs `plan`'s specialists start, but those `kept` names: each
/// whole, or in its parts when its diff is past `split`; and, run in
/// none, each specialist whose diff could not be split, as a run that
/// did not answer, its report saying why.
pub(crate) fn specialist_units(
    workdir: &Path,
    plan: &Plan,
    split: usize,
    kept: &dyn Fn(&str) -> bool,
) -> (Vec<Unit>, Vec<AgentRun>) {
    let mut units = Vec::new();
    let mut unsplit = Vec::new();
    for name in plan.specialists().into_iter().filter(|n| !kept(n)) {
        match parts::prepare(workdir, &name, split) {
            Ok(list) if list.is_empty() => units.push(Unit::whole(name)),
            Ok(list) => units.extend(list.into_iter().map(|p| Unit {
                name: name.clone(),
                kind: Kind::Specialist,
                part: Some(p),
            })),
            Err(e) => {
                let why = format!("its diff could not be split: {e}");
                let failure = match write_report(workdir, &name, &assemble(&[], "", Some(&why))) {
                    Ok(()) => why,
                    Err(e) => format!("{why}; {e}"),
                };
                unsplit.push(AgentRun {
                    parts: 0,
                    ..AgentRun::of(name, Kind::Specialist, Some(failure), RunLog::default(), 0)
                });
            }
        }
    }

    (units, unsplit)
}

/// One line on `run`, for the progress the command prints; a run that
/// took two attempts says so, and one run in parts how many.
#[must_use]
pub fn describe(run: &AgentRun) -> String {
    let kind = run.kind.word();
    if run.from_store {
        return format!("{} ({kind}): reused from the answer store", run.name);
    }
    if run.reused {
        return format!("{} ({kind}): reused from the earlier run", run.name);
    }
    let line = match (&run.failure, &run.cut) {
        (None, None) => format!(
            "{} ({kind}): answered ({:.4} USD, {} turns)",
            run.name, run.log.cost, run.log.turns
        ),
        (None, Some(cut)) => format!(
            "{} ({kind}): ran, its log read: {cut} ({:.4} USD, {} turns)",
            run.name, run.log.cost, run.log.turns
        ),
        (Some(why), _) => format!("{} ({kind}): did not run: {why}", run.name),
    };
    if run.parts > 1 {
        format!("{line} in {} parts, {} attempts", run.parts, run.attempts)
    } else if run.attempts > 2 {
        format!("{line} on its third attempt")
    } else if run.attempts > 1 {
        format!("{line} on its second attempt")
    } else {
        line
    }
}

/// Runs `unit` once, logging under the logs' directory, with `note`,
/// when there is one, added to its prompt, and for a specialist returns
/// the report read from its log, its ids held to `known`, written when
/// it runs whole. A run that never started has no attempts.
pub(crate) fn attempt(
    (host, workdir, plan): (&dyn Host, &Path, &Plan),
    unit: &Unit,
    (deadline, known): (Duration, &BTreeSet<String>),
    note: Option<&str>,
) -> (AgentRun, Option<String>) {
    let (name, kind) = (unit.name.as_str(), unit.kind);
    let component = match component::load(workdir, name) {
        Ok(component) => component,
        Err(e) => {
            let mut failure = format!("its definition does not read: {e}");
            // A whole specialist writes no report, so none from an
            // earlier run in this work directory is left for its
            // generalist to read.
            if let (Kind::Specialist, None) = (kind, &unit.part) {
                let path = workdir.join(report_file(name));
                match fs::remove_file(&path) {
                    Err(e) if e.kind() != io::ErrorKind::NotFound => failure.push_str(&format!(
                        "; its report from before could not be removed: {}: {e}",
                        path.display()
                    )),
                    _ => {}
                }
            }
            let report = unit
                .part
                .as_ref()
                .map(|_| assemble(&[], "", Some(&failure)));
            return (
                AgentRun::of(unit.label(), kind, Some(failure), RunLog::default(), 0),
                report,
            );
        }
    };
    let world = world_of(workdir, plan, name);
    let log_path = workdir.join(LOGS).join(format!("{}.jsonl", unit.stem()));
    let diff = unit.part.as_ref().map(|part| part.diff.as_str());
    let mut command = runner::command_over(&component, &world, None, &log_path, deadline, diff);
    if let Some(part) = &unit.part {
        told(&mut command.args, &part.note);
    }
    if let Some(note) = note {
        told(&mut command.args, note);
    }
    let (mut log, failure, cut) = match runner::run(host, &command) {
        Ok(log) => {
            let (failure, cut) = ended(&log);
            (log, failure, cut)
        }
        Err(e) => (RunLog::default(), Some(e), None),
    };
    let mut run = AgentRun::of(unit.label(), kind, failure, RunLog::default(), 1);
    run.cut = cut;
    let report =
        (kind == Kind::Specialist).then(|| read_report(workdir, unit, &mut run, &mut log, known));
    run.log = log;
    if let (None, Some(text)) = (&unit.part, &report)
        && let Err(e) = write_report(workdir, name, text)
    {
        run.failure = Some(run.failure.map_or(e.clone(), |f| format!("{f}; {e}")));
    }

    (run, report)
}

/// Why the run `log` reads did not run, its log holding no model call,
/// and why it ran but ended without a successful result: its result was
/// no success, or it answered nothing.
fn ended(log: &RunLog) -> (Option<String>, Option<String>) {
    let why = if !log.answered {
        Some(format!("its run ended with {}", log.subtype))
    } else if log.answer.trim().is_empty() {
        Some("its run answered nothing".to_owned())
    } else {
        None
    };
    if log.ran() { (None, why) } else { (why, None) }
}

/// The report specialist `unit`'s run gives from `log`, its ids held to
/// `known`, as [`reading::read`] reads it: `log`'s answer becomes the
/// outcome line the report ends with, and what it set aside, could not
/// read and did not read of its diff under `workdir` are noted on
/// `run`. A run that did not run says why, headed incomplete.
fn read_report(
    workdir: &Path,
    unit: &Unit,
    run: &mut AgentRun,
    log: &mut RunLog,
    known: &BTreeSet<String>,
) -> String {
    if let Some(why) = &run.failure {
        return assemble(&log.texts, &log.answer, Some(why));
    }
    let part = unit.part.as_ref().map(|p| p.index);
    let read = reading::read((&unit.name, part), &log.texts, &log.answer, known);
    log.answer = read.answer;
    run.ignored = read.ignored;
    run.unreadable = read.unreadable;
    run.partial
        .extend(reading::uncovered(workdir, &unit.stem(), &log.reads).map(|why| (part, why)));

    read.report
}

/// The world agent `name` of `plan` runs on in `workdir`: the review's
/// class and touched count, and for a generalist its specialists'
/// report files.
pub(crate) fn world_of(workdir: &Path, plan: &Plan, name: &str) -> World {
    World {
        main: workdir.to_path_buf(),
        class: plan.class.clone(),
        touched: plan.touched,
        seeded: plan.reports_of(name),
        notes: Vec::new(),
    }
}

/// Adds `note` to the prompt among `args`, the value after `-p`.
fn told(args: &mut [String], note: &str) {
    let at = args.iter().position(|a| a == "-p").map(|at| at + 1);
    if let Some(prompt) = at.and_then(|at| args.get_mut(at)) {
        prompt.push(' ');
        prompt.push_str(note);
    }
}

/// Writes `text` as specialist `name`'s report.
///
/// # Errors
///
/// Fails when the report cannot be written.
pub(crate) fn write_report(workdir: &Path, name: &str, text: &str) -> Result<(), String> {
    let path = workdir.join(report_file(name));
    fs::write(&path, text)
        .map_err(|e| format!("its report could not be written: {}: {e}", path.display()))
}

/// Moves the log named `name` and its errors, each when there is one,
/// to `<name>.<which>.jsonl` and `<name>.<which>.stderr`.
pub(crate) fn keep_log(workdir: &Path, name: &str, which: &str) -> Result<(), String> {
    let logs = workdir.join(LOGS);
    for extension in ["jsonl", "stderr"] {
        let from = logs.join(format!("{name}.{extension}"));
        let to = logs.join(format!("{name}.{which}.{extension}"));
        match fs::rename(&from, &to) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                return Err(format!(
                    "rename {} to {}: {e}",
                    from.display(),
                    to.display()
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
