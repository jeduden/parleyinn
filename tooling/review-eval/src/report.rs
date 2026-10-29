//! A report as data: the findings a reviewer component wrote, one per
//! line in the shape the shared report section asks for,
//! `path:line — blocking|nit — what it breaks — the ids it touches`,
//! and the outcome line the report ends with, as the review gate reads
//! it.

/// The outcomes a report may end with.
pub const OUTCOMES: [&str; 3] = ["sound", "nits", "blocking"];

/// One finding line, read.
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

/// A report, read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// The outcome the last line names.
    pub outcome: Option<String>,
    /// The last line as written, when it is an outcome line.
    pub outcome_line: Option<String>,
    /// Non-blank lines that are neither a finding, a heading nor the
    /// outcome line.
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
        } else if let Some(finding) = finding(line) {
            report.findings.push(finding);
        } else if !line.trim_start().starts_with('#') {
            report.other.push(line.trim().to_owned());
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

/// `line` without a leading list marker, emphasis or quote.
fn bare(line: &str) -> &str {
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

/// `line` as a finding, when it reads as one: `path:line`, a dash, a
/// severity word, a dash and the rest.
#[must_use]
pub fn finding(line: &str) -> Option<Finding> {
    let parts = split_dashes(bare(line));
    let [place, severity, rest @ ..] = parts.as_slice() else {
        return None;
    };
    let place = place.trim().trim_matches('`');
    let (path, number) = place.rsplit_once(':')?;
    let path = path.trim_matches('`').trim();
    let number: usize = number.trim().parse().ok()?;
    let severity = severity.trim().trim_matches(['`', '*', '_']);
    if path.is_empty() || severity.is_empty() || severity.contains(' ') {
        return None;
    }

    Some(Finding {
        path: path.to_owned(),
        line: number,
        severity: severity.to_lowercase(),
        text: rest.join(" — ").trim().to_owned(),
        raw: line.trim().to_owned(),
    })
}

/// `text` split at each dash that stands between spaces: an em dash,
/// an en dash, or one or two hyphens.
fn split_dashes(text: &str) -> Vec<&str> {
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
