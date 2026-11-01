//! ENG-28: a specialist whose diff is past `review-run`'s split limit
//! runs in parts. The check runs `review-run` on a work directory whose
//! one specialist has a diff of three files no two of which fit one
//! part, with a stand-in for the `claude` command line that states, in
//! each part's run, a finding every part states and one of its own
//! part's. Run whole, the specialist must run once per part, each told
//! its own part's diff file, and its report must be the merge of its
//! parts'; run with one part cut off at its turn budget, the review must
//! fail closed naming that part, every part's findings kept.

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use review_eval::host::{Command, Host, Output};
use review_run::parts::LIMIT;

use crate::Checkout;
use crate::review_runs::{FINDING, agent_of};

/// The specialist the check splits.
pub const SPECIALIST: &str = "test-engineer-unit";
/// The generalist that weighs it.
pub const GENERALIST: &str = "review-code";
/// The files of its diff, in path order, one part each.
pub const FILES: [&str; 3] = ["src/x1.rs", "src/x2.rs", "src/x3.rs"];

/// The finding a part's run states on its own part, `i` from one.
#[must_use]
pub fn own_finding(i: usize) -> String {
    format!("src/part{i}.rs:1 — if not addressed: part {i} breaks — I9")
}

/// The part of its diff a run's `prompt` tells it it has, from one.
#[must_use]
pub fn part_of(prompt: &str) -> Option<usize> {
    let rest = prompt.split("you are part ").nth(1)?;
    rest.split(' ').next()?.parse().ok()
}

/// A stand-in for the `claude` command line: each part of the
/// specialist states [`FINDING`] and its own part's, then its outcome
/// line, except part `cut`, whose run ends at its turn budget; the
/// generalist keeps the finding as a nit. Every specialist prompt is
/// kept.
#[derive(Debug, Default)]
pub struct PartHost {
    pub cut: Option<usize>,
    pub prompts: Mutex<Vec<String>>,
}

impl Host for PartHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if let Some(out) = crate::review_runs::version(command) {
            return Ok(out);
        }
        let agent = agent_of(command);
        let prompt = command.args.get(1).cloned().unwrap_or_default();
        let (text, subtype, result) = if agent == GENERALIST {
            let answer = format!(
                "src/a.rs:1 — nit — a long line — {SPECIALIST}\n{agent} finds its scope sound."
            );
            (answer.clone(), "success", answer)
        } else {
            let part = part_of(&prompt).unwrap_or_default();
            if let Ok(mut prompts) = self.prompts.lock() {
                prompts.push(prompt);
            }
            let text = format!("{FINDING}\n{}", own_finding(part));
            if self.cut == Some(part) {
                (text, "error_max_turns", String::new())
            } else {
                (text, "success", format!("outcome: findings — part {part}."))
            }
        };
        let log = command
            .stdout_to
            .as_deref()
            .ok_or("the run logs to no file")?;
        let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null, "message": {
            "id": "m", "model": "claude-haiku-5-5", "usage": {},
            "content": [{"type": "text", "text": text}]}});
        let line = serde_json::json!({"type": "result", "subtype": subtype,
            "is_error": subtype != "success", "result": result, "total_cost_usd": 0.01,
            "num_turns": 2});
        fs::write(log, format!("{call}\n{line}\n"))
            .map_err(|e| format!("write {}: {e}", log.display()))?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// The specialist's diff: each of [`FILES`] just over half of
/// [`LIMIT`], so no two fit one part.
#[must_use]
pub fn big_diff() -> String {
    let line = format!("+{}\n", "x".repeat(99));
    FILES
        .iter()
        .map(|path| {
            let head = format!("diff --git a/{path} b/{path}\n");
            let lines = (LIMIT / 2 + 1).div_ceil(line.len());
            format!("{head}{}", line.repeat(lines))
        })
        .collect()
}

/// Lays out in `workdir` what the review job prepares for `review-run`,
/// with the checkout's definitions of the two agents and the
/// specialist's diff past the limit.
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
    write(&format!("review/diffs/{SPECIALIST}.diff"), &big_diff())?;
    for name in [SPECIALIST, GENERALIST] {
        let rel = format!(".claude/agents/{name}.md");
        write(&rel, &checkout.read(&rel)?)?;
    }

    Ok(())
}

/// Runs `review-run` on `workdir`, prepared from `checkout`, with part
/// `cut` cut off: the prompts its specialist's runs got, or why it
/// could not run.
///
/// # Errors
///
/// Fails when the work directory cannot be prepared or review-run
/// exits with a failure.
pub fn run(checkout: &Checkout, workdir: &Path, cut: Option<usize>) -> Result<Vec<String>, String> {
    prepare(checkout, workdir)?;
    let host = PartHost {
        cut,
        ..PartHost::default()
    };
    let mut out = Vec::new();
    let mut err = Vec::new();
    let args = ["--workdir".to_owned(), workdir.display().to_string()];
    let code = review_run::cli::run(&host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        let why = err.lines().last().unwrap_or_default();
        return Err(format!("review-run exited {code}: {why}"));
    }

    Ok(host.prompts.into_inner().unwrap_or_default())
}

/// What is wrong with a whole run's `prompts` and what it wrote in
/// `workdir`: a part count other than one per file, a part not told its
/// own diff file, a report that is not its parts' merged, and a
/// specialist that answered in every part but is not consulted.
#[must_use]
pub fn whole_problems(prompts: &[String], workdir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    if prompts.len() != FILES.len() {
        problems.push(format!(
            "{SPECIALIST} ran {} times, not once per part of its diff",
            prompts.len()
        ));
    }
    for i in 1..=FILES.len() {
        let file = format!("`review/diffs/{SPECIALIST}.{i}.diff`");
        if !prompts
            .iter()
            .any(|p| part_of(p) == Some(i) && p.contains(&file))
        {
            problems.push(format!("no run of {SPECIALIST} was told part {i}, {file}"));
        }
    }
    let read = |rel: &str| fs::read_to_string(workdir.join(rel)).unwrap_or_default();
    let report = read(&format!("review/reports/{SPECIALIST}.md"));
    let mut want: Vec<String> = vec![FINDING.to_owned()];
    want.extend((1..=FILES.len()).map(own_finding));
    let outcome = (1..=FILES.len())
        .map(|i| format!("part {i}, findings — part {i}."))
        .collect::<Vec<_>>()
        .join("; ");
    want.push(format!(
        "outcome: findings — in {} parts: {outcome}",
        FILES.len()
    ));
    if report != format!("{}\n", want.join("\n")) {
        problems.push(format!(
            "{SPECIALIST}'s report is not the merge of its parts' reports: {report:?}"
        ));
    }
    if !read("review/consulted.json").contains(&format!("\"{SPECIALIST}\"")) {
        problems.push(format!(
            "{SPECIALIST} answered in every part but is not consulted"
        ));
    }

    problems
}

/// What is wrong with what a run whose part 2 was cut off wrote in
/// `workdir`: a review that does not fail naming that part, and a
/// report that does not keep every part's findings under its
/// incomplete line.
#[must_use]
pub fn cut_problems(workdir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let read = |rel: &str| fs::read_to_string(workdir.join(rel)).unwrap_or_default();
    let named = format!(
        "{SPECIALIST} did not answer (part 2 of {}: its run ended with error_max_turns)",
        FILES.len()
    );
    let blocks = review_gate::parse_outcome(&read("review/outcome.json")).is_ok_and(|o| {
        o.verdict == review_gate::REQUEST_CHANGES_VERDICT
            && o.findings
                .iter()
                .any(|f| f.severity == review_gate::BLOCKING && f.body.starts_with(&named))
    });
    if !blocks {
        problems.push(format!(
            "a part of {SPECIALIST} that did not answer does not fail the review naming it"
        ));
    }
    let report = read(&format!("review/reports/{SPECIALIST}.md"));
    let keeps = report.starts_with("incomplete: part 2 of ")
        && (1..=FILES.len()).all(|i| report.contains(&own_finding(i)));
    if !keeps {
        problems.push(format!(
            "{SPECIALIST}'s report does not keep every part's findings under its incomplete \
             line: {report:?}"
        ));
    }

    problems
}

/// Runs `review-run` on two work directories under `workdir`, from
/// `checkout`: one whole, one with the specialist's part 2 cut off, and
/// returns what is wrong with what each ran and wrote.
#[must_use]
pub fn review_runs_in_parts(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let whole = workdir.join("whole");
    let cut = workdir.join("cut");
    let mut problems = match run(checkout, &whole, None) {
        Ok(prompts) => whole_problems(&prompts, &whole),
        Err(e) => vec![e],
    };
    match run(checkout, &cut, Some(2)) {
        Ok(_) => problems.extend(cut_problems(&cut)),
        Err(e) => problems.push(e),
    }

    problems
}

#[cfg(test)]
mod tests;
