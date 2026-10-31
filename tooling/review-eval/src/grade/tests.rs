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

const REPORT: &str = "crates/x/src/a.rs:12 — if not addressed: an unwrap outside tests panics — ENG-16\n\noutcome: findings — it unwraps.\n";

fn run<'a>(log: &'a RunLog, report: Option<&'a str>) -> Run<'a> {
    Run { log, report }
}

#[test]
fn a_specialist_run_that_does_everything_right_passes() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        outcome: vec!["findings".into()],
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
    let mut log = answered_log("outcome: findings — it unwraps.\nreview/reports/lang-rust.md\n");
    log.writes = vec!["/work/main/review/reports/lang-rust.md".into()];

    let graded = grade(&c, &case("lang-rust", expect), &run(&log, Some(REPORT)));

    assert_eq!(graded, Graded::default());
    assert!(graded.passed());
}

#[test]
fn a_specialist_run_names_each_failed_assertion() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        outcome: vec!["none".into()],
        finding: vec![Expected {
            mentions: vec!["panic".into()],
            ..wanted("crates/x/src/b.rs")
        }],
        none: vec![wanted("crates/x/src/a.rs")],
        absent: vec!["everything".into()],
        ..Expect::default()
    };
    let mut log = answered_log("I looked.\noutcome: none\n");
    log.writes = vec!["review/reports/lang-toml.md".into()];
    let eleven = (1..=11)
        .map(|n| format!("crates/x/src/a.rs:{n} — if not addressed: one breaks — I2\n"))
        .collect::<String>()
        + "I read everything.\noutcome: findings\n";

    let graded = grade(&c, &case("lang-rust", expect), &run(&log, Some(&eleven)));

    assert_eq!(
        graded.failures,
        vec![
            "wrote review/reports/lang-toml.md",
            "outcome findings, want none",
            "no finding at crates/x/src/b.rs mentioning panic",
            "a finding at crates/x/src/a.rs: crates/x/src/a.rs:1 — if not addressed: one breaks — I2",
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
        outcome: vec!["findings".into()],
        finding: vec![Expected {
            mentions: vec!["unwrap".into()],
            line: Some(22),
            ..wanted("crates/x/src/a.rs")
        }],
        ..Expect::default()
    };
    let report = "Findings:\n\n- crates/x/src/a.rs:20-23 — if not addressed: an unwrap panics — ENG-16\n\noutcome: findings — it unwraps.\n";
    let log =
        answered_log("outcome: findings — unwraps, see the report.\nreview/reports/lang-rust.md\n");

    let graded = grade(&c, &case("lang-rust", expect), &run(&log, Some(report)));

    assert!(graded.passed(), "{:?}", graded.failures);
    assert_eq!(
        graded.slips,
        vec!["a line that is neither a finding nor the outcome: Findings:"]
    );
}

#[test]
fn answer_slips_name_what_the_answer_lacks_and_what_it_adds() {
    let own = "review/reports/lang-rust.md";
    let slips = |answer: &str| answer_slips(&answered_log(answer), "none", own);

    assert!(slips("outcome: none — fine.\nreview/reports/lang-rust.md").is_empty());
    assert!(slips("review/reports/lang-rust.md — Outcome: none — fine.").is_empty());
    assert_eq!(
        slips("review/reports/lang-rust.md"),
        vec!["answer does not name the report's outcome"]
    );
    assert_eq!(
        slips("outcome: none"),
        vec!["answer does not name the report's path"]
    );
    assert_eq!(
        slips("Done.\noutcome: none — fine.\nreview/reports/lang-rust.md"),
        vec!["answer holds more than the outcome and the path: Done."]
    );
}

#[test]
fn shape_slips_name_an_eleventh_finding_and_a_stray_line() {
    let eleven = (1..=11)
        .map(|n| format!("a.rs:{n} — request: add a rule — one\n"))
        .collect::<String>();
    let one = "a.rs:1 — if not addressed: one breaks — I2\n";

    assert!(
        shape_slips(&report::parse(&format!("{eleven}outcome: findings\n")).clone()).len() == 1
    );
    assert_eq!(
        shape_slips(&report::parse(&format!(
            "Findings:\n{one}outcome: findings\n"
        ))),
        vec!["a line that is neither a finding nor the outcome: Findings:"]
    );
    assert!(shape_slips(&report::parse(&format!("{one}outcome: findings\n"))).is_empty());
}

#[test]
fn a_specialist_that_did_not_answer_or_write_fails_on_that() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let log = RunLog {
        subtype: "error_max_turns".to_owned(),
        ..RunLog::default()
    };

    let graded = grade(&c, &case("lang-rust", Expect::default()), &run(&log, None));
    assert_eq!(
        graded.failures,
        vec!["did not answer: error_max_turns", "wrote no report"]
    );

    let graded = grade(
        &c,
        &case("lang-rust", Expect::default()),
        &run(&log, Some(" \n")),
    );
    assert_eq!(
        graded.failures,
        vec!["did not answer: error_max_turns", "wrote an empty report"]
    );

    let graded = grade(
        &c,
        &case("lang-rust", Expect::default()),
        &run(&log, Some("a.rs:1 — nit — x\nno outcome\n")),
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
        &run(&log, Some("outcome: none\n")),
    );
    assert_eq!(graded.failures, vec!["did not answer: error_max_turns"]);
}

#[test]
fn a_lenient_specialist_is_not_held_to_the_line_shape() {
    let c = component(Kind::Specialist, "test-engineer-unit", false);
    let report = "tooling/x/src/a.rs line 4: the function has no unit test.\n\
                  tooling/x/src/a.rs:4 — request: add a floor per module — none holds it\n\n\
                  outcome: findings — no tests.\n";
    let log = answered_log("outcome: findings — no tests.\nreview/reports/test-engineer-unit.md");
    let expect = Expect {
        finding: vec![Expected {
            line: Some(5),
            mentions: vec!["no unit test".into()],
            ..wanted("tooling/x/src/a.rs")
        }],
        ..Expect::default()
    };

    let graded = grade(
        &c,
        &case("test-engineer-unit", expect),
        &run(&log, Some(report)),
    );

    assert!(graded.passed(), "{:?}", graded.failures);
}

#[test]
fn a_specialist_report_the_review_would_refuse_fails_its_run() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let log = answered_log("outcome: findings — x.\nreview/reports/lang-rust.md");
    for (report, problem) in [
        (
            "a.rs:1 — blocking — an unwrap — ENG-16\noutcome: findings — x.\n",
            "has a report whose line 1 is neither a linked finding nor a requirement request",
        ),
        (
            "a.rs:1 — if not addressed: a panic — ENG-16\noutcome: none — x.\n",
            "has a report whose outcome is none but which holds findings",
        ),
    ] {
        let graded = grade(
            &c,
            &case("lang-rust", Expect::default()),
            &run(&log, Some(report)),
        );

        assert_eq!(graded.failures, vec![format!("report refused: {problem}")]);
    }
}

#[test]
fn a_specialist_s_line_is_matched_by_its_form_and_never_by_a_severity_word() {
    let form = |form: &str| Expected {
        form: Some(form.into()),
        ..wanted("a.rs")
    };
    let severity = |severity: &str| Expected {
        severity: Some(severity.into()),
        ..wanted("a.rs")
    };
    let linked = "a.rs:3 — if not addressed: a pin is lost — I3";
    let request = "a.rs:3 — request: change I3 — it reads two ways";
    let weighed = "a.rs:3 — blocking — a pin is lost — I3";
    let s = Kind::Specialist;

    assert!(line_matches(linked, &form("linked"), s));
    assert!(!line_matches(request, &form("linked"), s));
    assert!(line_matches(request, &form("request"), s));
    assert!(!line_matches(linked, &form("request"), s));
    // A case written while specialists weighed: blocking wants a
    // linked finding, a nit either form, and neither a severity word.
    assert!(line_matches(linked, &severity("blocking"), s));
    assert!(!line_matches(request, &severity("blocking"), s));
    assert!(line_matches(request, &severity("nit"), s));
    assert!(line_matches(linked, &severity("nit"), s));
    assert!(!line_matches(weighed, &severity("blocking"), s));
    assert!(!line_matches(weighed, &severity("nit"), s));
    assert!(line_matches(weighed, &wanted("a.rs"), s));
    // A generalist's line has no form.
    assert!(!line_matches(weighed, &form("linked"), Kind::Generalist));
    assert_eq!(describe(&form("request")), "at a.rs as a request line");
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
    let log = answered_log(answer);
    assert!(grade(&c, &case("review-code", forbids), &run(&log, None)).passed());

    let graded = grade(&c, &case("review-code", expect.clone()), &run(&log, None));
    assert!(graded.passed(), "{:?}", graded.failures);

    let mut wrote =
        answered_log("crates/x/src/a.rs:12 — blocking — unwraps — lang-rust\nkept everything\n");
    wrote.writes = vec!["review/reports/review-code.md".into()];
    let graded = grade(&c, &case("review-code", expect), &run(&wrote, None));
    assert_eq!(
        graded.failures,
        vec![
            "wrote review/reports/review-code.md",
            "did not drop at crates/x/src/b.rs mentioning no such line"
        ]
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

    let g = Kind::Generalist;
    assert!(line_matches(line, &full, g));
    assert!(!line_matches(&line.replace("12", "13"), &full, g));
    assert!(!line_matches(&line.replace("ENG-16", "ENG-17"), &full, g));
    assert!(!line_matches(&line.replace("tests", "code"), &full, g));
    assert!(!line_matches(&line.replace("lang-rust", "yours"), &full, g));
    assert!(!line_matches(&line.replace("blocking", "nit"), &full, g));
    assert!(!line_matches(
        "b.rs:12 — blocking — unwrap tests ENG-16 lang-rust",
        &full,
        g
    ));
    assert!(line_matches(
        "a.rs:9 — blocking — Unwrap in tests — ENG-16 by lang-rust",
        &full,
        g
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
    let log = answered_log(answer);

    let graded = grade(&c, &case("review-code", expect), &run(&log, None));

    assert!(graded.passed(), "{:?}", graded.failures);
}

#[test]
fn a_kept_finding_that_mentions_dropping_is_no_dropped_line() {
    let c = component(Kind::Generalist, "review-code", false);
    let expect = Expect {
        dropped: vec![Expected {
            mentions: vec!["no such line".into()],
            ..wanted("crates/x/src/b.rs")
        }],
        ..Expect::default()
    };
    let answer = "crates/x/src/b.rs:3 — nit — drops the no such line check — lang-toml\n";
    let log = answered_log(answer);

    let graded = grade(&c, &case("review-code", expect), &run(&log, None));

    assert_eq!(
        graded.failures,
        vec!["did not drop at crates/x/src/b.rs mentioning no such line"]
    );
}

#[test]
fn a_severity_is_read_from_its_slot_only() {
    let blocking = Expected {
        severity: Some("blocking".into()),
        ..wanted("a.rs")
    };

    let g = Kind::Generalist;
    assert!(line_matches(
        "a.rs:3 — blocking — it breaks I2",
        &blocking,
        g
    ));
    assert!(!line_matches(
        "a.rs:3 — nit — this is not blocking",
        &blocking,
        g
    ));
    // A line that is no finding line has no slot; its prose is read whole.
    assert!(line_matches("a.rs mentions blocking prose", &blocking, g));
    assert!(!line_matches("a.rs mentions nothing", &blocking, g));
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
    let found = answered_log(
        "plan/2610301001_x.md:14 — blocking — pins come back summarised — I3\n\
         The plan does not serve me.\n",
    );
    let graded = grade(
        &c,
        &case("persona-agent", expect.clone()),
        &run(&found, None),
    );
    assert!(graded.passed(), "{:?}", graded.failures);
    assert!(graded.slips.is_empty());

    let prose = answered_log("The summaries in plan/2610301001_x.md worry me.\n");
    let graded = grade(&c, &case("persona-agent", expect), &run(&prose, None));
    assert_eq!(
        graded.failures,
        vec![
            "no finding at plan/2610301001_x.md marked blocking mentioning one of summar, verbatim"
        ]
    );
}
