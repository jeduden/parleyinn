//! The host on the machine: real processes, each killed at its
//! deadline, with the variables a Claude Code session sets for its
//! children cleared. The `review-eval` and `review-run` executables
//! both start their commands through it; nothing else of either starts
//! a program.

use std::fs::File;
use std::io;
use std::process::{Command as Process, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::host::{Command, Host, Output};

/// How long a command without a deadline of its own may run.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(600);

/// How often a running command is looked at.
const POLL: Duration = Duration::from_millis(50);

/// What a Claude Code session sets for the commands it starts, which
/// would make an agent started here a child of that session: its tool
/// calls run in the background and its subagents are capped. The
/// review runs each agent as a session of its own, so the host clears
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
        let deadline = command.deadline.unwrap_or(DEFAULT_DEADLINE);
        let started = Instant::now();
        let wait_failed = |e: io::Error| format!("wait for {}: {e}", command.program);
        let timed_out = loop {
            if child.try_wait().map_err(wait_failed)?.is_some() {
                break false;
            }
            if started.elapsed() >= deadline {
                let _ = child.kill();
                break true;
            }
            thread::sleep(POLL);
        };
        let out = child.wait_with_output().map_err(wait_failed)?;

        Ok(Output {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            timed_out,
        })
    }
}

#[cfg(test)]
mod tests;
