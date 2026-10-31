//! The host on the machine: real processes, each killed at its
//! deadline, with the variables a Claude Code harness session sets for its
//! children cleared. The `review-eval` and `review-run` executables
//! both start their commands through it; nothing else of either starts
//! a program.

use std::fs::File;
use std::io::{self, Read};
use std::process::{Command as Process, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::host::{Command, Host, Output};

/// How long a command without a deadline of its own may run.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(600);

/// How often a running command is looked at.
const POLL: Duration = Duration::from_millis(50);

/// How long, once a command has ended, its output may still take to
/// arrive: a process it started may hold its pipes past its end.
const DRAIN: Duration = Duration::from_secs(2);

/// What stands for a pipe still held open once [`DRAIN`] has passed, so
/// the output lost is never silent.
pub const HELD: &str = "review-eval: a process the command started still holds this output\n";

/// What a Claude Code harness session sets for the commands it starts, which
/// would make an agent started here a child of that harness session: its tool
/// calls run in the background and its subagents are capped. The
/// review runs each agent as a harness session of its own, so the host clears
/// them.
pub const NESTED_SESSION_VARS: [&str; 4] = [
    "CLAUDE_AUTO_BACKGROUND_TASKS",
    "CLAUDE_CODE_BG_TASKS_REPORT_RUNNING",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH",
];

/// The machine: real processes, each killed at its deadline.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessHost;

impl Host for ProcessHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        let mut process = Process::new(&command.program);
        process
            .args(&command.args)
            .current_dir(&command.cwd)
            .envs(command.env.iter().cloned())
            .stdin(Stdio::null());
        for var in NESTED_SESSION_VARS {
            process.env_remove(var);
        }
        // A log file takes the output and the errors, so a chatty run
        // never fills a pipe nobody reads before it ends.
        match &command.stdout_to {
            Some(path) => {
                let create = |p: &std::path::Path| {
                    File::create(p).map_err(|e| format!("create {}: {e}", p.display()))
                };
                let log = create(path)?;
                let errors = create(&path.with_extension("stderr"))?;
                process.stdout(Stdio::from(log)).stderr(Stdio::from(errors));
            }
            None => {
                process.stdout(Stdio::piped()).stderr(Stdio::piped());
            }
        }
        let mut child = process
            .spawn()
            .map_err(|e| format!("start {}: {e}", command.program))?;
        // Each pipe is read as the command writes, so a chatty command
        // never blocks on a full pipe.
        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());
        let deadline = command.deadline.unwrap_or(DEFAULT_DEADLINE);
        let started = Instant::now();
        let wait_failed = |e: io::Error| format!("wait for {}: {e}", command.program);
        let timed_out = loop {
            if child.try_wait().map_err(wait_failed)?.is_some() {
                break false;
            }
            if started.elapsed() >= deadline {
                // `kill` answers Ok for a child that has already exited, so
                // an error means the command may still run past its deadline.
                child
                    .kill()
                    .map_err(|e| format!("kill {}: {e}", command.program))?;
                break true;
            }
            thread::sleep(POLL);
        };
        let status = child.wait().map_err(wait_failed)?;
        let until = Instant::now() + DRAIN;

        Ok(Output {
            code: status.code(),
            stdout: collect(&stdout, until),
            stderr: collect(&stderr, until),
            timed_out,
        })
    }
}

/// Reads `pipe` to its end on a thread of its own, and hands what it
/// read to the receiver.
fn drain(pipe: Option<impl Read + Send + 'static>) -> Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut read = Vec::new();
        if let Some(mut pipe) = pipe {
            // What was read before an error is kept, and a line says the
            // rest was lost, so a truncated output is never silent.
            if let Err(e) = pipe.read_to_end(&mut read) {
                read.extend_from_slice(
                    format!("\nreview-eval: reading this output failed: {e}\n").as_bytes(),
                );
            }
        }
        let _ = tx.send(read);
    });
    rx
}

/// What a pipe's reader read, waiting until `until` for a pipe a process
/// the command started still holds, and [`HELD`] in its place then.
fn collect(rx: &Receiver<Vec<u8>>, until: Instant) -> String {
    rx.recv_timeout(until.saturating_duration_since(Instant::now()))
        .map_or_else(
            |_| HELD.to_owned(),
            |read| String::from_utf8_lossy(&read).into_owned(),
        )
}

#[cfg(test)]
mod tests;
