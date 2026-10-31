//! ENG-28: what the review's runner names instead of passing by. The
//! gate holds no tree, so it reads a cited id by its shape alone; the
//! runner, which reads the SRS, names a specialist's report that cites
//! an id no requirement or invariant has in a blocking finding. A
//! generalist's finding line the runner cannot read is named the same
//! way, never dropped as prose. A refused report first runs its
//! specialist once more, told the refusal and the forms, and is judged
//! by that run alone. The check runs `review-run` on a work directory
//! with a stand-in for the `claude` command line whose specialist cites
//! such an id on every run and whose generalist answers such a line, and
//! requires the specialist run twice, its second run told why, and a
//! review that requests changes naming both.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use review_eval::host::{Command, Host, Output};

use crate::Checkout;
use crate::review_runs::agent_of;

/// The specialist the check runs.
pub const SPECIALIST: &str = "lang-rust";
/// The generalist that weighs it.
pub const GENERALIST: &str = "review-code";
/// The id the specialist cites, which no requirement or invariant has.
pub const UNKNOWN_ID: &str = "ENG-999";
/// The finding line the generalist answers, its place unreadable.
pub const UNREAD: &str = "src/a.rs:x — blocking — an unwrap — lang-rust";

/// A stand-in for the `claude` command line: the specialist states a
/// finding citing [`UNKNOWN_ID`] and answers its outcome line on every
/// run, each run's prompt kept; the generalist answers [`UNREAD`] and a
/// closing sentence.
#[derive(Debug, Default)]
pub struct RefusalHost {
    prompts: Mutex<Vec<String>>,
}

impl RefusalHost {
    /// The prompt of each specialist run, in the order they started.
    #[must_use]
    pub fn prompts(&self) -> Vec<String> {
        self.prompts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Host for RefusalHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        let agent = agent_of(command);
        if agent == SPECIALIST {
            let prompt = command.args.get(1).cloned().unwrap_or_default();
            self.prompts
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(prompt);
        }
        let (text, result) = if agent == GENERALIST {
            let answer = format!("{UNREAD}\n{agent} finds its scope sound.");
            (answer.clone(), answer)
        } else {
            (
                format!("src/a.rs:1 — if not addressed: a hook panics — {UNKNOWN_ID}"),
                "outcome: findings — one finding.".to_owned(),
            )
        };
        let log = command
            .stdout_to
            .as_deref()
            .ok_or("the run logs to no file")?;
        let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null, "message": {
            "id": "m", "model": "claude-haiku-5-5", "usage": {},
            "content": [{"type": "text", "text": text}]}});
        let line = serde_json::json!({"type": "result", "subtype": "success",
            "is_error": false, "result": result, "total_cost_usd": 0.01, "num_turns": 2});
        fs::write(log, format!("{call}\n{line}\n"))
            .map_err(|e| format!("write {}: {e}", log.display()))?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// Lays out in `workdir` what the review job prepares for `review-run`,
/// with the checkout's definitions of the two agents.
///
/// # Errors
///
/// Fails naming the file that cannot be read or written.
pub fn prepare(checkout: &Checkout, workdir: &Path) -> Result<(), String> {
    let write = |rel: &str, text: &str| {
        let path = workdir.join(rel);
        let dir = path.parent().unwrap_or(workdir);
        fs::create_dir_all(dir).map_err(|e| format!("make {}: {e}", dir.display()))?;
        fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
    };
    write(
        "review/dispatch.json",
        &format!(r#"{{"{GENERALIST}": ["{SPECIALIST}"]}}"#),
    )?;
    write(
        "class.txt",
        "class=standard\ntouched=1\nfindings=0\nconsults=2\n",
    )?;
    for name in [SPECIALIST, GENERALIST] {
        let rel = format!(".claude/agents/{name}.md");
        write(&rel, &checkout.read(&rel)?)?;
    }

    Ok(())
}

/// What is wrong with the outcome `review-run` wrote in `workdir`: a
/// review that does not request changes, and each of the two not named
/// in a blocking finding.
#[must_use]
pub fn outcome_problems(workdir: &Path) -> Vec<String> {
    let text = fs::read_to_string(workdir.join("review/outcome.json")).unwrap_or_default();
    let Ok(outcome) = review_gate::parse_outcome(&text) else {
        return vec!["review-run wrote no review outcome that reads".to_owned()];
    };
    let mut problems = Vec::new();
    if outcome.verdict != review_gate::REQUEST_CHANGES_VERDICT {
        problems.push(format!(
            "the review does not request changes: {}",
            outcome.verdict
        ));
    }
    let named = |path: &str, words: &str| {
        outcome.findings.iter().any(|f| {
            f.severity == review_gate::BLOCKING && f.path == path && f.body.contains(words)
        })
    };
    if !named(
        &format!("review/reports/{SPECIALIST}.md"),
        &format!("cites {UNKNOWN_ID}"),
    ) {
        problems.push(format!(
            "no blocking finding names {SPECIALIST}'s report citing {UNKNOWN_ID}"
        ));
    }
    if !named("review/", UNREAD) {
        problems.push(format!(
            "no blocking finding names {GENERALIST}'s finding line it cannot read"
        ));
    }

    problems
}

/// What is wrong with the specialist's runs, by `prompts`, each one's
/// prompt: not run exactly once more, or its second run not told that
/// its report cites [`UNKNOWN_ID`] and the forms again.
#[must_use]
pub fn retry_problems(prompts: &[String]) -> Vec<String> {
    let [_, second] = prompts else {
        return vec![format!(
            "review-run ran {SPECIALIST} {} time{}, not once more after its report was refused",
            prompts.len(),
            if prompts.len() == 1 { "" } else { "s" }
        )];
    };
    let mut problems = Vec::new();
    if !second.contains(&format!("cites {UNKNOWN_ID}")) {
        problems.push(format!(
            "{SPECIALIST}'s second run was not told its report cites {UNKNOWN_ID}"
        ));
    }
    if !second.contains(review_run::retry::FORMS) {
        problems.push(format!(
            "{SPECIALIST}'s second run was not told the forms again"
        ));
    }

    problems
}

/// Runs `review-run` on `workdir`, prepared from `checkout`, with the
/// [`RefusalHost`] standing in for the `claude` command line, and
/// returns what is wrong with the review it decided and the runs it
/// started.
#[must_use]
pub fn review_run_names_what_it_refuses(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    if let Err(e) = prepare(checkout, workdir) {
        return vec![e];
    }
    let mut out = Vec::new();
    let mut err = Vec::new();
    let args = ["--workdir".to_owned(), workdir.display().to_string()];
    let host = RefusalHost::default();
    let code = review_run::cli::run(&host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        let why = err.lines().last().unwrap_or_default();
        return vec![format!("review-run exited {code}: {why}")];
    }

    let mut problems = outcome_problems(workdir);
    problems.extend(retry_problems(&host.prompts()));

    problems
}

#[cfg(test)]
mod tests;
