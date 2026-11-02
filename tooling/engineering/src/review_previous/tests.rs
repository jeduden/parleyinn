use super::*;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

fn job() -> String {
    let flow = load(&repo(), WORKFLOW).unwrap();
    flow.jobs.into_iter().find(|j| j.name == JOB).unwrap().text
}

#[test]
fn the_checkout_s_review_carries_findings_from_main_s_review_workflow_only() {
    assert_eq!(review_carries_findings(&repo()), Ok(Vec::new()));
}

#[test]
fn a_checkout_without_the_workflow_is_named() {
    let dir = testkit::TempDir::new().unwrap();

    let e = review_carries_findings(&Checkout::new(dir.path())).unwrap_err();

    assert!(e.contains("review.yml"), "{e}");
}

#[test]
fn a_workflow_without_the_review_job_is_named() {
    let dir = testkit::TempDir::new().unwrap();
    let flows = dir.path().join(".github/workflows");
    std::fs::create_dir_all(&flows).unwrap();
    std::fs::write(
        flows.join("review.yml"),
        "on:\n  push:\njobs:\n  other:\n    steps: []\n",
    )
    .unwrap();

    let got = review_carries_findings(&Checkout::new(dir.path())).unwrap();

    assert_eq!(got, vec![format!("{WORKFLOW} has no job {JOB}")]);
}

#[test]
fn the_steps_of_a_job_split_at_their_dashes() {
    let text = "      - name: a\n        run: x\n      - name: b\n";
    assert_eq!(
        steps(text),
        vec!["      - name: a\n        run: x\n", "      - name: b\n"]
    );
    assert_eq!(
        steps("    steps:\n      - one\n      - two"),
        vec!["      - one\n", "      - two"]
    );
    assert!(steps("nothing").is_empty());
}

/// The review job with `from` replaced by `to`, which must be there.
fn mutated(from: &str, to: &str) -> String {
    let text = job();
    assert!(text.contains(from), "{from}");
    text.replacen(from, to, 1)
}

#[test]
fn a_job_that_cannot_read_artifacts_is_named() {
    let got = workflow_problems(&mutated("actions: read", "actions: none"));

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].contains("actions: read"), "{got:?}");
}

#[test]
fn a_fetch_that_never_picks_by_origin_is_named() {
    let got = workflow_problems(&mutated("previous select", "previous list"));

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].contains("previous select"), "{got:?}");
}

#[test]
fn a_fetch_that_does_not_check_the_file_or_look_for_the_pull_request_s_name_is_named() {
    let unchecked = workflow_problems(&mutated("previous check", "previous skip"));
    assert_eq!(unchecked.len(), 1, "{unchecked:?}");
    assert!(unchecked[0].contains("previous check"), "{unchecked:?}");

    let text = job();
    let nameless = workflow_problems(&text.replacen(
        "artifacts?name=review-findings-pr-",
        "artifacts?name=x-",
        1,
    ));
    assert!(
        nameless
            .iter()
            .any(|p| p.contains("pull request's own name")),
        "{nameless:?}"
    );
}

#[test]
fn findings_fetched_after_the_token_is_staged_are_named() {
    let text = job();
    let fetch = text.find("previous select").unwrap();
    let stage = "      - name: late stage\n        run: cargo run -p review-eval -- stage-credential --dir d\n";
    let late = format!("{text}{stage}");
    assert!(
        workflow_problems(&late).is_empty(),
        "stage after the fetch is fine"
    );
    let at = text[..fetch].rfind("\n      - ").unwrap() + 1;
    let early = format!("{}{stage}{}", &text[..at], &text[at..]);

    let got = workflow_problems(&early);

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(
        got[0].contains("after the Claude token is staged"),
        "{got:?}"
    );
}

#[test]
fn a_review_run_not_given_the_findings_is_named() {
    let got = workflow_problems(&mutated("--previous", "--prior"));

    assert!(got.iter().any(|p| p.contains("as --previous")), "{got:?}");
}

#[test]
fn findings_the_scrub_leaves_out_are_named() {
    let got = workflow_problems(&mutated(
        "--path review/findings.json",
        "--path review/x.json",
    ));

    assert!(got.iter().any(|p| p.contains("scrubbed")), "{got:?}");
}

#[test]
fn an_upload_that_is_not_per_pull_request_or_kept_short_is_named() {
    let anon = workflow_problems(&mutated(UPLOAD_NAME, "name: review-findings"));
    assert!(
        anon.iter()
            .any(|p| p.contains("one artifact name per pull request")),
        "{anon:?}"
    );

    let short = workflow_problems(&mutated("retention-days: 90", "retention-days: 1"));
    assert_eq!(short.len(), 1, "{short:?}");
    assert!(short[0].contains("retention-days: 90"), "{short:?}");

    let unscrubbed = workflow_problems(&mutated(
        "steps.review.outcome == 'success' && steps.scrub.outputs.scrubbed == 'true'",
        "steps.review.outcome == 'success'",
    ));
    assert!(
        unscrubbed.iter().any(|p| p.contains("scrubbed == 'true'")),
        "{unscrubbed:?}"
    );
}

#[test]
fn the_checkout_s_selection_trusts_main_s_review_workflow_alone() {
    assert_eq!(selection_problems(), Vec::<String>::new());
}
