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

/// A specialist's log: `texts` stated as it went, then `answer`, which
/// its last text block repeats as the harness logs it.
fn stated_log(texts: &[&str], answer: &str) -> RunLog {
    let mut log = answered_log(answer);
    log.texts = texts.iter().map(|t| (*t).to_owned()).collect();
    log.texts.push(answer.to_owned());
    log
}

const UNWRAP: &str =
    "crates/x/src/a.rs:12 — if not addressed: an unwrap outside tests panics — ENG-16";

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
        stated: vec![wanted("crates/x/src/a.rs")],
        absent: vec!["approved".into()],
        ..Expect::default()
    };
    let log = stated_log(
        &["Reading the diff.", UNWRAP, "Checking the tests."],
        "outcome: findings — it unwraps.\n",
    );

    let graded = grade(&c, &case("lang-rust", expect), &log);

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
        stated: vec![wanted("crates/x/src/b.rs")],
        withdrawn: vec![wanted("crates/x/src/a.rs")],
        absent: vec!["everything".into()],
        ..Expect::default()
    };
    let eleven = (1..=11)
        .map(|n| format!("crates/x/src/a.rs:{n} — if not addressed: everything breaks — I2"))
        .collect::<Vec<_>>();
    let texts: Vec<&str> = eleven.iter().map(String::as_str).collect();
    let mut log = stated_log(&texts, "I looked.\noutcome: findings\n");
    log.writes = vec!["review/reports/lang-toml.md".into()];

    let graded = grade(&c, &case("lang-rust", expect), &log);

    assert_eq!(
        graded.failures,
        vec![
            "wrote review/reports/lang-toml.md",
            "outcome findings, want none",
            "no finding at crates/x/src/b.rs mentioning panic",
            "a finding at crates/x/src/a.rs: crates/x/src/a.rs:1 — if not addressed: everything breaks — I2",
            "holds \"everything\"",
            "did not state before its answer at crates/x/src/b.rs",
            "did not withdraw or revise at crates/x/src/a.rs",
        ]
    );
    assert_eq!(
        graded.slips,
        vec![
            "answer holds more than the outcome line: I looked.",
            "11 findings, at most 10",
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
    let log = stated_log(
        &["Findings:\n\n- crates/x/src/a.rs:20-23 — if not addressed: an unwrap panics — ENG-16"],
        "Done.\noutcome: findings — it unwraps.\n",
    );

    let graded = grade(&c, &case("lang-rust", expect), &log);

    assert!(graded.passed(), "{:?}", graded.failures);
    assert_eq!(
        graded.slips,
        vec!["answer holds more than the outcome line: Done."]
    );
}

#[test]
fn a_finding_stated_only_in_the_answer_was_not_stated_before_it() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        finding: vec![wanted("crates/x/src/a.rs")],
        stated: vec![wanted("crates/x/src/a.rs")],
        ..Expect::default()
    };
    let answer = format!("{UNWRAP}\noutcome: findings — it unwraps.");
    let log = stated_log(&["Reading."], &answer);

    let graded = grade(&c, &case("lang-rust", expect), &log);

    assert_eq!(
        graded.failures,
        vec!["did not state before its answer at crates/x/src/a.rs"]
    );
}

#[test]
fn a_withdrawn_finding_leaves_the_report_and_counts_as_withdrawn() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let expect = Expect {
        outcome: vec!["none".into()],
        none: vec![wanted("crates/x/src/a.rs")],
        withdrawn: vec![Expected {
            mentions: vec!["test".into()],
            ..wanted("crates/x/src/a.rs")
        }],
        ..Expect::default()
    };
    let log = stated_log(
        &[
            UNWRAP,
            "withdrawn: crates/x/src/a.rs:12 — the function is a test's",
        ],
        "outcome: none — fine.",
    );
    assert!(grade(&c, &case("lang-rust", expect.clone()), &log).passed());

    let revised = stated_log(
        &[
            UNWRAP,
            "revised: crates/x/src/a.rs:12 — request: add a rule on test unwraps — none says so",
        ],
        "outcome: findings — a request.",
    );
    let graded = grade(&c, &case("lang-rust", expect), &revised);
    assert_eq!(
        graded.failures,
        vec![
            "outcome findings, want none",
            "a finding at crates/x/src/a.rs: crates/x/src/a.rs:12 — request: add a rule on test unwraps — none says so",
        ]
    );
}

#[test]
fn answer_slips_name_the_first_line_that_is_no_outcome_line() {
    let slips = |answer: &str| answer_slips(&answered_log(answer));

    assert!(slips("outcome: none — fine.").is_empty());
    assert!(slips("\n  **Outcome: none** — fine.\n\n").is_empty());
    assert_eq!(
        slips("Done.\noutcome: none — fine."),
        vec!["answer holds more than the outcome line: Done."]
    );
    assert_eq!(
        slips("outcome: none — fine.\nreview/reports/lang-rust.md"),
        vec!["answer holds more than the outcome line: review/reports/lang-rust.md"]
    );
}

#[test]
fn shape_slips_name_an_eleventh_finding() {
    let eleven = (1..=11)
        .map(|n| format!("a.rs:{n} — request: add a rule — one\n"))
        .collect::<String>();
    let one = "a.rs:1 — if not addressed: one breaks — I2\n";

    assert_eq!(
        shape_slips(&format!("{eleven}outcome: findings\n")),
        vec!["11 findings, at most 10"]
    );
    assert!(shape_slips(&format!("{one}outcome: findings\n")).is_empty());
}

#[test]
fn a_specialist_that_did_not_answer_fails_on_that_whatever_it_stated() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let log = RunLog {
        subtype: "error_max_turns".to_owned(),
        texts: vec![UNWRAP.to_owned()],
        writes: vec!["review/reports/lang-rust.md".to_owned()],
        ..RunLog::default()
    };

    let graded = grade(&c, &case("lang-rust", Expect::default()), &log);

    assert_eq!(
        graded.failures,
        vec![
            "did not answer: error_max_turns",
            "wrote review/reports/lang-rust.md"
        ]
    );
    assert!(graded.slips.is_empty());
}

#[test]
fn a_specialist_that_answered_no_outcome_line_fails_on_that() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let log = stated_log(&[UNWRAP], "I am done.\nreview/reports/lang-rust.md");

    let graded = grade(&c, &case("lang-rust", Expect::default()), &log);

    assert_eq!(graded.failures, vec!["answered no outcome line"]);
}

#[test]
fn a_lenient_specialist_is_not_held_to_the_line_count() {
    let c = component(Kind::Specialist, "test-engineer-unit", false);
    let twelve = (1..=12)
        .map(|n| {
            format!("tooling/x/src/a.rs:{n} — request: add a floor per module — none holds it")
        })
        .collect::<Vec<_>>();
    let texts: Vec<&str> = twelve.iter().map(String::as_str).collect();
    let log = stated_log(&texts, "outcome: findings — no tests.");
    let expect = Expect {
        finding: vec![Expected {
            line: Some(5),
            mentions: vec!["floor".into()],
            ..wanted("tooling/x/src/a.rs")
        }],
        ..Expect::default()
    };

    let graded = grade(&c, &case("test-engineer-unit", expect), &log);

    assert!(graded.passed(), "{:?}", graded.failures);
    assert!(graded.slips.is_empty());
}

#[test]
fn a_specialist_report_the_review_would_refuse_fails_its_run() {
    let c = component(Kind::Specialist, "lang-rust", true);
    for (texts, answer, problem) in [
        (
            vec!["a.rs:1 — blocking — an unwrap — ENG-16"],
            "outcome: findings — x.",
            "has a report whose line 1 is neither a linked finding nor a requirement request",
        ),
        (
            vec!["a.rs:1 — if not addressed: a panic — ENG-16"],
            "outcome: none — x.",
            "has a report whose outcome is none but which holds findings",
        ),
        (
            vec![
                "withdrawn: a.rs:1 — nothing was there",
                "revised: a.rs:2 — if not addressed: a panic — ENG-16",
            ],
            "outcome: none — x.",
            "has a report whose outcome is none but which holds findings",
        ),
    ] {
        let log = stated_log(&texts, answer);

        let graded = grade(&c, &case("lang-rust", Expect::default()), &log);

        assert_eq!(graded.failures, vec![format!("report refused: {problem}")]);
    }
}

#[test]
fn a_specialist_that_withdraws_what_it_only_checked_passes() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let log = stated_log(
        &["withdrawn: a.rs:1 — I checked it and it holds"],
        "outcome: none — x.",
    );
    let expect = Expect {
        outcome: vec!["none".into()],
        ..Expect::default()
    };

    let graded = grade(&c, &case("lang-rust", expect), &log);

    assert!(graded.passed(), "{:?}", graded.failures);
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
    assert!(grade(&c, &case("review-code", forbids), &log).passed());

    let graded = grade(&c, &case("review-code", expect.clone()), &log);
    assert!(graded.passed(), "{:?}", graded.failures);

    let mut wrote =
        answered_log("crates/x/src/a.rs:12 — blocking — unwraps — lang-rust\nkept everything\n");
    wrote.writes = vec!["review/reports/review-code.md".into()];
    let graded = grade(&c, &case("review-code", expect), &wrote);
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

    let graded = grade(&c, &case("review-code", expect), &log);

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

    let graded = grade(&c, &case("review-code", expect), &log);

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
    let graded = grade(&c, &case("persona-agent", expect.clone()), &found);
    assert!(graded.passed(), "{:?}", graded.failures);
    assert!(graded.slips.is_empty());

    let prose = answered_log("The summaries in plan/2610301001_x.md worry me.\n");
    let graded = grade(&c, &case("persona-agent", expect), &prose);
    assert_eq!(
        graded.failures,
        vec![
            "no finding at plan/2610301001_x.md marked blocking mentioning one of summar, verbatim"
        ]
    );
}

#[test]
fn a_consultant_s_line_in_a_specialist_s_form_is_kept_and_matched_by_its_form() {
    let c = component(Kind::Consultant, "persona-agent", false);
    let at = |form: &str| Expected {
        form: Some(form.into()),
        ..wanted("plan/2610301001_x.md")
    };
    let answer = answered_log(
        "plan/2610301001_x.md:14 — if not addressed: pins come back summarised — I3\n\
         plan/2610301001_x.md:20 — request: add a rule that a digest names its pins — none says so\n\
         outcome: findings — the plan does not serve me\n",
    );
    let expect = Expect {
        finding: vec![at("linked"), at("request")],
        ..Expect::default()
    };
    let graded = grade(&c, &case("persona-agent", expect), &answer);
    assert!(graded.passed(), "{:?}", graded.failures);

    // A linked line is what a case written while personas weighed
    // reads its blocking finding as; another consultant keeps its
    // severity word.
    let s = Kind::Consultant;
    let linked = "a.rs:3 — if not addressed: a pin is lost — I3";
    let blocking = Expected {
        severity: Some("blocking".into()),
        ..wanted("a.rs")
    };
    assert!(line_matches(linked, &blocking, s));
    assert!(line_matches(
        "a.rs:3 — blocking — a pin is lost",
        &blocking,
        s
    ));
    assert!(!line_matches("a.rs:3 — nit — a pin is lost", &blocking, s));
    assert!(!line_matches(linked, &at("request"), s));
}

#[test]
fn a_generalist_keeping_more_nits_than_the_case_allows_fails() {
    let c = component(Kind::Generalist, "review-code", false);
    let expect = Expect {
        nits: Some(2),
        ..Expect::default()
    };
    let three = "a.rs:1 — nit — one — lang-rust\n\
                 a.rs:2 — nit — two — lang-rust\n\
                 a.rs:3 — blocking — three — lang-rust\n\
                 a.rs:4 — dropped — four — lang-rust\n\
                 a.rs:5 — nit — five — lang-rust\n";

    let graded = grade(&c, &case("review-code", expect.clone()), &answered_log(three));
    assert_eq!(graded.failures, vec!["kept 3 nits, want at most 2"]);

    let two = three.replacen("a.rs:5 — nit", "a.rs:5 — blocking", 1);
    let graded = grade(&c, &case("review-code", expect), &answered_log(&two));
    assert!(graded.passed(), "{:?}", graded.failures);
}
