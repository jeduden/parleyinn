//! Retention policies at the node (REC-15 in part, SEC-22): while a
//! retention policy is set, `parinn ingest` records a time mark when one
//! is due and purges the ranges the policies have aged out, as the
//! node's own purge, each audited and counted naming its policy. It
//! never fails the ingest that ran it (I9): a failure is audited,
//! counted and tried again by the next.

use administration::node::Node;
use administration::retention::{self, Applied, Policy};
use audit::entry::{Counter, Kind, Occurrence};
use configuration::resolve::Settings;
use configuration::value::Value;
use key::random::Entropy;
use record::event::Provenance;
use record::id::RoomId;
use store::Store;

use crate::Context;
use crate::purge::audit_refusals;

/// The retention policies `settings` holds, by key. A key that does not
/// read as a room and a provenance class holds none.
pub(crate) fn policies(settings: &Settings) -> Vec<Policy> {
    settings
        .iter()
        .filter_map(|(key, value)| policy(key.name(), value))
        .collect()
}

/// The policy the key `name` sets to `value`, if it is one.
fn policy(name: &str, value: &Value) -> Option<Policy> {
    let parts: Vec<&str> = name.splitn(3, '.').collect();
    let ["retention_policy", room, class] = parts[..] else {
        return None;
    };
    let Value::Integer(days) = value else {
        return None;
    };
    let room = if room == "*" {
        None
    } else {
        Some(RoomId::parse(room).ok()?)
    };

    Some(Policy {
        key: name.to_owned(),
        room,
        class: Provenance::parse(class)?,
        days: u64::try_from(*days).ok()?,
    })
}

/// Applies the retention policies of `settings` in the home `store`
/// belongs to, auditing what came of it.
pub(crate) fn apply(
    ctx: &Context,
    store: &mut Store,
    node: &Node,
    settings: &Settings,
    entropy: &mut dyn Entropy,
) {
    let policies = policies(settings);
    let applied = retention::apply(store, node, &policies, (ctx.clock)(), entropy);
    applied.map_or_else(|e| failed(ctx, &e), |applied| audit(ctx, &applied));
}

/// Audits and counts a failure to apply the policies: the store, the
/// home or the random source failed; the next ingest tries again
/// (SEC-22, I6).
pub(crate) fn failed(ctx: &Context, cause: &dyn std::fmt::Display) {
    crate::logged("apply the retention policies", Err::<(), _>(cause));
    let occurrence = Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::RetentionFailed,
        operation: "retention",
        detail: "the retention policies could not be applied: the store or the random source failed",
    };
    crate::logged("audit a retention failure", ctx.audit(&occurrence));
}

/// Applies the policies in the store and node `opened` gave, or audits
/// and counts that it gave none.
pub(crate) fn apply_opened(
    ctx: &Context,
    opened: Result<(Store, Node), String>,
    settings: &Settings,
    entropy: &mut dyn Entropy,
) {
    match opened {
        Ok((mut store, node)) => apply(ctx, &mut store, &node, settings, entropy),
        Err(e) => failed(ctx, &e),
    }
}

/// Audits each range purged and each refused (SEC-22, ADM-07, I6).
fn audit(ctx: &Context, applied: &Applied) {
    for (range, done) in &applied.purged {
        let scope = format!("{}:{}-{}", range.writer, range.first, range.last);
        let occurrence = Occurrence::Purged {
            scope: &scope,
            reason: "retention",
            by: "retention policy",
            behind: &range.policy,
            counter: Some(Counter::RetentionPurged),
        };
        crate::logged("audit a retention purge", ctx.audit(&occurrence));
        if !done.finished.vacuumed {
            crate::purge::audit_residue(ctx);
        }
    }
    for (_, refusals) in &applied.refused {
        audit_refusals(ctx, refusals);
    }
}

#[cfg(test)]
mod tests;
