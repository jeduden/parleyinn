//! A review's verdict kept for a comment (ENG-28): a stakeholder's
//! comment brings a review, since consent is a comment, and that review
//! runs no agent when an earlier review of the same head already took
//! every dispatched agent's answer. `review-run previous verdict` weighs
//! what the earlier review kept, its run id, gaps, dispatch, outcome and
//! consulted agents, against the head and the dispatch of the review
//! the comment brought: it is reused only when the head is the same
//! commit, the dispatch names the same agents, the earlier review left
//! no gap and its outcome and consulted agents read. Anything else, a
//! file that is missing or does not read included, is named and never
//! reused, so the agents run (I6). The deterministic findings are never
//! reused: the review bundle states them again, the comment among its
//! inputs.

use std::fs;
use std::path::Path;

use crate::{CONSULTED, OUTCOME, REVIEW, gaps, run_id};

/// The earlier review's dispatch, under its review directory.
const DISPATCH: &str = "review/dispatch.json";

/// Whether the earlier review's verdict stands for this one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// It does: no agent runs.
    Reused,
    /// It does not, and why: the agents run.
    Fresh(String),
}

impl Verdict {
    /// The lines the job's outputs take.
    #[must_use]
    pub fn lines(&self) -> String {
        match self {
            Self::Reused => "reused=true\n".to_owned(),
            Self::Fresh(why) => format!("reused=false\nwhy={why}\n"),
        }
    }
}

/// What the earlier review kept, each file's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub run_id: String,
    pub gaps: String,
    pub dispatch: String,
    pub outcome: String,
    pub consulted: String,
}

/// The file `rel`, a path under the work directory's `review/`, under
/// the earlier review's directory `dir`.
fn kept_file(dir: &Path, rel: &str) -> Result<String, String> {
    let name = rel
        .strip_prefix(REVIEW)
        .map_or(rel, |r| r.trim_start_matches('/'));
    fs::read_to_string(dir.join(name))
        .map_err(|e| format!("the earlier review kept no {name}: {e}"))
}

/// Reads what the earlier review kept in `dir`.
///
/// # Errors
///
/// Fails naming the first file that is not there or does not read.
pub fn read(dir: &Path) -> Result<Kept, String> {
    Ok(Kept {
        run_id: kept_file(dir, run_id::RUN_ID)?,
        gaps: kept_file(dir, gaps::GAPS)?,
        dispatch: kept_file(dir, DISPATCH)?,
        outcome: kept_file(dir, OUTCOME)?,
        consulted: kept_file(dir, CONSULTED)?,
    })
}

/// Why `kept` does not stand for a review of `head` under `dispatch`,
/// the dispatch file's text; none when it does.
fn refusal(kept: &Kept, head: &str, dispatch: &str) -> Result<(), String> {
    let id = run_id::parse(&kept.run_id)?;
    match id.head.as_deref() {
        Some(was) if was == head => {}
        Some(was) => return Err(format!("the earlier review was of {was}, not {head}")),
        None => return Err("the earlier review recorded no head".to_owned()),
    }
    let json = |what: &str, text: &str| {
        serde_json::from_str::<serde_json::Value>(text)
            .map_err(|e| format!("{what} dispatch does not read: {e}"))
    };
    if json("the earlier review's", &kept.dispatch)? != json("this review's", dispatch)? {
        return Err("the dispatch changed since the earlier review".to_owned());
    }
    let gaps = gaps::parse(&kept.gaps)?;
    if !gaps.is_empty() {
        return Err(format!(
            "the earlier review left {} agent run(s) without an accepted report",
            gaps.len()
        ));
    }
    review_gate::parse_outcome(&kept.outcome)
        .map_err(|e| format!("the earlier review's outcome does not read: {e}"))?;
    review_gate::parse_consulted(&kept.consulted)
        .map_err(|e| format!("the earlier review's consulted agents do not read: {e}"))?;

    Ok(())
}

/// Weighs `kept` against a review of `head` under `dispatch`, the
/// dispatch file's text.
#[must_use]
pub fn weigh_kept(kept: &Kept, head: &str, dispatch: &str) -> Verdict {
    match refusal(kept, head, dispatch) {
        Ok(()) => Verdict::Reused,
        Err(why) => Verdict::Fresh(why),
    }
}

/// Weighs what the earlier review kept in `dir` against a review of
/// `head` under `dispatch`, the dispatch file's text.
#[must_use]
pub fn weigh(dir: &Path, head: &str, dispatch: &str) -> Verdict {
    match read(dir) {
        Ok(kept) => weigh_kept(&kept, head, dispatch),
        Err(why) => Verdict::Fresh(why),
    }
}

#[cfg(test)]
mod tests;
