//! What `parinn mcp` leaves of a recall beyond the envelope (RCL-03,
//! RCL-04, RCL-05, RCL-07, SEC-08, I6): the writer that appends each
//! recall event with its arguments redacted, and the audit entries for a
//! call cut at its cap, one whose scope extends past its run, one
//! refused for its scope or a gap, and one that went unrecorded.

use std::io::Write;

use audit::entry::{Counter, Kind, Occurrence};
use redaction::Redactor;

use crate::{Context, settings};

/// The redactor the MCP server writes by: none while a settings layer
/// fails, as nothing may be written without the redaction patterns it
/// names (SEC-08); the refusal is audited.
pub(crate) fn redactor(
    ctx: &Context,
    in_force: &settings::InForce,
    stderr: &mut dyn Write,
) -> Option<Redactor> {
    if in_force.loaded.principal_layer_failed() {
        crate::refuse_ingest(ctx, stderr);
        return None;
    }

    crate::redactor(ctx, &in_force.resolved.settings, stderr).ok()
}

/// What records the MCP server's recalls (RCL-07): in the device seat's
/// writer, their arguments redacted by `redactor`; where none is known,
/// no recall is recorded.
pub(crate) fn recall_writer(
    device_writer: record::id::WriterId,
    redactor: Option<Redactor>,
) -> mcp::RecallWriter {
    mcp::RecallWriter {
        writer: device_writer,
        redact: redactor.map(|redactor| -> mcp::recording::Redact {
            Box::new(move |text| {
                let redacted = redactor.redact_text(text);
                (redacted.text, redacted.rules)
            })
        }),
    }
}

/// Audits what the recall tools came to (RCL-03, RCL-05, RCL-07, SEC-08,
/// I6): a call cut at its cap, one whose scope extends past its run, one
/// whose recall event went unwritten, and each secret redacted from the
/// arguments it recorded.
pub(crate) fn audit_notes(ctx: &Context, notes: &[mcp::Note]) {
    for note in notes {
        let occurrence = match note {
            mcp::Note::Truncated(tool) => Occurrence::Degraded {
                kind: Kind::Truncated,
                counter: Counter::RecallTruncated,
                operation: tool,
                detail: "a recall tool stopped at its per-call cap and returned a cursor",
            },
            mcp::Note::Extended { tool, scope } => {
                tracing::info!(tool = *tool, scope = *scope, "recall extended");
                Occurrence::RecallExtended {
                    operation: tool,
                    detail: if *scope == "rooms" {
                        "recall extended past the current run to every room the run has a seat in"
                    } else {
                        "recall extended past the current run to a room the run has a seat in"
                    },
                }
            }
            mcp::Note::NotRecorded(tool) => Occurrence::Degraded {
                kind: Kind::Dropped,
                counter: Counter::RecallNotRecorded,
                operation: tool,
                detail: "a recall was served but its recall event could not be written",
            },
            mcp::Note::ScopeRefused(tool) => Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::RecallRefused,
                operation: tool,
                detail: "a recall call was refused: its scope is not one this server serves",
            },
            mcp::Note::RecordGap(tool) => Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::RecallRefused,
                operation: tool,
                detail: "a recall call was refused: the record has a gap inside the range asked for",
            },
            mcp::Note::LabelsFailed { tool, cause } => {
                tracing::warn!(tool = *tool, cause = cause.as_str(), "writer labels failed");
                Occurrence::Degraded {
                    kind: Kind::Failed,
                    counter: Counter::RecallRefused,
                    operation: tool,
                    detail: "a recall call was refused: the writer labels could not be assigned",
                }
            }
            mcp::Note::Redacted(rule) => Occurrence::Keyed {
                kind: Kind::Redacted,
                counter: Counter::Redacted,
                operation: "mcp",
                detail: "redacted from a recall's arguments by rule",
                key: rule,
            },
        };
        let _ = ctx.audit(&occurrence);
    }
}

#[cfg(test)]
mod tests;
