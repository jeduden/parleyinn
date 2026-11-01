//! ENG-28: what holds the review's runs back. `review-run` passes each
//! agent the cost cap of its tier, starts no run once the runs that
//! finished cost the review's cap and names each agent it did not start
//! over budget, never approving; starts no generalist once more than a
//! third of the dispatched specialists failed, opening the review
//! outcome with the abort; waits with backoff before it runs again an
//! agent the API turned away as overloaded; and never runs again an
//! agent a cost cap stopped or whose credential was refused. The checks
//! run `review-run` on the work directory [`review_runs::prepare`] lays
//! out, with a stand-in for the `claude` command line that keeps every
//! command and fails the runs it is told to as the command line would,
//! answering every other as [`Recorder`] does; the waits are on a clock
//! that does not sleep.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use review_eval::component::kind_of;
use review_eval::host::{Command, Host, Output};
use review_eval::runner::budget_usd;
use review_run::controls::{Clock, Fraction};
use review_run::run::{Options, Review, review_on};

use crate::Checkout;
use crate::review_runs::{self, Recorder, agent_of};

/// What the harness logs for a run the API turned away as overloaded.
pub const OVERLOADED: &str = r#"{"type":"result","subtype":"success","is_error":true,"result":"API Error: 529 {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}","total_cost_usd":0,"num_turns":1}"#;
/// What it logs for a run whose credential was refused.
pub const REFUSED: &str = r#"{"type":"result","subtype":"success","is_error":true,"result":"API Error: 401 {\"type\":\"error\",\"error\":{\"type\":\"authentication_error\",\"message\":\"invalid token\"}}","total_cost_usd":0,"num_turns":1}"#;
/// What it logs for a run its own cost cap stopped.
pub const CAPPED: &str = r#"{"type":"result","subtype":"error_max_budget_usd","is_error":true,"result":"","total_cost_usd":1.01,"num_turns":9}"#;

/// A stand-in for the `claude` command line: it keeps every command,
/// fails the first runs of each agent [`ControlHost::fails`] names as
/// it says, and answers every other run as [`Recorder`] does.
#[derive(Debug, Default)]
pub struct ControlHost {
    /// By agent: how many of its first runs fail, and the log each
    /// writes, or none for a run that cannot start.
    pub fails: BTreeMap<&'static str, (usize, Option<&'static str>)>,
    commands: Mutex<Vec<Command>>,
    recorder: Recorder,
}

impl ControlHost {
    /// The commands run, in the order they started.
    #[must_use]
    pub fn commands(&self) -> Vec<Command> {
        self.commands
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// How many times `agent` ran.
    #[must_use]
    pub fn runs_of(&self, agent: &str) -> usize {
        self.commands()
            .iter()
            .filter(|c| agent_of(c) == agent)
            .count()
    }
}

impl Host for ControlHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        let agent = agent_of(command);
        let ran = {
            let mut commands = self.commands.lock().unwrap_or_else(PoisonError::into_inner);
            commands.push(command.clone());
            commands.iter().filter(|c| agent_of(c) == agent).count()
        };
        match self.fails.get(agent.as_str()) {
            Some((first, Some(log))) if ran <= *first => {
                let path = command
                    .stdout_to
                    .as_deref()
                    .ok_or("the run logs to no file")?;
                fs::write(path, format!("{log}\n"))
                    .map_err(|e| format!("write {}: {e}", path.display()))?;
                Ok(Output {
                    code: Some(1),
                    ..Output::default()
                })
            }
            Some((first, None)) if ran <= *first => Err("claude: not found".to_owned()),
            _ => self.recorder.run(command),
        }
    }
}

/// Runs `review-run` on `workdir`, prepared from `checkout`, with
/// `host` and the flags `extra`: what it printed, or why it could not
/// run.
fn review(
    checkout: &Checkout,
    workdir: &Path,
    host: &ControlHost,
    extra: &[&str],
) -> Result<String, String> {
    review_runs::prepare(checkout, workdir)?;
    let mut args = vec!["--workdir".to_owned(), workdir.display().to_string()];
    args.extend(extra.iter().map(|a| (*a).to_owned()));
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = review_run::cli::run(host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        return Err(format!(
            "review-run exited {code}: {}",
            err.lines().last().unwrap_or_default()
        ));
    }

    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// The reasons of the gaps `review-run` wrote in `workdir`, by name.
fn gaps(workdir: &Path) -> BTreeMap<String, String> {
    let text = fs::read_to_string(workdir.join("review/gaps.json")).unwrap_or_default();
    review_run::gaps::parse(&text)
        .unwrap_or_default()
        .into_iter()
        .map(|g| (g.name, g.reason))
        .collect()
}

/// Each of `commands` run without the cost cap of its agent's tier.
#[must_use]
pub fn tier_problems(commands: &[Command]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|command| {
            let agent = agent_of(command);
            let want = budget_usd(kind_of(&agent));
            let capped = command
                .args
                .windows(2)
                .any(|w| w[0] == "--max-budget-usd" && w[1] == want);
            (!capped).then(|| format!("{agent} ran without its tier's cost cap {want}"))
        })
        .collect()
}

/// What is wrong with a review whose first run reached its cost cap,
/// `started` runs started, printing `out`, its gaps `gaps`: a further
/// run started, an agent not started that is not named over budget or
/// not counted, or a review that does not request changes.
#[must_use]
pub fn capped_problems(started: usize, out: &str, gaps: &BTreeMap<String, String>) -> Vec<String> {
    let mut problems = Vec::new();
    if started != 1 {
        problems.push(format!(
            "{started} runs started though the first reached the review's cost cap"
        ));
    }
    let over = gaps
        .values()
        .filter(|r| r.starts_with("over budget: "))
        .count();
    if over != 4 || !out.contains("\nover_budget=4\n") {
        problems.push(format!(
            "{over} agents named over budget, not the 4 not started: {out}"
        ));
    }
    if !out.starts_with("verdict=request_changes\n") {
        problems.push(format!(
            "a review past its cost cap does not request changes: {out}"
        ));
    }

    problems
}

/// What is wrong with the cost caps: a run without its tier's cap, and
/// what [`capped_problems`] names of a review whose cap the first run
/// reaches.
#[must_use]
pub fn caps_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let full = ControlHost::default();
    let capped = ControlHost::default();
    let dir = workdir.join("capped");
    let flags = ["--parallel", "1", "--max-cost-usd", "0.01"];
    let out = review(checkout, &workdir.join("full"), &full, &[])
        .and_then(|_| review(checkout, &dir, &capped, &flags));
    match out {
        Ok(out) => {
            let mut problems = tier_problems(&full.commands());
            problems.extend(capped_problems(capped.commands().len(), &out, &gaps(&dir)));
            problems
        }
        Err(e) => vec![e],
    }
}

/// What the abort names.
const ABORT: &str = "2 of 3 specialists failed";

/// What is wrong with a review two of whose three specialists failed,
/// printing `out`, its outcome `outcome`, its gaps `gaps`, the
/// generalists in `started` having started: a generalist started, or
/// not named aborted, or an outcome that does not open with the abort.
#[must_use]
pub fn aborted_problems(
    started: &[&str],
    out: &str,
    outcome: &str,
    gaps: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut problems: Vec<String> = started
        .iter()
        .map(|g| format!("{g} started though {ABORT}"))
        .collect();
    for (generalist, _, _) in review_runs::GENERALISTS {
        let reason = gaps.get(generalist).map_or("", String::as_str);
        if !reason.starts_with(&format!("aborted: {ABORT}")) {
            problems.push(format!("{generalist} is not named aborted: {reason:?}"));
        }
    }
    let opens = review_gate::parse_outcome(outcome).ok().is_some_and(|o| {
        o.findings
            .first()
            .is_some_and(|f| f.body.starts_with(&format!("review aborted: {ABORT}")))
    });
    if !opens || !out.contains("\naborted=2\n") {
        problems.push(format!(
            "the review outcome does not open with the abort: {outcome}"
        ));
    }

    problems
}

/// What [`aborted_problems`] names of the review of `workdir`, prepared
/// from `checkout`, two of whose three specialists cannot start.
#[must_use]
pub fn breaker_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let host = ControlHost {
        fails: BTreeMap::from([("lang-rust", (2, None)), ("test-engineer-unit", (2, None))]),
        ..ControlHost::default()
    };
    match review(checkout, workdir, &host, &[]) {
        Ok(out) => {
            let started: Vec<&str> = review_runs::GENERALISTS
                .iter()
                .map(|(g, _, _)| *g)
                .filter(|g| host.runs_of(g) > 0)
                .collect();
            let outcome =
                fs::read_to_string(workdir.join("review/outcome.json")).unwrap_or_default();
            aborted_problems(&started, &out, &outcome, &gaps(workdir))
        }
        Err(e) => vec![e],
    }
}

/// A clock that never sleeps: it keeps each wait and moves on by it.
#[derive(Debug, Default)]
pub struct Waits(Mutex<Vec<Duration>>);

impl Clock for Waits {
    fn now(&self) -> Duration {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .sum()
    }

    fn sleep(&self, duration: Duration) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(duration);
    }

    fn jitter(&self) -> f64 {
        0.0
    }
}

/// What is wrong with running again: an overloaded run not waited for
/// and run again, or a run its cost cap stopped or whose credential was
/// refused run again.
#[must_use]
pub fn retry_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    if let Err(e) = review_runs::prepare(checkout, workdir) {
        return vec![e];
    }
    let host = ControlHost {
        fails: BTreeMap::from([
            ("lang-rust", (1, Some(OVERLOADED))),
            ("test-engineer-unit", (1, Some(REFUSED))),
            ("lang-markdown", (1, Some(CAPPED))),
        ]),
        ..ControlHost::default()
    };
    let clock = Waits::default();
    let ran = review_run::plan::read(workdir).and_then(|plan| {
        review_on(&Review {
            host: &host,
            clock: &clock,
            workdir,
            plan: &plan,
            known: &review_gate::report::invariants(),
            options: Options {
                breaker: Fraction::WHOLE,
                ..Options::default()
            },
            resume: None,
            progress: &|_| {},
        })
    });
    if let Err(e) = ran {
        return vec![e];
    }
    let waited = clock.0.into_inner().unwrap_or_else(PoisonError::into_inner);
    retried_problems(&|agent| host.runs_of(agent), &waited)
}

/// What is wrong with how often each agent ran, `runs_of` it, and the
/// waits `waited`: lang-rust, turned away as overloaded, not run twice
/// after one wait of 15 s or more, and test-engineer-unit, whose
/// credential was refused, or lang-markdown, its cap reached, run again.
#[must_use]
pub fn retried_problems(runs_of: &dyn Fn(&str) -> usize, waited: &[Duration]) -> Vec<String> {
    let mut problems = Vec::new();
    let once = waited.len() == 1 && waited.iter().all(|w| *w >= Duration::from_secs(15));
    if runs_of("lang-rust") != 2 || !once {
        problems.push(format!(
            "lang-rust, turned away as overloaded, ran {} times after waits of {waited:?}, not \
             twice after one wait of 15 s or more",
            runs_of("lang-rust")
        ));
    }
    for agent in ["test-engineer-unit", "lang-markdown"] {
        if runs_of(agent) != 1 {
            problems.push(format!(
                "{agent}, which can never succeed by running again, ran {} times",
                runs_of(agent)
            ));
        }
    }

    problems
}

#[cfg(test)]
mod tests;
