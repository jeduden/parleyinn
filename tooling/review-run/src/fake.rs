//! A host for the tests: `claude` runs a closure the test hands in,
//! which writes the run's log, and its report for a specialist, as the
//! real command line would; every command is kept.

use std::fs;
use std::sync::Mutex;

use review_eval::host::{Command, Host, Output};
use serde_json::json;

pub(crate) type Claude = Box<dyn Fn(&Command) -> Result<Output, String> + Sync>;

pub(crate) struct FakeHost {
    pub claude: Claude,
    pub calls: Mutex<Vec<Command>>,
}

impl FakeHost {
    pub(crate) fn new(
        claude: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
    ) -> Self {
        Self {
            claude: Box::new(claude),
            calls: Mutex::new(Vec::new()),
        }
    }

    /// The agents run, in the order their runs started.
    pub(crate) fn agents(&self) -> Vec<String> {
        self.calls.lock().unwrap().iter().map(agent_of).collect()
    }
}

impl Host for FakeHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        self.calls.lock().unwrap().push(command.clone());
        (self.claude)(command)
    }
}

/// The agent `command` runs as.
pub(crate) fn agent_of(command: &Command) -> String {
    command
        .args
        .iter()
        .position(|a| a == "--agent")
        .and_then(|at| command.args.get(at + 1))
        .cloned()
        .unwrap_or_default()
}

/// The prompt `command` gives its agent.
pub(crate) fn prompt_of(command: &Command) -> String {
    command.args.get(1).cloned().unwrap_or_default()
}

/// A run's log: one model call with usage, and a result answering
/// `answer` at `cost`.
pub(crate) fn log(answer: &str, cost: f64) -> String {
    let call = json!({"type": "assistant", "parent_tool_use_id": null, "message": {
        "id": "m1", "model": "claude-haiku-5-5",
        "usage": {"input_tokens": 10, "cache_creation_input_tokens": 20,
                  "cache_read_input_tokens": 30, "output_tokens": 5},
        "content": []}});
    let result = json!({"type": "result", "subtype": "success", "is_error": false,
        "result": answer, "total_cost_usd": cost, "num_turns": 3, "duration_ms": 9});

    format!("{call}\n{result}\n")
}

/// Answers `command` as its agent would: `report` written to its report
/// file when there is one, and a log answering `answer`.
pub(crate) fn answer(
    command: &Command,
    answer: &str,
    report: Option<&str>,
) -> Result<Output, String> {
    if let Some(report) = report {
        let dir = command.cwd.join("review/reports");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{}.md", agent_of(command))), report).unwrap();
    }
    write_log(command, &log(answer, 0.01));

    Ok(Output {
        code: Some(0),
        ..Output::default()
    })
}

/// Writes `text` as `command`'s log.
pub(crate) fn write_log(command: &Command, text: &str) {
    fs::write(command.stdout_to.as_ref().unwrap(), text).unwrap();
}

/// A specialist's report with one nit, its outcome line and its answer.
pub(crate) fn specialist(command: &Command) -> Result<Output, String> {
    let name = agent_of(command);
    let outcome = format!("outcome: nits — {name} saw one nit.");
    answer(
        command,
        &format!("{outcome}\nreview/reports/{name}.md"),
        Some(&format!(
            "src/a.rs:3 — nit — a long line — ENG-02\n\n{outcome}\n"
        )),
    )
}

/// A generalist's answer: the specialist's nit kept, its closing
/// sentence.
pub(crate) fn generalist(command: &Command) -> Result<Output, String> {
    let name = agent_of(command);
    answer(
        command,
        &format!("src/a.rs:3 — nit — a long line — lang-rust\n{name} finds the change sound."),
        None,
    )
}

/// Writes agent `name`'s definition under `.claude/agents` of `dir`,
/// with `turns` as its turn budget: a specialist's tools for a name
/// that is no generalist's.
pub(crate) fn define(dir: &std::path::Path, name: &str, turns: usize) {
    let generalist = name.starts_with("review-");
    let (tools, model) = if generalist {
        ("Read, Grep, Glob", "opus")
    } else {
        ("Read, Grep, Glob, Write", "haiku")
    };
    let agents = dir.join(".claude/agents");
    fs::create_dir_all(&agents).unwrap();
    fs::write(
        agents.join(format!("{name}.md")),
        format!(
            "---\nname: {name}\ndescription: d\ntools: {tools}\nmodel: {model}\neffort: low\nmaxTurns: {turns}\n---\n# {name}\n"
        ),
    )
    .unwrap();
}

/// Answers as a specialist or a generalist, by the prompt.
pub(crate) fn either(command: &Command) -> Result<Output, String> {
    if prompt_of(command).contains("a specialist") {
        specialist(command)
    } else {
        generalist(command)
    }
}
