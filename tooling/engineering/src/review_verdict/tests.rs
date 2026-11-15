use super::*;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

fn job() -> String {
    let flow = load(&repo(), WORKFLOW).unwrap();
    flow.jobs.into_iter().find(|j| j.name == JOB).unwrap().text
}

/// The review job with `from` replaced by `to`, which must be there.
fn mutated(from: &str, to: &str) -> String {
    let text = job();
    assert!(text.contains(from), "{from}");
    text.replacen(from, to, 1)
}

#[test]
fn the_checkout_s_review_reuses_the_agents_verdict_after_a_comment() {
    assert_eq!(review_reuses_a_verdict(&repo()), Ok(Vec::new()));
}

#[test]
fn a_checkout_without_the_workflow_is_named() {
    let dir = testkit::TempDir::new().unwrap();

    let e = review_reuses_a_verdict(&Checkout::new(dir.path())).unwrap_err();

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

    let got = review_reuses_a_verdict(&Checkout::new(dir.path())).unwrap();

    assert_eq!(got, vec![format!("{WORKFLOW} has no job {JOB}")]);
}

#[test]
fn a_comment_s_review_that_cancels_a_running_one_is_named() {
    assert_eq!(queue_problems(QUEUES), Vec::<String>::new());
    assert_eq!(
        queue_problems("  cancel-in-progress: true\n"),
        ["a comment's review cancels a running review of its pull request"]
    );
}

#[test]
fn a_job_that_weighs_no_earlier_verdict_is_named() {
    let got = workflow_problems(&mutated(
        "review-run -- previous verdict",
        "review-run -- previous guess",
    ));

    assert_eq!(
        got,
        ["no step weighs an earlier review's verdict with review-run previous verdict"]
    );
}

#[test]
fn a_search_that_leaves_out_a_part_of_the_weighing_is_named() {
    for (from, to, why) in [
        (
            "[ \"$EVENT\" = issue_comment ] || decided false",
            "[ \"$EVENT\" = any ] || decided false",
            "looks only after a comment",
        ),
        (
            "spend/trusted-runs.txt",
            "spend/runs.txt",
            "looks only in the runs behind the spend it trusts",
        ),
        (
            "--head \"$HEAD_SHA\"",
            "--head any",
            "weighs the earlier review against this head",
        ),
        (
            "--dispatch review/dispatch.json",
            "--dispatch \"$kept/dispatch.json\"",
            "weighs the earlier review against this dispatch",
        ),
    ] {
        assert_eq!(
            workflow_problems(&mutated(from, to)),
            [format!(
                "the step that finds an earlier verdict never {why}"
            )]
        );
    }
}

#[test]
fn a_search_that_says_nothing_or_fails_the_review_is_named() {
    let silent = find_problems("previous verdict\n");
    assert!(
        silent.contains(
            &"the step that finds an earlier verdict never says in the job summary whether a \
              verdict is reused"
                .to_owned()
        ),
        "{silent:?}"
    );

    let failing = workflow_problems(&mutated(
        "  echo \"reused=$1\" >> \"$GITHUB_OUTPUT\"\n",
        "  echo \"reused=$1\" >> \"$GITHUB_OUTPUT\"\n  exit 1\n",
    ));
    assert_eq!(
        failing,
        ["the step that finds an earlier verdict fails the review"]
    );
}

#[test]
fn a_verdict_fetched_after_the_token_is_staged_is_named() {
    let job = "      - run: review-eval -- stage-credential\n      - run: |\n          \
               [ \"$EVENT\" = issue_comment ] || decided false\n          cat spend/trusted-runs.txt\n          \
               review-run -- previous verdict --head \"$HEAD_SHA\" --dispatch \
               review/dispatch.json >> $GITHUB_STEP_SUMMARY\n";

    let got = workflow_problems(job);

    assert_eq!(
        got,
        [
            "an earlier verdict is fetched after the Claude token is staged",
            "the agents run although an earlier verdict is reused",
            "no step uploads the reports as review-reports",
        ]
    );
}

#[test]
fn agents_that_run_on_a_reused_verdict_are_named() {
    let got = workflow_problems(&mutated(
        "number != '' && steps.verdict.outputs.reused != 'true'",
        "number != ''",
    ));

    assert_eq!(
        got,
        ["the agents run although an earlier verdict is reused"]
    );
}

#[test]
fn a_review_that_keeps_no_part_of_its_verdict_is_named() {
    for kept in KEPT {
        let got = workflow_problems(&mutated(&format!("            {kept}\n"), ""));

        assert_eq!(
            got,
            [format!(
                "a review does not keep {kept} for a comment's review"
            )]
        );
    }
}

#[test]
fn the_checkout_s_weighing_reuses_a_verdict_for_the_same_head_and_dispatch_alone() {
    assert_eq!(
        weighing_problems(&review_run::verdict::weigh_kept),
        Vec::<String>::new()
    );
}

#[test]
fn a_weighing_that_reuses_everything_or_nothing_is_named() {
    let all = weighing_problems(&|_, _, _| Verdict::Reused);
    assert_eq!(
        all,
        [
            "previous verdict reused a verdict for a head that moved",
            "previous verdict reused a verdict under a dispatch that changed",
            "previous verdict reused a verdict of a review that left a gap",
            "previous verdict reused a verdict whose outcome does not read",
        ]
    );

    let none = weighing_problems(&|_, _, _| Verdict::Fresh("never".to_owned()));
    assert_eq!(
        none,
        ["previous verdict did not reuse a verdict for the same head and dispatch"]
    );
}
