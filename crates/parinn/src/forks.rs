//! Two different events at one (writer, seq) (PEER-10, REC-21): the
//! node keeps the first head it saw, refuses the later one, and says
//! so. [`refuse`] is the one entry a history brought in from elsewhere
//! goes through: M3's backup restore (ADM-06), M9's room bundle import
//! (REC-23) and M8's peer sync call it, so the refusal is the same
//! wherever the second history comes from. It stores nothing of the
//! refused history but the writer, the seq range and the two digests
//! (I1), counts and audits each new refusal as `fork_conflict` (I6), and
//! marks no writer `equivocated`: until seals ship (REC-18, M7) nothing
//! authenticates a copy. `parinn verify` reads the refusals back and
//! reports them as integrity `refused` (VIEW-10).

use administration::home::Home;
use audit::entry::{Counter, Kind, Occurrence};
use record::chain::Link;
use record::fork::Fork;
use serde_json::{Value, json};
use store::{Store, StoreError};

/// What one [`refuse`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// The forks that were new: each is in the audit log, unless the log
    /// refused its entry.
    pub forks: Vec<Fork>,
    /// How many of those entries the audit log refused. The fork stays
    /// recorded and `parinn verify` reports it; the caller says on
    /// stderr that its entry is missing (I6).
    pub unaudited: usize,
}

/// The entry for `fork`, keyed by its writer and seq range.
fn occurrence<'a>(key: &'a str, operation: &'static str) -> Occurrence<'a> {
    Occurrence::Keyed {
        kind: Kind::Rejected,
        counter: Counter::ForkConflict,
        operation,
        detail: "refused a second event at one seq of writer",
        key,
    }
}

/// Refuses what `offered`, a history brought in by `operation`, holds
/// against the events `home`'s store holds at the same addresses: the
/// first head seen wins, nothing held is rewritten, and each new fork
/// is audited through `audit`, which says whether the audit log took
/// the entry.
///
/// # Errors
///
/// Fails when the store cannot be opened or read, or the refusal
/// cannot be recorded.
pub fn refuse(
    home: &Home,
    offered: &[Link],
    operation: &'static str,
    audit: &mut dyn FnMut(&Occurrence<'_>) -> bool,
) -> Result<Refused, StoreError> {
    let forks = Store::open(&home.store())?.refuse_forks(offered)?;
    let unaudited = forks
        .iter()
        .filter(|f| {
            let key = format!("{}:{}-{}", f.writer, f.first, f.last);
            !audit(&occurrence(&key, operation))
        })
        .count();

    Ok(Refused { forks, unaudited })
}

/// The refusals as `parinn verify --json` shows them: writer, seq range
/// and the two digests.
pub(crate) fn json(forks: &[Fork]) -> Value {
    forks
        .iter()
        .map(|f| {
            json!({
                "writer": f.writer.to_string(),
                "first": f.first.get(),
                "last": f.last.get(),
                "held": f.held.to_hex(),
                "offered": f.offered.to_hex(),
            })
        })
        .collect()
}

/// One line per refusal, for `parinn verify` and `parinn status`.
pub(crate) fn lines(forks: &[Fork]) -> Vec<String> {
    forks.iter().map(|f| format!("refused: {f}")).collect()
}

#[cfg(test)]
pub(crate) mod tests;
