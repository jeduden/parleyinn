//! What a recall leaves in the record (RCL-07, RCL-08) and what the
//! caller logs of it (I6).
//!
//! Every call that returns an envelope appends a recall event to the
//! device seat's writer: provenance `assistant`, the tool, the call's
//! arguments as redacted, the run and scope it was served in and the
//! addresses it returned. The event carries no `text`, so the search
//! index holds nothing of it and a search never echoes an earlier
//! query. A call whose event cannot be appended is served all the
//! same (I9) and noted, so the caller counts and audits it.
//!
//! A writer label is assigned by a structural event in the same
//! writer, the first time a call names one that has none
//! ([`administration::label`]).

use std::collections::BTreeMap;

use key::random::{OsEntropy, draw};
use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::WriterId;
use record::range::Label;
use serde_json::{Value, json};
use store::NewEvent;

use crate::{Served, Server};

/// Removes secrets from a text (SEC-08).
pub type Redact = Box<dyn Fn(&str) -> String>;

/// What records the recalls of a server.
pub struct Recorder {
    /// The device seat's writer, which the recall events go to.
    pub writer: WriterId,
    /// Removes secrets from a text before it is written (SEC-08); none
    /// while the redaction patterns are unknown, when nothing is
    /// written.
    pub redact: Option<Redact>,
}

/// What a call came to that the caller logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    /// The tool stopped at its per-call cap and returned a cursor
    /// (RCL-03).
    Truncated(&'static str),
    /// The call's recall scope extends past the calling run (RCL-05).
    Extended {
        tool: &'static str,
        scope: &'static str,
    },
    /// The call's recall event could not be appended (RCL-07).
    NotRecorded(&'static str),
}

/// `value` with every text in it redacted by `redact`.
fn redacted(value: &Value, redact: &dyn Fn(&str) -> String) -> Value {
    match value {
        Value::String(text) => Value::String(redact(text)),
        Value::Array(items) => items.iter().map(|v| redacted(v, redact)).collect(),
        Value::Object(fields) => fields
            .iter()
            .map(|(name, v)| (redact(name), redacted(v, redact)))
            .collect(),
        other => other.clone(),
    }
}

impl Server {
    /// Appends the recall event of `tool` called with `arguments` under
    /// `run`, which returned `served`; notes it when it cannot.
    pub(crate) fn record(
        &mut self,
        tool: &'static str,
        arguments: &Value,
        run: Option<&str>,
        served: &Served,
    ) {
        let Some(recorder) = &self.recorder else {
            return;
        };
        let written = recorder.redact.as_deref().and_then(|redact| {
            let content = json!({
                "tool": tool,
                "arguments": redacted(arguments, redact),
                "run": run.map(redact),
                "scope": served.scope.map(crate::scope::Name::as_str),
                "returned": served.envelope.items.iter().map(|i| i.address.as_str()).collect::<Vec<_>>(),
                "truncated": served.envelope.truncated,
            });
            let key = draw(&mut OsEntropy).ok()?;
            let event = NewEvent {
                kind: EventKind::Recall,
                provenance: Provenance::Assistant,
                content,
                commitment_key: CommitmentKey::from_bytes(key),
                transcript_timestamp: None,
            };

            Some((recorder.writer, event))
        });
        let appended =
            written.is_some_and(|(writer, event)| self.store.append(writer, &[event]).is_ok());
        if !appended {
            self.notes.borrow_mut().push(Note::NotRecorded(tool));
        }
    }

    /// The writer labels the device seat assigned (RCL-08).
    pub(crate) fn labels(&self) -> Result<BTreeMap<WriterId, Label>, String> {
        // A server that records assigns no label, so none is known.
        let Some(device) = self.recorder.as_ref().map(|r| r.writer) else {
            return Ok(BTreeMap::new());
        };

        self.store
            .writer_labels(device)
            .map_err(|_| crate::NO_INDEX.to_owned())
    }

    /// Assigns a label to each writer that has none, when this server
    /// records; a failure leaves the label unresolved, which the call
    /// answers.
    pub(crate) fn assign_labels(&mut self) {
        if let Some(recorder) = &self.recorder {
            let _ = administration::label::assign_missing(
                &mut self.store,
                recorder.writer,
                &mut OsEntropy,
            );
        }
    }
}

#[cfg(test)]
mod tests;
