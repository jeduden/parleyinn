use super::*;
use crate::review_workflow::{self, parse};
use crate::tests::checkout;

use testkit::TempDir;

const KEY: &str = "APP_KEY";

/// A workflow whose key job decides the review with `run`.
fn flow(run: &str) -> Workflow {
    parse(&format!(
        "jobs:
  post:
    steps:
      - env:
          K: ${{{{ secrets.{KEY} }}}}
        run: |
{run}"
    ))
}

const DRY: &str = "over budget: the pull request's reviews cost 0.3000 USD, and a run capped at \
                   0.30 USD does not fit in its cap of 0.31 USD, so it was not started";

fn gaps_of(names: &[&str]) -> BTreeMap<String, String> {
    names
        .iter()
        .map(|n| ((*n).to_owned(), DRY.to_owned()))
        .collect()
}

#[test]
fn the_checkout_s_gate_says_what_to_do_about_a_review_its_cost_cap_stopped() {
    let repo = Checkout::new(&testkit::repo_root());
    let work = TempDir::new().unwrap();
    let flow = review_workflow::load(&repo, ".github/workflows/review.yml").unwrap();

    let mut problems = check_problems(&flow, "JEDUDEN_REVIEW_AGENT_KEY").unwrap();
    problems.extend(capped_problems(&repo, work.path()));

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_job_that_drops_the_gate_s_error_is_named() {
    let says = "          if ! gate 2> gate-error.txt; then
            why=\"$(tail -n 1 gate-error.txt)\"
            post \"failed before deciding: $why\"
          fi
";
    assert_eq!(check_problems(&flow(says), KEY), Ok(vec![]));

    assert_eq!(
        check_problems(&flow("          gate || post failed\n"), KEY),
        Ok(vec![
            "job post does not keep the gate's errors: 2> gate-error.txt".to_owned(),
            "job post does not take the gate's error from them: tail -n 1 gate-error.txt"
                .to_owned(),
            "job post does not say the gate's error in the check it fails: failed before \
             deciding: $why"
                .to_owned(),
        ])
    );
    assert_eq!(
        check_problems(&flow(says), "OTHER").unwrap_err(),
        r#"0 jobs read "OTHER", want exactly one"#
    );
}

#[test]
fn a_gate_that_posts_a_review_a_cost_cap_kept_agents_out_of_is_named() {
    let review = review_gate::Review {
        commit_id: "c".into(),
        event: review_gate::REQUEST_CHANGES.into(),
        body: "b".into(),
    };

    assert_eq!(
        said_problems(&Ok(review), &gaps_of(&["review-code"])),
        ["the gate posts REQUEST_CHANGES for a review a cost cap kept agents out of"]
    );
}

#[test]
fn an_error_that_holds_each_reason_and_the_reset_has_no_problem() {
    let said = format!("review: dispatched but not consulted: lang-rust ({DRY}); {RESET}");
    let mut gaps = gaps_of(&["lang-rust"]);
    gaps.insert("lang-toml".into(), "partial: lines 3-9 unread".into());

    assert!(said_problems(&Err(said), &gaps).is_empty());
}

#[test]
fn an_error_without_an_agent_s_reason_or_the_reset_is_named() {
    let said = format!("review: dispatched but not consulted: lang-rust ({DRY}), review-code");

    let problems = said_problems(&Err(said.clone()), &gaps_of(&["lang-rust", "review-code"]));

    assert_eq!(
        problems,
        [
            format!("the gate's error does not say why review-code did not run: {said}"),
            format!(
                "the gate's error does not say how what the pull request's reviews spent is \
                 reset: {said}"
            ),
        ]
    );
}

#[test]
fn a_review_that_cannot_run_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = capped_problems(&c, work.path());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("read .claude/agents/"),
        "{problems:?}"
    );
}
