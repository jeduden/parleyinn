//! What the hook handlers observed of the harness sessions an ingest
//! reads (REC-19, PRV-08, REC-04): held at each transcript's ingest
//! marker, read here from the store. A `SessionStart` hook observation
//! with `source` = `fork` is a fork observation; one that read the
//! process ancestry lets a typed line be `user`, unless it found the
//! harness session nested; a `UserPromptSubmit` hook observation says
//! whether its handler found the attended mark. Each one ingest appends
//! becomes a structural event of the device seat.

use std::collections::BTreeSet;

use provenance::Session;
use record::id::WriterId;
use serde_json::{Value, json};
use store::{Held, Store};

/// The hook observations the store holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observed {
    held: Vec<Held>,
}

impl Observed {
    /// Every hook observation `store` holds.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn load(store: &Store) -> Result<Self, store::StoreError> {
        Ok(Self::of(store.hook_observations()?))
    }

    /// The observations `held`.
    #[must_use]
    pub fn of(held: Vec<Held>) -> Self {
        Self { held }
    }

    fn starts(&self, session: &str) -> impl Iterator<Item = &Held> {
        self.held.iter().filter(move |h| {
            h.observation.hook == "SessionStart" && h.observation.harness_session == session
        })
    }

    /// The harness sessions a fork observation names.
    #[must_use]
    pub fn forks(&self) -> BTreeSet<String> {
        self.held
            .iter()
            .filter(|h| {
                h.observation.hook == "SessionStart"
                    && h.observation.source.as_deref() == Some("fork")
            })
            .map(|h| h.observation.harness_session.clone())
            .collect()
    }

    /// What the hook handlers recorded of `session`'s process ancestry.
    #[must_use]
    pub fn session(&self, session: &str) -> Session {
        let ancestries: Vec<(u64, bool)> = self
            .starts(session)
            .filter_map(|h| h.observation.ancestry)
            .collect();

        Session {
            ancestry_observed: !ancestries.is_empty(),
            nested: ancestries.iter().any(|(_, nested)| *nested),
        }
    }

    /// Whether a `UserPromptSubmit` hook handler of `session` found the
    /// attended mark when it got the prompt `prompt_id`.
    #[must_use]
    pub fn attended(&self, session: &str, prompt_id: &str) -> bool {
        self.held.iter().any(|h| {
            let o = &h.observation;
            o.hook == "UserPromptSubmit"
                && o.harness_session == session
                && o.prompt_id.as_deref() == Some(prompt_id)
                && o.attended
        })
    }

    /// The observations of `session` not yet in the record, in the order
    /// they were held.
    #[must_use]
    pub fn pending(&self, session: &str) -> Vec<&Held> {
        self.held
            .iter()
            .filter(|h| h.observation.harness_session == session && !h.appended)
            .collect()
    }
}

/// The content of the hook observation event for `held`, a hook
/// observation of the run whose writer is `run`: its structural fields
/// alone, never a path or a prompt's text.
#[must_use]
pub fn content(held: &Held, run: WriterId) -> Value {
    let o = &held.observation;
    let mut content = json!({
        "hook": o.hook,
        "harness_session": o.harness_session,
        "run": run.to_string(),
        "offset": o.offset,
    });
    let fields = [
        ("source", o.source.as_ref().map(|s| json!(s))),
        ("prompt_id", o.prompt_id.as_ref().map(|p| json!(p))),
        (
            "attended",
            (o.hook == "UserPromptSubmit").then_some(json!(o.attended)),
        ),
        ("harness_processes", o.ancestry.map(|(h, _)| json!(h))),
        ("nested", o.ancestry.map(|(_, n)| json!(n))),
    ];
    for (key, value) in fields {
        if let Some(value) = value {
            content[key] = value;
        }
    }

    content
}

#[cfg(test)]
mod tests;
