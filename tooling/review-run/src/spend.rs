//! What a pull request's reviews spent (ENG-28): `review/spend.json`,
//! `{"spent_usd": <total>}`, the cost of the earlier reviews the review
//! was handed as `--spent-before` and of its own finished runs. It is
//! written when the review starts, again as each run ends, and once
//! more from the usage when the review ends, each time whole, so a
//! review cut off after it spent still leaves what it spent for the
//! next review of its pull request. A spend file past [`MAX_BYTES`] or
//! one that does not read is named, never taken for nothing spent (I6).

use std::fs;
use std::path::Path;

use serde::Deserialize;

/// Where, under the work directory, the review keeps what its pull
/// request's reviews spent.
pub const SPEND: &str = "review/spend.json";
/// The most bytes a spend file may hold.
pub const MAX_BYTES: usize = 1 << 10;

/// What a pull request's reviews spent, in US dollars.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Spend {
    spent_usd: f64,
}

/// The spend file's text for `spent` US dollars.
#[must_use]
pub fn text(spent: f64) -> String {
    format!("{{\"spent_usd\": {spent}}}\n")
}

/// Writes `text` to `path` whole: to a file beside it first, then moved
/// over it, so a reader never finds half of it.
///
/// # Errors
///
/// Fails when either file cannot be written or moved.
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    let part = path.with_extension("json.part");
    fs::write(&part, text).map_err(|e| format!("write {}: {e}", part.display()))?;
    fs::rename(&part, path)
        .map_err(|e| format!("rename {} to {}: {e}", part.display(), path.display()))
}

/// What the spend file's `text` says was spent.
///
/// # Errors
///
/// Fails when it is past [`MAX_BYTES`], does not read as a spend file,
/// or names no amount of zero or more.
pub fn check(text: &str) -> Result<f64, String> {
    if text.len() > MAX_BYTES {
        return Err(format!(
            "the spend file is {} bytes, past the {MAX_BYTES} a review reads",
            text.len()
        ));
    }
    let spend: Spend =
        serde_json::from_str(text).map_err(|e| format!("the spend file does not read: {e}"))?;
    if !(spend.spent_usd.is_finite() && spend.spent_usd >= 0.0) {
        return Err(format!(
            "the spend file names {} USD, no amount of zero or more",
            spend.spent_usd
        ));
    }

    Ok(spend.spent_usd)
}

#[cfg(test)]
mod tests;
