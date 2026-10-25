//! `stat_list`, `landmark_list` and `pin_list` (§9.2, RCL-01). The
//! record holds no landmark and no pin before M3 and M2 record them,
//! so those two answer an envelope with no item. `stat_list` answers an
//! envelope marked structural: counts derived from the record within
//! the recall scope, no content.

use envelope::Envelope;
use serde_json::Value;

use crate::scope::{self, RecallScope};
use crate::{Served, Server};

const BAD_TIER: &str = "the tier is not a short word";
const UNREADABLE: &str = "the record cannot be read: run parinn verify";

/// `arguments` with the scope `room` when they name none and the server
/// serves no run: its personal room is all it can serve. The server
/// chose that scope, not the caller, so it is not noted as an extension.
fn unbound_default(mut arguments: Value, run: Option<&str>) -> (Value, bool) {
    let chosen = run.is_none() && arguments["scope"].is_null();
    if chosen {
        arguments["scope"] = serde_json::json!("room");
    }

    (arguments, chosen)
}

impl Server {
    /// `landmark_list`: the landmark blocks of the scope, in the
    /// envelope; none are recorded yet.
    pub(crate) fn landmark_list(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        match &arguments["tier"] {
            Value::Null => {}
            Value::String(tier)
                if tier.len() <= 16 && tier.bytes().all(|b| b.is_ascii_alphanumeric()) => {}
            _ => return Err(BAD_TIER.to_owned()),
        }
        let scope = self.scope_of(arguments, run, "landmark_list")?;
        self.note_extended("landmark_list", &scope);

        Ok(Served {
            envelope: Envelope::default(),
            scope: Some(scope.name),
        })
    }

    /// `pin_list`: the qualifying pins that restore to the calling
    /// agent, or the pins of the room named, in the envelope; none are
    /// recorded yet. It is served in the scope the call names, like the
    /// other recall tools.
    pub(crate) fn pin_list(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        // A room alone asks for that room's pins (§9.2): the scope `room`.
        let mut arguments = arguments.clone();
        if arguments["scope"].is_null() && !arguments["room"].is_null() {
            arguments["scope"] = serde_json::json!("room");
        }
        let (arguments, chosen) = unbound_default(arguments, run);
        let scope = self.scope_of(&arguments, run, "pin_list")?;
        if !chosen {
            self.note_extended("pin_list", &scope);
        }

        Ok(Served {
            envelope: Envelope::default(),
            scope: Some(scope.name),
        })
    }

    /// `stat_list`: the counts within the scope the call names, in an
    /// envelope marked structural. A call that names no scope is served
    /// in the calling run, or in the personal room while the server
    /// serves no run.
    pub(crate) fn stat_list(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        let (arguments, chosen) = unbound_default(arguments.clone(), run);
        let scope = self.scope_of(&arguments, run, "stat_list")?;
        if !chosen {
            self.note_extended("stat_list", &scope);
        }
        let counts = self
            .store
            .stats(&scope.reach())
            .map_err(|_| UNREADABLE.to_owned())?;
        let runs = match scope.name {
            RecallScope::Run => u64::from(run.is_some()),
            _ => self.count_runs(&scope)?,
        };

        Ok(Served {
            envelope: Envelope {
                structural: Some(vec![
                    ("events", counts.events),
                    ("runs", runs),
                    ("compactions", counts.compactions),
                    ("recalls", counts.recalls),
                ]),
                ..Envelope::default()
            },
            scope: Some(scope.name),
        })
    }

    /// How many runs ingest recorded in the writers `scope` reaches.
    fn count_runs(&self, scope: &scope::Scope) -> Result<u64, String> {
        let runs = self
            .store
            .ingested_runs()
            .map_err(|_| UNREADABLE.to_owned())?;
        let mut names = std::collections::BTreeSet::new();
        for run in runs {
            let seat = self
                .store
                .seat(run.writer)
                .map_err(|_| UNREADABLE.to_owned())?;
            if seat.is_some_and(|s| scope.covers(run.writer, s.room)) {
                names.insert(run.run_name());
            }
        }

        Ok(names.len() as u64)
    }
}
