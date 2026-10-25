//! Quarantine (SEC-12, RCL-06): a recorded, reversible exclusion of
//! events from every path that carries content to an agent, on the node
//! that records it, keeping what it covers (I5). A quarantine is a
//! principal act, an `operator` event in this node's device seat naming
//! its selector and the principal's stated reason; its release is
//! another. The quarantine set is a function of those acts alone (I10):
//! an event is quarantined while an added selector matches it that no
//! release has taken back, so an event recorded after the act is held
//! out too.
//!
//! A quarantine changes no origin, no trust level and no key set: it is
//! read beside them, never into them (PRV-13).
//!
//! The selectors M1 reaches are an address range, a run, an author
//! seat, a room, a writer, a provenance class, a flag and a time
//! window. No event carries a flag before PRV-07 ships, so a flag
//! selector matches none yet; a time window reads the transcript
//! timestamp an event keeps, to the second, and no event without one
//! lies in it.

use std::cell::RefCell;
use std::collections::BTreeMap;

use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, SeatId, Seq, WriterId};
use serde_json::{Value, json};
use store::Store;

use crate::AdminError;

/// The longest run name, flag or time a selector holds.
const LONGEST: usize = 200;

/// How many characters of a time a window compares: `YYYY-MM-DDTHH:MM:SS`.
const SECONDS: usize = 19;

/// What a quarantine selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// The events `first` to `last` of one writer.
    Range {
        writer: WriterId,
        first: Seq,
        last: Seq,
    },
    /// The writers of the run with this name.
    Run(String),
    /// The writers of this seat.
    Author(SeatId),
    /// The writers of the seats in this room.
    Room(RoomId),
    /// This writer's events.
    Writer(WriterId),
    /// Events of this provenance class.
    Provenance(Provenance),
    /// Events carrying this flag (PRV-07).
    Flag(String),
    /// Events whose transcript timestamp is this time or later.
    Since(String),
}

/// Text that names no selector value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalid(pub &'static str);

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Invalid {}

/// `text` when it is a plain name of at most [`LONGEST`] characters.
fn plain(text: &str, allowed: fn(char) -> bool, what: &'static str) -> Result<String, Invalid> {
    let ok = !text.is_empty() && text.chars().count() <= LONGEST && text.chars().all(allowed);

    ok.then(|| text.to_owned()).ok_or(Invalid(what))
}

impl Selector {
    /// The run selector for `name`, a harness session, and the harness's
    /// agent id after a slash for a subagent's run.
    ///
    /// # Errors
    ///
    /// Fails on a name that is not plain.
    pub fn run(name: &str) -> Result<Self, Invalid> {
        let allowed = |c: char| c.is_ascii_alphanumeric() || "-_./:".contains(c);

        plain(name, allowed, "a run name is letters, digits and - _ . / :").map(Self::Run)
    }

    /// The flag selector for `name`.
    ///
    /// # Errors
    ///
    /// Fails on a name that is not lowercase letters and underscores.
    pub fn flag(name: &str) -> Result<Self, Invalid> {
        let allowed = |c: char| c.is_ascii_lowercase() || c == '_';

        plain(name, allowed, "a flag is lowercase letters and underscores").map(Self::Flag)
    }

    /// The time-window selector from `time`, an RFC 3339 time.
    ///
    /// # Errors
    ///
    /// Fails on text that does not begin `YYYY-MM-DD`, or holds a
    /// character an RFC 3339 time does not.
    pub fn since(time: &str) -> Result<Self, Invalid> {
        let allowed = |c: char| c.is_ascii_digit() || "-:T.Z+".contains(c);
        let dated = time.len() >= 10
            && time.as_bytes()[..4].iter().all(u8::is_ascii_digit)
            && time.as_bytes()[4] == b'-';
        let time = plain(
            time,
            allowed,
            "a time is RFC 3339, such as 2026-01-01T00:00:00Z",
        )?;

        if dated {
            Ok(Self::Since(time))
        } else {
            Err(Invalid("a time is RFC 3339, such as 2026-01-01T00:00:00Z"))
        }
    }

    /// The provenance selector for `class`.
    ///
    /// # Errors
    ///
    /// Fails on a name that is no provenance class.
    pub fn provenance(class: &str) -> Result<Self, Invalid> {
        Provenance::parse(class)
            .map(Self::Provenance)
            .ok_or(Invalid("not a provenance class"))
    }

    /// The selector as the principal act records it.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Range {
                writer,
                first,
                last,
            } => json!({"by": "range", "writer": writer.to_string(),
                        "first": first.get(), "last": last.get()}),
            Self::Run(name) => json!({"by": "run", "run": name}),
            Self::Author(seat) => json!({"by": "author", "seat": seat.to_string()}),
            Self::Room(room) => json!({"by": "room", "room": room.to_string()}),
            Self::Writer(writer) => json!({"by": "writer", "writer": writer.to_string()}),
            Self::Provenance(class) => json!({"by": "provenance", "class": class.name()}),
            Self::Flag(flag) => json!({"by": "flag", "flag": flag}),
            Self::Since(time) => json!({"by": "since", "time": time}),
        }
    }

    /// The selector `value` records; none for one that is not a
    /// selector.
    #[must_use]
    pub fn from_value(value: &Value) -> Option<Self> {
        let text = |name: &str| value.get(name)?.as_str();
        let seq = |name: &str| Seq::new(value.get(name)?.as_u64()?);

        match text("by")? {
            "range" => Some(Self::Range {
                writer: WriterId::parse(text("writer")?).ok()?,
                first: seq("first")?,
                last: seq("last")?,
            }),
            "run" => Self::run(text("run")?).ok(),
            "author" => Some(Self::Author(SeatId::parse(text("seat")?).ok()?)),
            "room" => Some(Self::Room(RoomId::parse(text("room")?).ok()?)),
            "writer" => Some(Self::Writer(WriterId::parse(text("writer")?).ok()?)),
            "provenance" => Self::provenance(text("class")?).ok(),
            "flag" => Self::flag(text("flag")?).ok(),
            "since" => Self::since(text("time")?).ok(),
            _ => None,
        }
    }

    /// The selector in words, as `parinn quarantine list` shows it.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Range {
                writer,
                first,
                last,
            } => format!("range {writer}:{first}-{last}"),
            Self::Run(name) => format!("run {name}"),
            Self::Author(seat) => format!("author {seat}"),
            Self::Room(room) => format!("room {room}"),
            Self::Writer(writer) => format!("writer {writer}"),
            Self::Provenance(class) => format!("provenance {}", class.name()),
            Self::Flag(flag) => format!("flag {flag}"),
            Self::Since(time) => format!("since {time}"),
        }
    }
}

/// What a selector reads of the writer an event lies in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriterFacts {
    pub writer: WriterId,
    pub room: RoomId,
    /// The seat that wrote it.
    pub seat: SeatId,
    /// The run it records, when ingest recorded one.
    pub run: Option<String>,
}

/// What a selector reads of one event.
#[derive(Debug, Clone, Copy)]
pub struct EventFacts<'a> {
    pub seq: Seq,
    pub provenance: &'a Provenance,
    pub timestamp: Option<&'a str>,
    pub flags: &'a [String],
}

/// The first [`SECONDS`] characters of `time`.
fn seconds(time: &str) -> &str {
    let end = time
        .char_indices()
        .nth(SECONDS)
        .map_or(time.len(), |(i, _)| i);

    &time[..end]
}

impl Selector {
    /// Whether the selector holds the event of `event` in `writer`.
    #[must_use]
    pub fn holds(&self, writer: &WriterFacts, event: &EventFacts<'_>) -> bool {
        match self {
            Self::Range {
                writer: w,
                first,
                last,
            } => *w == writer.writer && *first <= event.seq && event.seq <= *last,
            Self::Run(name) => writer.run.as_deref() == Some(name.as_str()),
            Self::Author(seat) => *seat == writer.seat,
            Self::Room(room) => *room == writer.room,
            Self::Writer(w) => *w == writer.writer,
            Self::Provenance(class) => class == event.provenance,
            Self::Flag(flag) => event.flags.contains(flag),
            Self::Since(time) => event
                .timestamp
                .is_some_and(|stamp| seconds(stamp) >= seconds(time)),
        }
    }
}

/// A selector in force, with the act that added it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Active {
    pub selector: Selector,
    /// The seq of the act in the device seat's writer.
    pub act: Seq,
    pub reason: String,
}

/// The selectors in force (SEC-12).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuarantineSet {
    active: Vec<Active>,
}

/// The act names of a quarantine and its release.
pub const ADD: &str = "quarantine_add";
pub const RELEASE: &str = "quarantine_release";

impl QuarantineSet {
    /// The quarantine set `store` holds for the node whose device seat's
    /// writer is `device`: the selectors its acts added and none
    /// released, in the order they were added. An act that does not
    /// parse adds or releases nothing.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn load(store: &Store, device: WriterId) -> Result<Self, AdminError> {
        let acts = store.contents(device, EventKind::PrincipalAct)?;
        let mut active: Vec<Active> = Vec::new();
        for (seq, act) in acts {
            let Some(selector) = Selector::from_value(&act["selector"]) else {
                continue;
            };
            match act["act"].as_str() {
                Some(ADD) => active.push(Active {
                    selector,
                    act: seq,
                    reason: act["reason"].as_str().unwrap_or_default().to_owned(),
                }),
                Some(RELEASE) => active.retain(|a| a.selector != selector),
                _ => {}
            }
        }

        Ok(Self { active })
    }

    /// The selectors in force, oldest first.
    #[must_use]
    pub fn active(&self) -> &[Active] {
        &self.active
    }

    /// Whether nothing is quarantined.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    /// Whether the event of `event` in `writer` is quarantined.
    #[must_use]
    pub fn holds(&self, writer: &WriterFacts, event: &EventFacts<'_>) -> bool {
        self.active.iter().any(|a| a.selector.holds(writer, event))
    }
}

/// The quarantine set of a node read against the events of its store:
/// what recall asks of each address (RCL-06). Nothing is read from the
/// store for an empty set.
pub struct Quarantine<'s> {
    store: &'s Store,
    set: QuarantineSet,
    /// The run each ingested writer records.
    runs: BTreeMap<WriterId, String>,
    /// What each writer asked about so far lies in.
    writers: RefCell<BTreeMap<WriterId, Option<WriterFacts>>>,
}

impl<'s> Quarantine<'s> {
    /// The quarantine of the node whose device seat's writer is
    /// `device`.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn load(store: &'s Store, device: WriterId) -> Result<Self, AdminError> {
        let set = QuarantineSet::load(store, device)?;
        let runs = if set.is_empty() {
            BTreeMap::new()
        } else {
            store
                .ingested_runs()?
                .into_iter()
                .map(|r| (r.writer, r.run_name()))
                .collect()
        };

        Ok(Self {
            store,
            set,
            runs,
            writers: RefCell::new(BTreeMap::new()),
        })
    }

    /// The quarantine set read.
    #[must_use]
    pub fn set(&self) -> &QuarantineSet {
        &self.set
    }

    /// Whether the event at `address` is quarantined. An event not in
    /// the store is not.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn holds(&self, address: Address) -> Result<bool, AdminError> {
        if self.set.is_empty() {
            return Ok(false);
        }
        let Some(writer) = self.writer(address.writer)? else {
            return Ok(false);
        };
        let Some(facts) = self.store.event_facts(address)? else {
            return Ok(false);
        };
        let event = EventFacts {
            seq: address.seq,
            provenance: &facts.provenance,
            timestamp: facts.timestamp.as_deref(),
            flags: &[],
        };

        Ok(self.set.holds(&writer, &event))
    }

    /// What `writer` lies in, looked up once.
    fn writer(&self, writer: WriterId) -> Result<Option<WriterFacts>, AdminError> {
        if let Some(known) = self.writers.borrow().get(&writer) {
            return Ok(known.clone());
        }
        let facts = self.store.seat(writer)?.map(|seat| WriterFacts {
            writer,
            room: seat.room,
            seat: seat.seat,
            run: self.runs.get(&writer).cloned(),
        });
        self.writers.borrow_mut().insert(writer, facts.clone());

        Ok(facts)
    }
}

#[cfg(test)]
mod tests;
