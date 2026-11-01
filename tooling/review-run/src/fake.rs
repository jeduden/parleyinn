//! A host for the tests: `claude` runs a closure the test hands in,
//! which writes the run's log as the real command line would: a
//! specialist's text blocks, its findings among them, and its result;
//! every command is kept.

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

/// A clock for the tests: it never sleeps, but moves on by each wait,
/// keeps every wait, and draws the jitter it is given.
#[derive(Debug, Default)]
pub(crate) struct FakeClock {
    pub now: Mutex<std::time::Duration>,
    pub slept: Mutex<Vec<std::time::Duration>>,
    pub jitter: f64,
}

impl crate::controls::Clock for FakeClock {
    fn now(&self) -> std::time::Duration {
        *self.now.lock().unwrap()
    }

    fn sleep(&self, duration: std::time::Duration) {
        *self.now.lock().unwrap() += duration;
        self.slept.lock().unwrap().push(duration);
    }

    fn jitter(&self) -> f64 {
        self.jitter
    }
}

impl FakeClock {
    /// The waits slept, in order.
    pub(crate) fn slept(&self) -> Vec<std::time::Duration> {
        self.slept.lock().unwrap().clone()
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

/// The finding a fake specialist states.
pub(crate) const FINDING: &str = "src/a.rs:3 — if not addressed: a long line hides a bug — I9";

/// A run's log: one model call with usage and a text block per entry of
/// `texts`, and a result of `subtype` answering `answer` at `cost`.
pub(crate) fn log_of(texts: &[&str], subtype: &str, answer: &str, cost: f64) -> String {
    let blocks: Vec<serde_json::Value> = texts
        .iter()
        .map(|t| json!({"type": "text", "text": t}))
        .collect();
    let call = json!({"type": "assistant", "parent_tool_use_id": null, "message": {
        "id": "m1", "model": "claude-haiku-5-5",
        "usage": {"input_tokens": 10, "cache_creation_input_tokens": 20,
                  "cache_read_input_tokens": 30, "output_tokens": 5},
        "content": blocks}});
    let result = json!({"type": "result", "subtype": subtype, "is_error": subtype != "success",
        "result": answer, "total_cost_usd": cost, "num_turns": 3, "duration_ms": 9});

    format!("{call}\n{result}\n")
}

/// A successful run's log answering `answer` at `cost`, with no text
/// but the answer's.
pub(crate) fn log(answer: &str, cost: f64) -> String {
    log_of(&[answer], "success", answer, cost)
}

/// Answers `command` as its agent would: a log whose text blocks are
/// `texts`, then a successful result answering `answer`.
pub(crate) fn answer(command: &Command, texts: &[&str], answer: &str) -> Result<Output, String> {
    write_log(command, &log_of(texts, "success", answer, 0.01));

    Ok(Output {
        code: Some(0),
        ..Output::default()
    })
}

/// Writes `text` as `command`'s log.
pub(crate) fn write_log(command: &Command, text: &str) {
    fs::write(command.stdout_to.as_ref().unwrap(), text).unwrap();
}

/// A specialist's run: one linked finding stated in its text, then its
/// outcome line as its answer.
pub(crate) fn specialist(command: &Command) -> Result<Output, String> {
    let name = agent_of(command);
    let outcome = format!("outcome: findings — {name} saw one finding.");
    answer(command, &[FINDING, &outcome], &outcome)
}

/// A generalist's answer: the specialist's nit kept, its closing
/// sentence.
pub(crate) fn generalist(command: &Command) -> Result<Output, String> {
    let name = agent_of(command);
    let text =
        format!("src/a.rs:3 — nit — a long line — lang-rust\n{name} finds the change sound.");
    answer(command, &[&text], &text)
}

/// Writes agent `name`'s definition under `.claude/agents` of `dir`,
/// with `turns` as its turn budget: every agent reads and writes
/// nothing; a name that is no generalist's runs on haiku.
pub(crate) fn define(dir: &std::path::Path, name: &str, turns: usize) {
    let model = if name.starts_with("review-") {
        "opus"
    } else {
        "haiku"
    };
    let agents = dir.join(".claude/agents");
    fs::create_dir_all(&agents).unwrap();
    fs::write(
        agents.join(format!("{name}.md")),
        format!(
            "---\nname: {name}\ndescription: d\ntools: Read, Grep, Glob\nmodel: {model}\neffort: low\nmaxTurns: {turns}\n---\n# {name}\n"
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
