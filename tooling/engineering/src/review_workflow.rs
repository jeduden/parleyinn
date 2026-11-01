//! ENG-28: the review workflow runs the review's agents by code, keeps
//! them away from the reviewer app's key, and the review gate that
//! posts the review decides as the requirement says.

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

/// What stages the Claude token where no agent reads it.
pub const STAGE: &str = "-p review-eval -- stage-credential";

/// What scrubs the files a job keeps of that token.
pub const SCRUB: &str = "-p review-eval -- scrub";

/// The roots that hold every process's environment and open files, no
/// agent's to read.
pub const PROCESS_FILES: [&str; 3] = ["/proc", "/sys", "/dev"];

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

/// Whether a `step` runs the review's agents: review-run, review-eval's
/// `run`, or the command line itself.
fn runs_agent_step(step: &str) -> bool {
    // A build of review-run passes it no arguments, and runs nothing.
    step.contains(&format!("{AGENT_RUN} --"))
        || step.contains(EVAL_RUN)
        || step.lines().any(starts_cli)
}

/// A step's name, or its first line when it has none.
fn step_name(step: &str) -> String {
    step.lines()
        .find_map(|l| {
            l.trim_start()
                .trim_start_matches("- ")
                .strip_prefix("name:")
        })
        .unwrap_or_else(|| step.lines().next().unwrap_or_default().trim())
        .trim()
        .to_owned()
}

/// The `id:` a step gives itself, when it gives one.
fn step_id(step: &str) -> Option<&str> {
    step.lines().find_map(|l| {
        l.trim_start()
            .trim_start_matches("- ")
            .strip_prefix("id:")
            .map(str::trim)
    })
}

/// The names of the upload steps among `steps` that no scrub gates: a
/// step that runs [`SCRUB`] after the last step that runs an agent,
/// whose `scrubbed` output the upload's `if:` requires.
fn unscrubbed_uploads(steps: &[String]) -> Vec<String> {
    let last_agent = steps.iter().rposition(|s| runs_agent_step(s));
    let gates: Vec<String> = steps
        .iter()
        .enumerate()
        .filter(|(at, s)| s.contains(SCRUB) && last_agent.is_none_or(|last| *at > last))
        .filter_map(|(_, s)| step_id(s))
        .map(|id| format!("steps.{id}.outputs.scrubbed == 'true'"))
        .collect();
    steps
        .iter()
        .filter(|s| s.contains(UPLOAD_ACTION))
        .filter(|s| {
            !s.lines()
                .any(|l| is_condition(l) && gates.iter().any(|g| l.contains(g.as_str())))
        })
        .map(|s| step_name(s))
        .collect()
}

/// Whether `line` is a step's `if:`, its first line or not.
fn is_condition(line: &str) -> bool {
    line.trim_start()
        .trim_start_matches("- ")
        .starts_with("if:")
}

/// The names of the upload steps among `steps` whose `if:` lacks
/// `always()`, so a failed step before them skips them.
fn uploads_skipped_on_failure(steps: &[String]) -> Vec<String> {
    steps
        .iter()
        .filter(|s| s.contains(UPLOAD_ACTION))
        .filter(|s| !s.lines().any(|l| is_condition(l) && l.contains("always()")))
        .map(|s| step_name(s))
        .collect()
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

    /// Requires review-run's agents, by `disallowed`, the rules it
    /// refuses them, and `settings`, the settings it runs them with, to
    /// be refused every read of [`PROCESS_FILES`] and outside their
    /// trees, where the environment of every process on the machine
    /// and the staged credential lie (ENG-28).
    #[must_use]
    pub fn reads_refused(disallowed: &str, settings: &str) -> Vec<String> {
        let rules: Vec<&str> = disallowed.split(',').map(str::trim).collect();
        let mut problems: Vec<String> = PROCESS_FILES
            .iter()
            .map(|root| format!("Read(/{root}/**)"))
            .filter(|rule| !rules.contains(&rule.as_str()))
            .map(|rule| format!("review-run does not refuse its agents {rule}"))
            .collect();
        if !settings.contains(r#""blockReadsOutsideWorkingDirectories":true"#) {
            problems
                .push("review-run does not refuse its agents reads outside their trees".to_owned());
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

    /// Requires every agent job to keep the Claude token `token`
    /// from its agents (ENG-28): one step alone holds it, and that step
    /// only stages it with [`STAGE`]; no step that runs an agent, and no
    /// job-wide variable, holds it; every step that runs agents passes
    /// them the staged credential with `--credential`; and every upload
    /// waits for a step that ran [`SCRUB`] after the last of them.
    #[must_use]
    pub fn token_kept_from_agents(&self, token: &str) -> Vec<String> {
        let held = format!("secrets.{token}");
        let mut problems = Vec::new();
        for j in self.agent_jobs() {
            let steps = steps(&j.text);
            let in_steps: usize = steps.iter().map(|s| s.matches(&held).count()).sum();
            if j.text.matches(&held).count() > in_steps {
                problems.push(format!("agent job {} sets {token} for every step", j.name));
            }
            let holders: Vec<&String> = steps.iter().filter(|s| s.contains(&held)).collect();
            if holders.is_empty() {
                problems.push(format!(
                    "agent job {} stages no {token} with {STAGE:?}",
                    j.name
                ));
            }
            problems.extend(
                holders
                    .iter()
                    .filter(|s| !s.contains(STAGE) || runs_agent_step(s))
                    .map(|s| {
                        format!(
                            "agent job {} hands {token} to step {:?}, which does more than stage it",
                            j.name,
                            step_name(s)
                        )
                    }),
            );
            problems.extend(
                steps
                    .iter()
                    .filter(|s| runs_agent_step(s) && !s.contains("--credential"))
                    .map(|s| {
                        format!(
                            "agent job {} runs agents in step {:?} without --credential",
                            j.name,
                            step_name(s)
                        )
                    }),
            );
            problems.extend(unscrubbed_uploads(&steps).into_iter().map(|s| {
                format!(
                    "agent job {} uploads in step {:?} before a scrub of {token} after its agents gates it",
                    j.name, s
                )
            }));
            problems.extend(uploads_skipped_on_failure(&steps).into_iter().map(|s| {
                format!(
                    "agent job {} uploads in step {s:?} only when every step before it succeeded, so a failed run keeps nothing for its author: its if: needs always()",
                    j.name
                )
            }));
        }

        problems
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

mod gate;
mod scripts;

pub use gate::*;
pub use scripts::{inline_scripts, load};

#[cfg(test)]
mod tests;
