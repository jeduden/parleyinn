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

#[test]
fn a_capped_review_is_named_when_its_gaps_or_its_count_alone_miss_the_agents_not_started() {
    let over = |n: usize| -> BTreeMap<String, String> {
        (0..n)
            .map(|i| (format!("a{i}"), "over budget: x".to_owned()))
            .collect()
    };
    let four = "verdict=request_changes\nover_budget=4\n";
    let three = "verdict=request_changes\nover_budget=3\n";

    let gaps_alone = capped_problems(1, four, &over(3));
    let count_alone = capped_problems(1, three, &over(4));

    assert_eq!(gaps_alone.len(), 1, "{gaps_alone:?}");
    assert!(gaps_alone[0].starts_with("3 agents named over budget, not the 4"));
    assert_eq!(count_alone.len(), 1, "{count_alone:?}");
    assert!(count_alone[0].starts_with("4 agents named over budget, not the 4"));
}

#[test]
fn an_abort_is_named_when_the_outcome_or_the_printed_count_alone_misses_it() {
    let gaps: BTreeMap<String, String> = ["review-code", "review-spec"]
        .into_iter()
        .map(|n| {
            (
                n.to_owned(),
                format!("aborted: {ABORT} (x), so it was not started"),
            )
        })
        .collect();
    let opening = format!(
        r#"{{"verdict": "request_changes", "summary": "s", "findings": [{{"path": "review/", "line": 0, "severity": "blocking", "body": "review aborted: {ABORT} (x)"}}]}}"#
    );
    let silent = r#"{"verdict": "request_changes", "summary": "s", "findings": []}"#;

    let outcome_alone = aborted_problems(&[], "aborted=2\n", silent, &gaps);
    let count_alone = aborted_problems(&[], "aborted=0\n", &opening, &gaps);

    assert_eq!(outcome_alone.len(), 1, "{outcome_alone:?}");
    assert!(outcome_alone[0].starts_with("the review outcome does not open with the abort"));
    assert_eq!(count_alone.len(), 1, "{count_alone:?}");
    assert!(count_alone[0].starts_with("the review outcome does not open with the abort"));
}

#[test]
fn an_overloaded_run_is_named_when_its_runs_or_its_waits_alone_are_wrong() {
    let runs =
        |lang_rust: usize| move |agent: &str| if agent == "lang-rust" { lang_rust } else { 1 };
    let fifteen = Duration::from_secs(15);

    let three_runs = retried_problems(&runs(3), &[fifteen]);
    let short_wait = retried_problems(&runs(2), &[Duration::from_secs(14)]);
    let two_waits = retried_problems(&runs(2), &[fifteen, fifteen * 2]);
    let no_wait = retried_problems(&runs(2), &[]);

    for problems in [three_runs, short_wait, two_waits, no_wait] {
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].starts_with("lang-rust, turned away as overloaded"));
    }
}

#[test]
fn a_gap_list_that_is_missing_or_does_not_parse_is_an_error_not_no_gaps() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("review/gaps.json");

    let missing = gaps(dir.path()).unwrap_err();
    assert!(
        missing.starts_with(&format!("read {}: ", path.display())),
        "{missing}"
    );

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{").unwrap();
    let malformed = gaps(dir.path()).unwrap_err();
    assert!(
        malformed.starts_with(&format!("{}: ", path.display())),
        "{malformed}"
    );

    fs::write(&path, "[]").unwrap();
    assert_eq!(gaps(dir.path()), Ok(BTreeMap::new()));
}
