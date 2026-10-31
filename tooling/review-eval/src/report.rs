//! A report as data. A specialist's report holds its linked findings,
//! `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`,
//! and its requirement requests,
//! `path:line — request: add|change|remove <requirement> — <motivation>`,
//! read by the review gate's own rule (`review_gate::report`), and ends
//! with the outcome line, `outcome: findings|none — <sentence>`. A
//! generalist's answer holds its kept findings,
//! `path:line — blocking|nit — what it breaks — its source`, and its
//! dropped ones, `dropped — path:line — why — source`: severity is the
//! generalists' alone.

use review_gate::report::Line;

pub use review_gate::report::{bare, split_dashes};

/// The outcomes a report may end with.
pub const OUTCOMES: [&str; 2] = review_gate::report::OUTCOMES;

/// One finding line of a generalist's answer, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub severity: String,
    /// The rest of the line after the severity.
    pub text: String,
    /// The whole line as written.
    pub raw: String,
}

/// A specialist's report, read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Its linked findings and requirement requests, in order.
    pub findings: Vec<Line>,
    /// Its finding lines in neither form, as written.
    pub malformed: Vec<String>,
    /// The outcome the last line names.
    pub outcome: Option<String>,
    /// The last line as written, when it is an outcome line.
    pub outcome_line: Option<String>,
    /// Non-blank lines that are neither a finding line, a heading nor
    /// the outcome line.
    pub other: Vec<String>,
}

/// Reads `text`.
#[must_use]
pub fn parse(text: &str) -> Report {
    let mut report = Report::default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let last = lines.len().saturating_sub(1);
    for (at, line) in lines.iter().enumerate() {
        if at == last
            && let Some(outcome) = outcome_word(line)
        {
            report.outcome = Some(outcome.to_owned());
            report.outcome_line = Some(line.trim().to_owned());
        } else {
            match review_gate::report::line(line) {
                Line::Malformed => report.malformed.push(line.trim().to_owned()),
                Line::Prose if line.trim_start().starts_with('#') => {}
                Line::Prose => report.other.push(line.trim().to_owned()),
                read => report.findings.push(read),
            }
        }
    }

    report
}

/// The outcome `line` names, when it is an outcome line: `outcome:`
/// and one of [`OUTCOMES`], after any emphasis or list marker.
#[must_use]
pub fn outcome_word(line: &str) -> Option<&'static str> {
    let rest = bare(line).strip_prefix("outcome:")?;
    let word = rest
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_ascii_alphanumeric());

    OUTCOMES.iter().find(|o| **o == word).copied()
}

/// The word a generalist's dropped-finding line starts with, or gives
/// as its severity.
pub const DROPPED: &str = "dropped";

/// Whether `line` reports a dropped finding: its first word is
/// [`DROPPED`], in any case, after any list marker or emphasis, as
/// `dropped — path:line — why — source`, or it reads as a finding whose
/// severity is [`DROPPED`]. A path that merely starts with the letters,
/// `dropped_events.rs:12`, is a finding on that path. The one rule the
/// evaluation's grader and the review's run both read a generalist's
/// answer by.
#[must_use]
pub fn dropped(line: &str) -> bool {
    bare(line).split_whitespace().next().is_some_and(|word| {
        word.trim_end_matches(['*', '_', '`', ':', ',', '.'])
            .eq_ignore_ascii_case(DROPPED)
    }) || finding(line).is_some_and(|f| f.severity == DROPPED)
}

/// The word a generalist's line starts with when it keeps, adds and
/// drops nothing.
pub const NONE: &str = "none";

/// Whether `line` says a generalist has nothing to add, in the shape
/// `none — nothing to add — source`: a first part of [`NONE`] alone,
/// in any case, after any list marker or emphasis, and a dash after it.
#[must_use]
pub fn nothing(line: &str) -> bool {
    match split_dashes(bare(line)).as_slice() {
        [first, _, ..] => first
            .trim()
            .trim_end_matches(['*', '_', '`'])
            .eq_ignore_ascii_case(NONE),
        _ => false,
    }
}

/// `line` as a generalist's finding, when it reads as one:
/// `path:line`, a dash, a severity word, a dash and the rest. A line
/// range, `path:12-20`, reads as its first line.
#[must_use]
pub fn finding(line: &str) -> Option<Finding> {
    let parts = split_dashes(bare(line));
    let [place, severity, rest @ ..] = parts.as_slice() else {
        return None;
    };
    let (path, number) = review_gate::report::place(place)?;
    let severity = severity.trim().trim_matches(['`', '*', '_']);
    if severity.is_empty() || severity.contains(' ') {
        return None;
    }

    Some(Finding {
        path,
        line: number,
        severity: severity.to_lowercase(),
        text: rest.join(" — ").trim().to_owned(),
        raw: line.trim().to_owned(),
    })
}

#[cfg(test)]
mod tests;
