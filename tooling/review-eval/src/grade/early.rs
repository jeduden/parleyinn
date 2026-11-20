//! The slip of a generalist that stated a weighing line before its
//! final answer and left it out there. Its instructions ask for the
//! whole weighing in the final answer: what the model writes between
//! its tool calls reaches the log only when it is short, so a line
//! stated early is one the review may never read. The review reads the
//! whole log all the same (`report::weighing`), the more lenient on
//! purpose, so the line counts and the slip is shown.

use crate::report::strict;

/// Each weighing line of `texts`, in the strict form a message before
/// the final answer is read in, that no line of `answer` is, once and
/// in order. A run with no final answer was cut off before it, which
/// is its slip: it has none here.
pub(super) fn slips(texts: &[String], answer: &str) -> Vec<String> {
    if answer.trim().is_empty() {
        return Vec::new();
    }
    let mut early: Vec<&str> = Vec::new();
    for line in texts.iter().flat_map(|t| t.lines()).map(str::trim) {
        let held = answer.lines().any(|l| l.trim() == line);
        if strict(line) && !held && !early.contains(&line) {
            early.push(line);
        }
    }

    early
        .iter()
        .map(|line| format!("stated before its final answer, which does not hold it: {line}"))
        .collect()
}

#[cfg(test)]
mod tests;
