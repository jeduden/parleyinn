//! The runs: every dispatched specialist, in parallel up to a bound,
//! then, once every one has finished, every dispatched generalist the
//! same way, each as `claude --agent` through the evaluation's runner,
//! with the review's tools, write rule and turn budget, and a deadline
//! of its own. A run that cannot start, ends without a successful
//! result, misses its deadline or answers nothing has not answered;
//! the review goes on without it.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use review_eval::component::{self, Kind};
use review_eval::host::Host;
use review_eval::log::RunLog;
use review_eval::runner;
use review_eval::world::World;

use crate::LOGS;
use crate::plan::{Plan, report_file};

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
}

impl Default for Options {
    fn default() -> Self {
        Self {
            parallel: PARALLEL,
            deadline: DEADLINE,
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
    /// How many times it ran: two when the first did not answer.
    pub attempts: u32,
}

impl AgentRun {
    /// Whether the run answered without an error.
    #[must_use]
    pub fn answered(&self) -> bool {
        self.failure.is_none()
    }
}

/// Runs every agent `plan` dispatches in the work directory `workdir`:
/// the specialists, then the generalists, telling `progress` of each.
///
/// # Errors
///
/// Fails when the logs' or the reports' directory cannot be made.
pub fn review(
    host: &dyn Host,
    workdir: &Path,
    plan: &Plan,
    options: Options,
    progress: &(dyn Fn(&str) + Sync),
) -> Result<Vec<AgentRun>, String> {
    for dir in [LOGS, crate::REPORTS] {
        let path = workdir.join(dir);
        fs::create_dir_all(&path).map_err(|e| format!("make {}: {e}", path.display()))?;
    }
    let one = |name: &String, kind: Kind| {
        let run = run_one(host, workdir, plan, name, kind, options.deadline, progress);
        progress(&describe(&run));
        run
    };
    let mut runs = parallel(&plan.specialists(), options.parallel, &|name| {
        one(name, Kind::Specialist)
    });
    runs.extend(parallel(&plan.generalists(), options.parallel, &|name| {
        one(name, Kind::Generalist)
    }));

    Ok(runs)
}

/// One line on `run`, for the progress the command prints; a run that
/// took two attempts says so.
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
    if run.attempts > 1 {
        format!("{line} on its second attempt")
    } else {
        line
    }
}

/// Runs agent `name`, a `kind`, and once more when its run did not
/// answer: a command that would not start, ended without a result,
/// missed its deadline or answered nothing. A definition that does not
/// read, or a report that cannot be cleared, fails at once, and a report
/// the agent wrote is judged where the outcome is decided, never rerun.
/// The first failure is told to `progress` before the second attempt,
/// and its log kept as `<name>.first.jsonl` beside the second's.
fn run_one(
    host: &dyn Host,
    workdir: &Path,
    plan: &Plan,
    name: &str,
    kind: Kind,
    deadline: Duration,
    progress: &(dyn Fn(&str) + Sync),
) -> AgentRun {
    let first = attempt(host, workdir, plan, name, kind, deadline);
    if first.answered() || first.attempts == 0 {
        return AgentRun {
            attempts: first.attempts.max(1),
            ..first
        };
    }
    progress(&format!("{}; running it once more", describe(&first)));
    if let Err(e) = keep_first_log(workdir, name) {
        progress(&format!(
            "{name} ({}): its first log was not kept: {e}",
            kind.word()
        ));
    }
    let mut second = attempt(host, workdir, plan, name, kind, deadline);
    second.log.cost += first.log.cost;
    second.attempts = 2;

    second
}

/// Runs agent `name` once, logging under the logs' directory. A
/// specialist's report file from before is removed first, so only what
/// this run wrote is read. A run that never started has no attempts.
fn attempt(
    host: &dyn Host,
    workdir: &Path,
    plan: &Plan,
    name: &str,
    kind: Kind,
    deadline: Duration,
) -> AgentRun {
    let failed = |failure: String| AgentRun {
        name: name.to_owned(),
        kind,
        failure: Some(failure),
        log: RunLog::default(),
        attempts: 0,
    };
    let component = match component::load(workdir, name) {
        Ok(component) => component,
        Err(e) => return failed(format!("its definition does not read: {e}")),
    };
    if kind == Kind::Specialist
        && let Err(e) = remove_stale(&workdir.join(report_file(name)))
    {
        return failed(e);
    }
    let world = World {
        main: workdir.to_path_buf(),
        class: plan.class.clone(),
        touched: plan.touched,
        seeded: plan.reports_of(name),
        notes: Vec::new(),
    };
    let log_path = workdir.join(LOGS).join(format!("{name}.jsonl"));
    let command = runner::command(&component, &world, None, &log_path, deadline);
    let log = match runner::run(host, &command) {
        Ok(log) => log,
        Err(e) => {
            return AgentRun {
                attempts: 1,
                ..failed(e)
            };
        }
    };
    let failure = if !log.answered {
        Some(format!("its run ended with {}", log.subtype))
    } else if log.answer.trim().is_empty() {
        Some("its run answered nothing".to_owned())
    } else {
        None
    };

    AgentRun {
        name: name.to_owned(),
        kind,
        failure,
        log,
        attempts: 1,
    }
}

/// Moves agent `name`'s log and its errors, each when there is one, to
/// `<name>.first.jsonl` and `<name>.first.stderr`, so its second
/// attempt does not write over them.
fn keep_first_log(workdir: &Path, name: &str) -> Result<(), String> {
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

/// Removes the file at `path` when there is one.
fn remove_stale(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => {
            Err(format!("remove {}: {e}", path.display()))
        }
        _ => Ok(()),
    }
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
            if let Ok(mut done) = done.lock() {
                done.push((at, out));
            }
        }
    };
    thread::scope(|scope| {
        for _ in 0..bound.max(1).min(items.len()) {
            scope.spawn(worker);
        }
    });
    let mut done = done.into_inner().unwrap_or_default();
    done.sort_by_key(|(at, _)| *at);

    done.into_iter().map(|(_, out)| out).collect()
}

#[cfg(test)]
mod tests;
