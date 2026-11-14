//! An answer's path against the reported one: a tail of it, at a `/`.

use super::*;

fn report_at(place: &str) -> String {
    format!("{place} — if not addressed: a long line hides a bug — I9")
}

fn kept_at(place: &str) -> String {
    format!("{place} — blocking — a long line hides a bug — I9")
}

#[test]
fn an_answer_that_shortens_the_reported_path_still_weighs_it() {
    let report = report_at("crates/parinn/src/index_gaps.rs:84");

    assert!(missed([report.as_str()], &kept_at("index_gaps.rs:84")).is_empty());
    assert!(missed([report.as_str()], &kept_at("src/index_gaps.rs:84")).is_empty());
    assert!(missed([report.as_str()], "dropped — index_gaps.rs:84 — noise").is_empty());
    assert_eq!(
        missed([report.as_str()], &kept_at("index_gaps.rs:85")),
        ["crates/parinn/src/index_gaps.rs:84"]
    );
}

#[test]
fn an_answer_that_lengthens_the_reported_path_still_weighs_it() {
    let report = report_at("trust-and-flow.md:24");
    let answer = kept_at("docs/domain-model/trust-and-flow.md:24");

    assert!(missed([report.as_str()], &answer).is_empty());
    assert_eq!(
        missed(
            [report.as_str()],
            &kept_at("docs/domain-model/trust-and-flow.md:25")
        ),
        ["trust-and-flow.md:24"]
    );
}

#[test]
fn an_ambiguous_basename_weighs_nothing() {
    let reports = [
        report_at("crates/a/src/lib.rs:5"),
        report_at("crates/b/src/lib.rs:5"),
    ];
    let both: Vec<&str> = reports.iter().map(String::as_str).collect();

    assert_eq!(
        missed(both.clone(), &kept_at("lib.rs:5")),
        ["crates/a/src/lib.rs:5", "crates/b/src/lib.rs:5"]
    );
    assert!(missed(both, &kept_at("crates/a/src/lib.rs:5")).len() == 1);

    let short = [report_at("lib.rs:5"), report_at("crates/b/lib.rs:5")];
    let both: Vec<&str> = short.iter().map(String::as_str).collect();

    assert_eq!(
        missed(both, &kept_at("x/lib.rs:5")),
        ["crates/b/lib.rs:5", "lib.rs:5"]
    );
    let dir = report_at("src/a.rs:3");

    assert_eq!(
        missed([dir.as_str()], &kept_at("lib/src/a.rs:3")),
        ["src/a.rs:3"]
    );
}

#[test]
fn a_lengthened_path_is_found_after_any_text_before_it() {
    let report = report_at("trust-and-flow.md:24");

    for answer in [
        "dropped — docs/model/trust-and-flow.md:24 — noise",
        "dropped —docs/model/trust-and-flow.md:24 — noise",
    ] {
        assert!(missed([report.as_str()], answer).is_empty(), "{answer}");
    }
}

#[test]
fn a_short_and_a_long_spelling_of_one_path_reported_together_are_one_place() {
    let reports = [
        report_at("trust-and-flow.md:24"),
        report_at("docs/domain-model/trust-and-flow.md:24"),
    ];
    let both: Vec<&str> = reports.iter().map(String::as_str).collect();
    let answer = kept_at("docs/domain-model/trust-and-flow.md:24");

    assert!(missed(both.clone(), &answer).is_empty());
    assert!(missed(both, &kept_at("trust-and-flow.md:24")).is_empty());
}

#[test]
fn a_bare_name_with_two_longer_spellings_reported_stays_ambiguous() {
    let reports = [
        report_at("lib.rs:5"),
        report_at("crates/a/lib.rs:5"),
        report_at("crates/b/lib.rs:5"),
    ];
    let all: Vec<&str> = reports.iter().map(String::as_str).collect();

    assert_eq!(
        missed(all, &kept_at("crates/a/lib.rs:5")),
        ["crates/b/lib.rs:5", "lib.rs:5"]
    );
}
