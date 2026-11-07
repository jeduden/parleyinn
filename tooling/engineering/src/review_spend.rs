//! ENG-28: a pull request's reviews share one cost cap. `review-run`
//! counts what the pull request's earlier reviews spent, handed in as
//! `--spent-before`, toward the cap, starts no run once that and its own
//! finished runs reach it, names each agent it did not start over budget
//! and keeps the total in `review/spend.json`. The review job fetches
//! that file with `scripts/review-spend.sh` before the Claude token is
//! staged, from the newest artifact under the pull request's own name a
//! run of main's review workflow uploaded, passes it to `review-run`,
//! scrubs the total it kept and uploads it for 90 days however the review
//! ended; `review-run previous spend` names a spend file past 1 KiB or
//! one that does not read, never taking it for nothing spent. The runs
//! use the stand-in `claude` of [`review_controls`], each run costing
//! 0.01 US dollars, and the cap `review-run` takes when none is given.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use review_run::controls::PULL_REQUEST_COST_USD;
use review_run::previous::Selection;

use crate::Checkout;
use crate::review_controls::{ControlHost, gaps, review};
use crate::review_previous::steps;
use crate::review_workflow::load;

/// The workflow the check reads.
pub const WORKFLOW: &str = ".github/workflows/review.yml";
/// The job that runs the agents.
const JOB: &str = "review";
/// How many agents the work directory dispatches.
const AGENTS: usize = 5;
/// What one run of the stand-in costs, in US dollars.
const RUN_COST: f64 = 0.01;
/// The artifact name the spend is kept under, as the upload step writes it.
const UPLOAD_NAME: &str = "name: review-spend-pr-${{ steps.pull_head.outputs.number }}";

/// What one review held to the pull request's cap did.
#[derive(Debug, Clone, PartialEq)]
pub struct Held<'a> {
    /// How many runs started.
    pub started: usize,
    /// What `review-run` printed.
    pub out: &'a str,
    /// Its gaps' reasons, by name.
    pub gaps: &'a BTreeMap<String, String>,
    /// What its `review/spend.json` says, or why it does not read.
    pub spend: Result<f64, String>,
}

/// What is wrong with review `case`, which `held` says what it did, when
/// `started` runs should have started and its spend file should say
/// `spent`: another count of runs, an agent not started that is not
/// named over budget or not counted, a review that does not request
/// changes, or a spend file that is missing what was spent.
#[must_use]
pub fn held_problems(case: &str, held: &Held, started: usize, spent: f64) -> Vec<String> {
    let mut problems = Vec::new();
    if held.started != started {
        problems.push(format!(
            "{case}: {} runs started, not {started}",
            held.started
        ));
    }
    let left = AGENTS - started;
    let over = held
        .gaps
        .values()
        .filter(|r| r.starts_with("over budget: "))
        .count();
    if over != left || !held.out.contains(&format!("\nover_budget={left}\n")) {
        problems.push(format!(
            "{case}: {over} agents named over budget, not the {left} not started: {}",
            held.out
        ));
    }
    if !held.out.starts_with("verdict=request_changes\n") {
        problems.push(format!(
            "{case}: a review its pull request's cap stopped does not request changes: {}",
            held.out
        ));
    }
    match &held.spend {
        Ok(got) if (got - spent).abs() < 1e-9 => {}
        Ok(got) => problems.push(format!(
            "{case}: review/spend.json says {got} USD, not the {spent} the earlier reviews and \
             this one spent"
        )),
        Err(e) => problems.push(format!("{case}: {e}")),
    }

    problems
}

/// What `review/spend.json` in `workdir` says was spent.
fn spend_of(workdir: &Path) -> Result<f64, String> {
    let path = workdir.join(review_run::spend::SPEND);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    review_run::spend::check(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// What is wrong with the pull request's cap: with the earlier reviews
/// a little under it, one run starts and nothing after it; with them at
/// it, no run starts at all. Each names the rest over budget, requests
/// changes and keeps what the earlier reviews and it spent.
#[must_use]
pub fn cap_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let under = PULL_REQUEST_COST_USD - RUN_COST / 2.0;
    let mut problems = Vec::new();
    for (case, before, started) in [
        ("a review a little under the cap", under, 1),
        ("a review at the cap", PULL_REQUEST_COST_USD, 0),
    ] {
        let host = ControlHost::default();
        let dir = workdir.join(started.to_string());
        let before_flag = before.to_string();
        let flags = ["--parallel", "1", "--spent-before", before_flag.as_str()];
        let checked = review(checkout, &dir, &host, &flags).and_then(|out| {
            let gaps = gaps(&dir)?;
            let held = Held {
                started: host.commands().len(),
                out: &out,
                gaps: &gaps,
                spend: spend_of(&dir),
            };
            #[allow(clippy::cast_precision_loss, reason = "a handful of runs")]
            let spent = before + RUN_COST * started as f64;
            Ok(held_problems(case, &held, started, spent))
        });
        match checked {
            Ok(found) => problems.extend(found),
            Err(e) => problems.push(format!("{case}: {e}")),
        }
    }

    problems
}

/// What is wrong with how the checkout's review carries what a pull
/// request's reviews spent from one review to the next.
///
/// # Errors
///
/// Fails when the workflow, or a script it runs, cannot be read.
pub fn carry_problems(checkout: &Checkout) -> Result<Vec<String>, String> {
    let flow = load(checkout, WORKFLOW)?;
    let Some(job) = flow.jobs.iter().find(|j| j.name == JOB) else {
        return Ok(vec![format!("{WORKFLOW} has no job {JOB}")]);
    };
    let mut problems = workflow_problems(&job.text);
    problems.extend(file_problems(&review_run::spend::check));
    problems.extend(selection_problems());

    Ok(problems)
}

/// What is wrong with the `step` that fetches what the earlier reviews
/// spent: it must pick by origin, read the file it fetched, look for
/// the pull request's own name, cut the file one byte past the size cap
/// and say in the job summary what it carried.
fn fetch_problems(step: &str) -> Vec<String> {
    [
        (
            "previous spend",
            "reads the file it fetched with review-run previous spend",
        ),
        (
            "review-spend-pr-",
            "looks for the artifact of the pull request's own name",
        ),
        (
            "head -c 1025",
            "cuts the unzipped file one byte past the size cap",
        ),
        (
            "GITHUB_STEP_SUMMARY",
            "says in the job summary what earlier reviews spent",
        ),
    ]
    .into_iter()
    .filter(|(needle, _)| !step.contains(needle))
    .map(|(_, why)| format!("the step that fetches what earlier reviews spent never {why}"))
    .collect()
}

/// What is wrong with the review job's text `job`, the scripts its steps
/// run inlined.
#[must_use]
pub fn workflow_problems(job: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let steps = steps(job);
    let at = |needle: &str| steps.iter().position(|s| s.contains(needle));
    // The command, not a comment that names it.
    match at("review-run -- previous select --of spend") {
        None => problems.push(
            "no step picks what earlier reviews spent with review-run previous select --of spend"
                .to_owned(),
        ),
        Some(fetch) => {
            problems.extend(fetch_problems(steps[fetch]));
            if at("stage-credential").is_some_and(|stage| stage < fetch) {
                problems.push(
                    "what earlier reviews spent is fetched after the Claude token is staged"
                        .to_owned(),
                );
            }
        }
    }
    if !steps
        .iter()
        .any(|s| s.contains("-p review-run -- --workdir") && s.contains("--spent-before \""))
    {
        problems.push(
            "review-run is not given what earlier reviews spent as --spent-before".to_owned(),
        );
    }
    if !steps
        .iter()
        .any(|s| s.contains("review-eval -- scrub") && s.contains("--path review/spend.json"))
    {
        problems.push("the spend file is not scrubbed of the Claude token".to_owned());
    }
    let upload = steps
        .iter()
        .find(|s| s.contains("upload-artifact") && s.contains(UPLOAD_NAME));
    match upload {
        None => problems.push(format!(
            "no step uploads what the pull request's reviews spent as {UPLOAD_NAME:?}"
        )),
        Some(step) => {
            for needle in [
                "if: always() && ",
                "scrubbed == 'true'",
                "path: review/spend.json",
                "retention-days: 90",
            ] {
                if !step.contains(needle) {
                    problems.push(format!(
                        "the step that uploads what the pull request's reviews spent lacks {needle:?}"
                    ));
                }
            }
        }
    }

    problems
}

/// `previous spend`'s reading, or what stands in for it: a spend file's
/// text in, what it says was spent out.
type Check<'a> = &'a dyn Fn(&str) -> Result<f64, String>;

/// What is wrong with the reading of a spend file: it must take one
/// that reads and refuse one past the size limit and one that does not
/// read, each by name, never as nothing spent.
#[must_use]
pub fn file_problems(check: Check) -> Vec<String> {
    let mut problems = Vec::new();
    match check(&review_run::spend::text(1.5)) {
        Ok(spent) if (spent - 1.5).abs() < 1e-9 => {}
        other => problems.push(format!(
            "previous spend read a spend file of 1.5 USD as {other:?}"
        )),
    }
    let big = format!(
        "{{\"spent_usd\": 1{}}}",
        " ".repeat(review_run::spend::MAX_BYTES)
    );
    for (what, text) in [
        ("past the size limit", big.as_str()),
        ("that does not read", "{\"spent_usd\": "),
        ("of no amount", "{\"spent_usd\": -1}"),
    ] {
        if let Ok(spent) = check(text) {
            problems.push(format!(
                "previous spend took a spend file {what} for {spent} USD"
            ));
        }
    }

    problems
}

/// What is wrong with `previous select --of spend`: it must take the
/// newest artifact of the spend's name a run of main's review workflow
/// uploaded, refusing and counting a newer one from the pull request's
/// own event, and never the findings' artifact.
#[must_use]
pub fn selection_problems() -> Vec<String> {
    selection_problems_with(&review_run::previous::select_of)
}

/// `previous select`, or what stands in for it: the name's start, the
/// pull request, the artifacts' listing and the runs' list in, the pick
/// out.
type Select<'a> = &'a dyn Fn(&str, &str, &str, &str) -> Result<Selection, String>;

/// [`selection_problems`] of `select`.
fn selection_problems_with(select: Select) -> Vec<String> {
    let artifact = |id: u64, name: &str, at: &str, run: u64| {
        format!(
            r#"{{"id":{id},"name":"{name}","expired":false,"created_at":"{at}","workflow_run":{{"id":{run}}}}}"#
        )
    };
    let artifacts = format!(
        r#"{{"artifacts":[{},{},{}]}}"#,
        artifact(3, "review-findings-pr-7", "2026-11-01T12:00:00Z", 10),
        artifact(2, "review-spend-pr-7", "2026-11-01T11:00:00Z", 20),
        artifact(1, "review-spend-pr-7", "2026-11-01T10:00:00Z", 10),
    );
    let runs = r#"[{"id":20,"path":".github/workflows/review.yml","event":"pull_request"},
        {"id":10,"path":".github/workflows/review.yml","event":"workflow_run"}]"#;
    let picked = select(review_run::previous::SPEND, "7", &artifacts, runs);
    match picked {
        Ok(Selection {
            artifact: Some(1),
            skipped: 1,
        }) => Vec::new(),
        other => vec![format!(
            "previous select --of spend gave {other:?}, not artifact 1 with 1 refused"
        )],
    }
}

#[cfg(test)]
mod tests;
