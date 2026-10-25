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

/// Removes secrets from a text (SEC-08): the text with its secrets
/// replaced, and the rule of each replacement.
pub type Redact = Box<dyn Fn(&str) -> (String, Vec<String>)>;

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// A secret was redacted from the call's arguments before they were
    /// recorded, by the rule named (SEC-08).
    Redacted(String),
}

/// The recall event of `content`, under a key of its own.
fn recall_event(content: Value) -> Option<NewEvent> {
    let key = draw(&mut OsEntropy).ok()?;

    Some(NewEvent {
        kind: EventKind::Recall,
        provenance: Provenance::Assistant,
        content,
        commitment_key: CommitmentKey::from_bytes(key),
        transcript_timestamp: None,
    })
}

/// `text` redacted by `redact`, the rule of each redaction added to
/// `rules`.
fn scrub(
    text: &str,
    redact: &dyn Fn(&str) -> (String, Vec<String>),
    rules: &mut Vec<String>,
) -> String {
    let (out, found) = redact(text);
    rules.extend(found);

    out
}

/// `value` with every text in it redacted by `redact`, the rule of each
/// redaction added to `rules`.
fn redacted(
    value: &Value,
    redact: &dyn Fn(&str) -> (String, Vec<String>),
    rules: &mut Vec<String>,
) -> Value {
    match value {
        Value::String(text) => Value::String(scrub(text, redact, rules)),
        Value::Array(items) => items.iter().map(|v| redacted(v, redact, rules)).collect(),
        Value::Object(fields) => fields
            .iter()
            .map(|(name, v)| (scrub(name, redact, rules), redacted(v, redact, rules)))
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
        let writer = recorder.writer;
        let mut rules = Vec::new();
        let event = recorder.redact.as_deref().and_then(|redact| {
            let content = json!({
                "tool": tool,
                "arguments": redacted(arguments, redact, &mut rules),
                "run": run,
                "scope": served.scope.map(crate::scope::Name::as_str),
                "returned": served.envelope.items.iter().map(|i| i.address.as_str()).collect::<Vec<_>>(),
                "truncated": served.envelope.truncated,
            });

            recall_event(content)
        });
        let appended = event.is_some_and(|event| self.store.append(writer, &[event]).is_ok());
        let mut notes = self.notes.borrow_mut();
        notes.extend(rules.into_iter().map(Note::Redacted));
        if !appended {
            notes.push(Note::NotRecorded(tool));
        }
    }

    /// The writer labels the device seat assigned (RCL-08).
    pub(crate) fn labels(&self) -> Result<BTreeMap<WriterId, Label>, String> {
        // A server that assigns no label knows none.
        let Some(device) = self.labeller else {
            return Ok(BTreeMap::new());
        };

        self.store
            .writer_labels(device)
            .map_err(|_| crate::NO_INDEX.to_owned())
    }

    /// Assigns a label to each writer that has none, when this server
    /// may; a failure leaves the label unresolved, which the call
    /// answers.
    pub(crate) fn assign_labels(&mut self) {
        if let Some(device) = self.labeller {
            let _ = administration::label::assign_missing(&mut self.store, device, &mut OsEntropy);
        }
    }
}

#[cfg(test)]
mod tests;
