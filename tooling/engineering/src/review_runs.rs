//! ENG-28: the review is run by code. `review-run`, given a dispatch,
//! runs each dispatched specialist as its own agent, assembles its
//! report from its run log, runs each dispatched generalist only once
//! its own specialists have ended and the breaker is decided, and
//! writes the review outcome from what they answered, telling each
//! specialist to read only its own diff and the context the review bundle
//! wrote for it, never the whole bundle. The check runs
//! `review-run` on a work directory holding the checkout's definitions
//! of the dispatched agents, with a stand-in for the `claude` command
//! line that records when each agent's run starts and ends, answers as
//! the agent would, and holds two specialists' runs as [`HOLDS`] says:
//! one a generalist does not weigh, until that generalist started, and
//! one it does, briefly, so a generalist that would not wait for it
//! starts first. The review's breaker never trips here, so it holds no
//! generalist back: the breaker has a check of its own.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

use review_bundle::cli::{BUNDLE, CONTEXTS, DIFFS};
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

/// The specialists the stand-in holds, each until the generalist named
/// beside it started or for as long as is named: lang-markdown, which
/// review-code does not weigh, until review-code started, which proves
/// review-code did not wait for it; and lang-rust, which review-code
/// weighs, briefly, so a review-code that did not wait for it would start
/// while it still ran.
pub const HOLDS: [(&str, &str, Duration); 2] = [
    ("lang-markdown", "review-code", Duration::from_secs(10)),
    ("lang-rust", "review-code", Duration::from_secs(2)),
];

/// When an agent's run started or ended, as the stand-in saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Started(String),
    Ended(String),
}

/// What a [`Recorder`] recorded: its events and each run's agent and prompt.
pub type Recorded = (Vec<Event>, Vec<(String, String)>);

/// A stand-in for the `claude` command line: it records each run's
/// start and end and each agent's prompt, and answers as the agent it
/// runs as; when it holds, each run [`HOLDS`] names ends only once its
/// generalist started, or after its time.
#[derive(Debug, Default)]
pub struct Recorder {
    pub events: Mutex<Vec<Event>>,
    /// Each run's agent and the prompt it was given.
    pub prompts: Mutex<Vec<(String, String)>>,
    /// Whether it holds the runs [`HOLDS`] names.
    pub holds: bool,
    started: Mutex<BTreeSet<String>>,
    start: Condvar,
}

impl Recorder {
    /// The events and the prompts it recorded; an error naming the lock
    /// a panicking run poisoned, so a check never reads a list as empty
    /// and passes.
    ///
    /// # Errors
    ///
    /// Fails when a lock of the recorder is poisoned.
    pub fn recorded(self) -> Result<Recorded, String> {
        let events = self
            .events
            .into_inner()
            .map_err(|_| "the stand-in's events lock is poisoned".to_owned())?;
        let prompts = self
            .prompts
            .into_inner()
            .map_err(|_| "the stand-in's prompts lock is poisoned".to_owned())?;
        Ok((events, prompts))
    }

    /// A recorder that holds the runs [`HOLDS`] names.
    #[must_use]
    pub fn holding() -> Self {
        Self {
            holds: true,
            ..Self::default()
        }
    }

    fn record(&self, event: Event) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    /// Notes that `agent` started, waking each run held for it, and
    /// holds `agent`'s run when [`HOLDS`] names it.
    fn hold(&self, agent: &str) {
        if !self.holds {
            return;
        }
        let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
        started.insert(agent.to_owned());
        self.start.notify_all();
        if let Some((_, until, most)) = HOLDS.iter().find(|(held, _, _)| *held == agent) {
            let _ = self
                .start
                .wait_timeout_while(started, *most, |s| !s.contains(*until));
        }
    }
}

impl Host for Recorder {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if let Some(out) = version(command) {
            return Ok(out);
        }
        let agent = agent_of(command);
        if let Ok(mut prompts) = self.prompts.lock() {
            prompts.push((agent.clone(), prompt_of(command)));
        }
        self.record(Event::Started(agent.clone()));
        self.hold(&agent);
        let answered = answer(command, &agent);
        self.record(Event::Ended(agent));
        answered?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// The version line a stand-in `claude --version` prints.
pub const VERSION: &str = "2.1.0 (Claude Code)";

/// What a stand-in answers `command`, when it asks `claude --version`,
/// the one run `review-run` starts that is no agent's.
#[must_use]
pub fn version(command: &Command) -> Option<Output> {
    (command.args == ["--version"]).then(|| Output {
        code: Some(0),
        stdout: format!("{VERSION}\n"),
        ..Output::default()
    })
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

/// The prompt `command` gives its agent, the value after `-p`.
#[must_use]
pub fn prompt_of(command: &Command) -> String {
    command
        .args
        .iter()
        .position(|a| a == "-p")
        .and_then(|at| command.args.get(at + 1))
        .cloned()
        .unwrap_or_default()
}

/// What is wrong with what each dispatched specialist among `prompts`
/// was told: not to read its own diff, not to read its own context,
/// or not to leave the bundle unread, which on a large change is too
/// large for one read.
#[must_use]
pub fn prompt_problems(prompts: &[(String, String)]) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, prompt) in prompts {
        if !SPECIALISTS.contains(&name.as_str()) {
            continue;
        }
        for own in [
            format!("review/{DIFFS}/{name}.diff"),
            format!("review/{CONTEXTS}/{name}.md"),
        ] {
            if !prompt.contains(&format!("`{own}`")) {
                problems.push(format!("{name} is not told to read {own}"));
            }
        }
        if !prompt.contains(&format!("never `review/{BUNDLE}`")) {
            problems.push(format!(
                "{name} is not told to leave review/{BUNDLE} unread"
            ));
        }
    }

    problems
}

/// What is wrong with what each part of specialist `name`'s split diff,
/// `count` parts, was told among `prompts`: not to read its own part's
/// diff alone, to read the whole diff, or not to leave the bundle
/// unread; and a part that never ran.
#[must_use]
pub fn part_problems(name: &str, count: usize, prompts: &[(String, String)]) -> Vec<String> {
    let mine: Vec<&str> = prompts
        .iter()
        .filter(|(agent, _)| agent == name)
        .map(|(_, prompt)| prompt.as_str())
        .collect();
    let mut problems = Vec::new();
    let whole = format!("Read only your own diff, `review/{DIFFS}/{name}.diff`");
    for index in 1..=count {
        let own = format!("Read only your own diff, `review/{DIFFS}/{name}.{index}.diff`");
        if !mine.iter().any(|p| p.contains(&own)) {
            problems.push(format!(
                "{name} part {index} is not told to read its own part's diff review/{DIFFS}/{name}.{index}.diff"
            ));
        }
    }
    for prompt in &mine {
        if prompt.contains(&whole) {
            problems.push(format!("{name} is told to read the whole diff in a part"));
        }
        if !prompt.contains(&format!("never `review/{BUNDLE}`")) {
            problems.push(format!(
                "{name} is not told to leave review/{BUNDLE} unread in a part"
            ));
        }
    }

    problems
}

/// The finding a stand-in specialist states in its text.
pub const FINDING: &str = "src/a.rs:1 — if not addressed: a long line hides a bug — I9";

/// Answers `command` as `agent` would, in its log alone: a specialist
/// states one linked finding in its text and answers its outcome line;
/// a generalist keeps, as a nit, the finding of the specialist
/// [`GENERALISTS`] names for it.
fn answer(command: &Command, agent: &str) -> Result<(), String> {
    let (text, result) =
        if let Some((_, path, from)) = GENERALISTS.iter().find(|(g, _, _)| *g == agent) {
            let answer =
                format!("{path}:1 — nit — a long line — {from}\n{agent} finds its scope sound.");
            (answer.clone(), answer)
        } else {
            (
                format!("I read the diff.\n{FINDING}"),
                format!("outcome: findings — {agent} saw one finding."),
            )
        };
    let log = command
        .stdout_to
        .as_deref()
        .ok_or("the run logs to no file")?;
    let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null, "message": {
        "id": format!("m-{agent}"), "model": "claude-haiku-5-5", "usage": {},
        "content": [{"type": "text", "text": text}]}});
    let line = serde_json::json!({"type": "result", "subtype": "success", "is_error": false,
        "result": result, "total_cost_usd": 0.01, "num_turns": 2});
    fs::write(log, format!("{call}\n{line}\n")).map_err(|e| format!("write {}: {e}", log.display()))
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
/// not run exactly once, an agent run that was not dispatched, a
/// generalist that started before one of its own specialists ended, and
/// a generalist waiting for the specialist [`HOLDS`] first names, which
/// it does not weigh.
#[must_use]
pub fn order_problems(events: &[Event]) -> Vec<String> {
    order_problems_by(events, DISPATCH)
}

/// [`order_problems`] by the dispatch text `dispatch`.
fn order_problems_by(events: &[Event], dispatch: &str) -> Vec<String> {
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
    let dispatch = match parse_dispatch(dispatch) {
        Ok(dispatch) => dispatch,
        Err(problem) => {
            problems.push(problem);
            return problems;
        }
    };
    for generalist in &generalists {
        let Some(&start) = started(generalist).first() else {
            continue;
        };
        for specialist in dispatch.get(*generalist).into_iter().flatten() {
            if at(Event::Ended(specialist.clone()))
                .iter()
                .any(|&end| end > start)
            {
                problems.push(format!(
                    "{generalist} started before its specialist {specialist} ended"
                ));
            }
        }
    }
    let (held, until, _) = HOLDS[0];
    let ended = at(Event::Ended(held.to_owned()));
    if let (Some(&start), Some(&end)) = (started(until).first(), ended.first())
        && start > end
    {
        problems.push(format!(
            "{until} did not start while {held}, a specialist it does not weigh, still ran"
        ));
    }

    problems
}

/// What is wrong with the files `review-run` wrote in `workdir`: an
/// outcome that does not read, a finding a generalist answered that it
/// leaves out, an agent that answered but is not consulted, and a
/// specialist's report not assembled from its log: the finding it
/// stated, then the outcome line it answered, nothing else.
#[must_use]
pub fn outcome_problems(workdir: &Path) -> Vec<String> {
    let read = |rel: &str| {
        let path = workdir.join(rel);
        fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
    };
    let mut problems = Vec::new();
    for name in SPECIALISTS {
        let want = format!("{FINDING}\noutcome: findings — {name} saw one finding.\n");
        match read(&format!("review/reports/{name}.md")) {
            Ok(report) if report == want => {}
            Ok(report) => problems.push(format!(
                "{name}'s report is not assembled from its log: {report:?}"
            )),
            Err(e) => problems.push(e),
        }
    }
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
    let host = Recorder::holding();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let args = [
        "--workdir".to_owned(),
        workdir.display().to_string(),
        "--breaker-fraction=1".to_owned(),
    ];
    let code = review_run::cli::run(&host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        let why = err.lines().last().unwrap_or_default();
        return vec![format!("review-run exited {code}: {why}")];
    }
    let (events, prompts) = match host.recorded() {
        Ok(recorded) => recorded,
        Err(e) => return vec![e],
    };
    let mut problems = order_problems(&events);
    problems.extend(prompt_problems(&prompts));
    problems.extend(outcome_problems(workdir));
    problems.extend(split_runs_by_code(checkout, &workdir.join("split")));
    problems.extend(context_problems(checkout, &workdir.join("bundle")));

    problems
}

#[cfg(test)]
mod tests;

/// The dispatch the host was given, or why it does not parse.
fn parse_dispatch(raw: &str) -> Result<std::collections::BTreeMap<String, Vec<String>>, String> {
    serde_json::from_str(raw)
        .map_err(|e| format!("the dispatch the host was given does not parse: {e}"))
}

/// The specialist whose diff the split check makes too large for one
/// run, and the files its two parts hold.
pub const SPLIT: (&str, [&str; 2]) = ("lang-rust", ["src/a.rs", "src/b.rs"]);

/// A diff of `path` too large with another for one run.
fn section(path: &str) -> String {
    let body = "+let x = 1;\n".repeat(2500);
    format!("diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -0,0 +1,2500 @@\n{body}")
}

/// Runs `review-run` on a work directory `workdir` prepared from
/// `checkout` whose specialist [`SPLIT`] names has a diff too large for
/// one run, and returns what is wrong with what each of its parts was
/// told.
#[must_use]
pub fn split_runs_by_code(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    if let Err(e) = prepare(checkout, workdir) {
        return vec![e];
    }
    let (name, files) = SPLIT;
    let dir = workdir.join("review").join(DIFFS);
    let diff: String = files.iter().map(|f| section(f)).collect();
    let written =
        fs::create_dir_all(&dir).and_then(|()| fs::write(dir.join(format!("{name}.diff")), diff));
    if let Err(e) = written {
        return vec![format!("write {}: {e}", dir.display())];
    }
    let host = Recorder::default();
    let args = [
        "--workdir".to_owned(),
        workdir.display().to_string(),
        "--breaker-fraction=1".to_owned(),
    ];
    let code = review_run::cli::run(&host, args, &mut Vec::new(), &mut Vec::new());
    if code != review_run::cli::EXIT_OK {
        return vec![format!("review-run exited {code} on a split diff")];
    }

    match host.recorded() {
        Ok((_, prompts)) => part_problems(name, files.len(), &prompts),
        Err(e) => vec![e],
    }
}

/// The specialist whose context the context check reads, the file the
/// change edits, and the requirement it names.
pub const CONTEXT: (&str, &str, &str) = (
    "lang-markdown",
    "docs/srs/10-engineering-quality.md",
    "ENG-28",
);

/// What is wrong with the context specialist [`CONTEXT`] names that the
/// review bundle wrote, run on `checkout` as both trees with a change
/// to the file [`CONTEXT`] names, into `out`: it lacks the review class,
/// the touched ids, a dispatch row that names the specialist or a
/// deterministic finding on its files.
#[must_use]
pub fn context_problems(checkout: &Checkout, out: &Path) -> Vec<String> {
    let (_, path, id) = CONTEXT;
    let files = out.join("in");
    let diff =
        format!("diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -1 +1 @@\n+{id}\n");
    let written = fs::create_dir_all(&files)
        .and_then(|()| fs::write(files.join("pr.diff"), diff))
        .and_then(|()| fs::write(files.join("consent.json"), "[]"));
    if let Err(e) = written {
        return vec![format!("write {}: {e}", files.display())];
    }
    let args = [
        "--base".to_owned(),
        checkout.root.display().to_string(),
        "--head".to_owned(),
        checkout.root.display().to_string(),
        "--diff".to_owned(),
        files.join("pr.diff").display().to_string(),
        "--out".to_owned(),
        out.display().to_string(),
        "--consent".to_owned(),
        files.join("consent.json").display().to_string(),
    ];
    let mut printed = Vec::new();
    let mut err = Vec::new();
    let code = review_bundle::cli::run(args, &mut printed, &mut err);
    if code != review_bundle::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        return vec![format!(
            "review-bundle exited {code}: {}",
            err.lines().last().unwrap_or_default()
        )];
    }
    context_in(out, &String::from_utf8_lossy(&printed))
}

/// What is wrong with the context [`CONTEXT`] names among the files the
/// review bundle wrote into `out`, `printed` its step outputs; a file
/// it did not write is named.
#[must_use]
pub fn context_in(out: &Path, printed: &str) -> Vec<String> {
    let (name, path, id) = CONTEXT;
    let read = |rel: &str| {
        let file = out.join(rel);
        fs::read_to_string(&file).map_err(|e| format!("read {}: {e}", file.display()))
    };
    match (
        read(review_bundle::cli::DISPATCH),
        read(review_bundle::cli::CHECKED),
        read(&format!("{CONTEXTS}/{name}.md")),
    ) {
        (Ok(dispatch), Ok(checked), Ok(context)) => {
            context_lacks(name, path, id, printed, &dispatch, &checked, &context)
        }
        (d, c, x) => [d, c, x].into_iter().filter_map(Result::err).collect(),
    }
}

/// What `context`, the context for `name` the review bundle wrote for a
/// change to `path` naming `id`, lacks of what the bundle's other
/// outputs say: `printed` its step outputs, `dispatch` and `checked`
/// its files.
#[must_use]
pub fn context_lacks(
    name: &str,
    path: &str,
    id: &str,
    printed: &str,
    dispatch: &str,
    checked: &str,
    context: &str,
) -> Vec<String> {
    let mut problems = Vec::new();
    let output = |key: &str| {
        printed
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key}=")))
            .unwrap_or_default()
    };
    let class = output("class");
    if class.is_empty() || !context.contains(&format!("Class: **{class}**")) {
        problems.push(format!(
            "the context for {name} lacks the review class {class:?}"
        ));
    }
    if output("touched") != "1" || !context.contains(&format!("touches 1 id: {id}")) {
        problems.push(format!("the context for {name} lacks the touched id {id}"));
    }
    let rows: Vec<(String, Vec<String>)> =
        serde_json::from_str::<std::collections::BTreeMap<String, Vec<String>>>(dispatch)
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, specialists)| specialists.iter().any(|s| s == name))
            .collect();
    if rows.is_empty() {
        problems.push(format!("the dispatch has no row naming {name}"));
    }
    for (generalist, _) in rows {
        if !context.contains(&format!("- `{generalist}`, weighing")) {
            problems.push(format!(
                "the context for {name} lacks its dispatch row under {generalist}"
            ));
        }
    }
    let findings = serde_json::from_str::<Vec<serde_json::Value>>(checked).unwrap_or_default();
    let on_path: Vec<_> = findings
        .iter()
        .filter(|f| f["path"].as_str() == Some(path))
        .collect();
    if on_path.is_empty() {
        problems.push(format!(
            "the bundle found nothing on {path} to put in the context"
        ));
    }
    for f in on_path {
        let line = f["line"].as_u64().unwrap_or_default();
        if !context.contains(&format!("`{path}:{line}`")) {
            problems.push(format!(
                "the context for {name} lacks the deterministic finding on {path}:{line}"
            ));
        }
    }

    problems
}
