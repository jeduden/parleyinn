//! ENG-28: the review workflow runs the review's agents by code, keeps
//! them away from the reviewer app's key, and the review gate that
//! posts the review decides as the requirement says.

use std::collections::BTreeMap;

use review_eval::runner::TOOLS;

use crate::list;
use crate::static_analysis::without_assignments;

/// What a job runs the review's agents with: review-run, which starts
/// every dispatched agent.
const AGENT_RUN: &str = "-p review-run";

/// An action that runs one harness session, which would route the review
/// itself; a job that uses one is held to the agents' rules too.
const AGENT_ACTION: &str = "anthropics/claude-code-action@";

/// The ways a job installs the Claude Code command line, which starts
/// an agent from any later step: the repository's composite action and
/// the npm package it installs.
const AGENT_CLI: [&str; 2] = ["./.github/actions/claude-code", "@anthropic-ai/claude-code"];

/// The run of review-eval, which may start the command line beside
/// review-run: it runs the reviewer evaluation's cases (ENG-32) and
/// routes no review. Its `plan` starts no agent, so it grants none.
const EVAL_RUN: &str = "cargo run --locked --quiet -p review-eval -- run";

/// Where review-run writes each agent's run log, which holds every tool
/// result the agent read: it stays in the agents' job.
pub const RUN_LOGS: &str = "review/logs";

/// The action that uploads an artifact out of the job.
const UPLOAD_ACTION: &str = "actions/upload-artifact@";

/// The tools review-run gives its agents, by kind, as the evaluation's
/// runner sets them: reading tools alone, a specialist's report being
/// assembled from its run log.
pub const RUN_TOOLS: [(&str, &str); 2] = [("specialist", TOOLS), ("generalist", TOOLS)];

/// The tools that read and nothing more, which every agent may use.
pub const READING: [&str; 3] = ["Read", "Glob", "Grep"];

/// A GitHub Actions workflow split the way the ENG-28 steps read it:
/// its triggers and its jobs, comments left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Workflow {
    pub triggers: Vec<String>,
    /// The lines of the `on:` block.
    pub on: String,
    pub jobs: Vec<Job>,
}

/// One job's name and its lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Job {
    pub name: String,
    pub text: String,
}

/// Splits `body` into its `on:` block and its jobs. It reads only the
/// block layout the repository's workflows use: two-space indents, and
/// triggers and jobs as keys one level down.
#[must_use]
pub fn parse(body: &str) -> Workflow {
    let mut w = Workflow::default();
    let mut top = "";
    for line in body.split_inclusive('\n').map(uncomment) {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(key) = top_key(&line) {
            top = key;
            continue;
        }
        match (top, sub_key(&line)) {
            ("on", Some(key)) => {
                w.triggers.push(key.to_owned());
                w.on.push_str(&line);
            }
            ("on", None) => w.on.push_str(&line),
            ("jobs", Some(key)) => w.jobs.push(Job {
                name: key.to_owned(),
                text: String::new(),
            }),
            ("jobs", None) => {
                if let Some(job) = w.jobs.last_mut() {
                    job.text.push_str(&line);
                }
            }
            _ => {}
        }
    }

    w
}

/// The key a line names at the top level, such as `on` in `on:`.
fn top_key(line: &str) -> Option<&'static str> {
    let (key, _) = line.split_once(':')?;
    let valid = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '_' || c == '-');
    // The two keys the steps read; any other top-level key ends them.
    valid.then_some(match key {
        "on" => "on",
        "jobs" => "jobs",
        _ => "other",
    })
}

/// The key a line names one level down, at a two-space indent.
fn sub_key(line: &str) -> Option<&str> {
    let (key, _) = line.strip_prefix("  ")?.split_once(':')?;
    let valid = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

    valid.then_some(key)
}

/// Drops a YAML comment: a whole comment line, or one that follows a
/// space.
pub(crate) fn uncomment(line: &str) -> String {
    if line.trim_start().starts_with('#') {
        return String::new();
    }
    match line.find(" #") {
        Some(i) => format!("{}\n", &line[..i]),
        None => line.to_owned(),
    }
}

/// Whether `text` grants a write permission: a `:` followed by
/// `write`, quoted or not, as a whole word, or `write-all`.
fn grants_write(text: &str) -> bool {
    text.match_indices(':').any(|(i, _)| {
        let rest = text[i + 1..].trim_start().trim_start_matches(['"', '\'']);
        rest.strip_prefix("write")
            .is_some_and(|after| !after.starts_with(is_word))
    })
}

/// Whether `text` holds `word` with no word character on either side.
fn has_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether a job's `text` installs the Claude Code command line.
fn installs_cli(text: &str) -> bool {
    AGENT_CLI.iter().any(|cli| text.contains(cli))
}

/// The commands that start the command after them, as `npx` and `env`
/// do, each skipped with its flags to find what a script line starts;
/// `timeout` with its duration too, and a shell with the `-c` command it
/// is given.
const WRAPPERS: [&str; 10] = [
    "env", "npx", "exec", "command", "nohup", "time", "sudo", "timeout", "bash", "sh",
];

/// Whether `line` of a job starts the Claude Code command line itself:
/// `claude`, a path to it or the npm package the first word of any
/// command on a script line, in a subshell or a command substitution
/// too, after its assignments and any wrapper such as `env`, `npx` or
/// `bash -c`, so whatever tools and permission mode it passes go past
/// review-run's and review-eval's.
fn starts_cli(line: &str) -> bool {
    let line = line.trim_start();
    let line = line.strip_prefix("- ").unwrap_or(line).trim_start();
    let line = line.strip_prefix("run:").unwrap_or(line).trim_start();
    if line.starts_with('#') {
        return false;
    }

    line.split(['&', '|', ';', '(', ')', '`']).any(|command| {
        let mut rest = without_assignments(command.trim());
        loop {
            let (word, after) = rest.split_once(' ').unwrap_or((rest, ""));
            let word = word.trim_matches(['"', '\'']);
            if !WRAPPERS.contains(&word) {
                return word == "claude"
                    || word.ends_with("/claude")
                    || word
                        .strip_prefix(AGENT_CLI[1])
                        .is_some_and(|v| v.is_empty() || v.starts_with('@'));
            }
            rest = after.trim_start();
            let skipped = |r: &str| {
                r.starts_with('-')
                    || (word == "timeout" && r.starts_with(|c: char| c.is_ascii_digit()))
            };
            while skipped(rest) {
                rest = rest.split_once(' ').map_or("", |(_, r)| r.trim_start());
            }
            rest = without_assignments(rest);
        }
    })
}

/// Whether a job's `text` runs the review's agents, a harness session,
/// or installs the command line any of its steps can start one with.
fn runs_agents(text: &str) -> bool {
    text.contains(AGENT_RUN) || text.contains(AGENT_ACTION) || installs_cli(text)
}

/// A job's steps, each from its `- ` line at the steps' indent.
fn steps(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.split_inclusive('\n') {
        if line.starts_with("      - ") {
            out.push(line.to_owned());
        } else if let Some(step) = out.last_mut() {
            step.push_str(line);
        }
    }

    out
}

/// The paths an upload step names, one inline or one a line in its
/// block; none for a step that uploads nothing.
fn upload_paths(step: &str) -> Vec<String> {
    if !step.contains(UPLOAD_ACTION) {
        return Vec::new();
    }
    let mut lines = step.lines();
    let Some(line) = lines.find(|l| l.trim_start().starts_with("path:")) else {
        return Vec::new();
    };
    let indent = line.len() - line.trim_start().len();
    let value = line.trim_start()["path:".len()..].trim();
    if !value.is_empty() && !value.starts_with(['|', '>']) {
        return vec![value.trim_matches(['"', '\'']).to_owned()];
    }

    lines
        .take_while(|l| l.len() - l.trim_start().len() > indent)
        .map(|l| l.trim().trim_matches(['"', '\'']).to_owned())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Whether the upload `paths` carry `dir` out of the job: a path at or
/// under it, or one above it, or a glob that can match it, that no `!`
/// path excludes it from. Only a
/// `!` path at `dir` or above it excludes it: one under it, a single
/// log, leaves the rest uploaded.
fn uploads_dir(paths: &[String], dir: &str) -> bool {
    let bare = |p: &str| {
        p.trim_start_matches("./")
            .trim_end_matches(['*', '/'])
            .to_owned()
    };
    let above = |p: &str| {
        p.is_empty()
            || p == "."
            || dir
                .strip_prefix(p)
                .is_some_and(|rest| rest.starts_with('/'))
    };
    let excluded = paths
        .iter()
        .filter_map(|p| p.strip_prefix('!'))
        .map(&bare)
        .any(|p| p == dir || above(&p));

    // A glob can match anything under what comes before its first
    // wildcard, `dir` included when that is a prefix of it.
    paths
        .iter()
        .filter(|p| !p.starts_with('!'))
        .any(|p| match p.find(['*', '?', '[']) {
            Some(at) if !excluded && dir.starts_with(&bare(&p[..at])) => true,
            _ => {
                let p = bare(p);
                p.starts_with(dir) || (!excluded && above(&p))
            }
        })
}

/// The tools of the comma-separated `tools` other than reading:
/// anything that writes, runs a command, reaches the web or starts an
/// agent, a scoped `Edit(...)` or `Write(...)` rule included.
fn beyond_reading(tools: &str) -> impl Iterator<Item = &str> {
    tools
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .filter(|t| {
            !READING
                .iter()
                .any(|r| t == r || t.starts_with(&format!("{r}(")))
        })
}

impl Workflow {
    /// Requires exactly two triggers: `workflow_run` on the named
    /// workflow's completion, and `issue_comment` on a new comment, so
    /// the stakeholder's consent in a comment brings a review. GitHub
    /// runs both as the default branch defines them, so a pull request
    /// cannot rewrite its own reviewer; any other trigger would let a
    /// branch's copy run.
    ///
    /// # Errors
    ///
    /// Fails on another trigger or a missing one, or on a trigger
    /// block that does not say what it must.
    pub fn runs_after(&self, name: &str) -> Result<(), String> {
        if self.triggers != ["workflow_run", "issue_comment"] {
            return Err(format!(
                "the review workflow triggers on {}, want workflow_run and issue_comment",
                list(&self.triggers)
            ));
        }
        let wants = [
            ("workflow_run", format!("workflows: [{name}]")),
            ("workflow_run", "types: [completed]".to_owned()),
            ("issue_comment", "types: [created]".to_owned()),
        ];
        for (trigger, want) in wants {
            if !self.trigger_block(trigger).contains(&want) {
                return Err(format!("the {trigger} trigger does not say {want:?}"));
            }
        }

        Ok(())
    }

    /// The lines of `on:` that belong to `trigger`: from its own key to
    /// the next trigger's.
    fn trigger_block(&self, trigger: &str) -> String {
        let mut inside = false;
        let mut out = String::new();
        for line in self.on.split_inclusive('\n') {
            if let Some(key) = sub_key(line) {
                inside = key == trigger;
            }
            if inside {
                out.push_str(line);
            }
        }

        out
    }

    /// The jobs that run the review's agents, or a harness session.
    fn agent_jobs(&self) -> impl Iterator<Item = &Job> {
        self.jobs.iter().filter(|j| runs_agents(&j.text))
    }

    /// Requires a job to run the review's agents with `cmd`, and no job
    /// to run a harness session, or the command line outside review-run
    /// and review-eval, that would route the review instead.
    #[must_use]
    pub fn agents_run_by(&self, cmd: &str) -> Vec<String> {
        let mut problems: Vec<String> = self
            .jobs
            .iter()
            .filter(|j| j.text.contains(AGENT_ACTION))
            .map(|j| {
                format!(
                    "job {} runs a harness session, {AGENT_ACTION}, which would route the review",
                    j.name
                )
            })
            .collect();
        for j in &self.jobs {
            let runner = j.text.contains(cmd) || j.text.contains(EVAL_RUN);
            if installs_cli(&j.text) && !runner {
                problems.push(format!(
                    "job {} starts the Claude Code command line outside review-run and review-eval, which would route the review",
                    j.name
                ));
            } else if j.text.lines().any(starts_cli) {
                problems.push(format!(
                    "job {} starts the Claude Code command line itself, outside review-run and review-eval, which would route the review",
                    j.name
                ));
            }
        }
        if !self.jobs.iter().any(|j| j.text.contains(cmd)) {
            problems.insert(0, format!("no job runs the review's agents with {cmd:?}"));
        }

        problems
    }

    /// Requires an agent job, and that none of them is granted a write
    /// permission or reads `key`.
    #[must_use]
    pub fn agents_unprivileged(&self, key: &str) -> Vec<String> {
        let mut problems = Vec::new();
        for j in self.agent_jobs() {
            if grants_write(&j.text) {
                problems.push(format!(
                    "agent job {} is granted a write permission",
                    j.name
                ));
            }
            if j.text.contains(&format!("secrets.{key}")) {
                problems.push(format!("agent job {} reads {key}", j.name));
            }
        }
        if self.agent_jobs().next().is_none() {
            problems.push("no job runs the review's agents".to_owned());
        }

        problems
    }

    /// Requires the agents to write nothing: `run`, the tools review-run
    /// gives each kind of agent, and every `--allowedTools` an agent job
    /// passes, allow the reading tools and no other, a scoped writing
    /// rule included. A specialist's report is assembled from its run
    /// log, so no agent needs to write.
    #[must_use]
    pub fn agents_write_nothing(&self, run: &[(&str, &str)]) -> Vec<String> {
        let want = format!("{} alone", READING.join(", "));
        let mut problems = Vec::new();
        for (kind, tools) in run {
            problems.extend(
                beyond_reading(tools)
                    .map(|t| format!("review-run's {kind} may use {t}, want {want}")),
            );
        }
        for j in self.agent_jobs() {
            for line in j.text.lines().filter(|l| l.contains("--allowedTools")) {
                let tools = line
                    .split_once("--allowedTools")
                    .map_or("", |(_, rest)| rest)
                    .trim()
                    .trim_matches('"');
                problems.extend(
                    beyond_reading(tools)
                        .map(|t| format!("agent job {} may use {t}, want {want}", j.name)),
                );
            }
        }

        problems
    }

    /// Requires every agent job to keep `logs`, its agents' run logs,
    /// inside the job: no step of one uploads them as an artifact, which
    /// would carry whatever an agent read, its environment included, out
    /// of the job.
    #[must_use]
    pub fn run_logs_stay(&self, logs: &str) -> Vec<String> {
        self.agent_jobs()
            .filter(|j| {
                steps(&j.text)
                    .iter()
                    .any(|s| uploads_dir(&upload_paths(s), logs))
            })
            .map(|j| {
                format!(
                    "agent job {} uploads {logs}, every tool result its agents read",
                    j.name
                )
            })
            .collect()
    }

    /// The one job that reads `key`.
    fn key_job(&self, key: &str) -> Result<&Job, String> {
        let holders: Vec<&Job> = self
            .jobs
            .iter()
            .filter(|j| j.text.contains(&format!("secrets.{key}")))
            .collect();
        match holders.as_slice() {
            [one] => Ok(one),
            _ => Err(format!(
                "{} jobs read {key:?}, want exactly one",
                holders.len()
            )),
        }
    }

    /// Requires one job to hold `key`, and that job to run no agent.
    ///
    /// # Errors
    ///
    /// Fails when no job or several hold the key, or its job runs the
    /// agent.
    pub fn key_held_apart(&self, key: &str) -> Result<(), String> {
        let j = self.key_job(key)?;
        if runs_agents(&j.text) {
            return Err(format!("job {} holds {key} and runs the agents", j.name));
        }

        Ok(())
    }

    /// Requires the key's job to need every agent job, and not to run
    /// when one of them failed.
    ///
    /// # Errors
    ///
    /// Fails when the key's job cannot be found.
    pub fn key_job_after_agents(&self, key: &str) -> Result<Vec<String>, String> {
        let j = self.key_job(key)?;
        let needs = j
            .text
            .lines()
            .rfind(|l| l.trim_start().starts_with("needs:"))
            .unwrap_or_default();
        let mut problems: Vec<String> = self
            .agent_jobs()
            .filter(|a| !has_word(needs, &a.name))
            .map(|a| format!("job {} does not need agent job {}", j.name, a.name))
            .collect();
        for over in ["always()", "failure()", "cancelled()"] {
            if j.text.contains(over) {
                problems.push(format!(
                    "job {} runs on {over}, even after a failed review",
                    j.name
                ));
            }
        }

        Ok(problems)
    }

    /// Requires the key's job to post its decision as the check `name`:
    /// to mint a token that writes checks, post to the check-runs
    /// endpoint under that name, and fail the check unless it approves.
    ///
    /// # Errors
    ///
    /// Fails when the key's job cannot be found.
    pub fn key_job_posts_check(&self, key: &str, name: &str) -> Result<Vec<String>, String> {
        let j = self.key_job(key)?;
        let named = format!("name={name}");
        Ok([
            ("mint", "permission-checks: write"),
            ("post to", "repos/$REPO/check-runs"),
            ("name the check", named.as_str()),
            ("fail the check by default:", "conclusion=failure"),
        ]
        .into_iter()
        .filter(|(_, text)| !j.text.contains(text))
        .map(|(what, text)| format!("job {} does not {what} {text}", j.name))
        .collect())
    }

    /// Requires the `evaluate` job (ENG-32): it runs `cmd`, needs every
    /// agent job, holds no write permission and not `key`, and the
    /// key's job needs it, so the evaluation's findings reach the gate.
    ///
    /// # Errors
    ///
    /// Fails when there is no evaluate job or the key's job cannot be
    /// found.
    pub fn evaluate_job_runs(&self, cmd: &str, key: &str) -> Result<Vec<String>, String> {
        let j = self
            .jobs
            .iter()
            .find(|j| j.name == "evaluate")
            .ok_or("no evaluate job in the review workflow")?;
        let needs = |job: &Job| {
            job.text
                .lines()
                .rfind(|l| l.trim_start().starts_with("needs:"))
                .unwrap_or_default()
                .to_owned()
        };
        let mut problems = Vec::new();
        if !j.text.contains(cmd) {
            problems.push(format!("job {} does not run {cmd:?}", j.name));
        }
        if grants_write(&j.text) {
            problems.push(format!("job {} is granted a write permission", j.name));
        }
        if j.text.contains(&format!("secrets.{key}")) {
            problems.push(format!("job {} reads {key}", j.name));
        }
        let own_needs = needs(j);
        problems.extend(
            self.agent_jobs()
                .filter(|a| a.name != j.name && !has_word(&own_needs, &a.name))
                .map(|a| format!("job {} does not need agent job {}", j.name, a.name)),
        );
        // The key's job among the others: an evaluate job that reads the
        // key is a problem named above, not a second holder.
        let holders: Vec<&Job> = self
            .jobs
            .iter()
            .filter(|job| job.name != j.name && job.text.contains(&format!("secrets.{key}")))
            .collect();
        let [key_job] = holders.as_slice() else {
            return Err(format!(
                "{} jobs read {key:?}, want exactly one",
                holders.len()
            ));
        };
        if !has_word(&needs(key_job), &j.name) {
            problems.push(format!("job {} does not need job {}", key_job.name, j.name));
        }

        Ok(problems)
    }

    /// Requires the key's job to run `cmd`.
    ///
    /// # Errors
    ///
    /// Fails when the key's job cannot be found or does not run `cmd`.
    pub fn key_job_runs(&self, key: &str, cmd: &str) -> Result<(), String> {
        let j = self.key_job(key)?;
        if !j.text.contains(cmd) {
            return Err(format!("job {} does not run {cmd:?}", j.name));
        }

        Ok(())
    }
}

/// Keys each body row of `rows` by the header row.
///
/// # Errors
///
/// Fails when the table has no rows under its header.
pub fn table_rows(rows: &[Vec<String>]) -> Result<Vec<BTreeMap<String, String>>, String> {
    let [header, body @ ..] = rows else {
        return Err("the table has no rows under its header".to_owned());
    };
    if body.is_empty() {
        return Err("the table has no rows under its header".to_owned());
    }

    Ok(body
        .iter()
        .map(|row| header.iter().cloned().zip(row.iter().cloned()).collect())
        .collect())
}

/// Runs every row through the review gate and compares the outcome
/// with the row's review column.
#[must_use]
pub fn gate_decides(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let got = gate_outcome(row);
            (row.get("review").map(String::as_str) != Some(got))
                .then(|| format!("row {} {row:?}: the review gate gives {got}", i + 1))
        })
        .collect()
}

/// The review gate's command line, as the review workflow runs it, on
/// the files [`gate_outcome`] writes.
fn gate_args(reviewed: &str, head: &str) -> Vec<String> {
    [
        "--outcome",
        "outcome.json",
        "--check-runs",
        "check-runs.jsonl",
        "--reviewed",
        reviewed,
        "--head",
        head,
        "--checked",
        "checked.json",
        "--dispatch",
        "dispatch.json",
        "--consulted",
        "consulted.json",
        "--reports",
        "reports",
        "--answers",
        "answers.json",
    ]
    .map(str::to_owned)
    .to_vec()
}

/// The report review-run assembled from the row's one specialist's run
/// log, and the answers file holding its final answer: a `specialist`
/// cell of `reported`, or none, hands the gate a report with a linked
/// finding that ends with the outcome line the specialist answered,
/// `requested` one with a requirement request instead, `no report` no
/// report, `empty report` a blank one, `incomplete` the report of a run
/// cut off before its answer, headed so and ending with no outcome, `no
/// outcome answered` an answer with no outcome line, `other outcome
/// answered` an answer whose outcome is not the report's, `malformed
/// finding` a logged finding that gives a severity where its
/// consequence and requirements belong, `revised nothing` the report
/// assembled from a log whose one finding line revises a finding the
/// specialist never stated, so only the revision standing as the finding
/// it states keeps the report's outcome of findings, `withdrew nothing`
/// the one assembled from a log that states a linked finding and
/// withdraws one it never stated, `none beside a finding` an
/// outcome of none after a finding, `findings with none` an outcome of
/// findings after none, and `unknown id` a finding citing ENG-999, an id
/// of the right shape that no requirement has, which the gate, holding
/// no tree, takes; `None` for any other value.
fn specialist(row: &BTreeMap<String, String>) -> Option<(Option<String>, &'static str)> {
    const FINDINGS: &str = r#"{"lang-rust":"outcome: findings — fine","review-code":"r"}"#;
    const NONE: &str = r#"{"lang-rust":"outcome: none — fine","review-code":"r"}"#;
    const NO_OUTCOME: &str = r#"{"lang-rust":"I am done.","review-code":"r"}"#;
    let (report, answers) = match row.get("specialist").map_or("reported", String::as_str) {
        "reported" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        "requested" => (
            Some("a.rs:1 — request: add a rule on x — none says so\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        "no report" => (None, FINDINGS),
        "empty report" => (Some("  \n"), FINDINGS),
        "incomplete" => (
            Some(
                "incomplete: its run ended with error_max_turns\na.rs:1 — if not addressed: x breaks — I2\n",
            ),
            NO_OUTCOME,
        ),
        "no outcome answered" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: findings — fine\n"),
            NO_OUTCOME,
        ),
        "other outcome answered" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: findings — fine\n"),
            NONE,
        ),
        "malformed finding" => (
            Some("a.rs:1 — nit — x\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        "revised nothing" => {
            return Some((
                Some(assembled("revised: b.rs:2 — request: add y — why")),
                FINDINGS,
            ));
        }
        "withdrew nothing" => {
            return Some((
                Some(assembled(
                    "a.rs:1 — if not addressed: x breaks — I2\nwithdrawn: b.rs:2 — I checked it and it holds",
                )),
                FINDINGS,
            ));
        }
        "none beside a finding" => (
            Some("a.rs:1 — if not addressed: x breaks — I2\noutcome: none — fine\n"),
            NONE,
        ),
        "findings with none" => (Some("outcome: findings — fine\n"), FINDINGS),
        "unknown id" => (
            Some("a.rs:1 — if not addressed: x breaks — ENG-999\noutcome: findings — fine\n"),
            FINDINGS,
        ),
        _ => return None,
    };

    Some((report.map(str::to_owned), answers))
}

/// The report review-run assembles from a run log that states `lines`
/// and answers an outcome of findings.
fn assembled(lines: &str) -> String {
    let texts = [lines.to_owned(), "outcome: findings — fine".to_owned()];
    review_gate::report::assemble(&texts, "outcome: findings — fine", None)
}

/// The dispatch a table row hands the gate by default: one generalist
/// with one specialist.
const DISPATCH: &str = r#"{"review-code":["lang-rust"]}"#;

/// What [`gate_outcome`] names for a row whose `dispatch`, `consulted`,
/// `specialist` or `own check` cell holds a value it does not define, so a
/// mistyped row fails rather than reads as the default.
pub const UNKNOWN_CELL: &str = "a cell value the table does not define";

/// The dispatch file a row hands the gate: [`DISPATCH`] for `given` or
/// no `dispatch` column, a file that is no map for `malformed`, and no
/// file (`Some(None)`) for `missing`; `None` for any other value.
fn dispatch(row: &BTreeMap<String, String>) -> Option<Option<&'static str>> {
    match row.get("dispatch").map_or("given", String::as_str) {
        "given" => Some(Some(DISPATCH)),
        "malformed" => Some(Some("[]")),
        "missing" => Some(None),
        _ => None,
    }
}

/// The agents a row says the review consulted: both of
/// [`DISPATCH`] for `all` or no `consulted` column, the generalist alone
/// for `not all`, and a file that is no list for `malformed`; `None` for
/// any other value.
fn consulted(row: &BTreeMap<String, String>) -> Option<&'static str> {
    match row.get("consulted").map_or("all", String::as_str) {
        "all" => Some(r#"["review-code","lang-rust"]"#),
        "not all" => Some(r#"["review-code"]"#),
        "malformed" => Some("nope"),
        _ => None,
    }
}

/// The check run line for the gate's own `review-gate` check a row
/// names: a failed one from the reviewer app for `failed`, none for
/// `none` or no `own check` column; `None` for any other value.
fn own_check(cell: &str) -> Option<String> {
    match cell {
        "failed" => Some(format!(
            "{{\"name\":{:?},\"status\":\"completed\",\"conclusion\":\"failure\",\"app\":{}}}\n",
            review_gate::OWN_CHECK,
            review_gate::REVIEWER_APP_ID
        )),
        "none" | "" => Some(String::new()),
        _ => None,
    }
}

/// Runs the review gate's command line, as the review workflow does,
/// on the files one table row describes, and names what it does: the
/// review event it prints, "nothing" for a head that moved, or "an
/// error" when it exits unsuccessfully. A `missing` verdict leaves the
/// outcome file out. The gate always gets `--checked`: a `check
/// finding` of `none`, or a row without the column, hands it an empty
/// list, `nit` or `blocking` one deterministic finding of that
/// severity, `malformed` a file that is no list, and `missing` no file.
/// It always gets `--dispatch`, `--consulted`, `--reports` and
/// `--answers` (see [`dispatch`], [`consulted`] and [`specialist`]), and a
/// row whose cell for any of them, or for `own check`, it does not
/// define gives [`UNKNOWN_CELL`].
#[must_use]
pub fn gate_outcome(row: &BTreeMap<String, String>) -> &'static str {
    const REVIEWED: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";
    const MOVED: &str = "490efa734617788d1dc1f831b26d0e23888e90fa";
    let cell = |name: &str| row.get(name).map(String::as_str).unwrap_or_default();
    let (Some(dispatched), Some(consulted), Some((report, answers)), Some(own)) = (
        dispatch(row),
        consulted(row),
        specialist(row),
        own_check(cell("own check")),
    ) else {
        return UNKNOWN_CELL;
    };
    let findings = match cell("finding") {
        "none" => "[]".to_owned(),
        severity => format!(r#"[{{"path":"a.rs","line":1,"severity":{severity:?},"body":"b"}}]"#),
    };
    let conclusion = |cell: &str| {
        if cell == "passed" {
            "success"
        } else {
            "failure"
        }
    };
    let check = match row.get("check finding").map_or("none", String::as_str) {
        "none" => Some("[]".to_owned()),
        "missing" => None,
        "malformed" => Some("nope".to_owned()),
        severity => Some(format!(
            r#"[{{"path":"features/x.feature","line":0,"severity":{severity:?},"body":"c"}}]"#
        )),
    };
    let mut files = BTreeMap::from([(
        "check-runs.jsonl",
        format!(
            "{{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":{:?}}}\n\
             {{\"name\":\"lint\",\"status\":\"completed\",\"conclusion\":{:?}}}\n{}",
            conclusion(cell("CI")),
            conclusion(cell("other check")),
            own
        ),
    )]);
    if cell("verdict") != "missing" {
        files.insert(
            "outcome.json",
            format!(
                r#"{{"verdict":{:?},"summary":"s","findings":{findings}}}"#,
                cell("verdict")
            ),
        );
    }
    let head = if cell("head") == "moved" {
        MOVED
    } else {
        REVIEWED
    };
    if let Some(body) = dispatched {
        files.insert("dispatch.json", body.to_owned());
    }
    files.insert("consulted.json", consulted.to_owned());
    if let Some(body) = report {
        files.insert("reports/lang-rust.md", body);
    }
    files.insert("answers.json", answers.to_owned());
    if let Some(body) = check {
        files.insert("checked.json", body);
    }
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = review_gate::cli::run(gate_args(REVIEWED, head), &mut out, &mut err, &read);
    match serde_json::from_slice::<review_gate::Review>(&out) {
        _ if code != review_gate::cli::EXIT_OK => "an error",
        _ if out.is_empty() => "nothing",
        Ok(review) if review.event == review_gate::APPROVE => review_gate::APPROVE,
        _ => review_gate::REQUEST_CHANGES,
    }
}

#[cfg(test)]
mod tests;
