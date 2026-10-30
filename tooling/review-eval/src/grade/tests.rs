use std::collections::BTreeSet;

use super::*;
use crate::case::Expected;
use crate::component::Kind;

fn component(kind: Kind, name: &str, strict_lines: bool) -> Component {
    Component {
        name: name.to_owned(),
        kind,
        file: format!(".claude/agents/{name}.md"),
        text: String::new(),
        model: "haiku".to_owned(),
        effort: "low".to_owned(),
        max_turns: 20,
        tools: Vec::new(),
        strict_lines,
    }
}

fn case(component: &str, expect: Expect) -> Case {
    Case {
        component: component.to_owned(),
        name: "x".to_owned(),
        kind: "defect".to_owned(),
        title: "x".to_owned(),
        consent: None,
        edits: Vec::new(),
        seeds: Vec::new(),
        expect,
        repetitions: 3,
        passes: 2,
    }
}

fn wanted(path: &str) -> Expected {
    Expected {
        path: path.to_owned(),
        ..Expected::default()
    }
}

fn answered_log(answer: &str) -> RunLog {
    RunLog {
        answered: true,
        subtype: "success".to_owned(),
        answer: answer.to_owned(),
        ..RunLog::default()
    }
}

const REPORT: &str = "crates/x/src/a.rs:12 — blocking — an unwrap outside tests — ENG-16\n\noutcome: blocking — it unwraps.\n";

fn run<'a>(log: &'a RunLog, report: Option<&'a str>, dispatched: &'a BTreeSet<String>) -> Run<'a> {
    Run {
        log,
        report,
        dispatched,
    }
}

#[test]
fn a_specialist_run_that_does_everything_right_passes() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        outcome: vec!["blocking".into()],
        finding: vec![Expected {
            severity: Some("blocking".into()),
            line: Some(10),
            mentions: vec!["unwrap".into()],
            ids: vec!["ENG-16".into()],
            ..wanted("crates/x/src/a.rs")
        }],
        none: vec![wanted("crates/x/src/lib.rs")],
        absent: vec!["approved".into()],
        ..Expect::default()
    };
    let mut log = answered_log("outcome: blocking — it unwraps.\nreview/reports/lang-rust.md\n");
    log.writes = vec!["/work/main/review/reports/lang-rust.md".into()];
    let none = BTreeSet::new();

    let graded = grade(
        &c,
        &case("lang-rust", expect),
        &run(&log, Some(REPORT), &none),
    );

    assert_eq!(graded, Graded::default());
    assert!(graded.passed());
}

#[test]
fn a_specialist_run_names_each_failed_assertion() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        outcome: vec!["sound".into()],
        finding: vec![Expected {
            mentions: vec!["panic".into()],
            ..wanted("crates/x/src/b.rs")
        }],
        none: vec![wanted("crates/x/src/a.rs")],
        absent: vec!["everything".into()],
        ..Expect::default()
    };
    let mut log = answered_log("I looked.\noutcome: sound\n");
    log.writes = vec!["review/reports/lang-toml.md".into()];
    let none = BTreeSet::new();
    let eleven = (1..=11)
        .map(|n| format!("crates/x/src/a.rs:{n} — nit — one\n"))
        .collect::<String>()
        + "I read everything.\noutcome: blocking\n";

    let graded = grade(
        &c,
        &case("lang-rust", expect),
        &run(&log, Some(&eleven), &none),
    );

    assert_eq!(
        graded.failures,
        vec![
            "wrote review/reports/lang-toml.md",
            "outcome blocking, want sound",
            "no finding at crates/x/src/b.rs mentioning panic",
            "a finding at crates/x/src/a.rs: crates/x/src/a.rs:1 — nit — one",
            "holds \"everything\"",
        ]
    );
    assert_eq!(
        graded.slips,
        vec![
            "answer does not name the report's outcome",
            "answer does not name the report's path",
            "answer holds more than the outcome and the path: I looked.",
            "11 findings, at most 10",
            "a line that is neither a finding nor the outcome: I read everything.",
        ]
    );
}

#[test]
fn a_specialist_run_that_only_slips_passes_with_its_slips_counted() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        outcome: vec!["blocking".into()],
        finding: vec![Expected {
            mentions: vec!["unwrap".into()],
            line: Some(22),
            ..wanted("crates/x/src/a.rs")
        }],
        ..Expect::default()
    };
    let report = "Findings:\n\n- crates/x/src/a.rs:20-23 — blocking — an unwrap outside tests — ENG-16\n\noutcome: blocking — it unwraps.\n";
    let log =
        answered_log("outcome: blocking — unwraps, see the report.\nreview/reports/lang-rust.md\n");
    let none = BTreeSet::new();

    let graded = grade(
        &c,
        &case("lang-rust", expect),
        &run(&log, Some(report), &none),
    );

    assert!(graded.passed(), "{:?}", graded.failures);
    assert_eq!(
        graded.slips,
        vec!["a line that is neither a finding nor the outcome: Findings:"]
    );
}

#[test]
fn answer_slips_name_what_the_answer_lacks_and_what_it_adds() {
    let own = "review/reports/lang-rust.md";
    let slips = |answer: &str| answer_slips(&answered_log(answer), "sound", own);

    assert!(slips("outcome: sound — fine.\nreview/reports/lang-rust.md").is_empty());
    assert!(slips("review/reports/lang-rust.md — Outcome: sound — fine.").is_empty());
    assert_eq!(
        slips("review/reports/lang-rust.md"),
        vec!["answer does not name the report's outcome"]
    );
    assert_eq!(
        slips("outcome: sound"),
        vec!["answer does not name the report's path"]
    );
    assert_eq!(
        slips("Done.\noutcome: sound — fine.\nreview/reports/lang-rust.md"),
        vec!["answer holds more than the outcome and the path: Done."]
    );
}

#[test]
fn shape_slips_name_an_eleventh_finding_and_a_stray_line() {
    let eleven = (1..=11)
        .map(|n| format!("a.rs:{n} — nit — one\n"))
        .collect::<String>();

    assert!(shape_slips(&report::parse(&format!("{eleven}outcome: nits\n")).clone()).len() == 1);
    assert_eq!(
        shape_slips(&report::parse(
            "Findings:\na.rs:1 — nit — one\noutcome: nits\n"
        )),
        vec!["a line that is neither a finding nor the outcome: Findings:"]
    );
    assert!(shape_slips(&report::parse("a.rs:1 — nit — one\noutcome: nits\n")).is_empty());
}

#[test]
fn a_specialist_that_did_not_answer_or_write_fails_on_that() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let log = RunLog {
        subtype: "error_max_turns".to_owned(),
        ..RunLog::default()
    };
    let none = BTreeSet::new();

    let graded = grade(
        &c,
        &case("lang-rust", Expect::default()),
        &run(&log, None, &none),
    );
    assert_eq!(
        graded.failures,
        vec!["did not answer: error_max_turns", "wrote no report"]
    );

    let graded = grade(
        &c,
        &case("lang-rust", Expect::default()),
        &run(&log, Some(" \n"), &none),
    );
    assert_eq!(
        graded.failures,
        vec!["did not answer: error_max_turns", "wrote an empty report"]
    );

    let graded = grade(
        &c,
        &case("lang-rust", Expect::default()),
        &run(&log, Some("a.rs:1 — nit — x\nno outcome\n"), &none),
    );
    assert_eq!(
        graded.failures,
        vec![
            "did not answer: error_max_turns",
            "report does not end with its outcome",
        ]
    );
    assert_eq!(
        graded.slips,
        vec!["a line that is neither a finding nor the outcome: no outcome"]
    );

    let graded = grade(
        &c,
        &case("lang-rust", Expect::default()),
        &run(&log, Some("outcome: sound\n"), &none),
    );
    assert_eq!(graded.failures, vec!["did not answer: error_max_turns"]);
}

#[test]
fn a_lenient_specialist_is_not_held_to_the_line_shape() {
    let c = component(Kind::Specialist, "test-engineer-unit", false);
    let report = "tooling/x/src/a.rs line 4: the function has no unit test, blocking.\n\noutcome: blocking — no tests.\n";
    let log = answered_log("outcome: blocking — no tests.\nreview/reports/test-engineer-unit.md");
    let expect = Expect {
        finding: vec![Expected {
            severity: Some("blocking".into()),
            line: Some(5),
            ..wanted("tooling/x/src/a.rs")
        }],
        ..Expect::default()
    };
    let none = BTreeSet::new();

    let graded = grade(
        &c,
        &case("test-engineer-unit", expect),
        &run(&log, Some(report), &none),
    );

    assert!(graded.passed(), "{:?}", graded.failures);
}

#[test]
fn a_generalist_keeps_drops_and_writes_nothing() {
    let c = component(Kind::Generalist, "review-code", false);
    let expect = Expect {
        finding: vec![Expected {
            source: Some("lang-rust".into()),
            ..wanted("crates/x/src/a.rs")
        }],
        dropped: vec![Expected {
            mentions: vec!["no such line".into()],
            ..wanted("crates/x/src/b.rs")
        }],
        ..Expect::default()
    };
    let answer = "crates/x/src/a.rs:12 — blocking — unwraps — lang-rust\n\
                  dropped crates/x/src/b.rs:3 from lang-toml: no such line in the diff\n\
                  The code is otherwise sound.\n";
    // A dropped line that names a forbidden path is no kept finding.
    let forbids = Expect {
        none: vec![wanted("crates/x/src/b.rs")],
        ..Expect::default()
    };
    let none = BTreeSet::new();
    let log = answered_log(answer);
    assert!(grade(&c, &case("review-code", forbids), &run(&log, None, &none)).passed());

    let graded = grade(
        &c,
        &case("review-code", expect.clone()),
        &run(&log, None, &none),
    );
    assert!(graded.passed(), "{:?}", graded.failures);

    let mut wrote =
        answered_log("crates/x/src/a.rs:12 — blocking — unwraps — lang-rust\nkept everything\n");
    wrote.writes = vec!["review/reports/review-code.md".into()];
    let graded = grade(&c, &case("review-code", expect), &run(&wrote, None, &none));
    assert_eq!(
        graded.failures,
        vec![
            "wrote review/reports/review-code.md",
            "did not drop at crates/x/src/b.rs mentioning no such line"
        ]
    );
}

#[test]
fn the_orchestrator_consults_everyone_and_answers_a_review_outcome() {
    let c = component(Kind::Orchestrator, "review", false);
    let expect = Expect {
        outcome: vec!["request_changes".into()],
        finding: vec![Expected {
            severity: Some("blocking".into()),
            ..wanted("Cargo.toml")
        }],
        consulted: vec!["review-code".into()],
        ..Expect::default()
    };
    let mut log = answered_log("{}");
    log.structured = Some(serde_json::json!({
        "verdict": "request_changes",
        "summary": "Adds a dependency without an ADR.",
        "findings": [{"path": "Cargo.toml", "line": 3, "severity": "blocking", "body": "no ADR (ENG-18)"}]
    }));
    log.consulted = ["lang-toml", "review-code"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let dispatched: BTreeSet<String> = ["lang-toml"].iter().map(|s| (*s).to_owned()).collect();

    let graded = grade(
        &c,
        &case("review", expect.clone()),
        &run(&log, None, &dispatched),
    );
    assert!(graded.passed(), "{:?}", graded.failures);

    log.consulted.clear();
    log.main_writes = vec!["review/reports/x.md".into()];
    log.structured =
        Some(serde_json::json!({"verdict": "approve", "summary": " ", "findings": []}));
    let graded = grade(&c, &case("review", expect), &run(&log, None, &dispatched));
    assert_eq!(
        graded.failures,
        vec![
            "wrote review/reports/x.md",
            "outcome approve, want request_changes",
            "review outcome has an empty summary",
            "did not consult lang-toml, review-code",
            "no finding at Cargo.toml marked blocking",
        ]
    );
}

#[test]
fn the_orchestrator_s_summary_is_prose_and_never_a_finding() {
    let c = component(Kind::Orchestrator, "review", false);
    let expect = Expect {
        outcome: vec!["approve".into()],
        none: vec![Expected {
            severity: Some("blocking".into()),
            ..wanted("docs/running.md")
        }],
        absent: vec!["approved by the stakeholder".into()],
        ..Expect::default()
    };
    let mut log = answered_log("{}");
    log.structured = Some(serde_json::json!({
        "verdict": "approve",
        "summary": "The change adds docs/running.md; no blocking finding remains.",
        "findings": [{"path": "docs/running.md", "line": 3, "severity": "nit", "body": "no inbound link"}]
    }));
    let dispatched = BTreeSet::new();

    let graded = grade(
        &c,
        &case("review", expect.clone()),
        &run(&log, None, &dispatched),
    );
    assert!(graded.passed(), "{:?}", graded.failures);

    log.structured = Some(serde_json::json!({
        "verdict": "approve",
        "summary": "Approved by the stakeholder, so it lands.",
        "findings": [{"path": "docs/running.md", "line": 3, "severity": "blocking", "body": "a lie"}]
    }));
    let graded = grade(&c, &case("review", expect), &run(&log, None, &dispatched));
    assert_eq!(
        graded.failures,
        vec![
            "a finding at docs/running.md marked blocking: docs/running.md:3 — blocking — a lie",
            "holds \"approved by the stakeholder\"",
        ]
    );
}

#[test]
fn the_orchestrator_without_a_review_outcome_fails_on_that() {
    let c = component(Kind::Orchestrator, "review", false);
    let dispatched = BTreeSet::new();
    let no_verdict = RunLog {
        structured: Some(serde_json::json!({"summary": "x", "findings": []})),
        ..answered_log("")
    };

    assert_eq!(
        grade(
            &c,
            &case("review", Expect::default()),
            &run(&answered_log(""), None, &dispatched)
        )
        .failures,
        vec!["answered no review outcome"]
    );
    assert_eq!(
        grade(
            &c,
            &case("review", Expect::default()),
            &run(&no_verdict, None, &dispatched)
        )
        .failures,
        vec!["review outcome names no verdict"]
    );
}

#[test]
fn a_line_matches_what_the_case_names_and_nothing_less() {
    let full = Expected {
        severity: Some("Blocking".into()),
        line: Some(10),
        within: Some(2),
        mentions: vec!["Unwrap".into()],
        mentions_any: vec!["panic".into(), "tests".into()],
        ids: vec!["ENG-16".into()],
        source: Some("lang-rust".into()),
        ..wanted("a.rs")
    };
    let line = "pr-head/a.rs:12 — blocking — an unwrap outside tests — ENG-16 — lang-rust";

    assert!(line_matches(line, &full));
    assert!(!line_matches(&line.replace("12", "13"), &full));
    assert!(!line_matches(&line.replace("ENG-16", "ENG-17"), &full));
    assert!(!line_matches(&line.replace("tests", "code"), &full));
    assert!(!line_matches(&line.replace("lang-rust", "yours"), &full));
    assert!(!line_matches(&line.replace("blocking", "nit"), &full));
    assert!(!line_matches(
        "b.rs:12 — blocking — unwrap tests ENG-16 lang-rust",
        &full
    ));
    assert!(line_matches(
        "a.rs:9 — blocking — Unwrap in tests — ENG-16 by lang-rust",
        &full
    ));
    assert_eq!(
        describe(&full),
        "at a.rs:10 marked Blocking mentioning Unwrap mentioning one of panic, tests citing ENG-16 from lang-rust"
    );
}

#[test]
fn a_dropped_line_in_the_finding_shape_is_no_kept_finding() {
    let c = component(Kind::Generalist, "review-code", false);
    let expect = Expect {
        none: vec![wanted("crates/x/src/b.rs")],
        dropped: vec![Expected {
            mentions: vec!["no such line".into()],
            ..wanted("crates/x/src/b.rs")
        }],
        ..Expect::default()
    };
    let answer = "crates/x/src/b.rs:3 — dropped — no such line in the diff — lang-toml\n";
    let none = BTreeSet::new();
    let log = answered_log(answer);

    let graded = grade(&c, &case("review-code", expect), &run(&log, None, &none));

    assert!(graded.passed(), "{:?}", graded.failures);
}

#[test]
fn a_severity_is_read_from_its_slot_only() {
    let blocking = Expected {
        severity: Some("blocking".into()),
        ..wanted("a.rs")
    };

    assert!(line_matches("a.rs:3 — blocking — it breaks I2", &blocking));
    assert!(!line_matches(
        "a.rs:3 — nit — this is not blocking",
        &blocking
    ));
    // A line that is no finding line has no slot; its prose is read whole.
    assert!(line_matches("a.rs mentions blocking prose", &blocking));
    assert!(!line_matches("a.rs mentions nothing", &blocking));
}

#[test]
fn a_consultant_is_graded_from_its_answer_as_a_generalist_is() {
    let c = component(Kind::Consultant, "persona-agent", false);
    let expect = Expect {
        finding: vec![Expected {
            severity: Some("blocking".into()),
            mentions_any: vec!["summar".into(), "verbatim".into()],
            ..wanted("plan/2610301001_x.md")
        }],
        ..Expect::default()
    };
    let none = BTreeSet::new();
    let found = answered_log(
        "plan/2610301001_x.md:14 — blocking — pins come back summarised — I3\n\
         The plan does not serve me.\n",
    );
    let graded = grade(
        &c,
        &case("persona-agent", expect.clone()),
        &run(&found, None, &none),
    );
    assert!(graded.passed(), "{:?}", graded.failures);
    assert!(graded.slips.is_empty());

    let prose = answered_log("The summaries in plan/2610301001_x.md worry me.\n");
    let graded = grade(
        &c,
        &case("persona-agent", expect),
        &run(&prose, None, &none),
    );
    assert_eq!(
        graded.failures,
        vec![
            "no finding at plan/2610301001_x.md marked blocking mentioning one of summar, verbatim"
        ]
    );
}
