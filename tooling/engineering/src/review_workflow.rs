//! ENG-28: the review workflow keeps the reviewing agent away from the
//! reviewer app's key, and the review gate that posts the review
//! decides as the requirement says.

use std::collections::BTreeMap;

use crate::list;

/// The action that runs the reviewing agent.
const AGENT_ACTION: &str = "anthropics/claude-code-action@";

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

    /// The jobs that run the reviewing agent.
    fn agent_jobs(&self) -> impl Iterator<Item = &Job> {
        self.jobs.iter().filter(|j| j.text.contains(AGENT_ACTION))
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
            problems.push("no job runs the reviewing agent".to_owned());
        }

        problems
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
        if j.text.contains(AGENT_ACTION) {
            return Err(format!("job {} holds {key} and runs the agent", j.name));
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
    ]
    .map(str::to_owned)
    .to_vec()
}

/// The dispatch a table row hands the gate by default: one generalist
/// with one specialist.
const DISPATCH: &str = r#"{"review-code":["lang-rust"]}"#;

/// What [`gate_outcome`] names for a row whose `dispatch` or `consulted`
/// cell holds a value it does not define, so a mistyped row fails
/// rather than reads as the default.
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

/// The agents a row says the orchestrator consulted: both of
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
/// names: a failed one from the reviewer app for `failed`, none
/// otherwise.
fn own_check(cell: &str) -> String {
    if cell == "failed" {
        format!(
            "{{\"name\":{:?},\"status\":\"completed\",\"conclusion\":\"failure\",\"app\":{}}}\n",
            review_gate::OWN_CHECK,
            review_gate::REVIEWER_APP_ID
        )
    } else {
        String::new()
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
/// It always gets `--dispatch` and `--consulted` (see [`dispatch`] and
/// [`consulted`]), and a row whose cell for either it does not define
/// gives [`UNKNOWN_CELL`].
#[must_use]
pub fn gate_outcome(row: &BTreeMap<String, String>) -> &'static str {
    const REVIEWED: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";
    const MOVED: &str = "490efa734617788d1dc1f831b26d0e23888e90fa";
    let (Some(dispatched), Some(consulted)) = (dispatch(row), consulted(row)) else {
        return UNKNOWN_CELL;
    };
    let cell = |name: &str| row.get(name).map(String::as_str).unwrap_or_default();
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
            own_check(cell("own check"))
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
