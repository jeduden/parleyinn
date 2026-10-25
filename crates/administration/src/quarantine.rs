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
//! The acts are read from the device seats of the node's personal room
//! alone: every device seat the node ever had lies there, those of
//! another node lie in rooms of their own, and an act of another node
//! is a request its principal applies, not a quarantine (I8, PEER-11).
//! Where a node identity change gave the node a second device seat, the
//! genesis seat's acts come first.
//!
//! The selectors M1 reaches are an address range, a run, an author
//! seat, a room, a writer, a provenance class, a flag and a time
//! window. No event carries a flag before PRV-07 ships, so a flag
//! selector matches none yet; a time window reads the transcript
//! timestamp an event keeps as the UTC instant it names (an offset is
//! applied), to the second, and no event without one, or with one that
//! is no RFC 3339 time, lies in it.

use std::cell::RefCell;
use std::collections::BTreeMap;

use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, SeatId, Seq, WriterId};
use serde_json::{Value, json};
use store::Store;

use crate::AdminError;
use crate::origin::is_device_seat;

/// The longest run name, flag or time a selector holds.
const LONGEST: usize = 200;

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
    /// Fails on text that is no RFC 3339 date or time: a date alone, or a
    /// time with `Z`, a `+HH:MM` or `-HH:MM` offset or none (read as UTC).
    pub fn since(time: &str) -> Result<Self, Invalid> {
        let allowed = |c: char| c.is_ascii_digit() || "-:T.Z+".contains(c);
        let dated = utc_seconds(time).is_some();
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

/// The digits of `text` as a number; `None` when any is no digit.
fn number(text: Option<&str>) -> Option<i64> {
    let text = text?;

    text.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| text.bytes().fold(0, |n, b| n * 10 + i64::from(b - b'0')))
}

/// The whole seconds since the Unix epoch of `time`, an RFC 3339 time
/// read as the UTC instant it names: its offset is applied, a fraction
/// of a second dropped, and a date alone or a time without an offset
/// read as UTC. `None` for text that is no such instant.
fn utc_seconds(time: &str) -> Option<i64> {
    let midnight = date_seconds(time)?;
    let Some(clock) = time.get(10..).filter(|rest| !rest.is_empty()) else {
        return Some(midnight);
    };
    let clock = clock.strip_prefix('T')?;
    let (hour, minute, second) = (
        number(clock.get(0..2))?,
        number(clock.get(3..5))?,
        number(clock.get(6..8))?,
    );
    let separated = clock.get(2..3)? == ":" && clock.get(5..6)? == ":";
    if !separated || hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    Some(midnight + hour * 3600 + minute * 60 + second - offset(clock.get(8..)?)?)
}

/// The seconds from the epoch to midnight UTC of the date `YYYY-MM-DD`
/// that begins `time`.
fn date_seconds(time: &str) -> Option<i64> {
    let (year, month, day) = (
        number(time.get(0..4))?,
        number(time.get(5..7))?,
        number(time.get(8..10))?,
    );
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let longest = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    let valid = time.get(4..5)? == "-"
        && time.get(7..8)? == "-"
        && (1..=12).contains(&month)
        && (1..=longest).contains(&day);

    valid.then(|| days_from_civil(year, month, day) * 86_400)
}

/// The seconds `zone`, the text after a clock, is ahead of UTC: an
/// optional fraction of a second, then nothing or `Z` (UTC) or
/// `+HH:MM` or `-HH:MM`.
fn offset(zone: &str) -> Option<i64> {
    let zone = match zone.strip_prefix('.') {
        Some(fraction) => {
            let end = fraction.bytes().take_while(u8::is_ascii_digit).count();
            if end == 0 {
                return None;
            }
            &fraction[end..]
        }
        None => zone,
    };
    if zone.is_empty() || zone == "Z" {
        return Some(0);
    }
    let sign = match zone.get(..1)? {
        "+" => 1,
        "-" => -1,
        _ => return None,
    };
    let (hours, minutes) = (number(zone.get(1..3))?, number(zone.get(4..6))?);
    let shaped = zone.len() == 6 && zone.get(3..4)? == ":" && hours <= 23 && minutes <= 59;

    shaped.then_some(sign * (hours * 3600 + minutes * 60))
}

/// The days from 1970-01-01 to the proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let of_day = of_era * 365 + of_era / 4 - of_era / 100 + of_year;

    era * 146_097 + of_day - 719_468
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
            Self::Since(time) => {
                let (Some(stamp), Some(from)) =
                    (event.timestamp.and_then(utc_seconds), utc_seconds(time))
                else {
                    return false;
                };

                stamp >= from
            }
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

/// The writers of the device seats in `room`, the node's personal room,
/// the genesis seat's first (see the module's notes).
///
/// # Errors
///
/// Fails when the store does.
pub fn device_writers(store: &Store, room: RoomId) -> Result<Vec<WriterId>, AdminError> {
    let mut found = Vec::new();
    for seat in store.seats()? {
        if seat.room != room || !is_device_seat(&seat) {
            continue;
        }
        let first = store.event(Address::new(seat.writer, Seq::FIRST))?;
        let changed = first.is_some_and(|f| f.link.header.kind == EventKind::NodeIdentityChange);
        found.push((changed, seat.writer));
    }
    found.sort();

    Ok(found.into_iter().map(|(_, writer)| writer).collect())
}

impl QuarantineSet {
    /// The quarantine set `store` holds for the node whose device seats'
    /// writers are `devices`: the selectors their acts added and none
    /// released, in the order they were added. An act that does not
    /// parse adds or releases nothing.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn load(store: &Store, devices: &[WriterId]) -> Result<Self, AdminError> {
        let mut active: Vec<Active> = Vec::new();
        for device in devices {
            for (seq, act) in store.contents(*device, EventKind::PrincipalAct)? {
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

    /// Whether a range or writer selector names `address`, which needs
    /// nothing of the event or its writer's seat.
    #[must_use]
    pub fn names(&self, address: Address) -> bool {
        self.active.iter().any(|a| match &a.selector {
            Selector::Range {
                writer,
                first,
                last,
            } => *writer == address.writer && *first <= address.seq && address.seq <= *last,
            Selector::Writer(writer) => *writer == address.writer,
            _ => false,
        })
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
    /// The quarantine of the node whose personal room is `room`.
    ///
    /// # Errors
    ///
    /// Fails when the store does.
    pub fn load(store: &'s Store, room: RoomId) -> Result<Self, AdminError> {
        let set = QuarantineSet::load(store, &device_writers(store, room)?)?;
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
        // A range or writer selector holds an event whatever its writer's
        // seat is, or whether it has one (I5: never fail open).
        if self.set.names(address) {
            return Ok(true);
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
