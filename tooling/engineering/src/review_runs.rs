//! ENG-28: the review is run by code. `review-run`, given a dispatch,
//! runs each dispatched specialist as its own agent, every dispatched
//! generalist only once every specialist has ended, and writes the
//! review outcome from what they answered. The check runs `review-run`
//! on a work directory holding the checkout's definitions of the
//! dispatched agents, with a stand-in for the `claude` command line that
//! records when each agent's run starts and ends and answers as the
//! agent would.

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use review_eval::host::{Command, Host, Output};

use crate::Checkout;

/// The dispatch the check runs: two generalists, three specialists.
pub const DISPATCH: &str =
    r#"{"review-code": ["lang-rust", "test-engineer-unit"], "review-spec": ["lang-markdown"]}"#;

/// The dispatched specialists, sorted.
pub const SPECIALISTS: [&str; 3] = ["lang-markdown", "lang-rust", "test-engineer-unit"];

/// The dispatched generalists, each with the file of the one finding it
/// keeps and the specialist it keeps it from.
pub const GENERALISTS: [(&str, &str, &str); 2] = [
    ("review-code", "src/a.rs", "lang-rust"),
    ("review-spec", "docs/a.md", "lang-markdown"),
];

/// When an agent's run started or ended, as the stand-in saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Started(String),
    Ended(String),
}

/// A stand-in for the `claude` command line: it records each run's
/// start and end, and answers as the agent it runs as.
#[derive(Debug, Default)]
pub struct Recorder {
    pub events: Mutex<Vec<Event>>,
}

impl Recorder {
    fn record(&self, event: Event) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

impl Host for Recorder {
    fn run(&self, command: &Command) -> Result<Output, String> {
        let agent = agent_of(command);
        self.record(Event::Started(agent.clone()));
        let answered = answer(command, &agent);
        self.record(Event::Ended(agent));
        answered?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// The agent `command` runs as.
#[must_use]
pub fn agent_of(command: &Command) -> String {
    command
        .args
        .iter()
        .position(|a| a == "--agent")
        .and_then(|at| command.args.get(at + 1))
        .cloned()
        .unwrap_or_default()
}

/// Answers `command` as `agent` would: a specialist writes its report
/// with one nit and answers its outcome line; a generalist keeps the
/// nit of the specialist [`GENERALISTS`] names for it.
fn answer(command: &Command, agent: &str) -> Result<(), String> {
    let write = |path: &Path, text: &str| {
        fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
    };
    let result = if let Some((_, path, from)) = GENERALISTS.iter().find(|(g, _, _)| *g == agent) {
        format!("{path}:1 — nit — a long line — {from}\n{agent} finds its scope sound.")
    } else {
        let outcome = format!("outcome: nits — {agent} saw one nit.");
        let report = command.cwd.join(format!("review/reports/{agent}.md"));
        write(
            &report,
            &format!("src/a.rs:1 — nit — a long line — ENG-02\n\n{outcome}\n"),
        )?;
        format!("{outcome}\nreview/reports/{agent}.md")
    };
    let log = command
        .stdout_to
        .as_deref()
        .ok_or("the run logs to no file")?;
    let line = serde_json::json!({"type": "result", "subtype": "success", "is_error": false,
        "result": result, "total_cost_usd": 0.01, "num_turns": 2});
    write(log, &format!("{line}\n"))
}

/// Lays out in `workdir` what the review job prepares for `review-run`:
/// the dispatch, the bundle's printed output and the checkout's
/// definitions of every dispatched agent.
///
/// # Errors
///
/// Fails naming the file that cannot be read or written.
pub fn prepare(checkout: &Checkout, workdir: &Path) -> Result<(), String> {
    let write = |dir: &str, file: &str, text: &str| {
        let dir = workdir.join(dir);
        fs::create_dir_all(&dir).map_err(|e| format!("make {}: {e}", dir.display()))?;
        let path = dir.join(file);
        fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
    };
    write("review", "dispatch.json", DISPATCH)?;
    write(
        "",
        "class.txt",
        "class=standard\ntouched=1\nfindings=0\nconsults=5\n",
    )?;
    let names = SPECIALISTS
        .iter()
        .chain(GENERALISTS.iter().map(|(g, _, _)| g));
    for name in names {
        let file = format!("{name}.md");
        let text = checkout.read(&format!(".claude/agents/{file}"))?;
        write(".claude/agents", &file, &text)?;
    }

    Ok(())
}

/// What is wrong with the order of `events`: a dispatched agent that did
/// not run exactly once, an agent run that was not dispatched, and a
/// generalist that started before a specialist ended.
#[must_use]
pub fn order_problems(events: &[Event]) -> Vec<String> {
    let mut problems = Vec::new();
    let generalists: Vec<&str> = GENERALISTS.iter().map(|(g, _, _)| *g).collect();
    let at = |want: Event| -> Vec<usize> {
        events
            .iter()
            .enumerate()
            .filter(|(_, e)| **e == want)
            .map(|(i, _)| i)
            .collect()
    };
    let started = |name: &str| at(Event::Started(name.to_owned()));
    for name in SPECIALISTS.iter().chain(&generalists) {
        let runs = started(name).len();
        if runs != 1 {
            problems.push(format!(
                "{name} ran {runs} times, not once as its own agent"
            ));
        }
    }
    for event in events {
        if let Event::Started(name) = event
            && !SPECIALISTS.contains(&name.as_str())
            && !generalists.contains(&name.as_str())
        {
            problems.push(format!("{name} ran but was not dispatched"));
        }
    }
    for generalist in &generalists {
        let Some(&start) = started(generalist).first() else {
            continue;
        };
        for specialist in SPECIALISTS {
            if at(Event::Ended(specialist.to_owned()))
                .iter()
                .any(|&end| end > start)
            {
                problems.push(format!("{generalist} started before {specialist} ended"));
            }
        }
    }

    problems
}

/// What is wrong with the files `review-run` wrote in `workdir`: an
/// outcome that does not read, a finding a generalist answered that it
/// leaves out, and an agent that answered but is not consulted.
#[must_use]
pub fn outcome_problems(workdir: &Path) -> Vec<String> {
    let read = |rel: &str| {
        let path = workdir.join(rel);
        fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
    };
    let mut problems = Vec::new();
    match read("review/outcome.json").and_then(|text| {
        review_gate::parse_outcome(&text).map_err(|e| format!("review/outcome.json: {e}"))
    }) {
        Ok(outcome) => {
            for (generalist, path, _) in GENERALISTS {
                if !outcome.findings.iter().any(|f| f.path == path) {
                    problems.push(format!(
                        "the review outcome leaves out the finding on {path} {generalist} answered"
                    ));
                }
            }
        }
        Err(e) => problems.push(e),
    }
    match read("review/consulted.json").and_then(|text| {
        serde_json::from_str::<Vec<String>>(&text)
            .map_err(|e| format!("review/consulted.json: {e}"))
    }) {
        Ok(consulted) => {
            let names = SPECIALISTS
                .iter()
                .chain(GENERALISTS.iter().map(|(g, _, _)| g));
            for name in names {
                if !consulted.iter().any(|c| c == name) {
                    problems.push(format!("{name} answered but is not consulted"));
                }
            }
        }
        Err(e) => problems.push(e),
    }

    problems
}

/// Runs `review-run` on the work directory `workdir`, prepared from
/// `checkout`, with the [`Recorder`] standing in for the `claude`
/// command line, and returns what is wrong with what it ran and wrote.
#[must_use]
pub fn review_runs_by_code(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    if let Err(e) = prepare(checkout, workdir) {
        return vec![e];
    }
    let host = Recorder::default();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let args = ["--workdir".to_owned(), workdir.display().to_string()];
    let code = review_run::cli::run(&host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        let why = err.lines().last().unwrap_or_default();
        return vec![format!("review-run exited {code}: {why}")];
    }
    let events = host.events.into_inner().unwrap_or_default();
    let mut problems = order_problems(&events);
    problems.extend(outcome_problems(workdir));

    problems
}

#[cfg(test)]
mod tests;
