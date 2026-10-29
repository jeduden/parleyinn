//! ENG-32: every reviewer component has its evaluation cases, every
//! case applies to the tree and makes the review bundle dispatch its
//! component, and the harness grades a run and decides a case as the
//! requirement says. The checks call the `review-eval` crate, which the
//! evaluate job runs; the decision tables here drive its grader and its
//! two-of-three rule without a model.

use std::collections::{BTreeMap, BTreeSet};

use review_eval::case::{Case, Expect, Expected};
use review_eval::component::{Component, Kind};
use review_eval::grade::{Run, grade};
use review_eval::log::RunLog;
use review_eval::score::decided;
use review_eval::{CASES, check};

use crate::Checkout;

/// The component's name and report the decision table grades.
const COMPONENT: &str = "lang-rust";
const REPORT: &str = "review/reports/lang-rust.md";
/// The paths the table's case expects a finding at, and forbids one at.
const WANTED: &str = "crates/x/src/a.rs";
const FORBIDDEN: &str = "crates/x/src/b.rs";

/// The components without one of their three kinds of case (ENG-32).
#[must_use]
pub fn components_have_their_cases(checkout: &Checkout) -> Vec<String> {
    check::kind_problems(&checkout.root, &checkout.root, &checkout.root.join(CASES))
}

/// The cases that do not read, apply to the checkout, or make the
/// review bundle dispatch their component (ENG-32).
#[must_use]
pub fn cases_apply_and_dispatch(checkout: &Checkout) -> Vec<String> {
    check::case_problems(&checkout.root, &checkout.root, &checkout.root.join(CASES))
}

/// Grades one run per row of the scenario's table and compares the
/// grader's answer with the row's `run` column.
#[must_use]
pub fn run_graded(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let got = run_outcome(row);
            (row.get("run").map(String::as_str) != Some(got.as_str()))
                .then(|| format!("row {} {row:?}: the grader says {got}", i + 1))
        })
        .collect()
}

/// The specialist the decision table grades: a strict-lined one on
/// haiku, as the language specialists are.
fn table_component() -> Component {
    Component {
        name: COMPONENT.to_owned(),
        kind: Kind::Specialist,
        file: format!(".claude/agents/{COMPONENT}.md"),
        text: "findings, each on one line".to_owned(),
        model: "haiku".to_owned(),
        effort: "low".to_owned(),
        max_turns: 20,
        tools: Vec::new(),
        strict_lines: true,
    }
}

/// The decision table's case: a defect that wants an unwrap finding at
/// one path and forbids a finding at another.
fn table_case() -> Case {
    Case {
        component: COMPONENT.to_owned(),
        name: "table".to_owned(),
        kind: "defect".to_owned(),
        title: "the table's case".to_owned(),
        consent: None,
        edits: Vec::new(),
        seeds: Vec::new(),
        expect: Expect {
            outcome: vec!["blocking".to_owned()],
            finding: vec![Expected {
                path: WANTED.to_owned(),
                mentions: vec!["unwrap".to_owned()],
                ..Expected::default()
            }],
            none: vec![Expected {
                path: FORBIDDEN.to_owned(),
                ..Expected::default()
            }],
            ..Expect::default()
        },
        repetitions: 3,
        passes: 2,
    }
}

/// What the grader says of the run `row` describes: `passes`, `passes,
/// slips on` what it slips on first, or `fails on` what it fails on
/// first.
fn run_outcome(row: &BTreeMap<String, String>) -> String {
    let column = |key: &str| row.get(key).map_or("", String::as_str);
    let (component, case) = (table_component(), table_case());
    let outcome_line = if column("outcome") == "as expected" {
        "outcome: blocking — it unwraps."
    } else {
        "outcome: sound — it is fine."
    };
    let mut report = String::new();
    if column("finding") == "found" {
        report.push_str(&format!(
            "{WANTED}:6 — blocking — an unwrap outside tests — ENG-16\n"
        ));
    }
    if column("forbidden") == "found" {
        report.push_str(&format!("{FORBIDDEN}:2 — nit — a long line — ENG-16\n"));
    }
    report.push_str(&format!("\n{outcome_line}\n"));
    let answer = if column("answer") == "more" {
        format!("I read the diff.\n{outcome_line}\n{REPORT}\n")
    } else {
        format!("{outcome_line}\n{REPORT}\n")
    };
    let log = RunLog {
        answered: column("answered") == "yes",
        subtype: if column("answered") == "yes" {
            "success".to_owned()
        } else {
            "error_max_turns".to_owned()
        },
        answer,
        writes: vec![REPORT.to_owned()],
        ..RunLog::default()
    };
    let dispatched = BTreeSet::new();
    let graded = grade(
        &component,
        &case,
        &Run {
            log: &log,
            report: (column("report") == "written").then_some(report.as_str()),
            dispatched: &dispatched,
        },
    );
    match (graded.failures.first(), graded.slips.first()) {
        (Some(f), _) => format!("fails on {}", failed_on(f)),
        (None, Some(s)) => format!("passes, slips on {}", failed_on(s)),
        (None, None) => "passes".to_owned(),
    }
}

/// What a failure is about, by its first words.
fn failed_on(failure: &str) -> &'static str {
    if failure.starts_with("did not answer") {
        "no answer"
    } else if failure.starts_with("wrote no report")
        || failure.starts_with("wrote an empty report")
        || failure.starts_with("report does not end")
    {
        "report"
    } else if failure.starts_with("outcome ") {
        "outcome"
    } else if failure.starts_with("no finding") {
        "finding"
    } else if failure.starts_with("a finding at") {
        "forbidden"
    } else if failure.starts_with("answer ") {
        "answer"
    } else {
        "something else"
    }
}

/// Decides one case per row of the scenario's table, each run in turn
/// until decided, and compares with the row's `case` and `runs made`
/// columns. A run of `none` did not answer, which fails it.
#[must_use]
pub fn case_decided(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let column = |key: &str| row.get(key).map_or("", String::as_str);
            let (verdict, made) = decide(column("runs"));
            let want = (column("case"), column("runs made"));
            (want != (verdict, made.to_string().as_str())).then(|| {
                format!(
                    "row {} {row:?}: the case {verdict} after {made} runs",
                    i + 1
                )
            })
        })
        .collect()
}

/// The case's verdict after the runs `runs` lists, and how many it
/// took: `passes` or `fails`, with two passes needed of three.
fn decide(runs: &str) -> (&'static str, usize) {
    let (mut passes, mut fails) = (0, 0);
    for (made, run) in runs.split(',').map(str::trim).enumerate() {
        if run == "pass" {
            passes += 1;
        } else {
            fails += 1;
        }
        if let Some(passed) = decided(passes, fails, 3, 2) {
            return (if passed { "passes" } else { "fails" }, made + 1);
        }
    }

    ("fails", runs.split(',').count())
}

#[cfg(test)]
mod tests;
