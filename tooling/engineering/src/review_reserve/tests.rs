use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn command(agent: &str, cap: Option<&str>) -> Command {
    let mut args = vec!["--agent", agent];
    if let Some(cap) = cap {
        args.extend(["--max-budget-usd", cap]);
    }

    Command::new("claude", &args, Path::new("/w"))
}

/// The five runs of a review that started every agent on 0.90 left.
fn sound() -> Vec<Command> {
    vec![
        command("lang-markdown", Some("0.30")),
        command("review-spec", Some("0.89")),
        command("lang-rust", Some("0.30")),
        command("test-engineer-unit", Some("0.30")),
        command("review-code", Some("0.86")),
    ]
}

const STARTED: &str = "verdict=approve\ncost=0.05\nover_budget=0\naborted=0\n";

#[test]
fn the_checkout_s_review_run_starts_a_generalist_on_what_is_left_of_the_cap() {
    let repo = Checkout::new(&testkit::repo_root());
    let work = TempDir::new().unwrap();

    let problems = reserve_problems(&repo, work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_command_s_cost_cap_is_read_when_it_passes_one_that_reads() {
    assert_eq!(cap_of(&command("lang-rust", Some("0.30"))), Some(0.3));
    assert_eq!(cap_of(&command("lang-rust", Some("much"))), None);
    assert_eq!(cap_of(&command("lang-rust", None)), None);
}

#[test]
fn a_run_capped_off_its_tier_s_cap_or_outside_what_was_left_is_named() {
    assert_eq!(cap_problems("lang-rust", Some(0.3), 0.9), None);
    assert_eq!(cap_problems("review-code", Some(0.75), 0.9), None);
    assert_eq!(cap_problems("review-code", Some(0.9), 0.9), None);

    for (agent, cap) in [
        ("lang-rust", Some(0.9)),
        ("review-code", Some(1.5)),
        ("review-code", Some(0.74)),
        ("review-code", None),
    ] {
        assert_eq!(
            cap_problems(agent, cap, 0.9),
            Some(format!(
                "{agent} ran with the cost cap {cap:?}, though 0.90 USD was left of the pull \
                 request's cap"
            ))
        );
    }
}

#[test]
fn a_review_that_started_every_agent_within_what_was_left_has_no_problem() {
    assert!(started_problems(&sound(), STARTED, &Ok(0.05)).is_empty());
}

#[test]
fn a_review_that_left_an_agent_out_capped_one_wrongly_or_kept_another_spend_is_named() {
    let mut runs = sound();
    runs.pop();
    runs[1] = command("review-spec", Some("1.50"));

    let problems = started_problems(&runs, "verdict=request_changes\n", &Ok(0.04));

    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(
        problems[0].starts_with("4 of 5 agents started with 0.90 USD left"),
        "{problems:?}"
    );
    assert!(
        problems[1].starts_with("review-spec ran with the cost cap Some(1.5)"),
        "{problems:?}"
    );
    assert_eq!(
        problems[2],
        "review/spend.json says 0.04 USD, not the 0.05 its runs cost"
    );
    // Every agent started, and the review still counts one over budget.
    let counted = started_problems(&sound(), "verdict=approve\nover_budget=1\n", &Ok(0.05));
    assert_eq!(counted.len(), 1, "{counted:?}");
    assert_eq!(
        started_problems(&sound(), STARTED, &Err("read spend.json: gone".into())),
        ["read spend.json: gone"]
    );
}

fn over_budget(why: &str) -> BTreeMap<String, String> {
    ["review-code", "review-spec"]
        .into_iter()
        .map(|name| (name.to_owned(), why.to_owned()))
        .chain([("lang-rust".to_owned(), "partial: lines unread".to_owned())])
        .collect()
}

#[test]
fn a_generalist_started_under_its_reserve_or_not_named_with_it_is_named() {
    let named = over_budget(
        "over budget: the pull request's reviews cost 4.3300 USD, and a run capped at 1.50 USD, \
         which starts with 0.75 USD at least, does not fit in its cap of 5.00 USD, so it was \
         not started",
    );
    let specialists = [command("lang-rust", Some("0.30"))];
    assert!(refused_problems(&specialists, &named).is_empty());

    let started = [command("review-code", Some("0.70"))];
    let unnamed = over_budget("over budget: a run capped at 1.50 USD does not fit");

    let problems = refused_problems(&started, &unnamed);

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert_eq!(
        problems[0],
        "review-code started with less than its reserve left"
    );
    assert!(
        problems[1].starts_with(
            "0 generalists named over budget with the reserve they start with, not 2: "
        ),
        "{problems:?}"
    );
}

#[test]
fn a_killed_generalist_charged_other_than_what_was_left_is_named() {
    let out = "verdict=request_changes\nover_budget=1\n";
    assert!(killed_problems(out, &Ok(0.9)).is_empty());

    let problems = killed_problems("verdict=approve\n", &Ok(1.53));

    assert_eq!(
        problems,
        [
            "review/spend.json says 1.53 USD after a generalist was killed on what was left, not \
             the 0.90 left: its tier's cap is 1.50",
            "a review whose generalist was killed with nothing left does not request changes: \
             verdict=approve\n",
        ]
    );
    assert_eq!(
        killed_problems(out, &Err("read spend.json: gone".into())),
        ["read spend.json: gone"]
    );
}

#[test]
fn a_review_that_cannot_run_is_named_for_each_run() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = reserve_problems(&c, work.path());

    assert_eq!(problems.len(), 3, "{problems:?}");
    for problem in &problems {
        assert!(problem.starts_with("read .claude/agents/"), "{problem}");
    }
}
