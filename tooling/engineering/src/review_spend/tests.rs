use super::*;

use testkit::TempDir;

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
fn the_checkout_s_review_run_holds_a_pull_request_s_reviews_to_its_cap() {
    let work = TempDir::new().unwrap();

    assert_eq!(cap_problems(&repo(), work.path()), Vec::<String>::new());
}

#[test]
fn the_checkout_s_review_carries_what_a_pull_request_s_reviews_spent() {
    assert_eq!(carry_problems(&repo()), Ok(Vec::new()));
}

#[test]
fn a_review_that_cannot_run_is_named_for_each_case() {
    let work = TempDir::new().unwrap();
    let file = work.path().join("file");
    std::fs::write(&file, "").unwrap();

    let got = cap_problems(&repo(), &file);

    assert_eq!(got.len(), 5, "{got:?}");
    assert!(
        got[0].starts_with("a review with room for one run: "),
        "{got:?}"
    );
    assert!(got[1].starts_with("a review at the cap: "), "{got:?}");
    assert!(
        got[2].starts_with("a review with room for two runs at once: "),
        "{got:?}"
    );
    assert!(
        got[3].starts_with("a review with a run cut off unbilled: "),
        "{got:?}"
    );
}

#[test]
fn a_spend_file_missing_or_unread_is_named() {
    let work = TempDir::new().unwrap();
    let missing = spend_of(work.path()).unwrap_err();
    assert!(
        missing.starts_with("read ") && missing.contains("spend.json"),
        "{missing}"
    );

    std::fs::create_dir_all(work.path().join("review")).unwrap();
    std::fs::write(work.path().join(review_run::spend::SPEND), "{").unwrap();
    let unread = spend_of(work.path()).unwrap_err();
    assert!(
        unread.contains("spend.json: the spend file does not read"),
        "{unread}"
    );
}

fn over(n: usize) -> BTreeMap<String, String> {
    (0..n)
        .map(|i| (format!("agent-{i}"), "over budget: x".to_owned()))
        .collect()
}

#[test]
fn a_review_held_to_the_cap_as_it_should_be_has_no_problem() {
    let gaps = over(4);
    let held = Held {
        started: 1,
        out: "verdict=request_changes\nover_budget=4\n",
        gaps: &gaps,
        spend: Ok(5.005),
        at_once: 1,
    };
    let want = Want {
        started: 1,
        at_once: 1,
        spent: 5.005,
    };

    assert!(held_problems("c", &held, want).is_empty());
}

#[test]
fn a_review_past_the_cap_that_runs_on_approves_or_loses_the_spend_is_named() {
    let gaps = over(1);
    let held = Held {
        started: 4,
        out: "verdict=approve\nover_budget=1\n",
        gaps: &gaps,
        spend: Ok(0.01),
        at_once: 3,
    };
    let want = Want {
        started: 0,
        at_once: 2,
        spent: 5.0,
    };

    let got = held_problems("c", &held, want);

    assert_eq!(
        got,
        [
            "c: 4 runs started, not 0",
            "c: 3 runs ran at once, past the 2 whose tier caps fit in the cap",
            "c: 1 agents named over budget, not the 5 not started: verdict=approve\nover_budget=1\n",
            "c: a review its pull request's cap stopped does not request changes: \
             verdict=approve\nover_budget=1\n",
            "c: review/spend.json says 0.01 USD, not the 5 this review spent",
        ]
    );
    let unread = Held {
        spend: Err("read x: gone".to_owned()),
        ..held
    };
    assert!(held_problems("c", &unread, want).contains(&"c: read x: gone".to_owned()));
}

#[test]
fn a_workflow_without_the_review_job_is_named() {
    let dir = TempDir::new().unwrap();
    let flows = dir.path().join(".github/workflows");
    std::fs::create_dir_all(&flows).unwrap();
    std::fs::write(
        flows.join("review.yml"),
        "on:\n  push:\njobs:\n  other:\n    steps: []\n",
    )
    .unwrap();

    let got = carry_problems(&Checkout::new(dir.path())).unwrap();

    assert_eq!(got, vec![format!("{WORKFLOW} has no job {JOB}")]);
    assert!(carry_problems(&Checkout::new(&dir.path().join("none"))).is_err());
}

#[test]
fn a_review_never_given_what_earlier_reviews_spent_is_named() {
    let got = workflow_problems(&mutated(" --spent-before \"$SPENT\"", ""));

    assert_eq!(
        got,
        ["review-run is not given what earlier reviews spent as --spent-before"]
    );
}

#[test]
fn a_spend_kept_only_when_the_review_succeeded_or_unscrubbed_is_named() {
    let text = job();
    let upload = text.find("name: review-spend-pr-").unwrap();
    let at = text[..upload].rfind("if: always() && ").unwrap();
    let only_on_success = format!(
        "{}if: {}",
        &text[..at],
        &text[at + "if: always() && ".len()..]
    );
    assert_eq!(
        workflow_problems(&only_on_success),
        ["the step that uploads what the pull request's reviews spent lacks \"if: always() && \""]
    );

    let unscrubbed = workflow_problems(&mutated(" --path review/spend.json", ""));
    assert_eq!(
        unscrubbed,
        ["the spend file is not scrubbed of the Claude token"]
    );

    let unkept = workflow_problems(&mutated("name: review-spend-pr-", "name: review-spend-"));
    assert_eq!(unkept.len(), 1, "{unkept:?}");
    assert!(unkept[0].starts_with("no step uploads what"), "{unkept:?}");
}

#[test]
fn a_fetch_that_picks_by_no_origin_or_reads_no_file_is_named() {
    let unpicked = workflow_problems(&mutated(
        "review-run -- previous select --of spend",
        "review-run -- previous list --of spend",
    ));
    assert_eq!(unpicked.len(), 1, "{unpicked:?}");
    assert!(
        unpicked[0].contains("previous select --of spend"),
        "{unpicked:?}"
    );

    let got = workflow_problems(&mutated("previous spend \"", "previous skip \""));
    assert_eq!(
        got,
        [
            "the step that fetches what earlier reviews spent never reads and sums the file of \
             every earlier review with review-run previous spend"
        ]
    );
    let all = workflow_problems(&mutated(" --except-run \"$GITHUB_RUN_ID\"", ""));
    assert_eq!(
        all,
        [
            "the step that fetches what earlier reviews spent never leaves out the spend its own run kept"
        ]
    );
}

#[test]
fn a_fetch_that_goes_on_past_a_run_it_could_not_look_up_is_named() {
    let past = "the step that fetches what earlier reviews spent goes on past a run it could not \
                look up";
    let warned = mutated(
        "    || fail \"Run $id, behind",
        "    || echo \"::warning::Run $id, behind",
    );
    assert_eq!(workflow_problems(&warned), [past]);
    // The loop's failures piped into jq end in its subshell, unseen.
    let piped = mutated(
        ": > \"$dir/runs.jsonl\"\n",
        ": > \"$dir/runs.jsonl\"\ndone | jq\n",
    );
    assert_eq!(workflow_problems(&piped), [past]);
    assert!(!lookup_fails("gh api \"repos/$REPO/actions/runs/$id\""));
}

#[test]
fn a_spend_fetched_after_the_token_is_staged_is_named() {
    let text = job();
    let fetch = text
        .find("review-run -- previous select --of spend")
        .unwrap();
    let stage = "      - name: early stage\n        run: cargo run -p review-eval -- stage-credential --dir d\n";
    let at = text[..fetch].rfind("\n      - ").unwrap() + 1;
    let early = format!("{}{stage}{}", &text[..at], &text[at..]);

    let got = workflow_problems(&early);

    assert!(
        got.contains(
            &"what earlier reviews spent is fetched after the Claude token is staged".to_owned()
        ),
        "{got:?}"
    );
}

#[test]
fn a_fetch_step_that_lacks_its_checks_names_each() {
    assert_eq!(fetch_problems("previous select --of spend").len(), 6);
}

#[test]
fn a_reading_that_takes_any_file_or_misreads_one_is_named() {
    let any = |_: &str| Ok(0.0);

    let got = file_problems(&any);

    assert_eq!(got.len(), 4, "{got:?}");
    assert!(got[0].starts_with("previous spend read a spend file of 1.5 USD as Ok(0.0)"));
    assert!(got[1].contains("past the size limit for 0 USD"), "{got:?}");
}

#[test]
fn an_upload_lacking_its_retention_is_named() {
    let got = workflow_problems(&mutated(
        "          overwrite: true\n          retention-days: 90\n",
        "          overwrite: true\n",
    ));

    assert_eq!(
        got,
        [
            "the step that uploads what the pull request's reviews spent lacks \"retention-days: 90\""
        ]
    );
    let kept = workflow_problems(&mutated(
        "          overwrite: true\n          retention-days: 90\n",
        "          retention-days: 90\n",
    ));
    assert_eq!(
        kept,
        ["the step that uploads what the pull request's reviews spent lacks \"overwrite: true\""]
    );
}

#[test]
fn a_selection_that_takes_another_origin_or_fails_is_named() {
    let failing = |_: &str, _: &str, _: &str, _: Option<u64>| Err("no listing".to_owned());

    let got = selection_problems_with(&failing);

    assert_eq!(
        got,
        [
            "previous select --of spend gave Err(\"no listing\"), not artifacts 1 and 4 with 1 \
             refused"
        ]
    );
}

#[test]
fn a_sum_that_drops_a_file_or_takes_a_broken_one_is_named() {
    let first = |texts: &[(String, String)]| review_run::spend::check(&texts[0].1);

    let got = sum_problems(&first);

    assert_eq!(
        got,
        [
            "previous spend summed spend files of 1.5 and 0.25 USD as Ok(1.5)",
            "previous spend summed a spend file that does not read as Ok(1.5), not naming it"
        ]
    );
    assert!(sum_problems(&review_run::previous::summed).is_empty());
}
