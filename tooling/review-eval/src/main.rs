//! The `review-eval` command: the evaluation of the review's own
//! reviewers (ENG-32). It starts the processes the library asks for,
//! `git` for the fixture worktrees, `mdsmith` for its pattern catalog
//! and `claude` for each run, each with a deadline, and nothing else.
//! It is CI tooling and never ships.

use std::fs::File;
use std::io;
use std::process::{Command as Process, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use review_eval::host::{Command, Host, Output};

/// How long a command without a deadline of its own may run.
const DEFAULT_DEADLINE: Duration = Duration::from_secs(600);

/// How often a running command is looked at.
const POLL: Duration = Duration::from_millis(100);

/// What a Claude Code session sets for the commands it starts, which
/// would make an agent started here a child of that session: its tool
/// calls run in the background and its subagents are capped. The
/// review runs each agent as a session of its own, so the harness
/// clears them.
const NESTED_SESSION_VARS: [&str; 4] = [
    "CLAUDE_AUTO_BACKGROUND_TASKS",
    "CLAUDE_CODE_BG_TASKS_REPORT_RUNNING",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH",
];

/// The machine: real processes, each killed at its deadline.
struct ProcessHost;

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
                let log =
                    File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
                let errors = File::create(path.with_extension("stderr"))
                    .map_err(|e| format!("create {}: {e}", path.display()))?;
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
        let timed_out = loop {
            match child.try_wait() {
                Ok(Some(_)) => break false,
                Ok(None) if started.elapsed() >= deadline => {
                    let _ = child.kill();
                    break true;
                }
                Ok(None) => thread::sleep(POLL),
                Err(e) => return Err(format!("wait for {}: {e}", command.program)),
            }
        };
        let out = child
            .wait_with_output()
            .map_err(|e| format!("wait for {}: {e}", command.program))?;

        Ok(Output {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            timed_out,
        })
    }
}

fn main() -> ExitCode {
    let code = review_eval::cli::run(
        &ProcessHost,
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
    );

    ExitCode::from(code)
}
