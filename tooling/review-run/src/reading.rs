//! How the review reads a specialist's run log, whatever result the run
//! ended with (ENG-28). Its report is assembled from the lines it
//! stated, split into what reads and what does not, and ended with an
//! outcome its lines give, the specialist's own when it agrees: a line
//! that says it is no finding is set aside and counted, and a
//! finding-shaped line that does not read is named on its own, never
//! discarding the findings beside it. Whether the specialist read its
//! whole diff is decided from the `Read` calls its log shows, never from
//! what it says: each range of its diff file no read covered is named.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

use review_bundle::cli::DIFFS;
use review_eval::coverage;
use review_eval::log::ReadCall;
use review_gate::report::{assemble, outcome_line, split_with, with_outcome};
use serde::{Deserialize, Serialize};

use crate::REVIEW;

/// The lines of the specialists' reports the review set aside as no
/// finding, beside the review outcome.
pub const IGNORED: &str = "review/ignored.json";

/// A line of a specialist's report the review set aside, or could not
/// read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportLine {
    /// The specialist.
    pub agent: String,
    /// The part of its split diff, from one; absent when it ran whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<usize>,
    /// Its 1-based number in the report as its log assembles it.
    pub line: usize,
    /// The line, trimmed.
    pub text: String,
    /// Why it is set aside, or the problem the report check gives it.
    pub why: String,
}

/// A specialist's report as the review reads it from its log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    /// The lines that read, ended with its outcome line.
    pub report: String,
    /// That outcome line: what the review takes as its answer.
    pub answer: String,
    /// The lines set aside as no finding.
    pub ignored: Vec<ReportLine>,
    /// The finding-shaped lines that do not read.
    pub unreadable: Vec<ReportLine>,
}

/// The report of specialist `agent`, or of its part `part`, read from
/// `texts`, its log's text blocks, and `answer`, its final answer, each
/// id held to `known`.
#[must_use]
pub fn read(
    (agent, part): (&str, Option<usize>),
    texts: &[String],
    answer: &str,
    known: &BTreeSet<String>,
) -> Read {
    let split = split_with(&assemble(texts, answer, None), Some(known));
    let report = with_outcome(&split.clean, answer);
    let line = |line: usize, text: String, why: String| ReportLine {
        agent: agent.to_owned(),
        part,
        line,
        text,
        why,
    };

    Read {
        answer: answer_of(&report),
        ignored: split
            .ignored
            .into_iter()
            .map(|i| line(i.line, i.text, i.reason.to_string()))
            .collect(),
        unreadable: split
            .unreadable
            .into_iter()
            .map(|u| line(u.line, u.text, u.problem.to_string()))
            .collect(),
        report,
    }
}

/// The outcome line `report` ends with, or nothing when it ends with
/// none.
#[must_use]
pub fn answer_of(report: &str) -> String {
    outcome_line(report).map_or_else(String::new, |(line, _)| line.to_owned())
}

/// The diff file whose log stem is `stem`, a specialist's name or a
/// part's, relative to the work directory.
#[must_use]
pub fn diff_file(stem: &str) -> String {
    format!("{REVIEW}/{DIFFS}/{stem}.diff")
}

/// Why the run of `stem`, whose log shows `reads`, did not read its
/// whole diff under `workdir`: the ranges of it no read covered, or that
/// its diff is not there or does not read; none when it read every line
/// of it, an empty diff having none. Every specialist, and every part,
/// is told to read its diff, so one not found is never a whole review. Only a read of that very file counts: a relative path
/// under the work directory, or an absolute one under it as named or as
/// resolved ([`coverage::unread`]).
#[must_use]
pub fn uncovered(workdir: &Path, stem: &str, reads: &[ReadCall]) -> Option<String> {
    let rel = diff_file(stem);
    let path = workdir.join(&rel);
    let text = match fs::read(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Some(format!(
                "its diff {rel} was not found, so no read of it can be shown"
            ));
        }
        Err(e) => return Some(format!("its diff {rel} does not read: {e}")),
    };
    let total = String::from_utf8_lossy(&text).lines().count();
    // The work directory as named and as resolved: an agent's tool may
    // log either in an absolute path.
    let roots: Vec<PathBuf> = [path::absolute(workdir).ok(), fs::canonicalize(workdir).ok()]
        .into_iter()
        .flatten()
        .collect();
    let roots: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
    let unread = coverage::unread(reads, &roots, &rel, total);
    (!unread.is_empty()).then(|| {
        let held = if total == 1 { "line" } else { "lines" };
        format!(
            "its log shows no Read of {} of {rel}, of {total} {held}",
            ranges(&unread)
        )
    })
}

/// `ranges`, 1-based and inclusive, said: `line 7`, `lines 1-40, 81`.
fn ranges(ranges: &[(usize, usize)]) -> String {
    let mut said = String::new();
    for (start, end) in ranges {
        if !said.is_empty() {
            said.push_str(", ");
        }
        if start == end {
            let _ = write!(said, "{start}");
        } else {
            let _ = write!(said, "{start}-{end}");
        }
    }
    let one = matches!(ranges, [(start, end)] if start == end);

    format!("{} {said}", if one { "line" } else { "lines" })
}

#[cfg(test)]
mod tests;
