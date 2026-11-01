use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_review_run_holds_its_agents_to_the_controls() {
    let work = TempDir::new().unwrap();

    let mut problems = caps_problems(&repo(), &work.path().join("caps"));
    problems.extend(breaker_problems(&repo(), &work.path().join("breaker")));
    problems.extend(retry_problems(&repo(), &work.path().join("retry")));

    assert!(problems.is_empty(), "{problems:?}");
}

fn command(args: &[&str]) -> Command {
    Command::new("claude", args, Path::new("/w"))
}

#[test]
fn a_run_without_its_tier_s_cap_is_named() {
    let commands = [
        command(&["--agent", "lang-rust", "--max-budget-usd", "1.00"]),
        command(&["--agent", "review-code", "--max-budget-usd", "1.00"]),
        command(&["--agent", "lang-shell"]),
    ];

    assert_eq!(
        tier_problems(&commands),
        [
            "review-code ran without its tier's cost cap 5.00",
            "lang-shell ran without its tier's cost cap 1.00"
        ]
    );
}

#[test]
fn a_review_past_its_cap_that_runs_on_names_no_gap_or_approves_is_named() {
    let over: BTreeMap<String, String> = ["a", "b", "c", "d"]
        .into_iter()
        .map(|n| (n.to_owned(), "over budget: x".to_owned()))
        .collect();
    let out = "verdict=request_changes\ncost=0.01\nover_budget=4\naborted=0\n";
    assert!(capped_problems(1, out, &over).is_empty());

    let problems = capped_problems(2, "verdict=approve\nover_budget=0\n", &BTreeMap::new());

    assert_eq!(problems.len(), 3, "{problems:?}");
    assert_eq!(
        problems[0],
        "2 runs started though the first reached the review's cost cap"
    );
    assert!(problems[1].starts_with("0 agents named over budget, not the 4 not started"));
    assert!(problems[2].starts_with("a review past its cost cap does not request changes"));
}

#[test]
fn a_review_that_starts_a_generalist_past_the_breaker_or_names_no_abort_is_named() {
    let gaps: BTreeMap<String, String> = ["review-code", "review-spec"]
        .into_iter()
        .map(|n| {
            (
                n.to_owned(),
                format!("aborted: {ABORT} (x), so it was not started"),
            )
        })
        .collect();
    let outcome = format!(
        r#"{{"verdict": "request_changes", "summary": "s", "findings": [{{"path": "review/", "line": 0, "severity": "blocking", "body": "review aborted: {ABORT} (x)"}}]}}"#
    );
    assert!(aborted_problems(&[], "aborted=0\naborted=2\n", &outcome, &gaps).is_empty());

    let problems = aborted_problems(&["review-spec"], "", "{", &BTreeMap::new());

    assert_eq!(
        problems,
        [
            "review-spec started though 2 of 3 specialists failed",
            "review-code is not named aborted: \"\"",
            "review-spec is not named aborted: \"\"",
            "the review outcome does not open with the abort: {"
        ]
    );
}

#[test]
fn an_overloaded_run_not_waited_for_or_one_that_cannot_succeed_run_again_is_named() {
    let runs = |n: usize| move |agent: &str| if agent == "lang-rust" { 2 } else { n };
    assert!(retried_problems(&runs(1), &[Duration::from_secs(15)]).is_empty());

    let problems = retried_problems(&runs(2), &[Duration::from_secs(1)]);

    assert_eq!(
        problems,
        [
            "lang-rust, turned away as overloaded, ran 2 times after waits of [1s], not twice \
             after one wait of 15 s or more",
            "test-engineer-unit, which can never succeed by running again, ran 2 times",
            "lang-markdown, which can never succeed by running again, ran 2 times"
        ]
    );
}

#[test]
fn a_checkout_without_the_agents_definitions_is_named_by_each_check() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    for problems in [
        caps_problems(&c, &work.path().join("caps")),
        breaker_problems(&c, &work.path().join("breaker")),
        retry_problems(&c, &work.path().join("retry")),
    ] {
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("read .claude/agents/"),
            "{problems:?}"
        );
    }
}

#[test]
fn a_review_that_cannot_run_is_named_by_each_check() {
    let work = TempDir::new().unwrap();
    for dir in ["caps/full", "breaker"] {
        fs::create_dir_all(work.path().join(dir).join("review/outcome.json")).unwrap();
    }
    fs::create_dir_all(work.path().join("retry/review")).unwrap();
    fs::write(work.path().join("retry/review/logs"), "a file").unwrap();

    let caps = caps_problems(&repo(), &work.path().join("caps"));
    let breaker = breaker_problems(&repo(), &work.path().join("breaker"));
    let retry = retry_problems(&repo(), &work.path().join("retry"));

    assert!(caps[0].starts_with("review-run exited 1: "), "{caps:?}");
    assert!(
        breaker[0].starts_with("review-run exited 1: "),
        "{breaker:?}"
    );
    assert!(retry[0].starts_with("make "), "{retry:?}");
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere_or_cannot_write_its_log() {
    let work = TempDir::new().unwrap();
    let host = ControlHost {
        fails: BTreeMap::from([("lang-rust", (1, Some(CAPPED)))]),
        ..ControlHost::default()
    };

    let nowhere = command(&["--agent", "lang-rust"]);
    assert_eq!(host.run(&nowhere).unwrap_err(), "the run logs to no file");
    let unwritable =
        command(&["--agent", "lang-rust"]).logging_to(&work.path().join("no/such/dir"));
    let host = ControlHost {
        fails: BTreeMap::from([("lang-rust", (1, Some(CAPPED)))]),
        ..ControlHost::default()
    };
    assert!(host.run(&unwritable).unwrap_err().starts_with("write "));
}

#[test]
fn the_clock_moves_on_by_each_wait_and_draws_no_jitter() {
    let clock = Waits::default();
    clock.sleep(Duration::from_secs(2));
    clock.sleep(Duration::from_secs(3));

    assert_eq!(clock.now(), Duration::from_secs(5));
    assert!(clock.jitter().abs() < f64::EPSILON);
}
