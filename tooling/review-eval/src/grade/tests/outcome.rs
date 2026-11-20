//! An outcome line among a report's lines, which is no finding.

use super::*;

#[test]
fn an_outcome_line_that_names_a_file_is_no_finding_at_it() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let named = "outcome: none — crates/x/src/a.rs changes one word and breaks no rule.";
    let forbids = Expect {
        outcome: vec!["none".into()],
        none: vec![wanted("crates/x/src/a.rs")],
        ..Expect::default()
    };
    assert!(grade(&c, &case("lang-rust", forbids), &stated_log(&[], named)).passed());

    // Nor is it the finding a case wants there.
    let wants = Expect {
        outcome: vec!["none".into()],
        finding: vec![wanted("crates/x/src/a.rs")],
        ..Expect::default()
    };
    assert_eq!(
        grade(&c, &case("lang-rust", wants), &stated_log(&[], named)).failures,
        vec!["no finding at crates/x/src/a.rs"]
    );
}

#[test]
fn a_generalist_s_line_before_its_final_answer_is_a_slip_and_a_consultant_s_is_none() {
    let early = "dropped — src/a.rs:9 — main has it — lang-rust";
    let log = stated_log(&[early], "none — nothing to add — review-code");

    let c = component(Kind::Generalist, "review-code", false);
    let graded = grade(&c, &case("review-code", Expect::default()), &log);
    assert!(graded.passed(), "{:?}", graded.failures);
    assert_eq!(
        graded.slips,
        [format!(
            "stated before its final answer, which does not hold it: {early}"
        )]
    );

    let c = component(Kind::Consultant, "persona-agent", false);
    let graded = grade(&c, &case("persona-agent", Expect::default()), &log);
    assert!(graded.slips.is_empty(), "{:?}", graded.slips);
}
