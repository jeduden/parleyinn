//! What the `bdd` target needs beside cucumber-rs: its command line,
//! which scenarios it runs, and an executor to drive the run on the
//! current thread.

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::future::Future;
use std::path::Path;
use std::pin::pin;
use std::process::Command;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

use cucumber::gherkin::tagexpr::TagOperation;
use cucumber::tag::Ext as _;

use crate::PENDING;

/// Which scenarios a run selects.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// A tag expression such as `@ENG-24`, from `--tags`.
    pub tags: Option<TagOperation>,
    /// Name filters, as `cargo test <filter>` passes them: a scenario is
    /// selected when its name contains one, or one names its tag.
    pub filters: Vec<String>,
    /// Filters from `--skip`, as libtest takes them: a scenario one of
    /// them matches, the way a name filter matches, is left out.
    pub skips: Vec<String>,
}

/// libtest's flags that take no value, accepted and ignored so that
/// `cargo test -- <flag>` across the workspace still runs the target.
const LIBTEST_FLAGS: [&str; 7] = [
    "--nocapture",
    "--show-output",
    "--quiet",
    "-q",
    "--ignored",
    "--include-ignored",
    "--exact",
];

/// libtest's flags that take a value, accepted and ignored likewise.
const LIBTEST_VALUED: [&str; 2] = ["--test-threads", "--color"];

/// Reads the target's command line: `--tags EXPR` (or `-t`), name
/// filters, `--skip FILTER`, and the libtest flags cargo may pass on.
///
/// # Errors
///
/// Fails on an unknown flag, a flag missing its value, or a tag
/// expression that does not parse.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with('-') => (flag, Some(value.to_owned())),
            _ => (arg.as_str(), None),
        };
        let mut value = || {
            inline
                .clone()
                .or_else(|| args.next().cloned())
                .ok_or(format!("{flag} needs a value"))
        };
        match flag {
            "--tags" | "-t" => {
                let expr = value()?;
                options.tags = Some(expr.parse().map_err(|e| format!("--tags {expr:?}: {e}"))?);
            }
            "--skip" => options.skips.push(value()?),
            f if LIBTEST_VALUED.contains(&f) => drop(value()?),
            f if LIBTEST_FLAGS.contains(&f) => {}
            f if f.starts_with('-') => return Err(format!("unknown flag {f:?}")),
            _ => options.filters.push(arg.clone()),
        }
    }

    Ok(options)
}

impl Options {
    /// Whether a run with these options runs the scenario `name`
    /// carrying `tags`, written without their `@`. A pending scenario is
    /// never run: it is declared, not written.
    #[must_use]
    pub fn selects<'a>(&self, name: &str, tags: impl Iterator<Item = &'a String> + Clone) -> bool {
        let matches_filter = |f: &String| {
            name.contains(f.as_str()) || tags.clone().any(|t| t == f.trim_start_matches('@'))
        };

        !tags.clone().any(|t| t == PENDING)
            && self
                .tags
                .as_ref()
                .is_none_or(|expr| expr.eval(tags.clone()))
            && (self.filters.is_empty() || self.filters.iter().any(matches_filter))
            && !self.skips.iter().any(matches_filter)
    }
}

/// Wakes the thread that drives a future.
struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

/// Drives `future` to completion on the current thread, parking it
/// while the future waits. cucumber-rs needs no more of an executor
/// than this.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        if let Poll::Ready(out) = future.as_mut().poll(&mut cx) {
            return out;
        }
        thread::park();
    }
}

/// The variable that marks the runner's own process as isolated: its
/// value is the home the parent created for it.
pub const ISOLATED_HOME: &str = "PARINN_BDD_HOME";

/// Whether a process with `HOME` set to `home`, and the marker set to
/// `marker`, runs isolated (ENG-14): the marker names its `HOME`, and
/// that home sits under the system's temporary directory `temp`, so an
/// exported marker cannot point the scenarios at a real home.
#[must_use]
pub fn is_isolated(home: Option<&OsStr>, marker: Option<&OsStr>, temp: &Path) -> bool {
    match (home, marker) {
        (Some(home), Some(marker)) => home == marker && Path::new(home).starts_with(temp),
        _ => false,
    }
}

/// Points `command` at `home`: its `HOME`, a `PARINN_HOME` inside it,
/// and the marker that says the process is isolated.
pub fn isolate<'a>(command: &'a mut Command, home: &Path) -> &'a mut Command {
    command
        .env("HOME", home)
        .env("PARINN_HOME", home.join(".parinn"))
        .env(ISOLATED_HOME, home)
}

/// A child's exit status code as this process's exit code: a signal,
/// or a code outside 0 to 255, is a failure.
#[must_use]
pub fn exit_code(code: Option<i32>) -> u8 {
    code.and_then(|c| u8::try_from(c).ok()).unwrap_or(1)
}

/// The sections' own state beside a scenario's world, one value per
/// type: created on first use and the same value for the rest of the
/// scenario. A section declares a struct for what its scenarios track
/// and never adds a field to the world.
#[derive(Default)]
pub struct Sections(HashMap<TypeId, Box<dyn Any>>);

impl Sections {
    /// The section state of type `T`.
    pub fn get<T: Default + 'static>(&mut self) -> &mut T {
        let slot = self
            .0
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()));
        // The map is keyed by the type, so the value is always a T.
        slot.downcast_mut()
            .unwrap_or_else(|| unreachable!("section {} holds another type", type_name::<T>()))
    }
}

impl fmt::Debug for Sections {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sections({})", self.0.len())
    }
}

#[cfg(test)]
mod tests;
