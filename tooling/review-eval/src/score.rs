//! The scores: each run's grade and usage folded into its case, each
//! case into its component's suite on one model, and the suites into a
//! scorecard, with the findings the review gate weighs and the text a
//! person reads.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::log::{RunLog, Usage};

/// One run of a case.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub passed: bool,
    pub failures: Vec<String>,
    /// Departures from the report's shape the review survives: counted
    /// and shown, they fail no run.
    #[serde(default)]
    pub slips: Vec<String>,
    pub answered: bool,
    pub cost: f64,
    pub turns: usize,
    pub duration_ms: u64,
    pub resolved_model: String,
    pub usage: BTreeMap<String, Usage>,
    pub first_write_turn: Option<usize>,
    /// What the world could not build as the workflow does; no
    /// failure, but kept so a run's conditions are visible.
    pub notes: Vec<String>,
}

impl RunRecord {
    /// The record of a run whose log is `log` and whose grade left
    /// `failures` and `slips`.
    #[must_use]
    pub fn new(log: &RunLog, failures: Vec<String>, slips: Vec<String>) -> Self {
        Self {
            passed: failures.is_empty(),
            failures,
            slips,
            answered: log.answered,
            cost: log.cost,
            turns: log.turns,
            duration_ms: log.duration_ms,
            resolved_model: log.resolved_model.clone(),
            usage: log.usage.clone(),
            first_write_turn: log.first_write_turn,
            notes: Vec::new(),
        }
    }

    /// The record of a run that could not start or be read.
    #[must_use]
    pub fn failed(why: &str) -> Self {
        Self {
            failures: vec![why.to_owned()],
            ..Self::default()
        }
    }
}

/// One case, with its runs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CaseRecord {
    pub component: String,
    pub name: String,
    pub kind: String,
    pub title: String,
    /// How many runs the case may take, and how many it must pass.
    pub repetitions: usize,
    pub passes: usize,
    pub runs: Vec<RunRecord>,
    pub passed: bool,
}

impl CaseRecord {
    /// How many of its runs passed.
    #[must_use]
    pub fn passed_runs(&self) -> usize {
        self.runs.iter().filter(|r| r.passed).count()
    }

    /// How many of its runs slipped.
    #[must_use]
    pub fn slipped_runs(&self) -> usize {
        self.runs.iter().filter(|r| !r.slips.is_empty()).count()
    }

    /// The failures of its first failed run, for a finding.
    #[must_use]
    pub fn first_failure(&self) -> String {
        self.runs
            .iter()
            .find(|r| !r.passed)
            .map(|r| r.failures.join("; "))
            .unwrap_or_default()
    }
}

/// Whether a case is decided after `passes` passing and `fails`
/// failing runs of at most `repetitions`, needing `needed` passes:
/// passed once the passes are there, failed once they cannot be.
#[must_use]
pub fn decided(passes: usize, fails: usize, repetitions: usize, needed: usize) -> Option<bool> {
    if passes >= needed {
        Some(true)
    } else if fails > repetitions.saturating_sub(needed) {
        Some(false)
    } else {
        None
    }
}

/// One component's cases on one model.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SuiteRecord {
    pub component: String,
    /// The model the runs named: the front matter's, or the one the
    /// run was told to use.
    pub model: String,
    /// The hash of the component's file as evaluated.
    pub hash: String,
    pub cases: Vec<CaseRecord>,
}

impl SuiteRecord {
    /// Whether every case passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.cases.iter().all(|c| c.passed)
    }

    /// Cases passed, of all.
    #[must_use]
    pub fn cases_passed(&self) -> (usize, usize) {
        (
            self.cases.iter().filter(|c| c.passed).count(),
            self.cases.len(),
        )
    }

    /// Runs passed, of all.
    #[must_use]
    pub fn runs_passed(&self) -> (usize, usize) {
        let runs: Vec<&RunRecord> = self.cases.iter().flat_map(|c| &c.runs).collect();
        (runs.iter().filter(|r| r.passed).count(), runs.len())
    }

    /// How many runs slipped.
    #[must_use]
    pub fn slipped_runs(&self) -> usize {
        self.cases.iter().map(CaseRecord::slipped_runs).sum()
    }

    /// What every run cost together.
    #[must_use]
    pub fn cost(&self) -> f64 {
        self.cases
            .iter()
            .flat_map(|c| &c.runs)
            .map(|r| r.cost)
            .sum()
    }

    /// The mean turns per run; 0 without runs.
    #[must_use]
    pub fn mean_turns(&self) -> f64 {
        mean(
            self.cases
                .iter()
                .flat_map(|c| &c.runs)
                .map(|r| r.turns as f64),
        )
    }

    /// The mean seconds per run; 0 without runs.
    #[must_use]
    pub fn mean_seconds(&self) -> f64 {
        mean(
            self.cases
                .iter()
                .flat_map(|c| &c.runs)
                .map(|r| r.duration_ms as f64 / 1000.0),
        )
    }

    /// The model most runs resolved to; `unknown` without one.
    #[must_use]
    pub fn resolved(&self) -> String {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for run in self.cases.iter().flat_map(|c| &c.runs) {
            if !run.resolved_model.is_empty() {
                *counts.entry(&run.resolved_model).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map_or("unknown", |(model, _)| model)
            .to_owned()
    }

    /// The 95% Wilson score interval of the runs' pass rate; none
    /// without runs.
    #[must_use]
    pub fn interval(&self) -> Option<(f64, f64)> {
        let (passed, runs) = self.runs_passed();
        wilson(passed, runs)
    }
}

/// The mean of `values`; 0 of none.
fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, n) = values.fold((0.0, 0_usize), |(s, n), v| (s + v, n + 1));
    if n == 0 { 0.0 } else { sum / n as f64 }
}

/// The 95% Wilson score interval of `found` out of `asked`; none when
/// nothing was asked.
#[must_use]
pub fn wilson(found: usize, asked: usize) -> Option<(f64, f64)> {
    if asked == 0 {
        return None;
    }
    let (n, p) = (asked as f64, found as f64 / asked as f64);
    let z = 1.96_f64;
    let centre = p + z * z / (2.0 * n);
    let spread = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
    let scale = 1.0 + z * z / n;

    Some(((centre - spread) / scale, (centre + spread) / scale))
}

/// Every suite of one evaluation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scorecard {
    pub suites: Vec<SuiteRecord>,
}

/// A finding in the shape the review gate reads, one per failed case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateFinding {
    pub path: String,
    pub line: usize,
    pub severity: String,
    pub body: String,
}

impl Scorecard {
    /// Whether every suite passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.suites.iter().all(SuiteRecord::passed)
    }

    /// One blocking finding per failed case, at the component's file,
    /// `file_of` naming each component's.
    #[must_use]
    pub fn findings(&self, file_of: &dyn Fn(&str) -> String) -> Vec<GateFinding> {
        self.suites
            .iter()
            .flat_map(|suite| {
                suite.cases.iter().filter(|c| !c.passed).map(move |c| GateFinding {
                    path: file_of(&suite.component),
                    line: 0,
                    severity: "blocking".to_owned(),
                    body: format!(
                        "reviewer evaluation: case {}/{} ({}) passed {} of {} runs on {}, needing {}: {}",
                        c.component,
                        c.name,
                        c.title,
                        c.passed_runs(),
                        c.runs.len(),
                        suite.resolved(),
                        c.passes,
                        c.first_failure()
                    ),
                })
            })
            .collect()
    }

    /// The scorecard as Markdown: one section per suite, its cases in
    /// a table, each failed run's failures under it.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = String::from("## Reviewer evaluation\n");
        if self.suites.is_empty() {
            out.push_str("\nNo component was evaluated.\n");
            return out;
        }
        for suite in &self.suites {
            let (cp, ct) = suite.cases_passed();
            let (rp, rt) = suite.runs_passed();
            let interval = suite.interval().map_or(String::new(), |(lo, hi)| {
                format!(" (95% interval {:.1}% to {:.1}%)", lo * 100.0, hi * 100.0)
            });
            let _ = write!(
                out,
                "\n### {} on {} ({}): {} of {} cases, {} of {} runs{}, {} slipped, ${:.2}, {:.1} turns, {:.0} s per run\n\n| case | kind | runs | cost | turns | result |\n|---|---|---:|---:|---:|---|\n",
                suite.component,
                suite.model,
                suite.resolved(),
                cp,
                ct,
                rp,
                rt,
                interval,
                suite.slipped_runs(),
                suite.cost(),
                suite.mean_turns(),
                suite.mean_seconds()
            );
            for case in &suite.cases {
                let cost: f64 = case.runs.iter().map(|r| r.cost).sum();
                let turns = mean(case.runs.iter().map(|r| r.turns as f64));
                let _ = writeln!(
                    out,
                    "| {} | {} | {} of {} | ${:.2} | {:.1} | {} |",
                    case.name,
                    case.kind,
                    case.passed_runs(),
                    case.runs.len(),
                    cost,
                    turns,
                    if case.passed { "passed" } else { "failed" }
                );
            }
            for case in &suite.cases {
                run_lines(&mut out, case);
            }
        }

        out
    }
}

/// Each failed run of `case` with its failures, each slipped run with
/// its slips, and each run's notes, one bullet each.
fn run_lines(out: &mut String, case: &CaseRecord) {
    for (n, run) in case.runs.iter().enumerate() {
        if !run.passed {
            let _ = writeln!(
                out,
                "\n- `{}` run {}: {}",
                case.name,
                n + 1,
                run.failures.join("; ")
            );
        }
        if !run.slips.is_empty() {
            let _ = writeln!(
                out,
                "\n- `{}` run {} slipped: {}",
                case.name,
                n + 1,
                run.slips.join("; ")
            );
        }
        for note in &run.notes {
            let _ = writeln!(out, "\n- `{}` run {} note: {note}", case.name, n + 1);
        }
    }
}

#[cfg(test)]
mod tests;
