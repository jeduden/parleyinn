//! A report line that names a file of the case by a tail of its path.

use super::*;

#[test]
fn a_finding_at_a_tail_of_the_case_s_path_is_the_finding_the_case_wants() {
    let mut case = case(
        "domain-model-harness-facts",
        Expect {
            finding: vec![Expected {
                mentions: vec!["ASM-99".into()],
                ..wanted("docs/domain-model/harness-facts.md")
            }],
            stated: vec![wanted("docs/domain-model/harness-facts.md")],
            ..Expect::default()
        },
    );
    case.edits = vec![crate::case::Edit::Remove {
        file: "docs/domain-model/harness-facts.md".into(),
    }];
    let line =
        "harness-facts.md:19 — if not addressed: it cites ASM-99, which nothing defines — ASM-03";
    let log = stated_log(&[line], "outcome: findings — one id is missing");

    let graded = grade(&component(Kind::Specialist, "x", true), &case, &log);

    assert_eq!(graded.failures, Vec::<String>::new());
    // The review withdraws and revises by the path as written, so the
    // tail is counted.
    assert_eq!(
        graded.slips,
        vec!["names docs/domain-model/harness-facts.md by its tail harness-facts.md"]
    );
}

/// A generalist's case that wants a blocking finding at line 3 of a
/// file, within one line.
fn at_line_three() -> Case {
    case(
        "review-code",
        Expect {
            finding: vec![Expected {
                severity: Some("blocking".into()),
                line: Some(3),
                within: Some(1),
                ..wanted("tooling/x/src/shape.rs")
            }],
            ..Expect::default()
        },
    )
}

fn failures_of(case: &Case, answer: &str) -> Vec<String> {
    grade(
        &component(Kind::Generalist, "review-code", false),
        case,
        &answered_log(answer),
    )
    .failures
}

#[test]
fn a_wanted_line_is_the_line_of_the_place_not_a_number_of_the_sentence() {
    let case = at_line_three();

    // At the line, beside it, and in a range that covers it.
    for place in ["shape.rs:3", "tooling/x/src/shape.rs:4", "shape.rs:1-2"] {
        let answer = format!("{place} — blocking — a comment speaks to the reviewer — yours");
        assert_eq!(failures_of(&case, &answer), Vec::<String>::new(), "{place}");
    }
    // Elsewhere in the file, though the sentence holds a 4: ENG-04, I4.
    let far = "tooling/x/src/shape.rs:14 — blocking — it breaks ENG-04 and I4 — yours";
    assert_eq!(failures_of(&case, far).len(), 1, "{far}");
}

#[test]
fn a_wanted_id_is_one_the_linked_line_cites_not_one_its_sentence_holds() {
    let case = case(
        "srs-security",
        Expect {
            finding: vec![Expected {
                ids: vec!["ENG-35".into()],
                ..wanted("docs/srs/06-security.md")
            }],
            ..Expect::default()
        },
    );
    let grade_of = |line: &str| {
        let log = stated_log(&[line], "outcome: findings — one sentence");
        grade(&component(Kind::Specialist, "x", true), &case, &log).failures
    };

    let cited = "docs/srs/06-security.md:8 — if not addressed: it speaks of a lane — ENG-35";
    assert_eq!(grade_of(cited), Vec::<String>::new());
    let in_the_sentence =
        "docs/srs/06-security.md:8 — if not addressed: it breaks ENG-35 for rooms — LANE-01";
    assert_eq!(grade_of(in_the_sentence).len(), 1, "{in_the_sentence}");
}
