use super::*;
use crate::report::{
    Line, Linked, OFF_FORM, Problem, Request, assemble, line, off_form, report_check, split_with,
};

const ANSWER: &str = "outcome: findings — one sentence.";

fn linked_of(text: &str) -> (String, Vec<String>) {
    match line(text) {
        Line::Linked(Linked {
            consequence, ids, ..
        }) => (consequence, ids),
        other => panic!("not linked: {text:?}: {other:?}"),
    }
}

fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

#[test]
fn a_linked_finding_with_a_remark_after_its_ids_reads_with_the_remark_in_its_consequence() {
    for (text, consequence, ids) in [
        (
            "a.rs:1 — if not addressed: x — I2 (provisional)",
            "x — (provisional)",
            &["I2"][..],
        ),
        (
            "- `a.rs:1 — if not addressed: x — ENG-11` (provisional id; confirming against the SRS)",
            "x — (provisional id; confirming against the SRS)",
            &["ENG-11"],
        ),
        (
            "crates/x/tests/e2e_a.rs:12 — if not addressed: it sleeps to wait — ENG-03, testing \
             rule in docs/testing.md (never sleep; wait on the exit with a deadline)",
            "it sleeps to wait — testing rule in docs/testing.md (never sleep; wait on the exit \
             with a deadline)",
            &["ENG-03"],
        ),
        (
            "a.rs:1 — if not addressed: a hook panics — ENG-04 and I9",
            "a hook panics",
            &["ENG-04", "I9"],
        ),
        (
            "a.rs:1 — if not addressed: a hook panics — or hangs — ENG-04, I9; both hold",
            "a hook panics — or hangs — both hold",
            &["ENG-04", "I9"],
        ),
    ] {
        assert_eq!(
            linked_of(text),
            (consequence.to_owned(), owned(ids)),
            "{text:?}"
        );
    }
}

#[test]
fn a_linked_finding_that_ends_with_its_ids_and_no_dash_reads_with_its_sentence_whole() {
    for (text, consequence, ids) in [
        (
            "a.rs:1 — if not addressed: the test sleeps, breaking ENG-03.",
            "the test sleeps, breaking ENG-03.",
            &["ENG-03"][..],
        ),
        (
            "a.rs:1 — if not addressed: a drop goes uncounted (I6, ENG-04)",
            "a drop goes uncounted (I6, ENG-04)",
            &["I6", "ENG-04"],
        ),
        (
            "a.rs:1 — if not addressed: x — which breaks `ENG-14` and I9.",
            "x — which breaks `ENG-14` and I9.",
            &["ENG-14", "I9"],
        ),
    ] {
        assert_eq!(
            linked_of(text),
            (consequence.to_owned(), owned(ids)),
            "{text:?}"
        );
    }
}

#[test]
fn a_request_with_no_motivation_set_apart_reads_with_the_reader_s_own() {
    for text in [
        "a.rs:1 — request: add a cap",
        "a.rs:1 — request: Add a cap — ",
    ] {
        assert_eq!(
            line(text),
            Line::Request(Request {
                path: "a.rs".to_owned(),
                line: 1,
                change: "add".to_owned(),
                requirement: "a cap".to_owned(),
                motivation: NO_MOTIVATION.to_owned(),
            }),
            "{text:?}"
        );
    }
}

#[test]
fn a_line_that_names_no_id_or_says_nothing_beside_its_ids_stays_malformed() {
    for text in [
        "a.rs:1 — if not addressed: a hook panics",
        "a.rs:1 — if not addressed: — ENG-04 and I9",
        "a.rs:1 — if not addressed: ENG-04",
        "a.rs:1 — if not addressed: ENG-04 breaks",
        "a.rs:1 — if not addressed: a hook panics — eng-04",
        "a.rs:1 — if not addressed: a hook panics — ENG-04 holds — as checked",
        "a.rs:1 — if not addressed: a hook panics and",
        "a.rs:1 — request: add",
        "a.rs:1 — request: rename a cap",
        "a.rs:1 if not addressed: a hook panics — ENG-04 (yes)",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn off_form_names_a_line_that_reads_only_as_a_near_form() {
    for text in [
        "a.rs:1 — if not addressed: x — I2 (provisional)",
        "- **a.rs:1** — if not addressed: the test sleeps, breaking ENG-03.",
        "a.rs:1 — request: add a cap",
        "a.rs:1 — request: add a cap; reports grow",
        "revised: a.rs:1 — if not addressed: x, breaking I2",
    ] {
        assert!(off_form(text), "{text:?}");
    }
    for text in [
        "a.rs:1 — if not addressed: x — I2",
        "a.rs:1 — request: add a cap — reports grow",
        "revised: a.rs:1 — if not addressed: x — I2",
        "withdrawn: a.rs:1 — it holds",
        "a.rs:1 — nit — x",
        "a.rs:1 — if not addressed: a hook panics",
        "I read it.",
    ] {
        assert!(!off_form(text), "{text:?}");
    }
}

fn texts(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|l| (*l).to_owned()).collect()
}

#[test]
fn an_assembled_report_states_a_near_form_line_in_its_form_and_notes_it_read_off_its_form() {
    let remark = "a.rs:1 — if not addressed: x — I2 (provisional)";
    let request = "c.rs:3 — request: add a cap";

    let got = assemble(
        &texts(&[
            remark,
            "b.rs:2 — if not addressed: y — I3",
            &format!("- {request}"),
        ]),
        ANSWER,
        None,
    );

    assert_eq!(
        got,
        format!(
            "a.rs:1 — if not addressed: x — (provisional) — I2\n\
             {OFF_FORM} {remark}\n\
             b.rs:2 — if not addressed: y — I3\n\
             c.rs:3 — request: add a cap — {NO_MOTIVATION}\n\
             {OFF_FORM} {request}\n\
             {ANSWER}\n"
        )
    );
    assert_eq!(report_check(Some(&got), Some(ANSWER), None), None);
    assert_eq!(line(&format!("{OFF_FORM} {remark}")), Line::Prose);
}

#[test]
fn a_near_form_line_stated_twice_or_in_a_revision_is_noted_once_each() {
    let near = "a.rs:1 — if not addressed: x, breaking I2";
    let revised = "revised: a.rs:1 — if not addressed: y, breaking I3";

    let got = assemble(&texts(&[near, near, revised]), ANSWER, None);

    assert_eq!(
        got,
        format!(
            "a.rs:1 — if not addressed: y, breaking I3 — I3\n\
             {OFF_FORM} {near}\n\
             {OFF_FORM} {revised}\n\
             {ANSWER}\n"
        )
    );
}

#[test]
fn a_near_form_line_citing_an_id_no_requirement_has_does_not_read() {
    let known = ["I2".to_owned()].into_iter().collect();
    let report = assemble(
        &texts(&["a.rs:1 — if not addressed: bytes are cut, breaking UTF-8"]),
        ANSWER,
        None,
    );

    let got = split_with(&report, Some(&known));

    assert_eq!(
        got.unreadable
            .iter()
            .map(|u| u.problem.clone())
            .collect::<Vec<_>>(),
        [Problem::UnknownId {
            line: 1,
            id: "UTF-8".to_owned()
        }]
    );
}

#[test]
fn run_takes_the_ids_words_start_with_and_passes_one_and_between_two() {
    let run_of = |text: &str| run(text.split_whitespace());

    assert_eq!(run_of("ENG-04, `I9`. more"), (owned(&["ENG-04", "I9"]), 2));
    assert_eq!(run_of("ENG-04 and I9 and"), (owned(&["ENG-04", "I9"]), 3));
    assert_eq!(run_of("ENG-04 and and I9"), (owned(&["ENG-04"]), 1));
    assert_eq!(run_of("and I9"), (Vec::new(), 0));
    assert_eq!(run_of(""), (Vec::new(), 0));
}

#[test]
fn a_request_is_cut_at_its_first_reason_and_whole_without_one() {
    assert_eq!(
        reasoned("a cap; it grows: fast"),
        ("a cap", "it grows: fast")
    );
    assert_eq!(
        reasoned("a cap on docs/ci.md:23"),
        ("a cap on docs/ci.md:23", "")
    );
    for (asked, motivation) in [
        ("a cap since none holds it", "none holds it"),
        ("a cap, since none holds it", "none holds it"),
        ("a cap because it grows", "it grows"),
        ("a cap, because it grows", "it grows"),
        ("a cap: it grows", "it grows"),
        ("a cap. It grows", "It grows"),
    ] {
        assert_eq!(reasoned(asked), ("a cap", motivation), "{asked:?}");
    }
}

#[test]
fn motivated_gives_a_request_the_reader_s_motivation_when_it_states_none() {
    assert_eq!(motivated("a cap; it grows"), ("a cap", "it grows"));
    assert_eq!(motivated("a cap"), ("a cap", NO_MOTIVATION));
    assert_eq!(motivated("a cap —"), ("a cap", NO_MOTIVATION));
}
