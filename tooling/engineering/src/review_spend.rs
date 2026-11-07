//! ENG-28: a pull request's reviews share one cost cap. `review-run`
//! counts what the pull request's earlier reviews spent, handed in as
//! `--spent-before`, toward the cap, starts a run only once its tier's
//! cap fits beside that, its own finished runs and the caps of its runs
//! in flight, names each agent it did not start over budget
//! and keeps what it alone spent in `review/spend.json`. The review job
//! fetches those files with `scripts/review-spend.sh` before the Claude
//! token is staged, summing every artifact under the pull request's own name a
//! run of main's review workflow uploaded, but its own run's, passes the
//! sum to `review-run`, which keeps what this review alone spent, scrubs
//! it and uploads it for 90 days however the review ended, replacing
//! only its own run's; a run behind an artifact that cannot be looked
//! up fails the step; `review-run previous spend` names a spend file
//! past 1 KiB or one that does not read, never taking it for nothing
//! spent. The runs
//! use the stand-in `claude` of [`review_controls`], each run costing
//! 0.01 US dollars, and the cap `review-run` takes when none is given.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use review_eval::runner::SPECIALIST_BUDGET_USD;
use review_run::controls::PULL_REQUEST_COST_USD;
use review_run::previous::Spends;

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
/// The specialist cut off unbilled, and the log of its run: model calls
/// that used tokens and no result, so no cost reported.
const CUT_OFF: &str = "lang-markdown";
const UNBILLED: &str = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"output_tokens":5},"content":[{"type":"text","text":"reading"}]}}"#;
/// How long each run of the stand-in takes, so runs started together
/// overlap.
const HOLD: std::time::Duration = std::time::Duration::from_millis(30);
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
    /// The most runs that ran at once.
    pub at_once: usize,
}

/// What review `case` should have done: how many runs start, the most
/// that run at once, and what its spend file says.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Want {
    pub started: usize,
    pub at_once: usize,
    pub spent: f64,
}

/// What is wrong with review `case`, which `held` says what it did,
/// against what it should have done, `want`: another count of runs,
/// more runs at once than their tier caps fit beside what was spent, an
/// agent not started that is not named over budget or not counted, a
/// review that does not request changes, or a spend file that is
/// missing what was spent.
#[must_use]
pub fn held_problems(case: &str, held: &Held, want: Want) -> Vec<String> {
    let Want {
        started,
        at_once,
        spent,
    } = want;
    let mut problems = Vec::new();
    if held.started != started {
        problems.push(format!(
            "{case}: {} runs started, not {started}",
            held.started
        ));
    }
    if held.at_once > at_once {
        problems.push(format!(
            "{case}: {} runs ran at once, past the {at_once} whose tier caps fit in the cap",
            held.at_once
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
            "{case}: review/spend.json says {got} USD, not the {spent} this review spent"
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
/// leaving room for one specialist's tier cap, one run starts and
/// nothing after it; with them at it, no run starts at all; with room
/// for two specialists' caps and eight runs allowed at once, never more
/// than two run at once, the third waits for them and no generalist's
/// cap fits; and with a specialist killed after its model calls, with
/// no cost reported, every run starts and the killed one is charged its
/// tier's cap. Each names the rest over budget, requests changes and
/// keeps what it alone spent.
#[must_use]
pub fn cap_problems(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let one = PULL_REQUEST_COST_USD - SPECIALIST_BUDGET_USD - RUN_COST / 2.0;
    let two = PULL_REQUEST_COST_USD - 2.0 * SPECIALIST_BUDGET_USD;
    let mut problems = Vec::new();
    for (case, before, parallel, started, at_once, cut) in [
        ("a review with room for one run", one, "1", 1, 1, false),
        (
            "a review at the cap",
            PULL_REQUEST_COST_USD,
            "1",
            0,
            0,
            false,
        ),
        (
            "a review with room for two runs at once",
            two,
            "8",
            3,
            2,
            false,
        ),
        (
            "a review with a run cut off unbilled",
            0.0,
            "1",
            AGENTS,
            1,
            true,
        ),
    ] {
        let mut host = ControlHost::holding(HOLD);
        if cut {
            host.fails.insert(CUT_OFF, (1, Some(UNBILLED)));
        }
        let dir = workdir.join(started.to_string());
        let before_flag = before.to_string();
        let flags = [
            "--parallel",
            parallel,
            "--spent-before",
            before_flag.as_str(),
        ];
        let checked = review(checkout, &dir, &host, &flags).and_then(|out| {
            let gaps = gaps(&dir)?;
            let held = Held {
                started: host.commands().len(),
                out: &out,
                gaps: &gaps,
                spend: spend_of(&dir),
                at_once: host.at_once(),
            };
            let billed = usize::from(cut);
            #[allow(clippy::cast_precision_loss, reason = "a handful of runs")]
            let spent =
                RUN_COST * (started - billed) as f64 + SPECIALIST_BUDGET_USD * billed as f64;
            let want = Want {
                started,
                at_once,
                spent,
            };
            Ok(held_problems(case, &held, want))
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
    problems.extend(sum_problems(&review_run::previous::summed));
    problems.extend(selection_problems());

    Ok(problems)
}

/// What is wrong with the `step` that fetches what the earlier reviews
/// spent: it must pick by origin, leave out the run's own, read and sum
/// the files it fetched, look for the pull request's own name, cut each
/// file one byte past the size cap, say in the job summary what it
/// carried, and fail on a run it cannot look up.
fn fetch_problems(step: &str) -> Vec<String> {
    let mut problems: Vec<String> = [
        (
            "previous spend \"${files[@]}\"",
            "reads and sums the file of every earlier review with review-run previous spend",
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
        (
            "--except-run \"$GITHUB_RUN_ID\"",
            "leaves out the spend its own run kept",
        ),
    ]
    .into_iter()
    .filter(|(needle, _)| !step.contains(needle))
    .map(|(_, why)| format!("the step that fetches what earlier reviews spent never {why}"))
    .collect();
    if !lookup_fails(step) {
        problems.push(
            "the step that fetches what earlier reviews spent goes on past a run it could not \
             look up"
                .to_owned(),
        );
    }

    problems
}

/// Whether `step` fails when the run behind a spend artifact cannot be
/// looked up: the line that looks it up, `actions/runs/$id`, is followed
/// by `|| fail`, outside any pipe whose subshell would swallow the
/// failure, and nothing only warns.
fn lookup_fails(step: &str) -> bool {
    let lines: Vec<&str> = step.lines().map(str::trim).collect();
    let fails = lines
        .iter()
        .position(|l| l.contains("actions/runs/$id"))
        .and_then(|at| lines.get(at + 1))
        .is_some_and(|next| next.starts_with("|| fail "));
    fails && !lines.iter().any(|l| l.starts_with("done |")) && !step.contains("::warning::")
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
                "overwrite: true",
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

/// `previous spend`'s sum, or what stands in for it: each spend file's
/// path and text in, what they say was spent out.
type Sum<'a> = &'a dyn Fn(&[(String, String)]) -> Result<f64, String>;

/// What is wrong with the sum of what earlier reviews spent: it must add
/// up every file, and refuse a sum one of whose files does not read,
/// naming that file.
#[must_use]
pub fn sum_problems(sum: Sum) -> Vec<String> {
    let file = |path: &str, text: String| (path.to_owned(), text);
    let two = [
        file("a", review_run::spend::text(1.5)),
        file("b", review_run::spend::text(0.25)),
    ];
    let mut problems = Vec::new();
    match sum(&two) {
        Ok(spent) if (spent - 1.75).abs() < 1e-9 => {}
        other => problems.push(format!(
            "previous spend summed spend files of 1.5 and 0.25 USD as {other:?}"
        )),
    }
    let broken = [two[0].clone(), file("b", "{".to_owned())];
    match sum(&broken) {
        Err(e) if e.starts_with("b: ") => {}
        other => problems.push(format!(
            "previous spend summed a spend file that does not read as {other:?}, not naming it"
        )),
    }

    problems
}

/// What is wrong with `previous select --of spend`: it must take every
/// artifact of the spend's name a run of main's review workflow
/// uploaded, but the review's own run's, refusing and counting one from
/// the pull request's own event, and never the findings' artifact.
#[must_use]
pub fn selection_problems() -> Vec<String> {
    selection_problems_with(&review_run::previous::spends)
}

/// `previous select --of spend`, or what stands in for it: the pull
/// request, the artifacts' listing, the runs' list and the run to leave
/// out in, the picks out.
type Select<'a> = &'a dyn Fn(&str, &str, &str, Option<u64>) -> Result<Spends, String>;

/// [`selection_problems`] of `select`.
fn selection_problems_with(select: Select) -> Vec<String> {
    let artifact = |id: u64, name: &str, at: &str, run: u64| {
        format!(
            r#"{{"id":{id},"name":"{name}","expired":false,"created_at":"{at}","workflow_run":{{"id":{run}}}}}"#
        )
    };
    let artifacts = format!(
        r#"{{"artifacts":[{},{},{},{},{}]}}"#,
        artifact(3, "review-findings-pr-7", "2026-11-01T12:00:00Z", 10),
        artifact(2, "review-spend-pr-7", "2026-11-01T11:00:00Z", 20),
        artifact(1, "review-spend-pr-7", "2026-11-01T10:00:00Z", 10),
        artifact(4, "review-spend-pr-7", "2026-11-01T09:00:00Z", 30),
        artifact(5, "review-spend-pr-7", "2026-11-01T13:00:00Z", 40),
    );
    let runs = r#"[{"id":20,"path":".github/workflows/review.yml","event":"pull_request"},
        {"id":10,"path":".github/workflows/review.yml","event":"workflow_run"},
        {"id":30,"path":".github/workflows/review.yml","event":"issue_comment"},
        {"id":40,"path":".github/workflows/review.yml","event":"workflow_run"}]"#;
    // Run 40 is the review's own.
    match select("7", &artifacts, runs, Some(40)) {
        Ok(Spends {
            artifacts,
            skipped: 1,
        }) if artifacts == [1, 4] => Vec::new(),
        other => vec![format!(
            "previous select --of spend gave {other:?}, not artifacts 1 and 4 with 1 refused"
        )],
    }
}

#[cfg(test)]
mod tests;
