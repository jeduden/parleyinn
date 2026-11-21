//! A finding's ids are held to the requirements and invariants the
//! case's world holds, as the review holds them (ENG-32).

use super::*;

const MISSING: &str = "docs/domain-model/places.md:11 — if not addressed: it cites SEC-99, which no section defines \
     — SEC-99, ENG-16";

#[test]
fn a_finding_that_cites_an_id_no_requirement_has_fails_its_run_as_the_review_refuses_it() {
    let c = component(Kind::Specialist, "domain-model-places", true);
    let log = stated_log(&[MISSING], "outcome: findings — x.");
    let expect = Expect {
        finding: vec![wanted("docs/domain-model/places.md")],
        ..Expect::default()
    };

    let graded = grade(&c, &case("domain-model-places", expect), &log);

    assert_eq!(
        graded.failures,
        [
            "no finding at docs/domain-model/places.md".to_owned(),
            format!(
                "report line does not read: has a report whose line 1 cites SEC-99, which no \
                 requirement or invariant has: `{MISSING}`"
            ),
        ]
    );
}

#[test]
fn a_finding_that_names_a_missing_id_in_its_sentence_and_cites_a_known_one_passes() {
    let c = component(Kind::Specialist, "domain-model-places", true);
    let line = MISSING.replace("SEC-99, ENG-16", "ENG-16");
    let log = stated_log(&[&line], "outcome: findings — x.");
    let expect = Expect {
        finding: vec![Expected {
            ids: vec!["ENG-16".into()],
            ..wanted("docs/domain-model/places.md")
        }],
        ..Expect::default()
    };

    let graded = grade(&c, &case("domain-model-places", expect), &log);

    assert!(graded.passed(), "{:?}", graded.failures);
}

#[test]
fn the_ids_are_the_world_s_and_no_others() {
    let c = component(Kind::Specialist, "domain-model-places", true);
    let line = MISSING.replace("SEC-99, ENG-16", "ENG-16, I8");
    let log = stated_log(&[&line], "outcome: findings — x.");
    let case = case("domain-model-places", Expect::default());
    let world = |ids: &[&str]| ids.iter().map(|id| (*id).to_owned()).collect();

    let held = super::super::grade(&c, &case, &log, &world(&["ENG-16", "I8"]));
    let lacking = super::super::grade(&c, &case, &log, &world(&["ENG-16"]));

    assert!(held.passed(), "{:?}", held.failures);
    assert_eq!(lacking.failures.len(), 1, "{:?}", lacking.failures);
    assert!(
        lacking.failures[0].contains("cites I8"),
        "{:?}",
        lacking.failures
    );
}
