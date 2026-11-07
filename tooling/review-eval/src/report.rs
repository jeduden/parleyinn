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

/// A generalist's weighing as its run log gives it: each weighing line
/// it stated in a message before its last, a finding, a dropped one or
/// its `none —`, or a line shaped as a finding the review cannot read
/// that starts with a path-like word ([`placed`]), each once and in
/// order, then its final answer as it stands. A line at a place, a
/// finding or a dropped one, supersedes every line stated before it at
/// that place, the final answer's lines included: the last statement
/// wins, so a draft finding can be dropped or revised later. A run that
/// was cut off before its final answer has the lines it stated and no
/// more, so the review reads what was paid for.
#[must_use]
pub fn weighing(texts: &[String], answer: &str) -> String {
    let said: Vec<&str> = answer.lines().map(str::trim).collect();
    let mut stated: Vec<&str> = Vec::new();
    for line in texts.iter().flat_map(|t| t.lines()).map(str::trim) {
        // Mid-run narration may read as a finding's shape, `Severity —
        // nit — …`; only a line that names a path is one the agent meant
        // as a finding, so only such a line is held unreadable.
        let unreadable = unread(line) && placed(line);
        let weighs = finding(line).is_some() || dropped(line) || nothing(line) || unreadable;
        if weighs && !said.contains(&line) && !stated.contains(&line) {
            supersede(&mut stated, line);
            stated.push(line);
        }
    }
    for line in &said {
        supersede(&mut stated, line);
    }
    let mut out = stated.join("\n");
    if !out.is_empty() && !answer.is_empty() {
        out.push('\n');
    }
    out.push_str(answer);

    out
}

/// Removes from `stated` each line at the place `line` is at, when it
/// is at one.
fn supersede(stated: &mut Vec<&str>, line: &str) {
    if let Some(at) = place_of(line) {
        stated.retain(|l| place_of(l).as_ref() != Some(&at));
    }
}

/// The place, path and line, a generalist's finding or dropped line is
/// at: a finding's own, or the part after a leading `dropped —`; none
/// for any other line.
#[must_use]
pub fn place_of(line: &str) -> Option<(String, usize)> {
    if let Some(f) = finding(line) {
        return Some((f.path, f.line));
    }
    if !dropped(line) {
        return None;
    }
    split_dashes(bare(line))
        .get(1)
        .and_then(|part| review_gate::report::place(part))
}

/// Whether `line` starts with a path-like word: its first part, markup
/// aside, is one word holding a `/`, a `.` or a `:`.
#[must_use]
pub fn placed(line: &str) -> bool {
    split_dashes(bare(line)).first().is_some_and(|part| {
        let word = part.trim().trim_matches(['`', '*', '_']);
        !word.is_empty() && !word.contains(char::is_whitespace) && word.contains(['/', '.', ':'])
    })
}

/// The severities a generalist's finding gives.
pub const SEVERITIES: [&str; 2] = ["blocking", "nit"];

/// Whether `line` is a generalist's finding the review cannot read: a
/// part after its first is a severity of [`SEVERITIES`] alone, markup
/// aside, yet the line reads as no finding and no dropped one, its
/// place unreadable. Such a line is never left out as prose.
#[must_use]
pub fn unread(line: &str) -> bool {
    let shaped = split_dashes(bare(line)).iter().skip(1).any(|part| {
        let word = part.trim().trim_matches(['`', '*', '_']);
        SEVERITIES.iter().any(|s| s.eq_ignore_ascii_case(word))
    });

    shaped && finding(line).is_none() && !dropped(line)
}

#[cfg(test)]
mod tests;
