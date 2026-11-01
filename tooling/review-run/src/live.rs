//! Live progress: while the agents run, every so often a block on what
//! is running, read from the tail of each running agent's stream-json
//! log (its model calls and tokens so far from its assistant messages'
//! usage), how many agents have finished of how many, and what the
//! finished runs cost so far. When the review ends, the table of every
//! run for the job summary. The formatting is pure; the watch reads the
//! logs on the injected clock's time.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

use review_eval::component::Kind;
use review_eval::log::{self, RunLog};

use crate::controls::Clock;
use crate::run::AgentRun;

/// How often the block is printed unless the run says otherwise.
pub const EVERY: Duration = Duration::from_secs(20);

/// One run under way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Running {
    pub kind: Kind,
    /// Its stream-json log.
    pub log: PathBuf,
    /// When it started, on the review's clock.
    pub since: Duration,
}

/// What the watch reads while the agents run.
#[derive(Debug, Default)]
pub struct Live {
    /// The runs under way, by how progress names them.
    running: Mutex<BTreeMap<String, Running>>,
    finished: AtomicUsize,
    stopped: Mutex<bool>,
    stop: Condvar,
}

/// One running agent's line of the block, as its log reads so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub label: String,
    pub kind: Kind,
    /// How long it has run.
    pub ran: Duration,
    pub calls: u64,
    /// Fresh input, cache writes and cache reads.
    pub input: u64,
    pub output: u64,
}

impl Live {
    /// Notes that `label` started, logging to `running.log`.
    pub fn start(&self, label: &str, running: Running) {
        self.lock().insert(label.to_owned(), running);
    }

    /// Notes that `label` ended.
    pub fn end(&self, label: &str) {
        self.lock().remove(label);
    }

    /// Counts one more agent finished.
    pub fn finish(&self) {
        self.finished.fetch_add(1, Ordering::SeqCst);
    }

    /// How many agents have finished.
    #[must_use]
    pub fn finished(&self) -> usize {
        self.finished.load(Ordering::SeqCst)
    }

    /// Each run under way as its log reads at `now`.
    #[must_use]
    pub fn rows(&self, now: Duration) -> Vec<Row> {
        let running = self.lock().clone();
        running
            .into_iter()
            .map(|(label, r)| row(label, &r, now, &read(&r.log)))
            .collect()
    }

    /// Ends the watch.
    pub fn stop(&self) {
        *self.stopped.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.stop.notify_all();
    }

    /// Waits up to `every` for the watch to end; whether it ended.
    #[must_use]
    pub fn wait(&self, every: Duration) -> bool {
        let stopped = self.stopped.lock().unwrap_or_else(PoisonError::into_inner);
        let (stopped, _) = self
            .stop
            .wait_timeout_while(stopped, every, |s| !*s)
            .unwrap_or_else(PoisonError::into_inner);
        *stopped
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Running>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// What the watch prints, and how often.
pub struct Watch<'a> {
    pub clock: &'a dyn Clock,
    pub every: Duration,
    /// How many agents the review dispatched.
    pub total: usize,
    /// What the finished runs cost so far.
    pub cost: &'a (dyn Fn() -> f64 + Sync),
    pub progress: &'a (dyn Fn(&str) + Sync),
}

/// Ends the watch when the work it watches ends, panicking or not.
struct Stop<'a>(&'a Live);

impl Drop for Stop<'_> {
    fn drop(&mut self) {
        self.0.stop();
    }
}

/// `work`, with a block on `live` printed as `watch` says every so
/// often while it runs.
pub fn watching<T>(live: &Live, watch: &Watch, work: impl FnOnce() -> T) -> T {
    std::thread::scope(|scope| {
        scope.spawn(|| {
            while !live.wait(watch.every) {
                let rows = live.rows(watch.clock.now());
                (watch.progress)(&block(&rows, live.finished(), watch.total, (watch.cost)()));
            }
        });
        let _stop = Stop(live);
        work()
    })
}

/// The log at `path` so far: a run cut mid-line reads as far as it
/// went, and one that wrote nothing yet as nothing.
fn read(path: &Path) -> RunLog {
    log::parse(&fs::read_to_string(path).unwrap_or_default())
}

/// `label`'s row from `running` and its log `so_far`, at `now`.
fn row(label: String, running: &Running, now: Duration, so_far: &RunLog) -> Row {
    let mut out = Row {
        label,
        kind: running.kind,
        ran: now.saturating_sub(running.since),
        calls: 0,
        input: 0,
        output: 0,
    };
    for u in so_far.usage.values() {
        out.calls += u.calls;
        out.input += u.fresh + u.cache_write + u.cache_read;
        out.output += u.output;
    }
    out
}

/// The block: how many of `total` agents have `finished`, what the
/// finished runs `cost`, and each of `rows`.
#[must_use]
pub fn block(rows: &[Row], finished: usize, total: usize, cost: f64) -> String {
    let mut s = format!(
        "progress: {finished} of {total} agents finished, {cost:.4} USD so far, {} running",
        rows.len()
    );
    for r in rows {
        let _ = write!(
            s,
            "\n  {} ({}), {}: {} calls, {} tokens in, {} out",
            r.label,
            r.kind.word(),
            clock(r.ran),
            r.calls,
            r.input,
            r.output
        );
    }
    s
}

/// `d` as minutes and seconds.
#[must_use]
pub fn clock(d: Duration) -> String {
    let s = d.as_secs();
    format!("{}m{:02}s", s / 60, s % 60)
}

/// The table of every run for the job summary: its kind, how it ended,
/// its attempts, turns, tokens and cost.
#[must_use]
pub fn table(runs: &[AgentRun]) -> String {
    let mut s = "### How each agent ran\n\n\
                 | agent | kind | result | attempts | turns | tokens in | out | cost |\n\
                 | --- | --- | --- | --- | --- | --- | --- | --- |\n"
        .to_owned();
    for run in runs {
        let (mut input, mut output) = (0, 0);
        for u in run.log.usage.values() {
            input += u.fresh + u.cache_write + u.cache_read;
            output += u.output;
        }
        let result = match (&run.failure, run.reused) {
            (_, true) => "reused".to_owned(),
            (None, false) => "answered".to_owned(),
            (Some(why), false) => cell(why),
        };
        let _ = writeln!(
            s,
            "| {} | {} | {result} | {} | {} | {input} | {output} | {:.4} |",
            run.name,
            run.kind.word(),
            run.attempts,
            run.log.turns,
            run.log.cost
        );
    }
    s
}

/// `text` safe in a table cell: one line, no pipe, at most 120
/// characters.
fn cell(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c == '|' || c.is_control() { ' ' } else { c })
        .collect();
    if flat.chars().count() > 120 {
        format!("{}…", flat.chars().take(119).collect::<String>())
    } else {
        flat
    }
}

#[cfg(test)]
mod tests;
