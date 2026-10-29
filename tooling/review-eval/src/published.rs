//! The scorecard the repository publishes: one line per component,
//! with the model its front matter named, the model it resolved to,
//! the cases and runs passed, what the runs cost and took, and the
//! hash of the file evaluated. The stakeholder owns the file; a
//! scheduled run compares its measurement with it.

use std::fmt::Write as _;

use crate::score::{Scorecard, SuiteRecord};

/// The file's first line.
pub const HEADER: &str = "reviewer evaluation scorecard v1";

/// One component's line.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub component: String,
    pub model: String,
    pub resolved: String,
    pub cases_passed: usize,
    pub cases: usize,
    pub runs_passed: usize,
    pub runs: usize,
    /// What every run cost together, in dollars.
    pub cost: f64,
    /// The mean turns per run.
    pub turns: f64,
    /// The mean seconds per run.
    pub seconds: f64,
    pub hash: String,
}

impl Line {
    /// The runs' pass rate; 0 without runs.
    #[must_use]
    pub fn rate(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.runs_passed as f64 / self.runs as f64
        }
    }

    /// What one run cost on average; 0 without runs.
    #[must_use]
    pub fn cost_per_run(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.cost / self.runs as f64
        }
    }
}

impl From<&SuiteRecord> for Line {
    fn from(suite: &SuiteRecord) -> Self {
        let (cases_passed, cases) = suite.cases_passed();
        let (runs_passed, runs) = suite.runs_passed();
        Self {
            component: suite.component.clone(),
            model: suite.model.clone(),
            resolved: suite.resolved(),
            cases_passed,
            cases,
            runs_passed,
            runs,
            cost: suite.cost(),
            turns: suite.mean_turns(),
            seconds: suite.mean_seconds(),
            hash: suite.hash.clone(),
        }
    }
}

/// Each suite of `scorecard` as a line.
#[must_use]
pub fn lines(scorecard: &Scorecard) -> Vec<Line> {
    scorecard.suites.iter().map(Line::from).collect()
}

/// Reads a published scorecard.
///
/// # Errors
///
/// Fails on a missing header, or a line that does not read.
pub fn parse(text: &str) -> Result<Vec<Line>, String> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some(HEADER) {
        return Err(format!("the scorecard does not open with {HEADER:?}"));
    }
    lines
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| parse_line(l).map_err(|e| format!("line {}: {e}", i + 2)))
        .collect()
}

/// Reads one line of `key=value` fields.
fn parse_line(line: &str) -> Result<Line, String> {
    let field = |key: &str| -> Result<&str, String> {
        line.split_whitespace()
            .find_map(|f| f.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
            .ok_or_else(|| format!("no {key}"))
    };
    let number = |key: &str| -> Result<f64, String> {
        field(key)?
            .parse()
            .map_err(|_| format!("{key} is not a number"))
    };
    let ratio = |key: &str| -> Result<(usize, usize), String> {
        let (a, b) = field(key)?
            .split_once('/')
            .ok_or_else(|| format!("{key} is not passed/all"))?;
        match (a.parse(), b.parse()) {
            (Ok(a), Ok(b)) => Ok((a, b)),
            _ => Err(format!("{key} is not passed/all")),
        }
    };
    let component = field("component")?.to_owned();
    let model = field("model")?.to_owned();
    let resolved = field("resolved")?.to_owned();
    let (cases_passed, cases) = ratio("cases")?;
    let (runs_passed, runs) = ratio("runs")?;
    let cost = number("cost")?;
    let turns = number("turns")?;
    let seconds = number("seconds")?;
    let hash = field("hash")?.to_owned();

    Ok(Line {
        component,
        model,
        resolved,
        cases_passed,
        cases,
        runs_passed,
        runs,
        cost,
        turns,
        seconds,
        hash,
    })
}

/// Writes `lines` as a published scorecard, in component order.
#[must_use]
pub fn render(lines: &[Line]) -> String {
    let mut sorted: Vec<&Line> = lines.iter().collect();
    sorted.sort_by(|a, b| a.component.cmp(&b.component));
    let mut out = format!("{HEADER}\n");
    for l in sorted {
        let _ = writeln!(
            out,
            "component={} model={} resolved={} cases={}/{} runs={}/{} cost={:.4} turns={:.1} seconds={:.0} hash={}",
            l.component,
            l.model,
            l.resolved,
            l.cases_passed,
            l.cases,
            l.runs_passed,
            l.runs,
            l.cost,
            l.turns,
            l.seconds,
            l.hash
        );
    }

    out
}

/// `existing` with each of `new` in place of its component's line.
#[must_use]
pub fn merge(existing: Vec<Line>, new: Vec<Line>) -> Vec<Line> {
    let mut out: Vec<Line> = existing
        .into_iter()
        .filter(|e| !new.iter().any(|n| n.component == e.component))
        .collect();
    out.extend(new);
    out.sort_by(|a, b| a.component.cmp(&b.component));

    out
}

#[cfg(test)]
mod tests;
