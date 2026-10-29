use super::*;
use crate::score::{CaseRecord, RunRecord};

fn line(component: &str, cases_passed: usize, runs_passed: usize, runs: usize, cost: f64) -> Line {
    Line {
        component: component.to_owned(),
        model: "haiku".to_owned(),
        resolved: "claude-haiku-5-5".to_owned(),
        cases_passed,
        cases: 4,
        runs_passed,
        runs,
        slipped: 2,
        cost,
        turns: 7.25,
        seconds: 48.0,
        hash: "0123456789abcdef".to_owned(),
    }
}

#[test]
fn a_line_renders_and_reads_back() {
    let lines = vec![
        line("lang-toml", 4, 11, 12, 0.42),
        line("lang-rust", 3, 9, 12, 1.0),
    ];

    let text = render(&lines);

    assert_eq!(
        text,
        "reviewer evaluation scorecard v1\n\
         component=lang-rust model=haiku resolved=claude-haiku-5-5 cases=3/4 runs=9/12 slipped=2 cost=1.0000 turns=7.2 seconds=48 hash=0123456789abcdef\n\
         component=lang-toml model=haiku resolved=claude-haiku-5-5 cases=4/4 runs=11/12 slipped=2 cost=0.4200 turns=7.2 seconds=48 hash=0123456789abcdef\n"
    );
    let back = parse(&text).unwrap();
    assert_eq!(back.len(), 2);
    assert_eq!(back[0].component, "lang-rust");
    assert_eq!((back[0].cases_passed, back[0].cases), (3, 4));
    assert_eq!((back[0].runs_passed, back[0].runs), (9, 12));
    assert_eq!(back[0].slipped, 2);
    assert!((back[0].cost - 1.0).abs() < 1e-9);
    assert!((back[0].turns - 7.2).abs() < 1e-9);
    assert!((back[0].seconds - 48.0).abs() < 1e-9);
    assert_eq!(back[0].hash, "0123456789abcdef");
    assert!((back[1].rate() - 11.0 / 12.0).abs() < 1e-9);
    assert!((back[1].cost_per_run() - 0.035).abs() < 1e-9);
}

#[test]
fn an_empty_scorecard_is_its_header_alone() {
    assert_eq!(render(&[]), "reviewer evaluation scorecard v1\n");
    assert_eq!(
        parse("reviewer evaluation scorecard v1\n\n"),
        Ok(Vec::new())
    );
}

#[test]
fn parse_names_each_problem() {
    assert_eq!(
        parse("scorecard\n").unwrap_err(),
        "the scorecard does not open with \"reviewer evaluation scorecard v1\""
    );
    let bad = |fields: &str| parse(&format!("{HEADER}\n{fields}\n")).unwrap_err();
    assert_eq!(bad("model=haiku"), "line 2: no component");
    assert_eq!(
        bad(
            "component=x model=h resolved=r cases=1 runs=1/1 slipped=0 cost=1 turns=1 seconds=1 hash=h"
        ),
        "line 2: cases is not passed/all"
    );
    assert_eq!(
        bad(
            "component=x model=h resolved=r cases=a/b runs=1/1 slipped=0 cost=1 turns=1 seconds=1 hash=h"
        ),
        "line 2: cases is not passed/all"
    );
    assert_eq!(
        bad(
            "component=x model=h resolved=r cases=1/1 runs=1/1 slipped=some cost=1 turns=1 seconds=1 hash=h"
        ),
        "line 2: slipped is not a count"
    );
    assert_eq!(
        bad(
            "component=x model=h resolved=r cases=1/1 runs=1/1 slipped=0 cost=much turns=1 seconds=1 hash=h"
        ),
        "line 2: cost is not a number"
    );
    assert_eq!(
        bad("component=x model=h resolved=r cases=1/1 runs=1/1 slipped=0 cost=1 turns=1 seconds=1"),
        "line 2: no hash"
    );
}

#[test]
fn a_line_without_runs_has_no_rate_or_cost() {
    let l = line("x", 0, 0, 0, 0.0);

    assert!(l.rate().abs() < 1e-9);
    assert!(l.cost_per_run().abs() < 1e-9);
}

#[test]
fn merge_replaces_a_component_s_line_and_keeps_the_rest() {
    let existing = vec![
        line("lang-rust", 1, 1, 3, 0.1),
        line("lang-toml", 4, 12, 12, 0.5),
    ];
    let new = vec![
        line("lang-rust", 4, 12, 12, 0.9),
        line("review", 2, 6, 6, 2.0),
    ];

    let merged = merge(existing, new);

    assert_eq!(
        merged
            .iter()
            .map(|l| (l.component.as_str(), l.cases_passed))
            .collect::<Vec<_>>(),
        vec![("lang-rust", 4), ("lang-toml", 4), ("review", 2)]
    );
}

#[test]
fn a_suite_becomes_a_line() {
    let run = |passed: bool| RunRecord {
        passed,
        slips: vec!["answer does not hold the report's outcome line".to_owned()],
        cost: 0.25,
        turns: 6,
        duration_ms: 3000,
        resolved_model: "claude-haiku-5-5".to_owned(),
        ..RunRecord::default()
    };
    let scorecard = Scorecard {
        suites: vec![SuiteRecord {
            component: "lang-rust".to_owned(),
            model: "haiku".to_owned(),
            hash: "h".to_owned(),
            cases: vec![CaseRecord {
                component: "lang-rust".to_owned(),
                name: "unwrap".to_owned(),
                runs: vec![run(true), run(true)],
                passed: true,
                ..CaseRecord::default()
            }],
        }],
    };

    let got = lines(&scorecard);

    assert_eq!(
        got,
        vec![Line {
            component: "lang-rust".to_owned(),
            model: "haiku".to_owned(),
            resolved: "claude-haiku-5-5".to_owned(),
            cases_passed: 1,
            cases: 1,
            runs_passed: 2,
            runs: 2,
            slipped: 2,
            cost: 0.5,
            turns: 6.0,
            seconds: 3.0,
            hash: "h".to_owned(),
        }]
    );
}
