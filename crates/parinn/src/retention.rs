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
        .filter_map(|(key, value)| {
            let name = key.name();
            let mut parts = name.splitn(3, '.');
            let (Some("retention_policy"), Some(room), Some(class)) =
                (parts.next(), parts.next(), parts.next())
            else {
                return None;
            };
            let Value::Integer(days) = value else {
                return None;
            };

            Some(Policy {
                key: name.to_owned(),
                room: match room {
                    "*" => None,
                    room => Some(RoomId::parse(room).ok()?),
                },
                class: Provenance::parse(class)?,
                days: u64::try_from(*days).ok()?,
            })
        })
        .collect()
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
    match applied {
        Ok(applied) => audit(ctx, &applied),
        Err(e) => {
            tracing::warn!(
                error = e.to_string().as_str(),
                "apply the retention policies"
            );
            let occurrence = Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::RetentionFailed,
                operation: "retention",
                detail: "the retention policies could not be applied: the store or the random source failed",
            };
            if let Err(e) = ctx.audit(&occurrence) {
                tracing::warn!(error = e.to_string().as_str(), "audit a retention failure");
            }
        }
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
        if let Err(e) = ctx.audit(&occurrence) {
            tracing::warn!(error = e.to_string().as_str(), "audit a retention purge");
        }
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
