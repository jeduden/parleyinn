//! ENG-28: a review in which an agent's output held the Claude Code
//! OAuth token. Every text the review writes or prints is scrubbed of
//! the token first, and the review requests changes, names the leak in
//! a blocking finding and fails, so nothing it decided is posted. The
//! check stages a stand-in token as the workflow stages the real one,
//! and runs `review-run` on a work directory with a stand-in for the
//! `claude` command line whose every agent reads the staged credential,
//! as an agent whose read was not refused would, and states the token
//! in its text and its answer. It requires no file under `review/` and
//! no line printed to hold the token, and the review to fail closed.

use std::fs;
use std::path::Path;

use review_eval::credential;
use review_eval::host::{Command, Host, Output};
use review_eval::secret::REDACTED;

use crate::Checkout;
use crate::review_runs::agent_of;

/// The stand-in token the check stages; never a real one.
pub const TOKEN: &str = "dummy-oauth-token-0123456789-never-a-real-one";

/// Where under the work directory the check stages it, outside
/// `review/`.
pub const CREDENTIAL: &str = "credential";

/// How the blocking finding that names the leak starts.
pub const NAMED: &str = "an agent's output contained the Claude token: ";

/// A stand-in for the `claude` command line: each agent reads the
/// credential it was pointed at and states the token in its text and
/// its answer.
#[derive(Debug, Default)]
pub struct LeakHost;

impl Host for LeakHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        let dir = command
            .env
            .iter()
            .find(|(k, _)| k == credential::CONFIG_VAR)
            .map(|(_, v)| v.clone())
            .ok_or("the run was pointed at no credential")?;
        let token = credential::read(Path::new(&dir))?;
        let log = command
            .stdout_to
            .as_deref()
            .ok_or("the run logs to no file")?;
        let agent = agent_of(command);
        let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null, "message": {
            "id": "m", "model": "claude-haiku-5-5", "usage": {},
            "content": [{"type": "text", "text": format!("{agent} read {token}")}]}});
        let line = serde_json::json!({"type": "result", "subtype": "success",
            "is_error": false, "result": format!("outcome: none — {token}"),
            "total_cost_usd": 0.01, "num_turns": 2});
        fs::write(log, format!("{call}\n{line}\n"))
            .map_err(|e| format!("write {}: {e}", log.display()))?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// How `review-run` ended: its exit code and every line it printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    pub code: u8,
    pub printed: String,
}

/// Prepares `workdir` from `checkout`, stages [`TOKEN`] under it and
/// runs `review-run` there with the [`LeakHost`], handed the credential
/// as the workflow hands it.
///
/// # Errors
///
/// Fails naming what could not be prepared or staged.
pub fn run_leaking(checkout: &Checkout, workdir: &Path) -> Result<Ran, String> {
    crate::review_refusals::prepare(checkout, workdir)?;
    let cred = workdir.join(CREDENTIAL);
    credential::stage(&cred, TOKEN)?;
    let args = [
        "--workdir".to_owned(),
        workdir.display().to_string(),
        "--credential".to_owned(),
        cred.display().to_string(),
    ];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = review_run::cli::run(&LeakHost, args, &mut out, &mut err);
    out.extend(err);

    Ok(Ran {
        code,
        printed: String::from_utf8_lossy(&out).into_owned(),
    })
}

/// Every file under `dir`, at any depth, by its path under `root`, read.
fn files(root: &Path, dir: &Path) -> Vec<(String, String)> {
    let mut all = Vec::new();
    for path in fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
    {
        if path.is_dir() {
            all.extend(files(root, &path));
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            let text = String::from_utf8_lossy(&fs::read(&path).unwrap_or_default()).into_owned();
            all.push((rel, text));
        }
    }

    all
}

/// What is wrong with what a review that met the token left in
/// `workdir` and `printed`: a file under `review/` or a line that holds
/// it, and no file holding it redacted, so the leak was never scrubbed.
#[must_use]
pub fn scrub_problems(workdir: &Path, printed: &str) -> Vec<String> {
    let written = files(workdir, &workdir.join(review_run::REVIEW));
    let mut problems: Vec<String> = written
        .iter()
        .filter(|(_, text)| text.contains(TOKEN))
        .map(|(rel, _)| format!("review-run left the token in {rel}"))
        .collect();
    if printed.contains(TOKEN) {
        problems.push("review-run printed the token".to_owned());
    }
    if !written.iter().any(|(_, text)| text.contains(REDACTED)) {
        problems.push(
            "no file under review/ holds the token redacted, so the leak was never scrubbed"
                .to_owned(),
        );
    }

    problems
}

/// What is wrong with how a review that met the token in `workdir`
/// ended, by `ran`: an exit the workflow posts after, a verdict printed
/// for it, an outcome that does not request changes, and no blocking
/// finding naming the leak.
#[must_use]
pub fn fail_closed_problems(workdir: &Path, ran: &Ran) -> Vec<String> {
    let mut problems = Vec::new();
    if ran.code == review_run::cli::EXIT_OK {
        problems.push(
            "review-run exited 0 on a leak, so the workflow would post what it decided".to_owned(),
        );
    }
    if ran.printed.lines().any(|l| l.starts_with("verdict=")) {
        problems.push("review-run printed a verdict on a leak".to_owned());
    }
    let text = fs::read_to_string(workdir.join(review_run::OUTCOME)).unwrap_or_default();
    let Ok(outcome) = review_gate::parse_outcome(&text) else {
        problems.push("review-run wrote no review outcome that reads".to_owned());
        return problems;
    };
    if outcome.verdict != review_gate::REQUEST_CHANGES_VERDICT {
        problems.push(format!(
            "the review does not request changes on a leak: {}",
            outcome.verdict
        ));
    }
    if !outcome
        .findings
        .iter()
        .any(|f| f.severity == review_gate::BLOCKING && f.body.starts_with(NAMED))
    {
        problems.push("no blocking finding names the leak".to_owned());
    }

    problems
}

/// Runs the review with the [`LeakHost`] in `workdir`, prepared from
/// `checkout`, and returns what it left holding the token.
#[must_use]
pub fn review_run_scrubs_what_it_writes(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    match run_leaking(checkout, workdir) {
        Ok(ran) => scrub_problems(workdir, &ran.printed),
        Err(e) => vec![e],
    }
}

/// Runs the review with the [`LeakHost`] in `workdir`, prepared from
/// `checkout`, and returns what is wrong with how it ended.
#[must_use]
pub fn review_run_fails_closed_on_a_leak(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    match run_leaking(checkout, workdir) {
        Ok(ran) => fail_closed_problems(workdir, &ran),
        Err(e) => vec![e],
    }
}

#[cfg(test)]
mod tests;
