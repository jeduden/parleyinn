//! `parinn purge --range RANGE [--reason TEXT] [--json]` (ADM-07, SEC-14,
//! REC-17): erases the content of an address range and records a
//! tombstone for it, a widening principal act at a terminal. The range
//! may be written in any form ParleyInn shows or takes (RCL-08).
//!
//! The command refuses, audits and counts a range it may not purge, and
//! names each cause, never content (I6): an event the key set or room
//! state derives from, a run with a pending ingest marker, or nothing
//! left to erase. M1 purges an address range alone: ADM-07's other
//! scopes come with M3.

use std::io::Write;

use administration::act::Act;
use administration::node::Node;
use administration::purge::{self, Planned, Reason, Refusal};
use audit::entry::{Counter, Kind, Occurrence};
use key::random::OsEntropy;
use record::id::{Seq, WriterId};
use record::range::{Range, Writer};
use serde_json::json;
use store::Store;

use crate::act::{Taking, at_terminal};
use crate::{Context, EXIT_FAILURE, EXIT_USAGE, failed, written};

const OP: &str = "purge";

/// What a command line asked for.
#[derive(Debug, PartialEq, Eq)]
struct Asked {
    range: String,
    reason: Option<String>,
    as_json: bool,
}

/// Reads the command line after `purge`.
fn parse(mut args: impl Iterator<Item = String>) -> Result<Asked, String> {
    let (mut range, mut reason, mut as_json) = (None, None, false);
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| args.next().ok_or(format!("{flag} wants a value"));
        match arg.as_str() {
            "--json" => as_json = true,
            "--range" => range = Some(value("--range")?),
            "--reason" => reason = Some(value("--reason")?),
            flag if flag.starts_with("--") => {
                return Err(format!(
                    "unknown flag {flag:?}; M1 purges an address range alone: --range"
                ));
            }
            other => return Err(format!("unexpected {other:?}")),
        }
    }
    let range = range.ok_or("want --range, such as A1:30-40")?;

    Ok(Asked {
        range,
        reason,
        as_json,
    })
}

/// The writer and seqs `text` names, in any form ParleyInn takes. A
/// writer label no one assigned yet is assigned (RCL-08).
///
/// # Errors
///
/// Fails, saying why, on text that is no range, a label that names no
/// writer, or a full address whose room is not the writer's.
pub(crate) fn resolve(
    store: &mut Store,
    node: &Node,
    text: &str,
) -> Result<(WriterId, Seq, Seq), String> {
    let range = Range::parse(text).map_err(|e| e.to_string())?;
    let writer = match range.writer {
        Writer::Id(writer) => writer,
        Writer::Label(label) => {
            let find = |store: &Store| -> Result<Option<WriterId>, String> {
                let labels = store
                    .writer_labels(node.device_writer())
                    .map_err(|e| e.to_string())?;

                Ok(labels
                    .into_iter()
                    .find(|(_, l)| *l == label)
                    .map(|(w, _)| w))
            };
            match find(store)? {
                Some(writer) => writer,
                None => {
                    administration::label::assign_missing(
                        store,
                        node.device_writer(),
                        &mut OsEntropy,
                    )
                    .map_err(|e| e.to_string())?;
                    find(store)?.ok_or("no writer has that label")?
                }
            }
        }
    };
    if let Some(room) = range.room {
        let seat = store.seat(writer).map_err(|e| e.to_string())?;
        if seat.is_none_or(|s| s.room != room) {
            return Err("the writer has no seat in the room the address names".to_owned());
        }
    }

    Ok((writer, range.from, range.to))
}

/// Why a purge did not finish as asked.
enum Failure {
    /// A check already reported it, with this exit code.
    Code(u8),
    /// Something failed; the message says what.
    Plain(String),
    /// The range may not be purged (ADM-07).
    Refused(Vec<Refusal>),
    /// The transaction failed, or its erasure stands but a step after it
    /// failed: whether residue is journalled, and why.
    Unfinished { pending: bool, cause: String },
    /// The erasure stands, but the logs were held open and the vacuum
    /// waits.
    Residue,
}

/// `parinn purge`.
pub(crate) fn purge(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let asked = match parse(args) {
        Ok(asked) => asked,
        Err(e) => {
            let _ = writeln!(stderr, "parinn purge: {e}");
            return EXIT_USAGE;
        }
    };
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    match carry(ctx, &asked, stderr) {
        Ok(line) => written(writeln!(stdout, "{line}"), stderr),
        Err(Failure::Code(code)) => code,
        Err(Failure::Plain(why)) => failed(OP, &why, stderr),
        Err(Failure::Refused(refusals)) => refused(ctx, &refusals, stderr),
        Err(Failure::Unfinished { pending, cause }) => unfinished(ctx, pending, &cause, stderr),
        Err(Failure::Residue) => residue(ctx, stderr),
    }
}

/// A failure of the work, whatever failed, by its message.
fn plain(e: impl std::fmt::Display) -> Failure {
    Failure::Plain(e.to_string())
}

/// Takes the purge `asked` for and says what it did.
fn carry(ctx: &Context, asked: &Asked, stderr: &mut dyn Write) -> Result<String, Failure> {
    let mut taking = Taking::open(ctx).map_err(plain)?;
    let reason = asked.reason.clone();
    let first_look = Act::Purge {
        ranges: Vec::new(),
        reason,
    };
    taking
        .permit(ctx, OP, &first_look, stderr)
        .map_err(Failure::Code)?;
    let (store, node) = (&mut taking.store, &taking.node);
    let (writer, first, last) = resolve(store, node, &asked.range).map_err(plain)?;
    let planned = purge::plan(store, writer, first, last, &Reason::Purged).map_err(plain)?;
    let plan = match planned {
        Planned::Ready(plan) => plan,
        Planned::Refused(refusals) => return Err(Failure::Refused(refusals)),
    };
    let scope = format!("{writer}:{}-{}", plan.erase.first, plan.erase.last);
    let act = Act::Purge {
        ranges: vec![scope.clone()],
        reason: asked.reason.clone(),
    };
    let event = administration::act::event(&act, &mut OsEntropy).map_err(plain)?;
    let done = purge::carry_out(store, node, &plan, Some(event), &mut OsEntropy);
    let done = done.map_err(|e| stands(store, &e))?;
    let act_at = done.act.map(|a| a.to_string()).unwrap_or_default();
    let said = Occurrence::Purged {
        scope: &scope,
        reason: "purged",
        by: "principal act",
        behind: &act_at,
        counter: None,
    };
    ctx.audit(&said).map_err(unnoted)?;
    if !done.finished.vacuumed {
        return Err(Failure::Residue);
    }

    Ok(if asked.as_json {
        json!({"purged": done.erased, "range": scope, "act": act_at,
               "tombstone": done.tombstone.to_string()})
        .to_string()
    } else {
        format!(
            "purged {} events of {scope}; tombstone {}",
            done.erased, done.tombstone
        )
    })
}

/// The failure to note a recorded purge in the audit log.
fn unnoted(e: impl std::fmt::Display) -> Failure {
    plain(format!(
        "the purge is recorded, but the audit log took no note of it: {e}"
    ))
}

/// A purge that failed after its transaction left residue journalled, or
/// failed before it.
fn stands(store: &Store, cause: &dyn std::fmt::Display) -> Failure {
    Failure::Unfinished {
        pending: store.purge_pending().unwrap_or(false),
        cause: cause.to_string(),
    }
}

/// Audits and counts each cause that refused the range, and names it.
fn refused(ctx: &Context, refusals: &[Refusal], stderr: &mut dyn Write) -> u8 {
    audit_refusals(ctx, refusals);
    let _ = writeln!(
        stderr,
        "parinn purge: refused:\n{}",
        describe(refusals, SHOWN)
    );

    EXIT_FAILURE
}

/// Audits and counts each cause that refused a range: one entry for each
/// kind of event kept, not one for each event (ADM-07, I6).
pub(crate) fn audit_refusals(ctx: &Context, refusals: &[Refusal]) {
    let mut kept: Vec<&str> = Vec::new();
    for refusal in refusals {
        let key = match refusal {
            Refusal::NothingHeld => "range".to_owned(),
            Refusal::Kept { kind, .. } => kind.as_str().to_owned(),
            Refusal::PendingIngest { writer } => writer.to_string(),
        };
        if let Refusal::Kept { kind, .. } = refusal {
            if kept.contains(&kind.as_str()) {
                continue;
            }
            kept.push(kind.as_str());
        }
        let occurrence = Occurrence::Keyed {
            kind: Kind::Rejected,
            counter: Counter::PurgeRefused,
            operation: OP,
            detail: refusal.detail(),
            key: &key,
        };
        crate::logged("audit a purge's refusal", ctx.audit(&occurrence));
    }
}

/// How many causes a refusal names on the terminal; every one is audited.
const SHOWN: usize = 20;

/// The causes of a refusal, one to a line, the first `limit` of them and
/// how many more there are.
fn describe(refusals: &[Refusal], limit: usize) -> String {
    let mut lines: Vec<String> = refusals
        .iter()
        .take(limit)
        .map(|r| format!("  {r}"))
        .collect();
    if refusals.len() > limit {
        lines.push(format!("  and {} more", refusals.len() - limit));
    }

    lines.join("\n")
}

/// Finishes a purge a crash or a busy log cut short, when `store`
/// holds one (ADM-07, SEC-14): the next ingest, rebuild or verify does
/// it. A
/// step that still cannot finish is audited and counted (I6), and
/// never stops the command that found it (I9).
pub(crate) fn finish(ctx: &Context, store: &mut Store) {
    let done = store.finish_purge();
    crate::logged("finish a purge", done.as_ref());
    if left_over(&done) {
        audit_residue(ctx);
    }
}

/// Whether finishing a purge left residue: a step failed, or the logs
/// could not be checkpointed because another connection held them.
fn left_over(done: &Result<store::Finished, store::StoreError>) -> bool {
    match done {
        Ok(done) => !(done.nothing || done.vacuumed),
        Err(_) => true,
    }
}

/// Audits and counts the residue a purge left.
pub(crate) fn audit_residue(ctx: &Context) {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::PurgeUnfinished,
        operation: OP,
        detail: "a purge erased its range but could not finish clearing the residue",
    };
    crate::logged("audit a purge's residue", ctx.audit(&occurrence));
}

/// Reports a purge whose transaction failed, or whose erasure stands but
/// whose finishing steps could not complete.
fn unfinished(ctx: &Context, pending: bool, cause: &str, stderr: &mut dyn Write) -> u8 {
    if pending {
        let _ = residue(ctx, stderr);
    }

    failed(OP, &cause, stderr)
}

/// Audits and counts the residue a purge left for its next attempt, and
/// says so: the erasure stands, the checkpoint and vacuum do not yet.
fn residue(ctx: &Context, stderr: &mut dyn Write) -> u8 {
    audit_residue(ctx);
    let _ = writeln!(
        stderr,
        "parinn purge: the range is erased, but the store could not yet be vacuumed; run parinn rebuild or parinn ingest to finish"
    );

    EXIT_FAILURE
}

#[cfg(test)]
mod tests;
