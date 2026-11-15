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
