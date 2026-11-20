//! A finding line off its form, which the review reads as the finding
//! it states and counts as a slip.

use super::*;

#[test]
fn a_finding_off_its_form_is_graded_as_the_finding_it_states_and_slips() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let near = "crates/x/src/a.rs:12 — if not addressed: an unwrap outside tests panics, \
                breaking ENG-16.";
    let log = stated_log(&[near], "outcome: findings — it unwraps.");
    let expect = Expect {
        outcome: vec!["findings".into()],
        finding: vec![Expected {
            line: Some(12),
            form: Some("linked".into()),
            ids: vec!["ENG-16".into()],
            ..wanted("crates/x/src/a.rs")
        }],
        stated: vec![wanted("crates/x/src/a.rs")],
        ..Expect::default()
    };

    let graded = grade(&c, &case("lang-rust", expect), &log);

    assert!(graded.passed(), "{:?}", graded.failures);
    assert_eq!(
        graded.slips,
        [format!(
            "read off its form, as the finding it states: {near}"
        )]
    );
}

#[test]
fn a_request_with_no_motivation_set_apart_is_graded_as_a_request_and_slips() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let near = "crates/x/src/a.rs:12 — request: add a rule on unwrap outside tests";
    let log = stated_log(&[near], "outcome: findings — a rule is missing.");
    let expect = Expect {
        finding: vec![Expected {
            form: Some("request".into()),
            ..wanted("crates/x/src/a.rs")
        }],
        ..Expect::default()
    };

    let graded = grade(&c, &case("lang-rust", expect), &log);

    assert!(graded.passed(), "{:?}", graded.failures);
    assert_eq!(
        graded.slips,
        [format!(
            "read off its form, as the finding it states: {near}"
        )]
    );
}

#[test]
fn a_line_that_names_no_id_still_fails_its_run() {
    let c = component(Kind::Specialist, "lang-rust", true);
    let bad = "crates/x/src/a.rs:12 — if not addressed: an unwrap outside tests panics";
    let log = stated_log(&[bad], "outcome: findings — it unwraps.");

    let graded = grade(&c, &case("lang-rust", Expect::default()), &log);

    assert_eq!(
        graded.failures,
        [format!(
            "report line does not read: has a report whose line 1 cites no requirement or \
             invariant id: a line saying all is well is not a finding, so leave it out: `{bad}`"
        )]
    );
}
