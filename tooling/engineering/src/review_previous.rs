//! ENG-28: a review carries the findings of the previous review of its
//! pull request, and trusts them only from a run of main's review
//! workflow. The review job fetches them with `scripts/review-previous.sh`
//! before the Claude token is staged, passes `review-run` the file as
//! `--previous`, scrubs and uploads the findings it kept under the pull
//! request's own artifact name for 90 days; and `review-run previous
//! select`, the checkout's, takes the newest artifact of that name a
//! run of main's review workflow uploaded, never one a workflow the pull
//! request defines uploaded under the same name.

use review_run::previous::Selection;

use crate::Checkout;
use crate::review_workflow::load;

/// The workflow the check reads.
pub const WORKFLOW: &str = ".github/workflows/review.yml";
/// The job that runs the agents.
const JOB: &str = "review";
/// The artifact name a pull request's findings are kept under, as the
/// upload step writes it.
const UPLOAD_NAME: &str = "name: review-findings-pr-${{ steps.pull_head.outputs.number }}";

/// What is wrong with how the checkout's review carries findings from
/// one review of a pull request to the next.
///
/// # Errors
///
/// Fails when the workflow, or a script it runs, cannot be read.
pub fn review_carries_findings(checkout: &Checkout) -> Result<Vec<String>, String> {
    let flow = load(checkout, WORKFLOW)?;
    let Some(job) = flow.jobs.iter().find(|j| j.name == JOB) else {
        return Ok(vec![format!("{WORKFLOW} has no job {JOB}")]);
    };
    let mut problems = workflow_problems(&job.text);
    problems.extend(selection_problems());
    problems.extend(file_problems(&review_run::previous::check));

    Ok(problems)
}

/// A job's steps, each from its `- ` line at the steps' indent.
pub(crate) fn steps(text: &str) -> Vec<&str> {
    let mut starts: Vec<usize> = text
        .match_indices("\n      - ")
        .map(|(at, _)| at + 1)
        .collect();
    if text.starts_with("      - ") {
        starts.insert(0, 0);
    }
    starts
        .iter()
        .enumerate()
        .map(|(n, &from)| &text[from..starts.get(n + 1).copied().unwrap_or(text.len())])
        .collect()
}

/// What is wrong with the `step` that fetches the previous findings: it
/// must check the file it fetched, look for the pull request's own name,
/// cut the unzipped file at the size cap, say in the job summary what it
/// carried, and keep its warnings out of the runs' lines for `jq`.
fn fetch_problems(step: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (needle, why) in [
        (
            "previous check",
            "checks the file it fetched with review-run previous check",
        ),
        (
            "review-findings-pr-",
            "looks for the artifact of the pull request's own name",
        ),
        (
            "head -c 1048577",
            "cuts the unzipped file one byte past the size cap",
        ),
        (
            "GITHUB_STEP_SUMMARY",
            "says in the job summary which earlier review it carried",
        ),
    ] {
        if !step.contains(needle) {
            problems.push(format!(
                "the step that fetches the previous findings does not read it: it never {why}"
            ));
        }
    }
    // The runs' lines go to `jq -s`: a warning among them on stdout
    // is no JSON, and ends the step before it says anything.
    if step
        .lines()
        .any(|l| l.contains("::warning::") && !l.contains(">&2"))
    {
        problems.push(
            "a warning of the step that fetches the previous findings goes to stdout, into jq"
                .to_owned(),
        );
    }

    problems
}

/// What is wrong with the review job's text `job`, the scripts its steps
/// run inlined.
#[must_use]
pub fn workflow_problems(job: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !job.contains("actions: read") {
        problems.push(format!(
            "the {JOB} job cannot read the artifacts of earlier runs: its permissions lack actions: read"
        ));
    }
    let steps = steps(job);
    let at = |needle: &str| steps.iter().position(|s| s.contains(needle));
    // The spend is picked by the same command, `--of spend`: it is not
    // the findings' fetch.
    let fetch = steps
        .iter()
        .position(|s| s.contains("previous select") && !s.contains("--of spend"));
    match fetch {
        None => problems.push(
            "no step picks the previous review's findings with review-run previous select"
                .to_owned(),
        ),
        Some(fetch) => {
            problems.extend(fetch_problems(steps[fetch]));
            if at("stage-credential").is_some_and(|stage| stage < fetch) {
                problems.push(
                    "the previous findings are fetched after the Claude token is staged".to_owned(),
                );
            }
        }
    }
    if !steps
        .iter()
        .any(|s| s.contains("-p review-run -- --workdir") && s.contains("--previous"))
    {
        problems.push("review-run is not given the fetched findings as --previous".to_owned());
    }
    if !steps.iter().any(|s| {
        s.contains("review-eval -- scrub")
            && s.contains("--path review/findings.json")
            && s.contains("--path review/findings.md")
    }) {
        problems.push("the tracked findings are not scrubbed of the Claude token".to_owned());
    }
    let upload = steps
        .iter()
        .find(|s| s.contains("upload-artifact") && s.contains(UPLOAD_NAME));
    match upload {
        None => problems.push(format!(
            "no step uploads the tracked findings as {UPLOAD_NAME:?}, one artifact name per pull request"
        )),
        Some(step) => {
            for needle in [
                "review/findings.json",
                "retention-days: 90",
                "scrubbed == 'true'",
                "steps.scrub.outcome == 'success'",
            ] {
                if !step.contains(needle) {
                    problems.push(format!(
                        "the step that uploads the tracked findings lacks {needle:?}"
                    ));
                }
            }
        }
    }

    problems
}

/// What is wrong with the checkout's `review-run previous select`: it
/// must take the newest artifact a run of main's review workflow
/// uploaded, over a newer one another workflow or the pull request's
/// own definition uploaded, count each it refused, and take none when
/// none qualifies.
#[must_use]
pub fn selection_problems() -> Vec<String> {
    selection_problems_with(&review_run::previous::select)
}

/// `previous select`, or what stands in for it: the pull request, the
/// artifacts' listing and the runs' list in, the pick out.
type Select<'a> = &'a dyn Fn(&str, &str, &str) -> Result<Selection, String>;

/// [`selection_problems`] of `select`.
fn selection_problems_with(select: Select) -> Vec<String> {
    const NAME: &str = "review-findings-pr-7";
    let artifact = |id: u64, at: &str, run: u64| {
        format!(
            r#"{{"id":{id},"name":"{NAME}","expired":false,"created_at":"{at}","workflow_run":{{"id":{run}}}}}"#
        )
    };
    let run = |id: u64, path: &str, event: &str| {
        format!(r#"{{"id":{id},"path":"{path}","event":"{event}"}}"#)
    };
    let review = ".github/workflows/review.yml";
    let artifacts = format!(
        r#"{{"artifacts":[{},{},{}]}}"#,
        artifact(3, "2026-11-01T12:00:00Z", 30),
        artifact(2, "2026-11-01T11:00:00Z", 20),
        artifact(1, "2026-11-01T10:00:00Z", 10),
    );
    let runs = format!(
        "[{},{},{}]",
        run(30, ".github/workflows/ci.yml", "workflow_run"),
        run(20, review, "pull_request"),
        run(10, review, "workflow_run"),
    );
    let mut problems = Vec::new();
    let want = Selection {
        artifact: Some(1),
        skipped: 2,
    };
    match select("7", &artifacts, &runs) {
        Ok(got) if got == want => {}
        Ok(got) => problems.push(format!(
            "previous select took {got:?} over two newer artifacts of the name from other runs, not {want:?}"
        )),
        Err(e) => problems.push(format!("previous select failed: {e}")),
    }
    // An artifact whose run is in no list of runs has an origin no one
    // checked.
    let unknown = format!(
        r#"{{"artifacts":[{}]}}"#,
        artifact(9, "2026-11-01T13:00:00Z", 90)
    );
    match select("7", &unknown, "[]") {
        Ok(Selection {
            artifact: None,
            skipped: 1,
        }) => {}
        other => problems.push(format!(
            "previous select took or missed an artifact whose run is unknown: {other:?}, want none with 1 refused"
        )),
    }
    problems.extend(trigger_problems(select));
    let forged = format!(
        "[{},{},{}]",
        run(30, ".github/workflows/ci.yml", "workflow_run"),
        run(20, review, "pull_request"),
        run(10, review, "push"),
    );
    match select("7", &artifacts, &forged) {
        Ok(Selection {
            artifact: None,
            skipped: 3,
        }) => {}
        other => problems.push(format!(
            "previous select with no artifact from main's review workflow gave {other:?}, not none with 3 refused"
        )),
    }

    problems
}

/// What is wrong with `previous select` for one artifact: each trigger
/// that runs main's definition is trusted, and an expired artifact is
/// taken from no one.
fn trigger_problems(select: Select) -> Vec<String> {
    let run = |event: &str| {
        format!(r#"[{{"id":40,"path":".github/workflows/review.yml","event":"{event}"}}]"#)
    };
    let mut problems = Vec::new();
    let one = |event: &str, expired: bool| {
        let listing = format!(
            r#"{{"artifacts":[{{"id":4,"name":"review-findings-pr-7","expired":{expired},"created_at":"2026-11-01T12:00:00Z","workflow_run":{{"id":40}}}}]}}"#
        );
        select("7", &listing, &run(event))
    };
    for (event, expired, want) in [
        ("issue_comment", false, Some(4)),
        ("workflow_run", false, Some(4)),
        ("workflow_run", true, None),
    ] {
        match one(event, expired) {
            Ok(got) if got.artifact == want => {}
            other => problems.push(format!(
                "previous select of an artifact from a {event} run, expired {expired}, gave {other:?}, want {want:?}"
            )),
        }
    }

    problems
}

/// `previous check`, or what stands in for it: a findings file's text in,
/// whether it reads out.
type Check<'a> = &'a dyn Fn(&str) -> Result<(), String>;

/// What is wrong with `previous check`: it must take findings that read
/// and refuse a file past the size limit and one that does not read, each
/// by name, never as an empty history.
#[must_use]
pub fn file_problems(check: Check) -> Vec<String> {
    let mut problems = Vec::new();
    if let Err(e) = check("[]") {
        problems.push(format!("previous check refused findings that read: {e}"));
    }
    let big = format!("[{}]", " ".repeat(review_run::previous::MAX_BYTES));
    for (what, text) in [
        ("past the size limit", big.as_str()),
        ("that do not read", "{"),
    ] {
        if check(text).is_ok() {
            problems.push(format!("previous check took findings {what}"));
        }
    }

    problems
}

#[cfg(test)]
mod tests;
