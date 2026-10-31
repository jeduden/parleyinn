//! A specialist's report, line by line (ENG-28). A specialist carries no
//! severity: each of its findings says what happens if it is not
//! addressed and links the requirements that outcome breaks, or asks
//! for a requirement to be added, changed or removed. Its generalist
//! alone weighs a finding into `blocking` or `nit`. The report ends
//! with its outcome: whether it holds findings, and one sentence.
//!
//! The specialist writes no file: it states each finding as one line in
//! its text as it finds it, and its final answer is the outcome line.
//! The review assembles the report from its run log ([`assemble`]):
//!
//! - a linked finding: `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`,
//!   each id an SRS requirement id or an invariant, `I1` to `I10`;
//! - a requirement request: `path:line — request: add|change|remove <requirement> — <motivation>`;
//! - a revision, `revised: <finding line>`, which replaces every earlier
//!   finding at the same `path:line`, or
//!   `revised: <old path:line> → <finding line>` when the place moved;
//! - a withdrawal, `withdrawn: path:line — <reason>`, which removes
//!   every earlier finding at that place;
//! - a note the assembly writes, never a specialist:
//!   `note: withdrew nothing at path:line — <reason>`, which a withdrawal
//!   naming a place no earlier line found at leaves in its stead;
//! - the outcome line, the report's last: `outcome: findings — <sentence>`
//!   or `outcome: none — <sentence>`.
//!
//! A line that starts with a place, `path:line` and a dash, is a finding
//! line and must take one of the two forms; a line that starts with
//! `revised:` or `withdrawn:` must take its form; any other line is
//! prose, which the assembly leaves out and the report check passes by.
//! A place may stand in emphasis, backticks or a link, `**a.rs:1**`,
//! or end a label, `finding: a.rs:1`, when `if not addressed:` or
//! `request:` follows its dash. A line whose place cannot be read, yet
//! with either after a dash, is a finding line too, so it is malformed,
//! never left out as prose; so is one missing the dash after its place,
//! `a.rs:1 if not addressed: …`, or with a dash and no place. A
//! malformed line whose place is the forms' own `path:line` is refused
//! naming that, so the reader sees the form was copied, not filled in.
//! A revision naming a place no earlier line found at states its
//! finding line, which is a whole finding; a withdrawal naming one
//! removes nothing and leaves its note, so it is neither an error nor
//! silent (I6).
//! A finding, revision or withdrawal line in no form is malformed, and
//! the report holding it is refused rather than handed to the generalist
//! marked unlinked: a severity word where the consequence belongs, or a
//! finding that links nothing, is the categorizing this format exists to
//! end, and a refused report fails the review closed, as a report that
//! ends with no outcome does. A linked finding citing an id no
//! requirement has is refused too, naming the id, so a made-up
//! requirement cannot carry a finding (`report_check` with the known
//! ids; the gate, which holds no tree, checks each id's shape alone).

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
/// What a revision starts with.
pub const REVISED: &str = "revised:";
/// What a withdrawal starts with.
pub const WITHDRAWN: &str = "withdrawn:";
/// What the note a withdrawal that removes nothing leaves starts with,
/// before its place; a line that starts with it is prose.
pub const WITHDREW_NOTHING: &str = "note: withdrew nothing at";
/// What stands between a revision's old place and its finding line.
pub const ARROWS: [&str; 2] = [" → ", " -> "];
/// What the first line of a report assembled from a run that did not
/// finish starts with.
pub const INCOMPLETE: &str = "incomplete:";
/// The place the forms are written with: a line that names it where its
/// place belongs copied a form instead of filling it in.
pub const TEMPLATE_PLACE: &str = "path:line";

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

/// A revision: the finding line that replaces every earlier finding at
/// `path:line`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    /// The place of the finding it replaces.
    pub path: String,
    pub line: usize,
    /// The finding line that stands in its place, bare: a linked
    /// finding or a request.
    pub finding: String,
}

/// A withdrawal: every earlier finding at `path:line` is removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Withdrawal {
    pub path: String,
    pub line: usize,
    pub reason: String,
}

/// One line of a specialist's report, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Linked(Linked),
    Request(Request),
    Revised(Revision),
    Withdrawn(Withdrawal),
    /// A finding, revision or withdrawal line in no form.
    Malformed,
    /// Any other line: blank, a heading or prose.
    Prose,
}

/// What a specialist's report is refused for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// The review holds no report for the specialist.
    Missing,
    /// Its report holds only whitespace.
    Empty,
    /// Its last line is no outcome line: its final answer gave none, or
    /// its run did not finish.
    NoOutcome,
    /// A finding, revision or withdrawal line, its line number in the
    /// report, in no form.
    Malformed { line: usize },
    /// A finding, revision or withdrawal line, its line number in the
    /// report, whose place is the forms' template, [`TEMPLATE_PLACE`],
    /// copied rather than filled in.
    TemplatePlace { line: usize },
    /// A linked finding, its line number in the report, cites an id no
    /// requirement or invariant has.
    UnknownId { line: usize, id: String },
    /// The outcome says none, yet the report holds a finding.
    NoneWithFindings,
    /// The outcome says findings, yet the report holds none.
    FindingsWithNone,
    /// Its final answer in the run log holds no outcome line.
    NoAnswerOutcome,
    /// Its final answer's outcome is not the one its report ends with.
    AnswerDiffers,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "has no report"),
            Self::Empty => write!(f, "has an empty report"),
            Self::NoOutcome => write!(f, "has a report that does not end with its outcome"),
            Self::Malformed { line } => write!(
                f,
                "has a report whose line {line} is neither a linked finding nor a requirement request"
            ),
            Self::TemplatePlace { line } => write!(
                f,
                "has a report whose line {line} names the place {TEMPLATE_PLACE}, the form's template, not a file and line"
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
            Self::NoAnswerOutcome => write!(f, "answered no outcome line in the run log"),
            Self::AnswerDiffers => write!(
                f,
                "answered another outcome in the run log than its report ends with"
            ),
        }
    }
}

/// Reads one line of a specialist's report.
#[must_use]
pub fn line(text: &str) -> Line {
    let text = bare(text);
    if strip_word(text, WITHDREW_NOTHING).is_some() {
        return Line::Prose;
    }
    if let Some(rest) = strip_word(text, REVISED) {
        return revision(rest).map_or(Line::Malformed, Line::Revised);
    }
    if let Some(rest) = strip_word(text, WITHDRAWN) {
        return withdrawal(rest).map_or(Line::Malformed, Line::Withdrawn);
    }
    let parts = split_dashes(text);
    let [first, second, rest @ ..] = parts.as_slice() else {
        return Line::Prose;
    };
    // A path holds no space: `I read line: 3 — fine` is prose. A line
    // that reads as a finding after a label, `finding: a.rs:1 — if not
    // addressed: …`, is read at the place the label ends with; one
    // whose place cannot be read is malformed, never dropped.
    let solid = |at: &(String, usize)| !at.0.contains(char::is_whitespace);
    let labelled = || {
        let last = first.split_whitespace().last()?;
        place(last).filter(|_| finding_shaped(&parts[..2]))
    };
    let Some((path, number)) = place(first).filter(solid).or_else(labelled) else {
        return if finding_shaped(&parts) || undashed(text) {
            Line::Malformed
        } else {
            Line::Prose
        };
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

/// Whether `parts`, a line cut at its dashes, reads as a finding: a
/// part after the first starts with `if not addressed:` or `request:`.
fn finding_shaped(parts: &[&str]) -> bool {
    parts.iter().skip(1).any(|part| {
        let part = part.trim();
        strip_word(part, IF_NOT_ADDRESSED).is_some() || strip_word(part, REQUEST).is_some()
    })
}

/// Whether `text` reads as a finding missing the spaced dash after its
/// place: a place or a dash, then `if not addressed:` or `request:`,
/// as `a.rs:1 if not addressed: …`, `a.rs:1— request: …` or
/// `— if not addressed: …`.
fn undashed(text: &str) -> bool {
    let (head, rest) = text.split_once(' ').unwrap_or((text, ""));
    let rest = match place(head.trim_end_matches(DASHES)) {
        Some(_) => rest,
        None if text.starts_with(DASHES) => text,
        None => return false,
    };
    let rest = rest.trim_start_matches(DASHES);

    strip_word(rest, IF_NOT_ADDRESSED).is_some() || strip_word(rest, REQUEST).is_some()
}

/// The dashes a place stands before, and a space.
const DASHES: [char; 4] = ['—', '–', '-', ' '];

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

/// A revision, `rest` after `revised:`: a finding line in one of the two
/// forms, after the old place and an arrow when the place moved.
fn revision(rest: &str) -> Option<Revision> {
    let (old, finding) = ARROWS
        .iter()
        .find_map(|arrow| rest.split_once(arrow))
        .map_or((None, rest), |(old, finding)| (Some(old), finding));
    let finding = bare(finding);
    let (path, number) = match line(finding) {
        Line::Linked(l) => (l.path, l.line),
        Line::Request(r) => (r.path, r.line),
        Line::Revised(_) | Line::Withdrawn(_) | Line::Malformed | Line::Prose => return None,
    };
    let (path, number) = match old {
        Some(old) => place(old).filter(|(p, _)| !p.contains(char::is_whitespace))?,
        None => (path, number),
    };

    Some(Revision {
        path,
        line: number,
        finding: finding.to_owned(),
    })
}

/// A withdrawal, `rest` after `withdrawn:`: a place, a dash and a
/// reason.
fn withdrawal(rest: &str) -> Option<Withdrawal> {
    let parts = split_dashes(rest);
    let (first, reason) = parts.split_first()?;
    let (path, number) = place(first).filter(|(p, _)| !p.contains(char::is_whitespace))?;
    let reason = reason.join(" — ").trim().to_owned();

    (!reason.is_empty()).then_some(Withdrawal {
        path,
        line: number,
        reason,
    })
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

/// `line` as an outcome line, trimmed, and the word it gives: it starts
/// with `outcome:`, in any case, and its first word, markup aside, is
/// one of [`OUTCOMES`].
fn outcome_of(line: &str) -> Option<(&str, &'static str)> {
    let line = line.trim();
    let rest = strip_word(bare(line), "outcome:")?;
    let word = rest
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_ascii_alphanumeric());

    OUTCOMES
        .iter()
        .find(|o| **o == word)
        .map(|outcome| (line, *outcome))
}

/// The outcome line a report `text` ends with, and the word it gives:
/// its last non-empty line, when that line is an outcome line.
#[must_use]
pub fn outcome_line(text: &str) -> Option<(&str, &'static str)> {
    let line = text.lines().map(str::trim).rfind(|l| !l.is_empty())?;
    outcome_of(line)
}

/// The outcome line a specialist's final `answer` gives, and the word
/// it gives: its last line that is one, so a blank or a stray line
/// after it is tolerated, and nothing else of the answer is read.
#[must_use]
pub fn answer_outcome(answer: &str) -> Option<(&str, &'static str)> {
    answer.lines().rev().find_map(outcome_of)
}

/// The report of a specialist, assembled from `texts`, its run log's
/// text blocks in order, and `answer`, its final answer: every finding
/// line replayed, a line identical to an earlier one dropped, a revision
/// put in place of every earlier finding at its place, a withdrawal
/// removing them, then the answer's outcome line. A revision that finds
/// nothing to replace states its finding line; a withdrawal that finds
/// nothing to remove leaves a note in its stead
/// ([`WITHDREW_NOTHING`]), so the reader sees it. A malformed line
/// stays as written, so the report check names it; prose is left out. A run
/// that did not finish passes `incomplete`, which heads the report and
/// says why, so a reader sees the report is partial.
#[must_use]
pub fn assemble(texts: &[String], answer: &str, incomplete: Option<&str>) -> String {
    let mut kept: Vec<Kept> = Vec::new();
    for raw in texts.iter().flat_map(|t| t.lines()) {
        let text = bare(raw).to_owned();
        match line(&text) {
            Line::Prose => {}
            read @ (Line::Linked(_) | Line::Request(_)) => {
                push_unique(&mut kept, Kept::of(render(&read).unwrap_or(text)));
            }
            Line::Malformed => push_unique(&mut kept, Kept::of(text)),
            Line::Revised(r) => {
                let finding = render(&line(&r.finding)).unwrap_or(r.finding);
                let finding = Kept::of(finding);
                if !replace(&mut kept, (&r.path, r.line), finding.clone()) {
                    push_unique(&mut kept, finding);
                }
            }
            Line::Withdrawn(w) => {
                let before = kept.len();
                kept.retain(|k| !k.at((&w.path, w.line)));
                if kept.len() == before {
                    let note = format!("{WITHDREW_NOTHING} {}:{} — {}", w.path, w.line, w.reason);
                    push_unique(&mut kept, Kept::of(note));
                }
            }
        }
    }
    let mut out = String::new();
    if let Some(why) = incomplete {
        out.push_str(&format!("{INCOMPLETE} {why}\n"));
    }
    for k in &kept {
        out.push_str(&k.text);
        out.push('\n');
    }
    if let Some((outcome, _)) = answer_outcome(answer) {
        out.push_str(outcome);
        out.push('\n');
    }

    out
}

/// `read`, a linked finding or a request, in its canonical shape, so
/// two writings of one finding are one line; none for any other line.
fn render(read: &Line) -> Option<String> {
    match read {
        Line::Linked(l) => Some(format!(
            "{}:{} — {IF_NOT_ADDRESSED} {} — {}",
            l.path,
            l.line,
            l.consequence,
            l.ids.join(", ")
        )),
        Line::Request(r) => Some(format!(
            "{}:{} — {REQUEST} {} {} — {}",
            r.path, r.line, r.change, r.requirement, r.motivation
        )),
        Line::Revised(_) | Line::Withdrawn(_) | Line::Malformed | Line::Prose => None,
    }
}

/// A line the assembly keeps, with the place it stands at when it has
/// one.
#[derive(Clone)]
struct Kept {
    text: String,
    place: Option<(String, usize)>,
}

impl Kept {
    /// `text`, a bare finding line, with its place: the first part
    /// before a dash, read as one.
    fn of(text: String) -> Self {
        let place = split_dashes(&text).first().and_then(|first| place(first));
        Self { text, place }
    }

    /// Whether the line stands at `at`.
    fn at(&self, at: (&str, usize)) -> bool {
        self.place
            .as_ref()
            .is_some_and(|(p, n)| (p.as_str(), *n) == at)
    }
}

/// Adds `k` to `kept` unless a line identical to it is there already.
fn push_unique(kept: &mut Vec<Kept>, k: Kept) {
    if !kept.iter().any(|have| have.text == k.text) {
        kept.push(k);
    }
}

/// Puts `with` where the first of `kept` at `at` stands and removes
/// the rest at `at`, unless a line identical to `with` stands elsewhere
/// already; whether anything stood at `at`.
fn replace(kept: &mut Vec<Kept>, at: (&str, usize), with: Kept) -> bool {
    let Some(first) = kept.iter().position(|k| k.at(at)) else {
        return false;
    };
    kept.retain(|k| !k.at(at));
    if !kept.iter().any(|k| k.text == with.text) {
        kept.insert(first, with);
    }

    true
}

/// The problem with a specialist's `report`, the text the review
/// assembled when it holds one, beside `answer`, its final answer in the
/// run log when it gave one, or none. With `known`, the ids a linked
/// finding may cite; without, each id is held to its shape alone. What
/// the gate refuses a review for, so the review's runner and the
/// evaluation's grader name the same problem.
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
    let Some((_, outcome)) = outcome_line(text) else {
        return Some(Problem::NoOutcome);
    };
    if let Some(problem) = findings_check(text, known) {
        return Some(problem);
    }
    let Some((_, answered)) = answer.and_then(answer_outcome) else {
        return Some(Problem::NoAnswerOutcome);
    };

    (answered != outcome).then_some(Problem::AnswerDiffers)
}

/// The problem with the lines of `text`, a report that ends with its
/// outcome line, or none: the first line in no form, the first id
/// outside `known` when it is given, or an outcome that says otherwise
/// than the lines do. A revision left in the text is read as the finding
/// it states, and a withdrawal left in it as nothing, as the assembly
/// reads them. A report with no outcome line is refused for that.
#[must_use]
pub fn findings_check(text: &str, known: Option<&BTreeSet<String>>) -> Option<Problem> {
    let Some((_, outcome)) = outcome_line(text) else {
        return Some(Problem::NoOutcome);
    };
    let mut held = false;
    for (at, l) in text.lines().enumerate() {
        let number = at + 1;
        match line(l) {
            Line::Prose | Line::Withdrawn(_) => {}
            Line::Malformed if at_template(l) => {
                return Some(Problem::TemplatePlace { line: number });
            }
            Line::Malformed => return Some(Problem::Malformed { line: number }),
            Line::Request(_) => held = true,
            Line::Revised(Revision { finding, .. }) => {
                held = true;
                if let Some(id) = unknown_id(&line(&finding), known) {
                    return Some(Problem::UnknownId { line: number, id });
                }
            }
            Line::Linked(linked) => {
                held = true;
                if let Some(id) = unknown_id(&Line::Linked(linked), known) {
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

/// Whether `text`, a malformed line, names [`TEMPLATE_PLACE`] where its
/// place belongs: the last word before its first dash, after the
/// `revised:` or `withdrawn:` it may start with.
fn at_template(text: &str) -> bool {
    let text = bare(text);
    let text = strip_word(text, REVISED)
        .or_else(|| strip_word(text, WITHDRAWN))
        .unwrap_or(text);
    split_dashes(text)
        .first()
        .and_then(|first| first.split_whitespace().last())
        .is_some_and(|last| {
            last.trim_matches(MARKUP)
                .eq_ignore_ascii_case(TEMPLATE_PLACE)
        })
}

/// The first id `read`, a linked finding, cites outside `known`, when
/// `known` is given; none for a request or any other line.
fn unknown_id(read: &Line, known: Option<&BTreeSet<String>>) -> Option<String> {
    let (Line::Linked(linked), Some(known)) = (read, known) else {
        return None;
    };
    linked.ids.iter().find(|id| !known.contains(*id)).cloned()
}

/// The markup a place may stand in: emphasis and backticks.
const MARKUP: [char; 3] = ['`', '*', '_'];

/// A finding's place, `path:line`, read: the path and the line, or the
/// first line of a range, `path:12-20`; emphasis and backticks around
/// either part or the whole are dropped, from either end, as is a
/// link's target: `[path:line](url)` reads as `path:line`.
#[must_use]
pub fn place(part: &str) -> Option<(String, usize)> {
    let part = link_text(part.trim().trim_matches(MARKUP)).trim_matches(MARKUP);
    let (path, number) = part.rsplit_once(':')?;
    let path = path.trim_matches(MARKUP).trim();
    let number: usize = number
        .trim()
        .split(['-', '–'])
        .next()?
        .trim()
        .parse()
        .ok()?;

    (!path.is_empty()).then(|| (path.to_owned(), number))
}

/// The text of `part` when it is a link, `[text](url)` or `[text]`;
/// `part` itself otherwise.
fn link_text(part: &str) -> &str {
    let Some(inner) = part.strip_prefix('[') else {
        return part;
    };
    match inner.split_once(']') {
        Some((text, "")) => text,
        Some((text, target)) if target.starts_with('(') && target.ends_with(')') => text,
        _ => part,
    }
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
