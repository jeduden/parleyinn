//! A specialist's report, line by line (ENG-28). A specialist carries no
//! severity: each of its findings says what happens if it is not
//! addressed and links the requirements that outcome breaks, or asks
//! for a requirement to be added, changed or removed. Its generalist
//! alone weighs a finding into `blocking` or `nit`. The report ends
//! with its outcome: whether it holds findings, and one sentence.
//!
//! - a linked finding: `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`,
//!   each id an SRS requirement id or an invariant, `I1` to `I10`;
//! - a requirement request: `path:line — request: add|change|remove <requirement> — <motivation>`;
//! - the outcome line, the report's last: `outcome: findings — <sentence>`
//!   or `outcome: none — <sentence>`.
//!
//! A line that starts with a place, `path:line` and a dash, is a finding
//! line and must take one of the two forms; any other line is prose,
//! which the report check passes by as it always has. A finding line in
//! neither form is malformed, and the report holding it is refused
//! rather than handed to the generalist marked unlinked: a severity
//! word where the consequence belongs, or a finding that links nothing,
//! is the categorizing this format exists to end, and a refused report
//! fails the review closed, as a report that ends with no outcome does.
//! A linked finding citing an id no requirement has is refused too,
//! naming the id, so a made-up requirement cannot carry a finding
//! (`report_check` with the known ids; the gate, which holds no tree,
//! checks each id's shape alone).

use std::collections::BTreeSet;
use std::fmt;

/// The outcome of a report that holds a finding or a request.
pub const FINDINGS: &str = "findings";
/// The outcome of a report that holds neither.
pub const NONE: &str = "none";
/// The words a report's outcome line may give.
pub const OUTCOMES: [&str; 2] = [FINDINGS, NONE];

/// What a linked finding's second part starts with.
pub const IF_NOT_ADDRESSED: &str = "if not addressed:";
/// What a requirement request's second part starts with.
pub const REQUEST: &str = "request:";
/// The changes a requirement request may ask for.
pub const CHANGES: [&str; 3] = ["add", "change", "remove"];

/// A finding linked to the requirements its consequence breaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linked {
    pub path: String,
    pub line: usize,
    /// What happens if the finding is not addressed.
    pub consequence: String,
    /// The requirement ids and invariants it links, in order.
    pub ids: Vec<String>,
}

/// A request that a requirement be added, changed or removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub path: String,
    pub line: usize,
    /// `add`, `change` or `remove`.
    pub change: String,
    /// The requirement, in the specialist's words.
    pub requirement: String,
    /// Why, informally.
    pub motivation: String,
}

/// One line of a specialist's report, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Linked(Linked),
    Request(Request),
    /// A finding line, one that starts with a place, in neither form.
    Malformed,
    /// Any other line: blank, a heading or prose.
    Prose,
}

/// What a specialist's report is refused for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// The specialist wrote no report file.
    Missing,
    /// Its report holds only whitespace.
    Empty,
    /// Its last line is no outcome line.
    NoOutcome,
    /// A finding line, its line number in the file, in neither form.
    Malformed { line: usize },
    /// A linked finding, its line number in the file, cites an id no
    /// requirement or invariant has.
    UnknownId { line: usize, id: String },
    /// The outcome says none, yet the report holds a finding.
    NoneWithFindings,
    /// The outcome says findings, yet the report holds none.
    FindingsWithNone,
    /// No line of its answer repeats its outcome line.
    AnswerDiffers,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "wrote no report"),
            Self::Empty => write!(f, "wrote an empty report"),
            Self::NoOutcome => write!(f, "has a report that does not end with its outcome"),
            Self::Malformed { line } => write!(
                f,
                "has a report whose line {line} is neither a linked finding nor a requirement request"
            ),
            Self::UnknownId { line, id } => write!(
                f,
                "has a report whose line {line} cites {id}, which no requirement or invariant has"
            ),
            Self::NoneWithFindings => {
                write!(
                    f,
                    "has a report whose outcome is none but which holds findings"
                )
            }
            Self::FindingsWithNone => {
                write!(
                    f,
                    "has a report whose outcome is findings but which holds none"
                )
            }
            Self::AnswerDiffers => write!(
                f,
                "has a report whose outcome differs from its answer in the run log"
            ),
        }
    }
}

/// Reads one line of a specialist's report.
#[must_use]
pub fn line(text: &str) -> Line {
    let parts = split_dashes(bare(text));
    let [first, second, rest @ ..] = parts.as_slice() else {
        return Line::Prose;
    };
    // A path holds no space: `I read line: 3 — fine` is prose.
    let Some((path, number)) = place(first).filter(|(p, _)| !p.contains(char::is_whitespace))
    else {
        return Line::Prose;
    };
    let second = second.trim();
    let read = if let Some(consequence) = strip_word(second, IF_NOT_ADDRESSED) {
        linked(path, number, consequence, rest).map(Line::Linked)
    } else if let Some(asked) = strip_word(second, REQUEST) {
        request(path, number, asked, rest).map(Line::Request)
    } else {
        None
    };

    read.unwrap_or(Line::Malformed)
}

/// `text` after `word`, in any case, when it starts with it.
fn strip_word<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    text.get(..word.len())
        .filter(|head| head.eq_ignore_ascii_case(word))
        .map(|_| text[word.len()..].trim())
}

/// A linked finding at `path:number`: a consequence, and after the last
/// dash its ids, each well formed.
fn linked(path: String, number: usize, consequence: &str, rest: &[&str]) -> Option<Linked> {
    let [middle @ .., ids] = rest else {
        return None;
    };
    let consequence = std::iter::once(consequence)
        .chain(middle.iter().map(|p| p.trim()))
        .collect::<Vec<_>>()
        .join(" — ");
    let ids: Vec<String> = ids
        .split(',')
        .map(|id| id.trim().trim_matches(['`', '*', '_', '.']).to_owned())
        .collect();
    (!consequence.is_empty() && ids.iter().all(|id| id_shaped(id))).then_some(Linked {
        path,
        line: number,
        consequence,
        ids,
    })
}

/// A requirement request at `path:number`: a change word and the
/// requirement, then after a dash its motivation.
fn request(path: String, number: usize, asked: &str, rest: &[&str]) -> Option<Request> {
    let (change, requirement) = asked.split_once(' ')?;
    let change = change.to_lowercase();
    let requirement = requirement.trim();
    let motivation = rest.join(" — ").trim().to_owned();
    (CHANGES.contains(&change.as_str()) && !requirement.is_empty() && !motivation.is_empty()).then(
        || Request {
            path,
            line: number,
            change,
            requirement: requirement.to_owned(),
            motivation,
        },
    )
}

/// Whether `id` has the shape of a requirement id, a family and a
/// number, `ENG-28`, or of an invariant, `I2`.
fn id_shaped(id: &str) -> bool {
    let digits = |n: &str| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit());
    match id.split_once('-') {
        Some((family, n)) => {
            !family.is_empty() && family.bytes().all(|b| b.is_ascii_uppercase()) && digits(n)
        }
        None => id.strip_prefix('I').is_some_and(digits),
    }
}

/// The invariants every report may cite, `I1` to `I10`.
#[must_use]
pub fn invariants() -> BTreeSet<String> {
    (1..=10).map(|n| format!("I{n}")).collect()
}

/// The outcome line `text` ends with, trimmed, and the word it gives:
/// its last non-empty line, when that line starts with `outcome:`, in
/// any case, and its first word, markup aside, is one of [`OUTCOMES`].
#[must_use]
pub fn outcome_line(text: &str) -> Option<(&str, &'static str)> {
    let line = text.lines().map(str::trim).rfind(|l| !l.is_empty())?;
    let rest = strip_word(line, "outcome:")?;
    let word = rest
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_ascii_alphanumeric());

    OUTCOMES
        .iter()
        .find(|o| **o == word)
        .map(|outcome| (line, *outcome))
}

/// The problem with a specialist's `report`, its file's text when it
/// has one, beside `answer`, its final answer when it gave one, or none.
/// With `known`, the ids a linked finding may cite; without, each id is
/// held to its shape alone. What the gate refuses a review for, so the
/// review's runner and the evaluation's grader name the same problem.
#[must_use]
pub fn report_check(
    report: Option<&str>,
    answer: Option<&str>,
    known: Option<&BTreeSet<String>>,
) -> Option<Problem> {
    let text = match report.map(str::trim) {
        None => return Some(Problem::Missing),
        Some("") => return Some(Problem::Empty),
        Some(text) => text,
    };
    let Some((outcome, _)) = outcome_line(text) else {
        return Some(Problem::NoOutcome);
    };
    if let Some(problem) = findings_check(text, known) {
        return Some(problem);
    }

    (!answer.is_some_and(|answer| answer.lines().any(|l| same_outcome(outcome, l))))
        .then_some(Problem::AnswerDiffers)
}

/// The problem with the lines of `text`, a report that ends with its
/// outcome line, or none: the first finding line in neither form, the
/// first id outside `known` when it is given, or an outcome that says
/// otherwise than the lines do. A report with no outcome line is
/// refused for that.
#[must_use]
pub fn findings_check(text: &str, known: Option<&BTreeSet<String>>) -> Option<Problem> {
    let Some((_, outcome)) = outcome_line(text) else {
        return Some(Problem::NoOutcome);
    };
    let mut held = false;
    for (at, l) in text.lines().enumerate() {
        let number = at + 1;
        match line(l) {
            Line::Prose => {}
            Line::Malformed => return Some(Problem::Malformed { line: number }),
            Line::Request(_) => held = true,
            Line::Linked(linked) => {
                held = true;
                let unknown = known.and_then(|k| linked.ids.into_iter().find(|id| !k.contains(id)));
                if let Some(id) = unknown {
                    return Some(Problem::UnknownId { line: number, id });
                }
            }
        }
    }

    match (outcome == NONE, held) {
        (true, true) => Some(Problem::NoneWithFindings),
        (false, false) => Some(Problem::FindingsWithNone),
        _ => None,
    }
}

/// Whether an answer's `line` repeats the report's `outcome` line:
/// the same words, whatever their case, spacing, emphasis markup, kind
/// of dash or trailing punctuation.
#[must_use]
pub fn same_outcome(outcome: &str, line: &str) -> bool {
    normalised(line) == normalised(outcome)
}

/// `text` lowercased, without emphasis or code markup, with every dash
/// a hyphen, its whitespace collapsed to single spaces and no trailing
/// punctuation: what a harmless retelling of a line does not change.
fn normalised(text: &str) -> String {
    let words: String = text
        .chars()
        .filter(|c| !matches!(c, '*' | '_' | '`' | '~'))
        .map(|c| match c {
            '\u{2012}'..='\u{2015}' | '\u{2212}' => '-',
            c => c.to_ascii_lowercase(),
        })
        .collect();
    let spaced = words.split_whitespace().collect::<Vec<_>>().join(" ");

    spaced
        .trim_end_matches(['.', ',', ';', ':', '!', '?', ' '])
        .to_owned()
}

/// A finding's place, `path:line`, read: the path and the line, or the
/// first line of a range, `path:12-20`; backticks around either part or
/// the whole are dropped.
#[must_use]
pub fn place(part: &str) -> Option<(String, usize)> {
    let part = part.trim().trim_matches('`');
    let (path, number) = part.rsplit_once(':')?;
    let path = path.trim_matches('`').trim();
    let number: usize = number
        .trim()
        .split(['-', '–'])
        .next()?
        .trim()
        .parse()
        .ok()?;

    (!path.is_empty()).then(|| (path.to_owned(), number))
}

/// `line` without a leading list marker, emphasis or quote.
#[must_use]
pub fn bare(line: &str) -> &str {
    let line = line.trim();
    let line = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .unwrap_or(line);
    let line = strip_number(line);
    line.trim_start_matches(['*', '_', '`', '>', ' '])
}

/// `line` without a leading `N. `.
fn strip_number(line: &str) -> &str {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    match line[digits..].strip_prefix(". ") {
        Some(rest) if digits > 0 => rest,
        _ => line,
    }
}

/// `text` split at each dash that stands between spaces: an em dash,
/// an en dash, or one or two hyphens.
#[must_use]
pub fn split_dashes(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        let cut = [" — ", " – ", " -- ", " - "]
            .iter()
            .filter_map(|sep| rest.find(sep).map(|at| (at, sep.len())))
            .min();
        match cut {
            Some((at, len)) => {
                out.push(&rest[..at]);
                rest = &rest[at + len..];
            }
            None => {
                out.push(rest);
                return out;
            }
        }
    }
}

#[cfg(test)]
mod tests;
