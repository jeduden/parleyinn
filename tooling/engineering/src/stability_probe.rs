//! ENG-32: the stability probe, and the trace and page every review
//! leaves. The probe's workflow runs on dispatch alone (a probe is nine
//! reviews, so the stakeholder turns a schedule on), reviews each state
//! of the committed corpus afresh with review-run and compares the
//! reviews with `review-report stability`; the review workflow keeps
//! each review's trace and page with its reports. The check also drives
//! review-report itself: a verdict that flips is measured as unstable,
//! a run's cost is the total its log reported, and a name or reason a
//! run shaped reaches the page as escaped text, never as markup.

use std::collections::{BTreeMap, BTreeSet};

use review_report::corpus;
use review_report::html;
use review_report::identity::Fid;
use review_report::inputs::Gap;
use review_report::stability::{self, Run, Stability};
use review_report::trace::{self, LogFile};

use crate::Checkout;

/// The probe's workflow.
pub const PROBE: &str = ".github/workflows/stability.yml";
/// The review's workflow.
pub const REVIEW: &str = ".github/workflows/review.yml";

/// What a hostile run could make an agent's name and its reason.
const HOSTILE: &str = "<script>alert(\"x\")</script>";

/// The problems with the probe's workflow `text`.
#[must_use]
pub fn probe_problems(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let has = |want: &str| text.contains(want);
    if !has("  workflow_dispatch:") {
        problems.push(format!("{PROBE} cannot be started on demand"));
    }
    if text
        .lines()
        .any(|l| l.trim_start().starts_with("schedule:"))
    {
        problems.push(format!(
            "{PROBE} runs on a schedule, which only the stakeholder turns on"
        ));
    }
    for (want, what) in [
        (
            "target/debug/review-run\" --workdir \"$RUNNER_TEMP/work\"",
            "reviews no state with review-run in a fresh work directory",
        ),
        (
            "-p review-report -- stability --out stability.json",
            "compares no reviews with review-report stability",
        ),
        (
            "corpus tooling/review-eval/stability-corpus.toml",
            "reads no corpus",
        ),
        (
            "permissions: {}",
            "does not drop every permission by default",
        ),
        ("concurrency:", "lets two probes run at once"),
        ("timeout-minutes: 120", "bounds no review's job"),
    ] {
        if !has(want) {
            problems.push(format!("{PROBE} {what}"));
        }
    }
    if has("review/logs") {
        problems.push(format!(
            "{PROBE} uploads review/logs, every tool result its agents read"
        ));
    }

    problems
}

/// The problems with the review workflow `text`: it keeps no trace or
/// no page with the reports.
#[must_use]
pub fn review_problems(text: &str) -> Vec<String> {
    ["review/report.html", "review/trace.jsonl"]
        .iter()
        .filter(|f| !text.contains(&format!("            {f}\n")))
        .map(|f| format!("{REVIEW} keeps no {f} with the reports"))
        .collect()
}

/// The problems with the corpus `text`, the probe reviewing each of
/// its states `samples` times: it does not read, or holds no state; or
/// the probe does not review each state three times.
#[must_use]
pub fn corpus_problems(text: &str, samples: usize) -> Vec<String> {
    let mut problems = match corpus::parse(text) {
        Ok(_) => Vec::new(),
        Err(e) => vec![format!("{}: {e}", corpus::CORPUS)],
    };
    if samples != 3 {
        problems.push(format!(
            "the probe reviews each state {samples} times, not three"
        ));
    }

    problems
}

/// One run of a review whose verdict is `verdict`, its specialist
/// finding at `path`.
fn run(verdict: &str, path: &str) -> Run {
    let fid = Fid::new(path, vec!["ENG-28".to_owned()], "the gate posts unread");
    Run {
        verdict: verdict.to_owned(),
        findings: BTreeMap::from([("lang-rust".to_owned(), BTreeSet::from([fid]))]),
        ..Run::default()
    }
}

/// What review-report made of a review: two identical runs measured
/// as `same`, three whose verdict flips once as `flip`, the cost of a
/// run whose log reported 0.25 USD and whose models' costs sum to 9, and
/// the page of a gap whose name and reason are a script.
#[derive(Debug, Clone, PartialEq)]
pub struct Made {
    pub same: Stability,
    pub flip: Stability,
    pub cost: f64,
    pub page: String,
}

/// What review-report makes of the reviews [`Made`] names.
#[must_use]
pub fn made() -> Made {
    let same = stability::compare(&[run("approve", "a.rs"), run("approve", "a.rs")]);
    let flip = stability::compare(&[
        run("approve", "a.rs"),
        run("request_changes", "a.rs"),
        run("approve", "a.rs"),
    ]);
    let log = LogFile {
        file: "lang-rust.jsonl".to_owned(),
        text: r#"{"type":"result","subtype":"success","result":"x","total_cost_usd":0.25,"modelUsage":{"m":{"costUSD":9.0}}}"#.to_owned(),
        ..LogFile::default()
    };
    let cost = trace::spans(&[log], &[], &[], &|_| Default::default())
        .iter()
        .map(|s| s.cost_usd)
        .sum();
    let gap = Gap {
        name: HOSTILE.to_owned(),
        reason: HOSTILE.to_owned(),
        ..Gap::default()
    };
    let page = html::render(&trace::Trace::default(), &[gap]);

    Made {
        same,
        flip,
        cost,
        page,
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
            "a span's cost is {}, not the 0.25 its run log reported",
            made.cost
        ));
    }
    if made.page.contains("<script") {
        problems.push("the report page holds a run's text as markup".to_owned());
    }

    problems
}

/// Every problem with the probe, the corpus, the review workflow and
/// review-report in `checkout`.
///
/// # Errors
///
/// Fails when a file cannot be read.
pub fn problems(checkout: &Checkout, probe: &str, corpus: &str) -> Result<Vec<String>, String> {
    let mut problems = probe_problems(&checkout.read(probe)?);
    problems.extend(corpus_problems(&checkout.read(corpus)?, corpus::SAMPLES));
    problems.extend(review_problems(&checkout.read(REVIEW)?));
    problems.extend(report_problems(&made()));

    Ok(problems)
}

#[cfg(test)]
mod tests;
