//! A host for the tests: `claude` runs a closure the test hands in,
//! which writes the run's log as the real command line would: a
//! specialist's reads of its diff, its text blocks, its findings among
//! them, and its result; every command is kept.

use std::fs;
use std::sync::Mutex;

use review_eval::component::Kind;
use review_eval::host::{Command, Host, Output};
use serde_json::json;

pub(crate) type Claude = Box<dyn Fn(&Command) -> Result<Output, String> + Sync>;

pub(crate) struct FakeHost {
    pub claude: Claude,
    pub calls: Mutex<Vec<Command>>,
    /// Each `claude --version` asked, kept apart from the agents' runs.
    pub versions: Mutex<Vec<Command>>,
}

/// What the fake's `claude --version` prints.
pub(crate) const VERSION: &str = "2.1.0 (Claude Code)";

impl FakeHost {
    pub(crate) fn new(
        claude: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
    ) -> Self {
        Self {
            claude: Box::new(claude),
            calls: Mutex::new(Vec::new()),
            versions: Mutex::new(Vec::new()),
        }
    }

    /// The agents run, in the order their runs started.
    pub(crate) fn agents(&self) -> Vec<String> {
        self.calls.lock().unwrap().iter().map(agent_of).collect()
    }
}

impl Host for FakeHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        // A confined probe carries the refused tools after it (ENG-28).
        if command.args.first().is_some_and(|a| a == "--version") {
            self.versions.lock().unwrap().push(command.clone());
            return Ok(Output {
                code: Some(0),
                stdout: format!("{VERSION}\n"),
                ..Output::default()
            });
        }
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
    reading_log("", texts, subtype, answer, cost)
}

/// [`log_of`] after `reads`, the stream-json lines of the reads the
/// run's one model call made.
pub(crate) fn reading_log(
    reads: &str,
    texts: &[&str],
    subtype: &str,
    answer: &str,
    cost: f64,
) -> String {
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

    format!("{reads}{call}\n{result}\n")
}

/// The lines one `Read` takes, as the agent's file tool reads them.
pub(crate) const CHUNK: usize = review_eval::coverage::DEFAULT_LIMIT;

/// The diff file `command`'s prompt tells its specialist to read, when
/// it names one.
pub(crate) fn diff_of(command: &Command) -> Option<String> {
    let prompt = prompt_of(command);
    let rest = prompt.split("Read only your own diff, `").nth(1)?;
    rest.split('`').next().map(str::to_owned)
}

/// The stream-json lines of a `Read` of `path`, `limit` lines from
/// `offset`, in the run's model call `m1`, and its result.
pub(crate) fn read_lines(path: &str, offset: usize, limit: usize) -> String {
    let id = format!("read-{offset}");
    let call = json!({"type": "assistant", "parent_tool_use_id": null, "message": {
        "id": "m1", "model": "claude-haiku-5-5", "usage": {},
        "content": [{"type": "tool_use", "id": id, "name": "Read",
                     "input": {"file_path": path, "offset": offset, "limit": limit}}]}});
    let result = json!({"type": "user", "parent_tool_use_id": null, "message": {
        "content": [{"type": "tool_result", "tool_use_id": id, "content": "+a"}]}});
    format!("{call}\n{result}\n")
}

/// The reads of the whole diff `command`'s prompt names, in chunks of
/// [`CHUNK`] lines from its first, as stream-json lines; none when it
/// names none.
pub(crate) fn reads(command: &Command) -> String {
    let Some(rel) = diff_of(command) else {
        return String::new();
    };
    let path = command.cwd.join(&rel);
    let lines = fs::read_to_string(&path).map_or(0, |t| t.lines().count());
    let path = path.display().to_string();
    (0..lines.max(1).div_ceil(CHUNK))
        .map(|at| read_lines(&path, at * CHUNK + 1, CHUNK))
        .collect()
}

/// A successful run's log answering `answer` at `cost`, with no text
/// but the answer's.
pub(crate) fn log(answer: &str, cost: f64) -> String {
    log_of(&[answer], "success", answer, cost)
}

/// Answers `command` as its agent would: a log whose reads cover the
/// whole diff its prompt names, whose text blocks are `texts`, then a
/// successful result answering `answer`.
pub(crate) fn answer(command: &Command, texts: &[&str], answer: &str) -> Result<Output, String> {
    let log = reading_log(&reads(command), texts, "success", answer, 0.01);
    write_log(command, &log);

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
    // Every specialist is told to read its diff: one the test gives none
    // has an empty one, with no line to cover.
    let diff = dir.join(format!("review/diffs/{name}.diff"));
    if review_eval::component::kind_of(name) == Kind::Specialist && !diff.exists() {
        fs::create_dir_all(diff.parent().unwrap()).unwrap();
        fs::write(diff, "").unwrap();
    }
}

/// Answers as a specialist or a generalist, by the prompt.
pub(crate) fn either(command: &Command) -> Result<Output, String> {
    if prompt_of(command).contains("a specialist") {
        specialist(command)
    } else {
        generalist(command)
    }
}
