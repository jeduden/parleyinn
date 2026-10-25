use std::collections::BTreeMap;
use std::path::PathBuf;

use coverage::plan::{Crate, Floors, Plan};

use super::*;
use crate::review_workflow;

const ROW: &str = "Line coverage MUST be ≥ 90% for security-sensitive crates (`provenance`, \
                   `redaction`, `flag`) and ≥ 80% overall.";

fn floors(all: Option<f64>) -> Floors {
    Floors {
        all,
        unit: Some(99.0),
        ..Floors::default()
    }
}

fn plan(names: &[&str], default: Option<f64>, over: &[(&str, Option<f64>)]) -> Plan {
    Plan {
        crates: names
            .iter()
            .map(|n| Crate {
                name: (*n).to_owned(),
                dir: PathBuf::from(format!("crates/{n}")),
            })
            .collect(),
        floors: floors(default),
        crate_floors: over
            .iter()
            .map(|(n, all)| {
                (
                    (*n).to_owned(),
                    Floors {
                        all: *all,
                        ..Floors::default()
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
        ..Plan::default()
    }
}

fn row() -> Row {
    Row::parse(ROW).unwrap()
}

#[test]
fn the_row_names_the_sensitive_crates_and_both_floors() {
    assert_eq!(
        row(),
        Row {
            sensitive: vec!["provenance".into(), "redaction".into(), "flag".into()],
            sensitive_floor: 90.0,
            overall_floor: 80.0,
        }
    );
}

#[test]
fn a_row_without_its_list_or_a_floor_is_refused() {
    assert_eq!(
        Row::parse("Line coverage MUST be high."),
        Err("the ENG-11 row names no security-sensitive crates in parentheses".to_owned())
    );
    assert_eq!(
        Row::parse("for security-sensitive crates (`a`) and overall."),
        Err("the ENG-11 row states no floor before \"for security-sensitive crates\"".to_owned())
    );
    assert_eq!(
        Row::parse("≥ 90% for security-sensitive crates (`a`) and overall."),
        Err("the ENG-11 row states no floor before \"overall\"".to_owned())
    );
}

#[test]
fn a_built_sensitive_crate_held_to_a_floor_at_or_above_the_row_s_passes() {
    let p = plan(&["provenance", "redaction", "store"], Some(100.0), &[]);
    assert!(sensitive_problems(&p, &row()).is_empty());
}

#[test]
fn a_sensitive_crate_held_lower_or_to_no_floor_is_named() {
    let p = plan(
        &["provenance", "redaction"],
        Some(100.0),
        &[("provenance", Some(85.0))],
    );
    assert_eq!(
        sensitive_problems(&p, &row()),
        vec!["provenance is held to 85% of its lines, below ENG-11's 90%".to_owned()]
    );
    let none = plan(&["redaction"], None, &[]);
    assert_eq!(
        sensitive_problems(&none, &row()),
        vec!["redaction is held to no floor over every test layer".to_owned()]
    );
}

#[test]
fn the_built_sensitive_crates_are_listed_and_the_rest_named_as_not_yet_built() {
    let p = plan(&["provenance", "store"], Some(100.0), &[]);
    assert_eq!(
        built(&p, &row()),
        (
            vec!["provenance".to_owned()],
            vec!["redaction".to_owned(), "flag".to_owned()]
        )
    );
}

#[test]
fn every_crate_at_the_overall_floor_holds_the_whole_workspace_to_it() {
    let p = plan(
        &["provenance", "store"],
        Some(100.0),
        &[("store", Some(80.0))],
    );
    assert!(overall_problems(&p, &row()).is_empty());
    let low = plan(&["store"], Some(100.0), &[("store", Some(79.5))]);
    assert_eq!(
        overall_problems(&low, &row()),
        vec!["store is held to 79.5% of its lines, below ENG-11's 80% overall".to_owned()]
    );
    let none = plan(&["store"], None, &[]);
    assert_eq!(
        overall_problems(&none, &row()),
        vec!["store is held to no floor over every test layer".to_owned()]
    );
}

const CI: &str = "jobs:
  coverage:
    needs: [coverage-layer]
    steps:
      - run: cargo run --locked -p coverage -- --merge
  ci:
    needs: [test, coverage]
";

#[test]
fn ci_holds_the_merged_profile_to_the_floors_in_a_job_ci_needs() {
    assert!(gated(&review_workflow::parse(CI)).is_empty());
}

#[test]
fn ci_without_the_merge_or_the_aggregator_s_need_is_named() {
    let unmerged = CI.replace(" -- --merge", "");
    assert_eq!(
        gated(&review_workflow::parse(&unmerged)),
        vec!["job coverage does not run \"-p coverage -- --merge\"".to_owned()]
    );
    let unneeded = CI.replace("[test, coverage]", "[test]");
    assert_eq!(
        gated(&review_workflow::parse(&unneeded)),
        vec!["job ci does not need coverage".to_owned()]
    );
    assert_eq!(
        gated(&review_workflow::parse("jobs:\n  ci:\n    needs: [test]\n")),
        vec!["the CI workflow has no job coverage".to_owned()]
    );
}
