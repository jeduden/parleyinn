use super::*;

fn ids(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

const ANSWER: &str = "outcome: findings — one sentence.\nreview/reports/lang-rust.md";

#[test]
fn a_linked_finding_reads_its_place_consequence_and_ids() {
    let got = line("crates/x/src/a.rs:12 — if not addressed: a hook panics — ENG-04, I9");

    assert_eq!(
        got,
        Line::Linked(Linked {
            path: "crates/x/src/a.rs".to_owned(),
            line: 12,
            consequence: "a hook panics".to_owned(),
            ids: vec!["ENG-04".to_owned(), "I9".to_owned()],
        })
    );
}

#[test]
fn a_linked_finding_tolerates_markup_dashes_and_a_dash_in_its_consequence() {
    let got =
        line("- `a/b.md:3-5` -- If not addressed: the pin is lost -- or summarised -- `PIN-02`.");

    let Line::Linked(l) = got else {
        panic!("not linked: {got:?}");
    };
    assert_eq!((l.path.as_str(), l.line), ("a/b.md", 3));
    assert_eq!(l.consequence, "the pin is lost — or summarised");
    assert_eq!(l.ids, ["PIN-02"]);
}

#[test]
fn a_request_reads_its_change_requirement_and_motivation() {
    let got = line(
        "docs/srs/10-engineering-quality.md:59 — request: add a cap on report length — reports grow — a lot",
    );

    assert_eq!(
        got,
        Line::Request(Request {
            path: "docs/srs/10-engineering-quality.md".to_owned(),
            line: 59,
            change: "add".to_owned(),
            requirement: "a cap on report length".to_owned(),
            motivation: "reports grow — a lot".to_owned(),
        })
    );
    for change in ["Change", "remove"] {
        let Line::Request(r) = line(&format!("a.rs:1 — Request: {change} ENG-28 — why")) else {
            panic!("{change}");
        };
        assert_eq!(r.change, change.to_lowercase());
    }
}

#[test]
fn a_finding_line_in_neither_form_is_malformed() {
    for text in [
        "a.rs:1 — blocking — an unwrap — ENG-16",
        "a.rs:1 — nit — x",
        "a.rs:1 — if not addressed: a hook panics",
        "a.rs:1 — if not addressed: — ENG-04",
        "a.rs:1 — if not addressed: a hook panics — ENG-04 and I9",
        "a.rs:1 — if not addressed: a hook panics — eng-04",
        "a.rs:1 — if not addressed: a hook panics — -04",
        "a.rs:1 — if not addressed: a hook panics — ENG-",
        "a.rs:1 — if not addressed: a hook panics — I",
        "a.rs:1 — if not addressed: a hook panics — ",
        "a.rs:1 — request: add a cap",
        "a.rs:1 — request: add a cap — ",
        "a.rs:1 — request: add — why",
        "a.rs:1 — request: rename ENG-28 — why",
        "a.rs:1 — requested: add ENG-28 — why",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn a_line_with_no_place_is_prose() {
    for text in [
        "",
        "# lang-rust",
        "I read the diff.",
        "outcome: findings — one sentence.",
        "I read line: 3 — and it is fine",
        "a.rs — if not addressed: x — ENG-04",
        "a.rs:x — if not addressed: x — ENG-04",
        "a.rs:12",
    ] {
        assert_eq!(line(text), Line::Prose, "{text:?}");
    }
}

#[test]
fn the_invariants_are_i1_to_i10() {
    let all = invariants();

    assert_eq!(all.len(), 10);
    assert!(all.contains("I1") && all.contains("I10"));
    assert!(!all.contains("I11"));
}

#[test]
fn an_outcome_line_names_findings_or_none_in_any_case_and_markup() {
    for (text, want) in [
        (
            "x\noutcome: findings — y\n\n",
            Some(("outcome: findings — y", FINDINGS)),
        ),
        ("Outcome: **none**. y", Some(("Outcome: **none**. y", NONE))),
        ("outcome: `none`", Some(("outcome: `none`", NONE))),
        ("outcome: none — y\nmore", None),
        ("outcome: nothing", None),
        ("outcome: blocking — y", None),
        ("outcome: sound — y", None),
        ("outcome:", None),
        ("outcomes", None),
        ("", None),
    ] {
        assert_eq!(outcome_line(text), want, "{text:?}");
    }
}

#[test]
fn a_report_check_takes_well_formed_findings_and_requests() {
    let report = "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
                  I read the hook.\n\n\
                  a.rs:2 — request: add a deadline rule — none says so\n\
                  outcome: findings — one sentence.\n";

    assert_eq!(report_check(Some(report), Some(ANSWER), None), None);
    assert_eq!(
        report_check(Some(report), Some(ANSWER), Some(&ids(&["ENG-04"]))),
        None
    );
    let none = "# lang-rust\nI read the diff.\noutcome: none — nothing to say.";
    assert_eq!(
        report_check(
            Some(none),
            Some("outcome: none — nothing to say."),
            Some(&ids(&[]))
        ),
        None
    );
}

#[test]
fn a_report_check_refuses_what_the_gate_refuses() {
    let finding = "a.rs:1 — if not addressed: a hook panics — ENG-04\n";
    let ending = "outcome: findings — one sentence.";
    let report = format!("{finding}{ending}");
    for (report, answer, want) in [
        (None, Some(ANSWER), Problem::Missing),
        (Some(" \n".to_owned()), Some(ANSWER), Problem::Empty),
        (Some(finding.to_owned()), Some(ANSWER), Problem::NoOutcome),
        (
            Some(format!("x\na.rs:1 — nit — x\n{ending}")),
            Some(ANSWER),
            Problem::Malformed { line: 2 },
        ),
        (
            Some(format!("{finding}outcome: none — one sentence.")),
            Some("outcome: none — one sentence."),
            Problem::NoneWithFindings,
        ),
        (
            Some(format!("I read it.\n{ending}")),
            Some(ANSWER),
            Problem::FindingsWithNone,
        ),
        (Some(report.clone()), None, Problem::AnswerDiffers),
        (
            Some(report.clone()),
            Some("outcome: none — one sentence."),
            Problem::AnswerDiffers,
        ),
    ] {
        assert_eq!(
            report_check(report.as_deref(), answer, None),
            Some(want.clone()),
            "{report:?}"
        );
    }
}

#[test]
fn a_report_check_with_the_known_ids_refuses_an_id_no_requirement_has() {
    let report = "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
                  b.rs:9 — if not addressed: a pin is lost — I3, PIN-99\n\
                  outcome: findings — one sentence.";
    let known: BTreeSet<String> = ids(&["ENG-04"]).into_iter().chain(invariants()).collect();

    assert_eq!(
        report_check(Some(report), Some(ANSWER), Some(&known)),
        Some(Problem::UnknownId {
            line: 2,
            id: "PIN-99".to_owned()
        })
    );
    assert_eq!(report_check(Some(report), Some(ANSWER), None), None);
}

#[test]
fn a_findings_check_refuses_a_report_without_an_outcome() {
    assert_eq!(
        findings_check("a.rs:1 — if not addressed: x — I2", None),
        Some(Problem::NoOutcome)
    );
}

#[test]
fn every_problem_reads_as_a_sentence_about_the_report() {
    for (problem, want) in [
        (Problem::Missing, "wrote no report"),
        (Problem::Empty, "wrote an empty report"),
        (
            Problem::NoOutcome,
            "has a report that does not end with its outcome",
        ),
        (
            Problem::Malformed { line: 3 },
            "has a report whose line 3 is neither a linked finding nor a requirement request",
        ),
        (
            Problem::UnknownId {
                line: 4,
                id: "ENG-99".to_owned(),
            },
            "has a report whose line 4 cites ENG-99, which no requirement or invariant has",
        ),
        (
            Problem::NoneWithFindings,
            "has a report whose outcome is none but which holds findings",
        ),
        (
            Problem::FindingsWithNone,
            "has a report whose outcome is findings but which holds none",
        ),
        (
            Problem::AnswerDiffers,
            "has a report whose outcome differs from its answer in the run log",
        ),
    ] {
        assert_eq!(problem.to_string(), want);
    }
}

#[test]
fn an_answer_line_is_the_outcome_whatever_its_whitespace_punctuation_dashes_or_markup() {
    let outcome = "outcome: findings — the change adds `x`.";
    for line in [
        "outcome: findings — the change adds `x`.",
        "  outcome:  findings —  the change adds `x`  ",
        "outcome: findings — the change adds x",
        "**Outcome: findings** - the change adds `x`;",
        "*outcome: findings* – the change adds _x_!",
        "`outcome: findings — the change adds x.`",
        "outcome: findings \u{2212} the change adds `x`.",
    ] {
        assert!(same_outcome(outcome, line), "{line:?}");
    }
    for line in [
        "outcome: none — the change adds `x`.",
        "outcome: findings — the change adds y.",
        "outcome: findings",
        "review/reports/lang-rust.md — outcome: findings — the change adds x.",
        "the outcome: findings — the change adds x. was it",
        "",
    ] {
        assert!(!same_outcome(outcome, line), "{line:?}");
    }
}

#[test]
fn a_place_reads_a_path_and_a_line_or_a_range() {
    assert_eq!(place("`a/b.rs:12`"), Some(("a/b.rs".to_owned(), 12)));
    assert_eq!(place("a/b.rs:3–4"), Some(("a/b.rs".to_owned(), 3)));
    assert_eq!(place("`a/b.rs`:7"), Some(("a/b.rs".to_owned(), 7)));
    assert_eq!(place("a/b.rs:-4"), None);
    assert_eq!(place(":4"), None);
    assert_eq!(place("a/b.rs"), None);
}

#[test]
fn bare_drops_list_markers_numbers_emphasis_and_quotes() {
    assert_eq!(bare("- a"), "a");
    assert_eq!(bare("* a"), "a");
    assert_eq!(bare("12. **a"), "a");
    assert_eq!(bare("> `a"), "a");
    assert_eq!(bare("12a"), "12a");
    assert_eq!(bare(". a"), ". a");
}

#[test]
fn split_dashes_cuts_at_each_spaced_dash() {
    assert_eq!(
        split_dashes("a — b – c -- d - e"),
        ["a", "b", "c", "d", "e"]
    );
    assert_eq!(split_dashes("a-b"), ["a-b"]);
}
