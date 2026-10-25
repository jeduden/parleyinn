//! Redaction before anything is written (SEC-08). A [`Redactor`] holds
//! the maintained redaction rule set, the harness account identifiers
//! learnt from a transcript, and the principal's redaction patterns
//! (`redaction.extra_patterns`). Every byte range one of them matches
//! is replaced by `[REDACTED:<rule>]`, which names the rule and carries
//! nothing derived from the secret. Ranges that overlap merge into one
//! marker, so no part of a secret two rules both match survives.

use std::fmt;

use serde_json::Value;

pub mod account;
pub mod rule;

use account::HarnessAccountIdentifiers;
use rule::{RedactionRule, rule_set};

/// Why a pattern does not compile, or compiles past the size limit.
pub type PatternError = regex::Error;

/// The rule name of the principal's `n`th redaction pattern, from 1.
#[must_use]
pub fn extra_pattern(n: usize) -> String {
    format!("extra_pattern_{n}")
}

/// A principal's redaction pattern that does not compile, or compiles
/// past the size limit: refused, while every other rule still applies.
#[derive(Debug, Clone)]
pub struct PatternRefused {
    /// The rule name it would have had.
    pub rule: String,
    pub source: regex::Error,
}

impl fmt::Display for PatternRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "redaction pattern {} refused", self.rule)
    }
}

impl std::error::Error for PatternRefused {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Text with its secrets replaced, and the rule of each replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted {
    pub text: String,
    /// One rule name per marker, in the order they appear.
    pub rules: Vec<String>,
    /// Whether the redaction scan limit was reached (`SCAN_LIMIT_FACTOR`) before
    /// the text beside the last matches was scanned again, so a secret
    /// a match hid from the boundary rules may stand in `text`. The
    /// caller counts it (I6); `text` loses nothing for it.
    pub limited: bool,
}

/// How many times the length of a text its scans may read together: a
/// clean text takes one, and the text around matches about one more.
const SCAN_LIMIT_FACTOR: usize = 8;

/// One step of [`Redactor::redact_text`]: text to scan, or the marker
/// of the rule that matched.
enum Work<'a> {
    Scan(&'a str),
    Marker(usize),
}

/// The rules redaction applies.
#[derive(Debug, Clone)]
pub struct Redactor {
    rules: Vec<RedactionRule>,
}

impl Redactor {
    /// The rule set and the principal's redaction patterns, with each
    /// pattern that is refused.
    ///
    /// # Errors
    ///
    /// Fails when the rule set does not compile, which its unit test
    /// rules out.
    pub fn new(extra_patterns: &[String]) -> Result<(Self, Vec<PatternRefused>), regex::Error> {
        let mut rules = rule_set()?;
        let mut refused = Vec::new();
        for (n, pattern) in (1..).zip(extra_patterns) {
            let rule = extra_pattern(n);
            match RedactionRule::new(&rule, pattern) {
                Ok(r) => rules.push(r),
                Err(source) => refused.push(PatternRefused { rule, source }),
            }
        }

        Ok((Self { rules }, refused))
    }

    /// This redactor with the harness account identifiers' rules ahead
    /// of the principal's redaction patterns.
    ///
    /// # Errors
    ///
    /// Fails when an identifier is too long to compile.
    pub fn with_identifiers(&self, ids: &HarnessAccountIdentifiers) -> Result<Self, regex::Error> {
        let set = rule::RULE_SET.len().min(self.rules.len());
        let mut rules = self.rules[..set].to_vec();
        rules.extend(ids.rules()?);
        rules.extend_from_slice(&self.rules[set..]);

        Ok(Self { rules })
    }

    /// `text` with every match replaced. The text around a match is
    /// scanned again on its own, because the match was what kept a
    /// secret beside it from matching (a key's word boundary hidden by
    /// a principal's pattern after it). A marker is never scanned, so
    /// no pattern matches inside one, and each rescan is of strictly
    /// less text, so it settles. The scans together read at most
    /// `SCAN_LIMIT_FACTOR` times the text, so a chain where every
    /// match uncovers only the next stays linear (I9); once the limit
    /// ends, the rest is scanned one last time without rescans and
    /// [`Redacted::limited`] says so. No text is dropped for it (I1).
    ///
    /// `REDACTION_RULE_SET_VERSION` stays as it is: it names the
    /// patterns, which this leaves alone, and a text the rescans
    /// change is one the patterns already matched under the same rule
    /// names, so the record's rule names still say which rules ran.
    #[must_use]
    pub fn redact_text(&self, text: &str) -> Redacted {
        self.redact_measured(text).0
    }

    /// [`Self::redact_text`] and how many bytes its scans read. The
    /// rescans read at most `SCAN_LIMIT_FACTOR - 1` times the text and
    /// the last scan, over disjoint stretches, at most once more, so
    /// the whole stays within `SCAN_LIMIT_FACTOR` times it.
    fn redact_measured(&self, text: &str) -> (Redacted, usize) {
        let mut out = Redacted {
            text: String::with_capacity(text.len()),
            rules: Vec::new(),
            limited: false,
        };
        let mut allowance = text.len().saturating_mul(SCAN_LIMIT_FACTOR - 1);
        let mut read = 0;
        let mut work = vec![Work::Scan(text)];
        while let Some(item) = work.pop() {
            match item {
                Work::Marker(rule) => self.marker(rule, &mut out),
                Work::Scan("") => {}
                Work::Scan(piece) if piece.len() > allowance => {
                    out.limited = true;
                    read += piece.len();
                    self.write_once(piece, &mut out);
                }
                Work::Scan(piece) => {
                    allowance -= piece.len();
                    read += piece.len();
                    self.scan(piece, &mut out, &mut work);
                }
            }
        }

        (out, read)
    }

    /// The marker of `rule` written to `out`.
    fn marker(&self, rule: usize, out: &mut Redacted) {
        let name = self.rules[rule].name();
        out.text.push_str(&format!("[REDACTED:{name}]"));
        out.rules.push(name.to_owned());
    }

    /// The merged matches of every rule in `piece`, in order.
    fn matches(&self, piece: &str) -> Vec<(usize, usize, usize)> {
        let mut spans: Vec<(usize, usize, usize)> = self
            .rules
            .iter()
            .enumerate()
            .flat_map(|(i, r)| r.spans(piece).map(move |(s, e)| (s, e, i)))
            .collect();
        spans.sort_unstable_by_key(|&(start, _, rule)| (start, rule));

        merged(spans)
    }

    /// One scan of `piece`: written out when nothing matches, else its
    /// stretches and markers queued in reverse on `work`.
    fn scan<'a>(&self, piece: &'a str, out: &mut Redacted, work: &mut Vec<Work<'a>>) {
        let spans = self.matches(piece);
        if spans.is_empty() {
            out.text.push_str(piece);
            return;
        }
        let mut at = piece.len();
        for (start, end, rule) in spans.into_iter().rev() {
            work.push(Work::Scan(&piece[end..at]));
            work.push(Work::Marker(rule));
            at = start;
        }
        work.push(Work::Scan(&piece[..at]));
    }

    /// `piece` with every match of one scan replaced and the stretches
    /// between left as they are.
    fn write_once(&self, piece: &str, out: &mut Redacted) {
        let mut at = 0;
        for (start, end, rule) in self.matches(piece) {
            out.text.push_str(&piece[at..start]);
            self.marker(rule, out);
            at = end;
        }
        out.text.push_str(&piece[at..]);
    }

    /// A transcript line or hook input redacted. Each JSON string in
    /// it is decoded and redacted on its own, so neither an escape nor
    /// a key the parser would drop as a duplicate hides a secret, and a
    /// line that is no JSON is redacted all the same; the text between
    /// strings is redacted as it stands. Only a string that is redacted
    /// changes, written back as a JSON string; every other byte stays.
    #[must_use]
    pub fn redact_line(&self, line: &str) -> Redacted {
        let mut out = Redacted {
            text: String::with_capacity(line.len()),
            rules: Vec::new(),
            limited: false,
        };
        for piece in pieces(line) {
            let r = match piece {
                Piece::Between(text) => self.redact_text(text),
                Piece::String(literal) => self.redact_literal(literal),
            };
            out.text.push_str(&r.text);
            out.rules.extend(r.rules);
            out.limited |= r.limited;
        }

        out
    }

    /// One JSON string literal, quotes included, redacted: decoded when
    /// it decodes, else as it stands.
    fn redact_literal(&self, literal: &str) -> Redacted {
        let Ok(decoded) = serde_json::from_str::<String>(literal) else {
            return self.redact_text(literal);
        };
        let r = self.redact_text(&decoded);
        if r.rules.is_empty() {
            return Redacted {
                text: literal.to_owned(),
                ..r
            };
        }

        Redacted {
            text: Value::String(r.text).to_string(),
            ..r
        }
    }
}

/// A piece of a line: a JSON string literal, quotes included, or the
/// text between two.
#[derive(Debug, PartialEq, Eq)]
enum Piece<'a> {
    Between(&'a str),
    String(&'a str),
}

/// `line` cut into its string literals and the text between them; a
/// literal left open runs to the end of the line.
fn pieces(line: &str) -> Vec<Piece<'_>> {
    let bytes = line.as_bytes();
    let (mut out, mut at) = (Vec::new(), 0);
    while let Some(open) = line[at..].find('"').map(|i| at + i) {
        if open > at {
            out.push(Piece::Between(&line[at..open]));
        }
        let close = literal_end(bytes, open + 1);
        out.push(Piece::String(&line[open..close]));
        at = close;
    }
    if at < line.len() {
        out.push(Piece::Between(&line[at..]));
    }

    out
}

/// The index past the quote that closes the literal whose content
/// starts at `from`, or the line's end when none does.
fn literal_end(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }

    bytes.len()
}

/// Sorted spans with each run of overlapping ones merged into the
/// first, which keeps its rule.
fn merged(spans: Vec<(usize, usize, usize)>) -> Vec<(usize, usize, usize)> {
    let mut out: Vec<(usize, usize, usize)> = Vec::with_capacity(spans.len());
    for (start, end, rule) in spans {
        match out.last_mut() {
            Some(last) if start < last.1 => last.1 = last.1.max(end),
            _ => out.push((start, end, rule)),
        }
    }

    out
}

#[cfg(test)]
mod tests;
