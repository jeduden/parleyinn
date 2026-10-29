use super::*;

fn run(passed: bool, cost: f64, turns: usize, model: &str) -> RunRecord {
    RunRecord {
        passed,
        failures: if passed {
            Vec::new()
        } else {
            vec!["outcome sound, want blocking".to_owned()]
        },
        answered: true,
        cost,
        turns,
        duration_ms: 2000,
        resolved_model: model.to_owned(),
        usage: BTreeMap::new(),
        first_write_turn: Some(2),
        notes: Vec::new(),
    }
}

fn case(name: &str, runs: Vec<RunRecord>) -> CaseRecord {
    let passed = runs.iter().filter(|r| r.passed).count() >= 2;
    CaseRecord {
        component: "lang-rust".to_owned(),
        name: name.to_owned(),
        kind: "defect".to_owned(),
        title: format!("the {name} case"),
        repetitions: 3,
        passes: 2,
        runs,
        passed,
    }
}

fn suite() -> SuiteRecord {
    SuiteRecord {
        component: "lang-rust".to_owned(),
        model: "haiku".to_owned(),
        hash: "abc".to_owned(),
        cases: vec![
            case(
                "unwrap",
                vec![
                    run(true, 0.1, 5, "claude-haiku-5-5"),
                    run(true, 0.1, 7, "claude-haiku-5-5"),
                ],
            ),
            case(
                "clean",
                vec![
                    run(false, 0.2, 9, "claude-haiku-5-5"),
                    run(true, 0.2, 9, "claude-haiku-5-5"),
                    run(false, 0.2, 10, "claude-haiku-5-6"),
                ],
            ),
        ],
    }
}

#[test]
fn a_run_record_takes_its_grade_and_usage_from_the_log() {
    let log = RunLog {
        answered: true,
        cost: 0.5,
        turns: 4,
        duration_ms: 10,
        resolved_model: "m".to_owned(),
        first_write_turn: Some(1),
        ..RunLog::default()
    };

    let passed = RunRecord::new(&log, Vec::new());
    let failed = RunRecord::new(&log, vec!["x".to_owned()]);
    let broken = RunRecord::failed("claude did not start");

    assert!(passed.passed && passed.answered);
    assert_eq!(
        (
            passed.turns,
            passed.duration_ms,
            passed.resolved_model.as_str()
        ),
        (4, 10, "m")
    );
    assert!(!failed.passed);
    assert_eq!(failed.failures, vec!["x"]);
    assert!(!broken.passed && !broken.answered);
    assert_eq!(broken.failures, vec!["claude did not start"]);
}

#[test]
fn a_case_counts_its_passed_runs_and_names_its_first_failure() {
    let s = suite();

    assert_eq!(s.cases[0].passed_runs(), 2);
    assert_eq!(s.cases[0].first_failure(), "");
    assert_eq!(s.cases[1].passed_runs(), 1);
    assert_eq!(s.cases[1].first_failure(), "outcome sound, want blocking");
}

#[test]
fn a_case_is_decided_as_soon_as_its_result_is_certain() {
    assert_eq!(decided(2, 0, 3, 2), Some(true));
    assert_eq!(decided(1, 1, 3, 2), None);
    assert_eq!(decided(1, 2, 3, 2), Some(false));
    assert_eq!(decided(0, 2, 3, 2), Some(false));
    assert_eq!(decided(0, 0, 3, 2), None);
    assert_eq!(decided(1, 0, 1, 1), Some(true));
    assert_eq!(decided(0, 1, 1, 1), Some(false));
    assert_eq!(decided(0, 1, 1, 2), Some(false));
}

#[test]
fn a_suite_sums_its_cases() {
    let s = suite();

    assert!(!s.passed());
    assert_eq!(s.cases_passed(), (1, 2));
    assert_eq!(s.runs_passed(), (3, 5));
    assert!((s.cost() - 0.8).abs() < 1e-9);
    assert!((s.mean_turns() - 8.0).abs() < 1e-9);
    assert!((s.mean_seconds() - 2.0).abs() < 1e-9);
    assert_eq!(s.resolved(), "claude-haiku-5-5");
    let (lo, hi) = s.interval().unwrap();
    assert!(lo > 0.2 && lo < 0.3, "{lo}");
    assert!(hi > 0.88 && hi < 0.89, "{hi}");
}

#[test]
fn an_empty_suite_passes_with_nothing_to_show() {
    let s = SuiteRecord::default();

    assert!(s.passed());
    assert_eq!(s.cases_passed(), (0, 0));
    assert_eq!(s.runs_passed(), (0, 0));
    assert!(s.cost().abs() < 1e-9);
    assert!(s.mean_turns().abs() < 1e-9);
    assert_eq!(s.resolved(), "unknown");
    assert_eq!(s.interval(), None);
}

#[test]
fn wilson_brackets_the_rate_and_widens_with_fewer_runs() {
    let (lo, hi) = wilson(72, 72).unwrap();
    assert!((lo - 0.949).abs() < 0.001, "{lo}");
    assert!((hi - 1.0).abs() < 1e-9, "{hi}");
    let (lo3, hi3) = wilson(3, 3).unwrap();
    assert!(lo3 < lo && (hi3 - 1.0).abs() < 1e-9);
    let (lo0, hi0) = wilson(0, 3).unwrap();
    assert!(lo0.abs() < 1e-9 && hi0 > 0.5);
    assert_eq!(wilson(0, 0), None);
}

#[test]
fn a_scorecard_makes_one_blocking_finding_per_failed_case() {
    let card = Scorecard {
        suites: vec![suite()],
    };

    assert!(!card.passed());
    let findings = card.findings(&|name| format!(".claude/agents/{name}.md"));

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].path, ".claude/agents/lang-rust.md");
    assert_eq!(findings[0].line, 0);
    assert_eq!(findings[0].severity, "blocking");
    assert_eq!(
        findings[0].body,
        "reviewer evaluation: case lang-rust/clean (the clean case) passed 1 of 3 runs on claude-haiku-5-5, needing 2: outcome sound, want blocking"
    );
    assert!(Scorecard::default().passed());
    assert!(Scorecard::default().findings(&|_| String::new()).is_empty());
}

#[test]
fn a_scorecard_renders_as_markdown() {
    let card = Scorecard {
        suites: vec![suite()],
    };

    let md = card.markdown();

    assert!(md.starts_with("## Reviewer evaluation\n"), "{md}");
    assert!(
        md.contains(
            "### lang-rust on haiku (claude-haiku-5-5): 1 of 2 cases, 3 of 5 runs (95% interval"
        ),
        "{md}"
    );
    assert!(md.contains("$0.80, 8.0 turns, 2 s per run"), "{md}");
    assert!(
        md.contains("| unwrap | defect | 2 of 2 | $0.20 | 6.0 | passed |"),
        "{md}"
    );
    assert!(
        md.contains("| clean | defect | 1 of 3 | $0.60 | 9.3 | failed |"),
        "{md}"
    );
    assert!(
        md.contains("- `clean` run 1: outcome sound, want blocking"),
        "{md}"
    );
    assert!(
        md.contains("- `clean` run 3: outcome sound, want blocking"),
        "{md}"
    );
    assert_eq!(
        Scorecard::default().markdown(),
        "## Reviewer evaluation\n\nNo component was evaluated.\n"
    );
}

#[test]
fn a_scorecard_round_trips_through_json() {
    let card = Scorecard {
        suites: vec![suite()],
    };

    let text = serde_json::to_string(&card).unwrap();
    let back: Scorecard = serde_json::from_str(&text).unwrap();

    assert_eq!(back, card);
}
