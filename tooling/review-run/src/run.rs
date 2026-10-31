//! The runs: every dispatched specialist, in parallel up to a bound,
//! then, once every one has finished, every dispatched generalist the
//! same way, each as `claude --agent` through the evaluation's runner,
//! with the review's reading tools and turn budget, and a deadline of
//! its own. A run that cannot start, ends without a successful result,
//! misses its deadline or answers nothing has not answered; the review
//! goes on without it. A specialist's report is assembled from its run
//! log, the finding lines it stated and its answer's outcome line, and
//! written to the report file its generalist reads; a run that did not
//! answer leaves what it stated there too, headed incomplete, so the
//! findings it logged are not lost, while the run stays one that did
//! not answer. A specialist whose diff is past the split limit runs
//! in parts, each a run of its own under the same bound, and its
//! report and run are merged from theirs; a part that did not answer
//! leaves the specialist one that did not answer. A run is tried once
//! more as `retry` says.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use review_eval::component::{self, Kind};
use review_eval::host::Host;
use review_eval::log::RunLog;
use review_eval::runner;
use review_eval::world::World;
use review_gate::report::assemble;

use crate::LOGS;
use crate::parts::{self, Part};
use crate::plan::{Plan, report_file};
use crate::retry::run_one;

/// How many agents run at once unless the run says otherwise.
pub const PARALLEL: usize = 8;
/// How long one agent may run before it is killed and counted as one
/// that did not answer.
pub const DEADLINE: Duration = Duration::from_secs(review_eval::DEADLINE_SECONDS);

/// How the runs go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// How many agents run at once, at least one.
    pub parallel: usize,
    /// Each run's deadline.
    pub deadline: Duration,
    /// The size, in bytes, past which a specialist's diff runs in parts.
    pub split: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            parallel: PARALLEL,
            deadline: DEADLINE,
            split: parts::LIMIT,
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
}

impl AgentRun {
    /// Whether the run answered without an error.
    #[must_use]
    pub fn answered(&self) -> bool {
        self.failure.is_none()
    }
}

/// Runs every agent `plan` dispatches in the work directory `workdir`:
/// the specialists, then the generalists, telling `progress` of each;
/// a specialist's report is held to the ids in `known` before it is
/// taken.
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
    progress: &(dyn Fn(&str) + Sync),
) -> Result<Vec<AgentRun>, String> {
    for dir in [LOGS, crate::REPORTS] {
        let path = workdir.join(dir);
        fs::create_dir_all(&path).map_err(|e| format!("make {}: {e}", path.display()))?;
    }
    let one = |unit: &Unit| {
        let (run, report) = run_one(host, workdir, plan, known, unit, options.deadline, progress);
        progress(&describe(&run));
        (run, report)
    };
    let (units, unsplit) = specialist_units(workdir, plan, options.split);
    let done = parallel(&units, options.parallel, &one);
    let mut runs = gather(workdir, plan, &units, done, unsplit, progress);
    let generalists: Vec<Unit> = plan.generalists().into_iter().map(Unit::whole).collect();
    let done = parallel(&generalists, options.parallel, &one);
    runs.extend(done.into_iter().map(|(run, _)| run));

    Ok(runs)
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
    fn whole(name: String) -> Self {
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

/// The runs `plan`'s specialists start: each whole, or in its parts
/// when its diff is past `split`; and, run in none, each specialist
/// whose diff could not be split, as a run that did not answer, its
/// report saying why.
fn specialist_units(workdir: &Path, plan: &Plan, split: usize) -> (Vec<Unit>, Vec<AgentRun>) {
    let mut units = Vec::new();
    let mut unsplit = Vec::new();
    for name in plan.specialists() {
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
                });
            }
        }
    }

    (units, unsplit)
}

/// Each of `plan`'s specialists' run, in its order: its one run, its
/// parts' runs in `done` merged, with the merged report written and
/// told to `progress`, or its run in `unsplit`.
fn gather(
    workdir: &Path,
    plan: &Plan,
    units: &[Unit],
    done: Vec<(AgentRun, Option<String>)>,
    mut unsplit: Vec<AgentRun>,
    progress: &(dyn Fn(&str) + Sync),
) -> Vec<AgentRun> {
    let mut by_name: Vec<(&Unit, (AgentRun, Option<String>))> = units.iter().zip(done).collect();
    let mut runs = Vec::new();
    for name in plan.specialists() {
        if let Some(at) = unsplit.iter().position(|r| r.name == name) {
            runs.push(unsplit.remove(at));
            continue;
        }
        let (own, rest): (Vec<_>, Vec<_>) = by_name.into_iter().partition(|(u, _)| u.name == name);
        by_name = rest;
        let split = own.iter().any(|(u, _)| u.part.is_some());
        let mut own: Vec<(AgentRun, Option<String>)> = own.into_iter().map(|(_, d)| d).collect();
        if !split {
            runs.extend(own.pop().map(|(run, _)| run));
            continue;
        }
        let (mut run, report) = parts::merge(
            &name,
            own.into_iter()
                .map(|(run, report)| (run, report.unwrap_or_default()))
                .collect(),
        );
        if let Err(e) = write_report(workdir, &name, &report) {
            run.failure = Some(run.failure.map_or(e.clone(), |f| format!("{f}; {e}")));
        }
        progress(&describe(&run));
        runs.push(run);
    }

    runs
}

/// One line on `run`, for the progress the command prints; a run that
/// took two attempts says so, and one run in parts how many.
#[must_use]
pub fn describe(run: &AgentRun) -> String {
    let kind = run.kind.word();
    let line = match &run.failure {
        None => format!(
            "{} ({kind}): answered ({:.4} USD, {} turns)",
            run.name, run.log.cost, run.log.turns
        ),
        Some(why) => format!("{} ({kind}): did not answer: {why}", run.name),
    };
    if run.parts > 1 {
        format!("{line} in {} parts, {} attempts", run.parts, run.attempts)
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
            let failure = format!("its definition does not read: {e}");
            let report = unit
                .part
                .as_ref()
                .map(|_| assemble(&[], "", Some(&failure)));
            return (
                AgentRun {
                    name: unit.label(),
                    kind,
                    failure: Some(failure),
                    log: RunLog::default(),
                    attempts: 0,
                    parts: 1,
                },
                report,
            );
        }
    };
    let world = World {
        main: workdir.to_path_buf(),
        class: plan.class.clone(),
        touched: plan.touched,
        seeded: plan.reports_of(name),
        notes: Vec::new(),
    };
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

    (
        AgentRun {
            name: unit.label(),
            kind,
            failure,
            log,
            attempts: 1,
            parts: 1,
        },
        report,
    )
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
/// to `<name>.first.jsonl` and `<name>.first.stderr`, so its second
/// attempt does not write over them.
pub(crate) fn keep_first_log(workdir: &Path, name: &str) -> Result<(), String> {
    let logs = workdir.join(LOGS);
    for extension in ["jsonl", "stderr"] {
        let from = logs.join(format!("{name}.{extension}"));
        let to = logs.join(format!("{name}.first.{extension}"));
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

/// `f` of each of `items`, at most `bound` at once, in the order of
/// `items`.
pub fn parallel<T: Sync, R: Send>(
    items: &[T],
    bound: usize,
    f: &(dyn Fn(&T) -> R + Sync),
) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let done: Mutex<Vec<(usize, R)>> = Mutex::new(Vec::with_capacity(items.len()));
    let worker = || {
        while let Some((at, item)) = {
            let at = next.fetch_add(1, Ordering::SeqCst);
            items.get(at).map(|item| (at, item))
        } {
            let out = f(item);
            // A poisoned lock still holds every result pushed before
            // the panic; this one joins them, so none is dropped.
            done.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push((at, out));
        }
    };
    thread::scope(|scope| {
        for _ in 0..bound.max(1).min(items.len()) {
            scope.spawn(worker);
        }
    });
    let mut done = done.into_inner().unwrap_or_else(PoisonError::into_inner);
    done.sort_by_key(|(at, _)| *at);

    done.into_iter().map(|(_, out)| out).collect()
}

#[cfg(test)]
mod tests;
