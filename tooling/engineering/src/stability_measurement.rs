//! ENG-32: the stability measurement, and the run list and the report every review
//! leaves. The measurement's workflow runs on dispatch alone (a measurement is nine
//! reviews, so the stakeholder turns a schedule on), reviews each snapshot
//! of the committed snapshots afresh with review-run and compares the
//! reviews with `review-report stability`; the review workflow keeps
//! each review's run list and report with its reports. The check also drives
//! review-report itself: a verdict that flips is measured as unstable,
//! a run's cost is the total its log reported, and a name or reason a
//! run shaped reaches the report as escaped text, never as markup.

use std::collections::{BTreeMap, BTreeSet};

use review_report::agent_runs::{self, LogFile};
use review_report::html;
use review_report::identity::Fid;
use review_report::inputs::Gap;
use review_report::snapshots;
use review_report::stability::{self, Review, Stability};

use crate::Checkout;

/// The measurement's workflow.
pub const MEASUREMENT: &str = ".github/workflows/stability.yml";
/// The review's workflow.
pub const REVIEW: &str = ".github/workflows/review.yml";

/// What a hostile run could make an agent's name and its reason.
const HOSTILE: &str = "<script>alert(\"x\")</script>";

/// The problems with the measurement's workflow `text`.
#[must_use]
pub fn measurement_problems(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let has = |want: &str| text.contains(want);
    if !has("  workflow_dispatch:") {
        problems.push(format!("{MEASUREMENT} cannot be started on demand"));
    }
    if text
        .lines()
        .any(|l| l.trim_start().starts_with("schedule:"))
    {
        problems.push(format!(
            "{MEASUREMENT} runs on a schedule, which only the stakeholder turns on"
        ));
    }
    for (want, what) in [
        (
            "target/debug/review-run\" --workdir \"$RUNNER_TEMP/work\"",
            "reviews no snapshot with review-run in a fresh work directory",
        ),
        (
            "-p review-report -- stability --out stability.json",
            "compares no reviews with review-report stability",
        ),
        (
            "snapshots tooling/review-eval/stability-snapshots.toml",
            "reads no snapshots",
        ),
        (
            "permissions: {}",
            "does not drop every permission by default",
        ),
        ("concurrency:", "lets two measurements run at once"),
        ("timeout-minutes: 120", "bounds no review's job"),
    ] {
        if !has(want) {
            problems.push(format!("{MEASUREMENT} {what}"));
        }
    }
    if has("review/logs") {
        problems.push(format!(
            "{MEASUREMENT} uploads review/logs, every tool result its agents read"
        ));
    }

    problems
}

/// The problems with the review workflow `text`: it keeps no run list or
/// no report with the reports.
#[must_use]
pub fn review_problems(text: &str) -> Vec<String> {
    ["review/report.html", "review/agent-runs.jsonl"]
        .iter()
        .filter(|f| !text.contains(&format!("            {f}\n")))
        .map(|f| format!("{REVIEW} keeps no {f} with the reports"))
        .collect()
}

/// The problems with the snapshots `text`, the measurement reviewing each of
/// its snapshots `samples` times: it does not read, or holds no snapshot; or
/// the measurement does not review each snapshot three times.
#[must_use]
pub fn states_problems(text: &str, samples: usize) -> Vec<String> {
    let mut problems = match snapshots::parse(text) {
        Ok(_) => Vec::new(),
        Err(e) => vec![format!("{}: {e}", snapshots::SNAPSHOTS)],
    };
    if samples != 3 {
        problems.push(format!(
            "the measurement reviews each snapshot {samples} times, not three"
        ));
    }

    problems
}

/// One review whose verdict is `verdict`, its specialist
/// finding at `path`.
fn review(verdict: &str, path: &str) -> Review {
    let fid = Fid::new(path, vec!["ENG-28".to_owned()], "the gate posts unread");
    Review {
        verdict: verdict.to_owned(),
        findings: BTreeMap::from([("lang-rust".to_owned(), BTreeSet::from([fid]))]),
        ..Review::default()
    }
}

/// What review-report made of a review: two identical reviews measured
/// as `same`, three whose verdict flips once as `flip`, the cost of a
/// run whose log reported 0.25 USD and whose models' costs sum to 9, and
/// the report of a gap whose name and reason are a script.
#[derive(Debug, Clone, PartialEq)]
pub struct Made {
    pub same: Stability,
    pub flip: Stability,
    pub cost: f64,
    pub report: String,
}

/// What review-report makes of the reviews [`Made`] names.
#[must_use]
pub fn made() -> Made {
    let same = stability::compare(&[review("approve", "a.rs"), review("approve", "a.rs")]);
    let flip = stability::compare(&[
        review("approve", "a.rs"),
        review("request_changes", "a.rs"),
        review("approve", "a.rs"),
    ]);
    let log = LogFile {
        file: "lang-rust.jsonl".to_owned(),
        text: r#"{"type":"result","subtype":"success","result":"x","total_cost_usd":0.25,"modelUsage":{"m":{"costUSD":9.0}}}"#.to_owned(),
        ..LogFile::default()
    };
    let cost = agent_runs::agent_runs(&[log], &[], &[], &|_| Default::default())
        .iter()
        .map(|s| s.cost_usd)
        .sum();
    let gap = Gap {
        name: HOSTILE.to_owned(),
        reason: HOSTILE.to_owned(),
        ..Gap::default()
    };
    let report = html::render(&agent_runs::AgentRuns::default(), &[gap]);

    Made {
        same,
        flip,
        cost,
        report,
    }
}

/// The problems with what review-report `made`.
#[must_use]
pub fn report_problems(made: &Made) -> Vec<String> {
    let mut problems = Vec::new();
    let (same, flip) = (&made.same, &made.flip);
    if !same.stable || flip.stable || flip.flip_rate <= 0.0 || flip.within_limits {
        problems.push("review-report stability does not find a verdict that flips".to_owned());
    }
    if (made.cost - 0.25).abs() > 1e-12 {
        problems.push(format!(
            "an agent run's cost is {}, not the 0.25 its run log reported",
            made.cost
        ));
    }
    if made.report.contains("<script") {
        problems.push("the report holds a run's text as markup".to_owned());
    }

    problems
}

/// Every problem with the measurement, the snapshots, the review workflow and
/// review-report in `checkout`.
///
/// # Errors
///
/// Fails when a file cannot be read.
pub fn problems(
    checkout: &Checkout,
    measurement: &str,
    snapshots: &str,
) -> Result<Vec<String>, String> {
    let mut problems = measurement_problems(&checkout.read(measurement)?);
    problems.extend(states_problems(
        &checkout.read(snapshots)?,
        snapshots::SAMPLES,
    ));
    problems.extend(review_problems(&checkout.read(REVIEW)?));
    problems.extend(report_problems(&made()));

    Ok(problems)
}

#[cfg(test)]
mod tests;
