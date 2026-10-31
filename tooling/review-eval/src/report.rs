//! A report as data. A specialist's report, as the review assembles it
//! from the specialist's run log, holds its linked findings,
//! `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`,
//! and its requirement requests,
//! `path:line — request: add|change|remove <requirement> — <motivation>`,
//! read by the review gate's own rule (`review_gate::report`), and ends
//! with the outcome line, `outcome: findings|none — <sentence>`. A
//! generalist's answer holds its kept findings,
//! `path:line — blocking|nit — what it breaks — its source`, and its
//! dropped ones, `dropped — path:line — why — source`: severity is the
//! generalists' alone.

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
