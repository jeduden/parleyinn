//! Timing one sample of a benchmark. Each blocking wait has a deadline
//! (ENG-03): a program that does not exit within it is an error, not a
//! slow sample, so a hung benchmark fails the run instead of stalling
//! the CI job until its timeout.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::fixture;
use crate::suite::{Benchmark, Fixture, HOOK_LINES};

/// How long one sample may take before it is an error.
pub const DEADLINE: Duration = Duration::from_secs(10);

/// How long preparing a sample's fixture may take.
pub const PREPARE_DEADLINE: Duration = Duration::from_secs(120);

/// Times `program` started with `args` until it exits, its standard
/// streams discarded. The time is taken on the thread that waits, so it
/// counts the program's start and run and nothing of the wait for the
/// result.
///
/// # Errors
///
/// Fails when the program cannot start, exits unsuccessfully, or has
/// not exited by `deadline`; it is then left to finish on its own.
pub fn time_program(program: &Path, args: &[&str], deadline: Duration) -> Result<Duration, String> {
    time_program_in(program, args, &[], deadline)
}

/// Times one sample of `benchmark` against the core executable
/// `program`: over a fresh fixture, made before and removed after, that
/// counts nothing of the time.
///
/// # Errors
///
/// Fails when the fixture cannot be made, or as [`time_program`].
pub fn time_benchmark(
    program: &Path,
    benchmark: &Benchmark,
    deadline: Duration,
) -> Result<Duration, String> {
    match benchmark.fixture {
        Fixture::None => time_program(program, benchmark.args, deadline),
        Fixture::Transcript { lines } => {
            let home =
                fixture::Home::with_transcript(&std::env::temp_dir(), &fixture::transcript(lines))
                    .map_err(|e| format!("make the fixture of {}: {e}", benchmark.name))?;
            time_program_in(program, benchmark.args, &home.env(), deadline)
        }
        Fixture::Search { lines, prepare } => {
            let home =
                fixture::Home::with_transcript(&std::env::temp_dir(), &fixture::transcript(lines))
                    .map_err(|e| format!("make the fixture of {}: {e}", benchmark.name))?;
            time_program_in(program, prepare, &home.env(), PREPARE_DEADLINE)
                .map_err(|e| format!("prepare the fixture of {}: {e}", benchmark.name))?;
            let input = fixture::SEARCH_REQUESTS.as_bytes().to_vec();
            time_program_fed(program, benchmark.args, &home.env(), input, deadline)
        }
        Fixture::HookInput { hook } => retried(ATTEMPTS, settle, || {
            hook_sample(program, benchmark, hook, deadline)
        }),
    }
}

/// How one attempt at a sample ended when it did not give a time.
#[derive(Debug, PartialEq, Eq)]
enum Attempt {
    /// The hook handler counted that it overran its hook budget.
    Overran(String),
    /// The sample could not be taken at all.
    Failed(String),
}

/// How many times a hook sample is taken before its overruns are the
/// handler's.
const ATTEMPTS: usize = 4;

/// How long the benchmark waits after an overrun before it takes the
/// sample again.
const SETTLE: Duration = Duration::from_millis(300);

/// How long the benchmark lets a flush of the disk take before it samples
/// anyway.
const FLUSH_DEADLINE: Duration = Duration::from_secs(5);

/// Waits [`SETTLE`] for the disk to calm down.
fn settle() {
    thread::sleep(SETTLE);
}

/// The time `attempt` gives, taken again after `settle` while its
/// handler overran its hook budget, up to `attempts` times. The handler
/// commits durably (NFR-02), and a disk that stalls its `fsync` past the
/// budget under one sample passes on a fresh home a moment later; a
/// handler that overruns every time fails the benchmark. Any other
/// failure is final.
fn retried(
    attempts: usize,
    mut settle: impl FnMut(),
    mut attempt: impl FnMut() -> Result<Duration, Attempt>,
) -> Result<Duration, String> {
    let mut taken = 1;
    loop {
        match attempt() {
            Ok(took) => return Ok(took),
            Err(Attempt::Failed(e)) => return Err(e),
            Err(Attempt::Overran(e)) if taken >= attempts => return Err(e),
            Err(Attempt::Overran(_)) => {
                taken += 1;
                settle();
            }
        }
    }
}

/// One sample of a hook handler over a fresh home, failed as
/// [`Attempt::Overran`] when its handler counted an overrun.
fn hook_sample(
    program: &Path,
    benchmark: &Benchmark,
    hook: &str,
    deadline: Duration,
) -> Result<Duration, Attempt> {
    let home =
        fixture::Home::with_transcript(&std::env::temp_dir(), &fixture::transcript(HOOK_LINES))
            .map_err(|e| Attempt::Failed(format!("make the fixture of {}: {e}", benchmark.name)))?;
    // Flush what earlier work left dirty, so the handler's own durable
    // commit does not wait on it. A flush that fails or times out only
    // costs a retry, and the host may have no `sync` at all.
    let _ = time_program(Path::new("sync"), &[], FLUSH_DEADLINE);
    let input = home.hook_input(hook).into_bytes();
    let took = time_program_fed(program, benchmark.args, &home.env(), input, deadline)
        .map_err(Attempt::Failed)?;
    unexceeded(&home, benchmark.name).map(|()| took)
}

/// Fails when the hook handler that ran over `home` counted that it
/// overran its hook budget (NFR-02): the benchmark times the unconfined
/// release core, where the absolute budget is meaningful.
fn unexceeded(home: &fixture::Home, name: &str) -> Result<(), Attempt> {
    const COUNTER: &str = "hook_budget_exceeded";
    let counted = home
        .counted(COUNTER)
        .map_err(|e| Attempt::Failed(format!("{name}: read the audit log: {e}")))?;
    if counted {
        return Err(Attempt::Overran(format!(
            "{name}: the handler counted {COUNTER}: it overran its hook budget"
        )));
    }

    Ok(())
}

/// Times `program` as [`time_program_in`] does, with `input` on its
/// stdin, which is then closed, as a harness hands a hook handler its
/// hook input.
///
/// # Errors
///
/// As [`time_program`].
pub fn time_program_fed(
    program: &Path,
    args: &[&str],
    env: &[(&str, std::path::PathBuf)],
    input: Vec<u8>,
    deadline: Duration,
) -> Result<Duration, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .envs(env.iter().map(|(k, v)| (*k, v.as_os_str())))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    waited(program, args, deadline, move || {
        let started = Instant::now();
        let status = command.spawn().and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                // A program that exits without reading its input is
                // timed all the same.
                let _ = stdin.write_all(&input);
            }
            child.wait()
        });
        (started.elapsed(), status)
    })
}

/// The time `run`, on a thread of its own, says its program took, by
/// `deadline`: an error when it fails to start, exits unsuccessfully or
/// has not exited by then.
fn waited(
    program: &Path,
    args: &[&str],
    deadline: Duration,
    run: impl FnOnce() -> (Duration, std::io::Result<std::process::ExitStatus>) + Send + 'static,
) -> Result<Duration, String> {
    let (done, result) = mpsc::channel();
    thread::spawn(move || {
        let _ = done.send(run());
    });

    let shown = program.display();
    match result.recv_timeout(deadline) {
        Ok((took, Ok(status))) if status.success() => Ok(took),
        Ok((_, Ok(status))) => Err(format!("{shown} {} exited {status}", args.join(" "))),
        Ok((_, Err(err))) => Err(format!("start {shown}: {err}")),
        Err(_) => Err(format!(
            "{shown} {} did not exit within {deadline:?}",
            args.join(" ")
        )),
    }
}

/// Times `program` as [`time_program`] does, with `env` set beside the
/// environment it inherits.
///
/// # Errors
///
/// As [`time_program`].
pub fn time_program_in(
    program: &Path,
    args: &[&str],
    env: &[(&str, std::path::PathBuf)],
    deadline: Duration,
) -> Result<Duration, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .envs(env.iter().map(|(k, v)| (*k, v.as_os_str())))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    waited(program, args, deadline, move || {
        let started = Instant::now();
        let status = command.status();
        (started.elapsed(), status)
    })
}

/// Takes `count` samples of `benchmark` with `time`, after one warm-up
/// sample it discards, so the first sample does not pay for a cold
/// page cache the others do not.
///
/// # Errors
///
/// Fails on the first sample `time` cannot take, naming the benchmark.
pub fn sample(
    benchmark: &Benchmark,
    count: usize,
    time: &mut dyn FnMut(&Benchmark) -> Result<Duration, String>,
) -> Result<Vec<u64>, String> {
    let name = benchmark.name;
    time(benchmark).map_err(|e| format!("{name}: {e}"))?;
    (0..count)
        .map(|_| {
            time(benchmark)
                .map(|took| u64::try_from(took.as_nanos()).unwrap_or(u64::MAX))
                .map_err(|e| format!("{name}: {e}"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
