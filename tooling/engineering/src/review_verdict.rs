//! ENG-28: a stakeholder's comment on a head the agents already reviewed
//! pays for no second review. The review job keeps each review's verdict
//! with what it was decided on, the run id, gaps, dispatch, outcome and
//! consulted agents, among its reports; after a comment,
//! `scripts/review-verdict.sh` looks through the runs behind the spend
//! `review-run previous select --of spend` trusted, runs of main's
//! review workflow alone, and `review-run previous verdict`, the
//! checkout's, reuses a verdict only for the same head under the same
//! dispatch from a review that left no gap. Reused, no agent runs; a
//! verdict that cannot be found or weighed is never an error, so the
//! agents run. A comment's review waits for a running one and cancels
//! none.

use review_run::verdict::{Kept, Verdict};

use crate::Checkout;
use crate::review_previous::steps;
use crate::review_workflow::load;

/// The workflow the check reads.
pub const WORKFLOW: &str = ".github/workflows/review.yml";
/// The job that runs the agents.
const JOB: &str = "review";
/// What a review keeps for a comment's review of the same head.
const KEPT: [&str; 5] = [
    "review/outcome.json",
    "review/consulted.json",
    "review/run-id.json",
    "review/gaps.json",
    "review/dispatch.json",
];
/// The condition that keeps the agents from running on a reused verdict.
const UNLESS_REUSED: &str = "steps.verdict.outputs.reused != 'true'";
/// A comment's review waits; only a CI run's cancels an older review.
const QUEUES: &str = "cancel-in-progress: ${{ github.event_name == 'workflow_run' }}";

/// What is wrong with how the checkout's review reuses the agents'
/// verdict after a comment.
///
/// # Errors
///
/// Fails when the workflow, or a script it runs, cannot be read.
pub fn review_reuses_a_verdict(checkout: &Checkout) -> Result<Vec<String>, String> {
    let text = checkout.read(WORKFLOW)?;
    let flow = load(checkout, WORKFLOW)?;
    let Some(job) = flow.jobs.iter().find(|j| j.name == JOB) else {
        return Ok(vec![format!("{WORKFLOW} has no job {JOB}")]);
    };
    let mut problems = queue_problems(&text);
    problems.extend(workflow_problems(&job.text));
    problems.extend(weighing_problems(&review_run::verdict::weigh_kept));

    Ok(problems)
}

/// What is wrong with the workflow's `text`: a comment's review must
/// wait for a running review of its pull request, not cancel it.
#[must_use]
pub fn queue_problems(text: &str) -> Vec<String> {
    if text.contains(QUEUES) {
        Vec::new()
    } else {
        vec!["a comment's review cancels a running review of its pull request".to_owned()]
    }
}

/// What is wrong with the `step` that looks for an earlier verdict.
fn find_problems(step: &str) -> Vec<String> {
    let mut problems: Vec<String> = [
        (
            "[ \"$EVENT\" = issue_comment ] || decided false",
            "looks only after a comment",
        ),
        (
            "spend/trusted-runs.txt",
            "looks only in the runs behind the spend it trusts",
        ),
        (
            "--head \"$HEAD_SHA\"",
            "weighs the earlier review against this head",
        ),
        (
            "--dispatch review/dispatch.json",
            "weighs the earlier review against this dispatch",
        ),
        (
            "GITHUB_STEP_SUMMARY",
            "says in the job summary whether a verdict is reused",
        ),
    ]
    .into_iter()
    .filter(|(needle, _)| !step.contains(needle))
    .map(|(_, why)| format!("the step that finds an earlier verdict never {why}"))
    .collect();
    if step.lines().any(|l| l.trim() == "exit 1") {
        problems.push("the step that finds an earlier verdict fails the review".to_owned());
    }

    problems
}

/// What is wrong with the review job's text `job`, the scripts its steps
/// run inlined.
#[must_use]
pub fn workflow_problems(job: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let steps = steps(job);
    let at = |needle: &str| steps.iter().position(|s| s.contains(needle));
    match at("review-run -- previous verdict") {
        None => problems.push(
            "no step weighs an earlier review's verdict with review-run previous verdict"
                .to_owned(),
        ),
        Some(find) => {
            problems.extend(find_problems(steps[find]));
            if at("stage-credential").is_some_and(|stage| stage < find) {
                problems.push(
                    "an earlier verdict is fetched after the Claude token is staged".to_owned(),
                );
            }
        }
    }
    if !steps
        .iter()
        .any(|s| s.contains("-p review-run -- --workdir") && s.contains(UNLESS_REUSED))
    {
        problems.push("the agents run although an earlier verdict is reused".to_owned());
    }
    let upload = steps
        .iter()
        .find(|s| s.contains("upload-artifact") && s.contains("name: review-reports"));
    match upload {
        None => problems.push("no step uploads the reports as review-reports".to_owned()),
        Some(step) => problems.extend(
            KEPT.iter()
                .filter(|kept| !step.contains(*kept))
                .map(|kept| format!("a review does not keep {kept} for a comment's review")),
        ),
    }

    problems
}

/// `previous verdict`'s weighing, or what stands in for it: what an
/// earlier review kept, this review's head and dispatch in, the verdict
/// out.
type Weigh<'a> = &'a dyn Fn(&Kept, &str, &str) -> Verdict;

/// The head of the earlier review the weighing is tried on.
const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
/// A head the earlier review was not of.
const MOVED: &str = "89abcdef0123456789abcdef0123456789abcdef";
/// The dispatch the earlier review ran under.
const DISPATCH: &str = r#"{"review-code": ["lang-rust"]}"#;

/// What an earlier review of [`HEAD`] under [`DISPATCH`] kept, no gap
/// among it.
fn kept() -> Kept {
    Kept {
        run_id: format!(r#"{{"head":"{HEAD}","inputs":"i","split":1,"agents":{{}}}}"#),
        gaps: "[]".to_owned(),
        dispatch: DISPATCH.to_owned(),
        outcome: r#"{"verdict":"approve","summary":"fine","findings":[]}"#.to_owned(),
        consulted: r#"["lang-rust","review-code"]"#.to_owned(),
    }
}

/// What is wrong with `previous verdict`: it must reuse a verdict for
/// the same head under the same dispatch and no other: not for a head
/// that moved, a dispatch that changed, a review that left a gap, or an
/// outcome that does not read.
#[must_use]
pub fn weighing_problems(weigh: Weigh) -> Vec<String> {
    let mut problems = Vec::new();
    if weigh(&kept(), HEAD, DISPATCH) != Verdict::Reused {
        problems.push(
            "previous verdict did not reuse a verdict for the same head and dispatch".to_owned(),
        );
    }
    let gap = Kept {
        gaps: r#"[{"name":"lang-rust","kind":"specialist","reason":"silent"}]"#.to_owned(),
        ..kept()
    };
    let unread = Kept {
        outcome: "{".to_owned(),
        ..kept()
    };
    for (what, kept, head, dispatch) in [
        ("for a head that moved", kept(), MOVED, DISPATCH),
        (
            "under a dispatch that changed",
            kept(),
            HEAD,
            r#"{"review-code": []}"#,
        ),
        ("of a review that left a gap", gap, HEAD, DISPATCH),
        ("whose outcome does not read", unread, HEAD, DISPATCH),
    ] {
        if weigh(&kept, head, dispatch) == Verdict::Reused {
            problems.push(format!("previous verdict reused a verdict {what}"));
        }
    }

    problems
}

#[cfg(test)]
mod tests;
