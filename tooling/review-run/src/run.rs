//! The runs: every dispatched specialist, in parallel up to a bound,
//! and every dispatched generalist the same way once its own
//! specialists have ended, each as `claude --agent` through the
//! evaluation's runner, with the review's reading tools, turn budget and
//! cost cap, and a deadline of its own. A run that cannot start, ends
//! without a successful result, misses its deadline or answers nothing
//! has not answered; the review goes on without it. A specialist's
//! report is assembled from its run log, the finding lines it stated
//! and its answer's outcome line, and written to the report file its
//! generalist reads; a run that did not answer leaves what it stated
//! there too, headed incomplete, so the findings it logged are not
//! lost, while the run stays one that did not answer. A specialist
//! whose diff is past the split limit runs in parts, each a run of its
//! own under the same bound, and its report and run are merged from
//! theirs; a part that did not answer leaves the specialist one that
//! did not answer. A run is tried again as `retry` says. What starts
//! when is `tasks`'s; what holds the runs back, `controls`'s.

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

use crate::LOGS;
use crate::controls::{
    BREAKER_FRACTION, Breaker, Budget, Clock, Controls, MAX_COST_USD, SystemClock,
};
use crate::live::{self, EVERY, Live, Watch};
use crate::parts::{self, Part};
use crate::plan::{Plan, report_file};
use crate::resume::Resume;
use crate::tasks;

/// How many agents run at once unless the run says otherwise.
pub const PARALLEL: usize = 8;
/// How long one agent may run before it is killed and counted as one
/// that did not answer.
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
    /// What the finished runs may cost, in US dollars, before no run
    /// starts.
    pub max_cost_usd: f64,
    /// The fraction of the dispatched specialists that may end in a gap
    /// before no generalist starts.
    pub breaker: f64,
    /// How often what is running is told.
    pub every: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            parallel: PARALLEL,
            deadline: DEADLINE,
            split: parts::LIMIT,
            max_cost_usd: MAX_COST_USD,
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
    /// Why it did not answer; none when it did.
    pub failure: Option<String>,
    /// Its log, read; empty when it logged nothing. After a second
    /// attempt, the last one's, with both costs.
    pub log: RunLog,
    /// How many times it ran: two when the first did not answer; for a
    /// specialist run in parts, every part's attempts.
    pub attempts: u32,
    /// How many parts it ran in: one unless its diff was split.
    pub parts: u32,
    /// For a specialist run in parts, each part that did not answer, by
    /// its place from one, with why.
    pub failed_parts: Vec<(usize, String)>,
    /// Whether its answer was taken from an earlier run of the same
    /// inputs rather than run again.
    pub reused: bool,
    /// Whether that answer came from the answer store.
    pub from_store: bool,
    /// Its answer key; none when it has none.
    pub answer_key: Option<String>,
}

impl AgentRun {
    /// The run of `name`, of `kind`, in one part, run `attempts` times,
    /// failing for `failure`, its log `log`; not reused.
    fn of(name: String, kind: Kind, failure: Option<String>, log: RunLog, attempts: u32) -> Self {
        Self {
            name,
            kind,
            failure,
            log,
            attempts,
            parts: 1,
            failed_parts: Vec::new(),
            reused: false,
            from_store: false,
            answer_key: None,
        }
    }

    /// Whether the run answered without an error.
    #[must_use]
    pub fn answered(&self) -> bool {
        self.failure.is_none()
    }
}

/// One review's runs: what starts them, the clock they keep, the work
/// directory, the plan, the ids a specialist's report may cite, how
/// they go, what a resumed review keeps, and what is told of each.
pub struct Review<'a> {
    pub host: &'a dyn Host,
    pub clock: &'a dyn Clock,
    pub workdir: &'a Path,
    pub plan: &'a Plan,
    pub known: &'a BTreeSet<String>,
    pub options: Options,
    pub resume: Option<&'a Resume>,
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
        progress,
    })
}

/// Runs every agent the review's plan dispatches: each specialist, and
/// each generalist once its own specialists have ended, telling
/// progress of each, and every so often of what is running; a
/// specialist's report is held to the known ids before it is taken.
/// What the resume plan keeps is not run again; no run starts once the
/// finished runs cost the review's cap, and no generalist once the
/// breaker tripped.
///
/// # Errors
///
/// Fails when the logs' or the reports' directory cannot be made.
pub fn review_on(r: &Review) -> Result<Vec<AgentRun>, String> {
    for dir in [LOGS, crate::REPORTS] {
        let path = r.workdir.join(dir);
        fs::create_dir_all(&path).map_err(|e| format!("make {}: {e}", path.display()))?;
    }
    let c = Controls {
        budget: Budget::new(r.options.max_cost_usd),
        breaker: Breaker::new(r.options.breaker, r.plan.specialists().len()),
        live: Live::default(),
    };
    let watch = Watch {
        clock: r.clock,
        every: r.options.every,
        total: r.plan.agents().len(),
        cost: &|| c.budget.spent(),
        progress: r.progress,
    };

    Ok(live::watching(&c.live, &watch, || tasks::run_all(r, &c)))
}

/// The run of `unit`, not started for `why`: it costs nothing and has
/// no attempt, so it is never tried again; a specialist's report says
/// why, headed incomplete, and is written when it runs whole.
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
                    name,
                    kind: Kind::Specialist,
                    failure: Some(failure),
                    log: RunLog::default(),
                    attempts: 0,
                    parts: 0,
                    failed_parts: Vec::new(),
                    reused: false,
                    from_store: false,
                    answer_key: None,
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
    if run.reused {
        return format!("{} ({kind}): reused from the earlier run", run.name);
    }
    let line = match &run.failure {
        None => format!(
            "{} ({kind}): answered ({:.4} USD, {} turns)",
            run.name, run.log.cost, run.log.turns
        ),
        Some(why) => format!("{} ({kind}): did not answer: {why}", run.name),
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
/// the report assembled from its log, written when it runs whole. A run
/// that never started has no attempts.
pub(crate) fn attempt(
    host: &dyn Host,
    workdir: &Path,
    plan: &Plan,
    unit: &Unit,
    deadline: Duration,
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
    let mut command = runner::command(&component, &world, None, &log_path, deadline);
    if let Some(part) = &unit.part {
        told(&mut command.args, &part.note);
    }
    if let Some(note) = note {
        told(&mut command.args, note);
    }
    let (log, failure) = match runner::run(host, &command) {
        Ok(log) => {
            let failure = if !log.answered {
                Some(format!("its run ended with {}", log.subtype))
            } else if log.answer.trim().is_empty() {
                Some("its run answered nothing".to_owned())
            } else {
                None
            };
            (log, failure)
        }
        Err(e) => (RunLog::default(), Some(e)),
    };
    let (failure, report) = match (kind, &unit.part) {
        (_, Some(_)) => {
            let text = assemble(&log.texts, &log.answer, failure.as_deref());
            (failure, Some(text))
        }
        (Kind::Specialist, None) => {
            let text = assemble(&log.texts, &log.answer, failure.as_deref());
            match write_report(workdir, name, &text) {
                Ok(()) => (failure, Some(text)),
                Err(e) => (
                    Some(failure.map_or(e.clone(), |f| format!("{f}; {e}"))),
                    Some(text),
                ),
            }
        }
        _ => (failure, None),
    };

    (AgentRun::of(unit.label(), kind, failure, log, 1), report)
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
