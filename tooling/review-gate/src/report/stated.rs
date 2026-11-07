//! What a specialist's report states, set apart from what it does not
//! (ENG-28, I6). A line that is no finding, or does not read as one,
//! must not discard the findings beside it: [`split`] keeps the lines
//! that read, sets aside the ones that say they are no finding, each
//! with why, and names each one that does not read with the problem the
//! report check gives it, so none is dropped unseen. [`with_outcome`]
//! then ends the lines that read with an outcome that agrees with them.

use std::collections::BTreeSet;
use std::fmt;

use super::{
    ALL_WELL, ALL_WELL_WORDS, FINDINGS, Line, MARKUP, Problem, REVISED, TEMPLATE_PLACE, WITHDRAWN,
    WITHDREW_NOTHING, all_well, answer_outcome, at_template, bare, defect_marked, form_problem,
    line, outcome_of, place, report_check, split_dashes, strip_word, unknown_id,
};

/// A report split into the lines that read and those that do not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split {
    /// Every line that reads, the outcome line among them, in order.
    pub clean: String,
    /// The lines that say they are no finding, set aside.
    pub ignored: Vec<Ignored>,
    /// The finding, revision and withdrawal lines that do not read.
    pub unreadable: Vec<Unreadable>,
}

/// A line set aside because it says it is no finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ignored {
    /// Its 1-based line number in the report split.
    pub line: usize,
    /// The line, trimmed.
    pub text: String,
    /// Why it is no finding.
    pub reason: Reason,
}

/// A finding, revision or withdrawal line that does not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreadable {
    /// Its 1-based line number in the report split.
    pub line: usize,
    /// The line, trimmed.
    pub text: String,
    /// What the report check refuses it for.
    pub problem: Problem,
}

/// Why a line is no finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// It says all is well at its place, or is the note that says so
    /// ([`ALL_WELL`]).
    AllWell,
    /// It is the note a withdrawal that removed nothing left
    /// ([`WITHDREW_NOTHING`]).
    WithdrewNothing,
    /// It is a revision or withdrawal whose place is a placeholder.
    Placeholder,
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AllWell => "says all is well, which is not a finding",
            Self::WithdrewNothing => "notes a withdrawal that removed nothing",
            Self::Placeholder => "names a placeholder where its place belongs, not a file and line",
        })
    }
}

/// `report` split into what reads and what does not, each id held to
/// its shape alone: [`split_with`] with no known ids.
#[must_use]
pub fn split(report: &str) -> Split {
    split_with(report, None)
}

/// `report` split into what reads and what does not. With `known`, a
/// finding citing an id outside it does not read.
#[must_use]
pub fn split_with(report: &str, known: Option<&BTreeSet<String>>) -> Split {
    let mut out = Split {
        clean: String::new(),
        ignored: Vec::new(),
        unreadable: Vec::new(),
    };
    for (at, raw) in report.lines().enumerate() {
        let number = at + 1;
        let text = raw.trim().to_owned();
        if let Some(reason) = reason(raw) {
            out.ignored.push(Ignored {
                line: number,
                text,
                reason,
            });
        } else if let Some(problem) = problem(raw, number, known) {
            out.unreadable.push(Unreadable {
                line: number,
                text,
                problem,
            });
        } else {
            out.clean.push_str(raw);
            out.clean.push('\n');
        }
    }

    out
}

/// Why `text`, a report's line, is no finding, or none when it may be
/// one.
fn reason(text: &str) -> Option<Reason> {
    let read = bare(text);
    if strip_word(read, ALL_WELL).is_some() {
        return Some(Reason::AllWell);
    }
    if strip_word(read, WITHDREW_NOTHING).is_some() {
        return Some(Reason::WithdrewNothing);
    }
    if line(text) != Line::Malformed {
        return None;
    }
    if placeholder(text) {
        Some(Reason::Placeholder)
    } else {
        (says_all_well(text) && !at_template(text)).then_some(Reason::AllWell)
    }
}

/// What the report check refuses `text`, the line at `number`, for, or
/// none when it reads.
fn problem(text: &str, number: usize, known: Option<&BTreeSet<String>>) -> Option<Problem> {
    let read = match line(text) {
        Line::Malformed => return Some(form_problem(text, number)),
        Line::Revised(r) => line(&r.finding),
        read => read,
    };
    unknown_id(&read, known).map(|id| Problem::UnknownId { line: number, id })
}

/// Whether `text`, a line at a place, says all is well there: it ends
/// with one of [`ALL_WELL_WORDS`] ([`all_well`]), or the part after its
/// place starts with one, alone or before punctuation, as `a.rs:1 — no
/// finding; the call sits in a test` reads, and what follows the word
/// holds none of [`super::DEFECT_MARKERS`]: `a.rs:1 — fine; but it
/// panics` may state a defect, so it is no note.
fn says_all_well(text: &str) -> bool {
    if all_well(text) {
        return true;
    }
    let parts = split_dashes(bare(text));
    let [first, second, rest @ ..] = parts.as_slice() else {
        return false;
    };
    let second = second.trim();
    place(first).is_some_and(|(path, _)| !path.contains(char::is_whitespace))
        && ALL_WELL_WORDS.iter().any(|word| {
            strip_word(second, word).is_some()
                && second[word.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| matches!(c, ';' | ':' | ',' | '.' | '!'))
                && !defect_marked(&second[word.len()..])
                && !rest.iter().any(|part| defect_marked(part))
        })
}

/// The places the forms' templates name in place of a file and line.
const TEMPLATE_PLACES: [&str; 2] = [TEMPLATE_PLACE, "<path>:<line>"];
/// What a placeholder path ends with: an ellipsis.
const ELLIPSES: [&str; 2] = ["…", "..."];

/// Whether `text` is a revision or withdrawal whose place, before its
/// first dash, is a placeholder: one of the forms' template places, or
/// a path that ends with an ellipsis and names no line, as
/// `docs/srs/…` does. A `<` or an ellipsis elsewhere does not make one:
/// such a line does not read.
fn placeholder(text: &str) -> bool {
    let read = bare(text);
    let Some(rest) = strip_word(read, REVISED).or_else(|| strip_word(read, WITHDRAWN)) else {
        return false;
    };
    let at = split_dashes(rest)
        .first()
        .copied()
        .unwrap_or_default()
        .trim()
        .trim_matches(MARKUP);
    TEMPLATE_PLACES.iter().any(|t| at.eq_ignore_ascii_case(t))
        || (ELLIPSES.iter().any(|e| at.ends_with(e))
            && !at.contains(char::is_whitespace)
            && place(at).is_none())
}

/// The number of findings and requests `clean`, a report's lines that
/// read, states: a revision left in it counts as the finding it states.
fn stated(clean: &str) -> usize {
    clean
        .lines()
        .filter(|l| {
            matches!(
                line(l),
                Line::Linked(_) | Line::Request(_) | Line::Revised(_)
            )
        })
        .count()
}

/// The outcome `clean`, a report's lines that read, gives: findings,
/// with how many it states, when it states one, else none.
#[must_use]
pub fn derived_outcome(clean: &str) -> String {
    match stated(clean) {
        0 => "outcome: none — nothing stated in the run log".to_owned(),
        1 => "outcome: findings — 1 finding stated in the run log".to_owned(),
        n => format!("outcome: findings — {n} findings stated in the run log"),
    }
}

/// `clean`, a report's lines that read, its outcome lines left out,
/// ended with an outcome that agrees with its lines: `answer`'s, the
/// specialist's final answer, when it gives one that agrees, else the
/// [`derived_outcome`]. So the report always ends with an outcome, and
/// never one its lines contradict.
#[must_use]
pub fn with_outcome(clean: &str, answer: &str) -> String {
    let held = stated(clean) > 0;
    let ending = answer_outcome(answer)
        .filter(|(_, word)| (*word == FINDINGS) == held)
        .map_or_else(|| derived_outcome(clean), |(l, _)| l.to_owned());
    let mut out: String = clean
        .lines()
        .filter(|l| outcome_of(l).is_none())
        .map(|l| format!("{l}\n"))
        .collect();
    out.push_str(&ending);
    out.push('\n');

    out
}

/// The problem with `report`, one [`with_outcome`] built, or none: the
/// report check with the report's own outcome as its answer, since the
/// outcome is the one its lines give.
#[must_use]
pub fn stated_check(report: &str, known: Option<&BTreeSet<String>>) -> Option<Problem> {
    report_check(Some(report), Some(report), known)
}

#[cfg(test)]
mod tests;
