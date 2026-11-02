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

#[test]
fn a_fetch_without_the_size_cap_or_the_summary_line_is_named() {
    let uncapped = workflow_problems(&mutated("head -c 1048577", "head -c 99"));
    assert_eq!(uncapped.len(), 1, "{uncapped:?}");
    assert!(uncapped[0].contains("size cap"), "{uncapped:?}");

    let silent = workflow_problems(&mutated(
        "echo \"$note\" >> \"$GITHUB_STEP_SUMMARY\"",
        "echo \"$note\" >> /dev/null",
    ));
    assert_eq!(silent.len(), 1, "{silent:?}");
    assert!(silent[0].contains("job summary"), "{silent:?}");
}

/// A `select` that gives `got` whatever it is asked.
fn gives(got: Result<Selection, String>) -> impl Fn(&str, &str, &str) -> Result<Selection, String> {
    move |_, _, _| got.clone()
}

#[test]
fn a_selection_that_takes_the_wrong_artifact_is_named_at_every_check() {
    let wrong = gives(Ok(Selection {
        artifact: Some(2),
        skipped: 0,
    }));

    let problems = selection_problems_with(&wrong);

    // The newest-trusted pick, the unknown run, the issue_comment and
    // expired cases, and the forged runs each say what they got.
    assert!(
        problems
            .iter()
            .any(|p| p.contains("not Selection { artifact: Some(1)")),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|p| p.contains("whose run is unknown")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.contains("from a issue_comment run")),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|p| p.contains("expired true")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.contains("not none with 3 refused")),
        "{problems:?}"
    );
}

#[test]
fn a_selection_that_fails_is_named() {
    let broken = gives(Err("boom".to_owned()));

    let problems = selection_problems_with(&broken);

    assert!(
        problems.iter().any(|p| p == "previous select failed: boom"),
        "{problems:?}"
    );
}

#[test]
fn a_selection_that_takes_none_of_main_s_artifacts_is_named_at_the_trigger_check() {
    let none = gives(Ok(Selection {
        artifact: None,
        skipped: 0,
    }));

    let problems = selection_problems_with(&none);

    assert!(
        problems
            .iter()
            .any(|p| p.contains("from a workflow_run run, expired false")),
        "{problems:?}"
    );
}

#[test]
fn the_checkout_s_check_refuses_a_file_past_the_limit_and_one_that_does_not_read() {
    assert_eq!(
        file_problems(&review_run::previous::check),
        Vec::<String>::new()
    );
}

#[test]
fn a_check_that_takes_what_it_should_refuse_or_refuses_what_it_should_take_is_named() {
    let takes_all = file_problems(&|_| Ok(()));
    assert_eq!(takes_all.len(), 2, "{takes_all:?}");
    assert!(
        takes_all[0].contains("past the size limit"),
        "{takes_all:?}"
    );
    assert!(takes_all[1].contains("that do not read"), "{takes_all:?}");

    let refuses_all = file_problems(&|_| Err("no".to_owned()));
    assert_eq!(
        refuses_all,
        vec!["previous check refused findings that read: no".to_owned()]
    );
}

#[test]
fn a_workflow_whose_findings_are_kept_after_a_leaking_scrub_is_named() {
    let got = workflow_problems(&mutated(" && steps.scrub.outcome == 'success'", ""));
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(
        got[0].contains("steps.scrub.outcome == 'success'"),
        "{got:?}"
    );
}

#[test]
fn a_scrub_or_an_upload_that_leaves_out_the_findings_files_is_named() {
    let no_scrub = workflow_problems(&mutated("review-eval -- scrub", "review-eval -- stage"));
    assert!(
        no_scrub.iter().any(|p| p.contains("not scrubbed")),
        "{no_scrub:?}"
    );

    let no_md = workflow_problems(&mutated("--path review/findings.md", "--path review/x.md"));
    assert!(
        no_md.iter().any(|p| p.contains("not scrubbed")),
        "{no_md:?}"
    );

    let no_json = workflow_problems(&mutated(
        "            review/findings.json\n            review/findings.md\n",
        "            review/other.json\n            review/findings.md\n",
    ));
    assert!(
        no_json
            .iter()
            .any(|p| p.contains("lacks \"review/findings.json\"")),
        "{no_json:?}"
    );
}
