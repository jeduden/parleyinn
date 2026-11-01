//! The host on the machine: real processes, each killed at its
//! deadline, with the variables a Claude Code harness session sets for
//! its children cleared and the model access token withheld. The
//! `review-eval` and `review-run` executables both start their commands
//! through it; nothing else of either starts a program.

use std::fs::File;
use std::io::{self, Read};
use std::process::{Child, Command as Process, ExitStatus, Stdio};
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

/// What no command the host starts finds in its environment: the
/// model access token, which a `claude` reads from its staged
/// credentials file instead, so an agent's tools never find it in the
/// environment they run in (ENG-28).
pub const WITHHELD_VARS: [&str; 1] = [crate::credential::TOKEN_VAR];

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
        for var in NESTED_SESSION_VARS.iter().chain(&WITHHELD_VARS) {
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
        let timed_out = watch(&mut child, &command.program, deadline)?;
        let status = child
            .wait()
            .map_err(|e| format!("wait for {}: {e}", command.program))?;
        let until = Instant::now() + DRAIN;

        Ok(Output {
            code: status.code(),
            stdout: collect(&stdout, until),
            stderr: collect(&stderr, until),
            timed_out,
        })
    }
}

/// What [`watch`] needs of a running child, so a test can stand in for
/// one whose wait or kill fails.
trait Watched {
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;
    fn kill(&mut self) -> io::Result<()>;
}

impl Watched for Child {
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Child::try_wait(self)
    }
    fn kill(&mut self) -> io::Result<()> {
        Child::kill(self)
    }
}

/// Looks at `child` until it ends or `deadline` passes, and answers
/// whether it was killed at its deadline. A child that cannot be waited
/// for is killed before the error returns, so it never runs on unwatched.
fn watch(child: &mut impl Watched, program: &str, deadline: Duration) -> Result<bool, String> {
    let started = Instant::now();
    // `kill` answers Ok for a child that has already exited, so an error
    // means the command may still run past its deadline.
    let kill = |child: &mut dyn Watched| child.kill().map_err(|e| format!("kill {program}: {e}"));
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(false),
            Ok(None) => {}
            Err(e) => {
                let wait = format!("wait for {program}: {e}");
                return Err(match kill(child) {
                    Ok(()) => format!("{wait}; killed it"),
                    Err(kill) => format!("{wait}; {kill}"),
                });
            }
        }
        if started.elapsed() >= deadline {
            kill(child)?;
            return Ok(true);
        }
        thread::sleep(POLL);
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
