//! The recall scope of a call (RCL-05, I8). A call is served in the
//! scope `run` unless it names a wider one: the calling run, through
//! every writer of its seats whatever room each lies in; `room`, one of
//! its principal's rooms the run has a seat in; or `rooms`, every such
//! room. Only an explicit `scope` widens it, and each widened call is
//! noted for the caller to log. No foreign room is reached by any of
//! them (RCL-10 names it).
//!
//! A run is found through the tool-use id of its call (PRV-14); a
//! server that holds no session binding serves no run, so it takes only
//! `room`, in the personal room unless the call names another.

use std::collections::BTreeSet;

use record::id::{RoomId, WriterId};
use serde_json::Value;
use store::{Reach, Store};

/// The refusals, each fixed text (RCL-04).
pub const BAD_SCOPE: &str = "the scope is none of run, room or rooms";
pub const BAD_ROOM: &str = "the room is not a room id";
pub const ROOM_CONFLICT: &str = "the address and the room argument name different rooms";
const NO_SEAT_IN_ROOM: &str = "the run has no seat in that room";
const ROOMS_NEED_A_RUN: &str = "scope \"rooms\" needs a run, and this server serves none yet";

/// How far a call reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Name {
    Run,
    Room,
    Rooms,
}

impl Name {
    /// The name a call spells it by.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Room => "room",
            Self::Rooms => "rooms",
        }
    }

    /// The scope `arguments` name: `run` when none.
    fn of(arguments: &Value) -> Result<Self, &'static str> {
        match &arguments["scope"] {
            Value::Null => Ok(Self::Run),
            Value::String(name) => match name.as_str() {
                "run" => Ok(Self::Run),
                "room" => Ok(Self::Room),
                "rooms" => Ok(Self::Rooms),
                _ => Err(BAD_SCOPE),
            },
            _ => Err(BAD_SCOPE),
        }
    }
}

/// What a call may reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub name: Name,
    /// The calling run's writers, none while the server serves no run.
    current: BTreeSet<WriterId>,
    /// The rooms reached, for `room` and `rooms`.
    rooms: BTreeSet<RoomId>,
}

/// Where a call is served from.
pub struct Call<'a> {
    pub store: &'a Store,
    pub personal_room: RoomId,
    /// The calling run's name, once the server serves runs.
    pub run: Option<&'a str>,
}

impl Call<'_> {
    /// The writers of the run `name` ingest recorded (REC-02).
    fn run_writers(&self, name: &str) -> Result<BTreeSet<WriterId>, store::StoreError> {
        Ok(self
            .store
            .ingested_runs()?
            .into_iter()
            .filter(|r| r.run_name() == name)
            .map(|r| r.writer)
            .collect())
    }

    /// The rooms the run has a seat in: its personal room from its first
    /// event, and the room of each seat of its writers (RCL-05).
    fn run_rooms(
        &self,
        writers: &BTreeSet<WriterId>,
    ) -> Result<BTreeSet<RoomId>, store::StoreError> {
        let mut rooms = BTreeSet::from([self.personal_room]);
        for writer in writers {
            rooms.extend(self.store.seat(*writer)?.map(|s| s.room));
        }

        Ok(rooms)
    }
}

/// Why a scope cannot be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// Fixed text for the agent.
    Text(&'static str),
    /// The record cannot be read.
    Unreadable,
}

impl From<&'static str> for Refused {
    fn from(text: &'static str) -> Self {
        Self::Text(text)
    }
}

impl From<store::StoreError> for Refused {
    fn from(_: store::StoreError) -> Self {
        Self::Unreadable
    }
}

/// The scope `arguments` ask for. `named` is the room an address names,
/// which the `room` argument must not contradict. `no_run` is what a
/// call for the scope `run` is told when the server serves no run.
///
/// # Errors
///
/// Fails with fixed text naming what is wrong, or when the record
/// cannot be read.
pub fn resolve(
    call: &Call<'_>,
    arguments: &Value,
    named: Option<RoomId>,
    no_run: &'static str,
) -> Result<Scope, Refused> {
    let name = Name::of(arguments)?;
    let argued = match &arguments["room"] {
        Value::Null => None,
        Value::String(text) => Some(RoomId::parse(text).map_err(|_| BAD_ROOM)?),
        _ => return Err(BAD_ROOM.into()),
    };
    if argued.zip(named).is_some_and(|(a, n)| a != n) {
        return Err(ROOM_CONFLICT.into());
    }
    let room = argued.or(named).unwrap_or(call.personal_room);
    let current = match call.run {
        Some(run) => call.run_writers(run)?,
        None => BTreeSet::new(),
    };
    let rooms = match (name, call.run) {
        (Name::Run, None) => return Err(no_run.into()),
        (Name::Run, Some(_)) => BTreeSet::new(),
        (Name::Room, None) => BTreeSet::from([room]),
        (Name::Room, Some(_)) => {
            let mine = call.run_rooms(&current)?;
            mine.contains(&room)
                .then_some(BTreeSet::from([room]))
                .ok_or(NO_SEAT_IN_ROOM)?
        }
        (Name::Rooms, None) => return Err(ROOMS_NEED_A_RUN.into()),
        (Name::Rooms, Some(_)) => call.run_rooms(&current)?,
    };

    Ok(Scope {
        name,
        current,
        rooms,
    })
}

impl Scope {
    /// Whether the scope reaches beyond the calling run, which each
    /// call is logged for.
    #[must_use]
    pub fn extends(&self) -> bool {
        self.name != Name::Run
    }

    /// Whether the scope reaches an event of `writer`, whose seat lies
    /// in `room`.
    #[must_use]
    pub fn covers(&self, writer: WriterId, room: RoomId) -> bool {
        match self.name {
            Name::Run => self.current.contains(&writer),
            Name::Room | Name::Rooms => self.rooms.contains(&room),
        }
    }

    /// Whether `writer` is one of the calling run's.
    #[must_use]
    pub fn is_current(&self, writer: WriterId) -> bool {
        self.current.contains(&writer)
    }

    /// The events a count over the scope reaches.
    #[must_use]
    pub fn reach(&self) -> Reach {
        match self.name {
            Name::Run => Reach::Writers(self.current.iter().copied().collect()),
            Name::Room | Name::Rooms => Reach::Rooms(self.rooms.iter().copied().collect()),
        }
    }

    /// The rooms and writers a search is limited to, narrowed to the
    /// writer a call's `run` argument names; none when nothing can
    /// match. A search with no writers and no rooms would reach every
    /// event, so a scope that reaches none returns none here.
    #[must_use]
    pub fn limits(&self, only: Option<WriterId>) -> Option<(Vec<RoomId>, Vec<WriterId>)> {
        let writers: Vec<WriterId> = match (self.name, only) {
            (Name::Run, None) => self.current.iter().copied().collect(),
            (Name::Run, Some(w)) => self.current.contains(&w).then_some(w).into_iter().collect(),
            (_, only) => only.into_iter().collect(),
        };
        let rooms: Vec<RoomId> = self.rooms.iter().copied().collect();
        let nothing = match self.name {
            Name::Run => writers.is_empty(),
            Name::Room | Name::Rooms => rooms.is_empty(),
        };

        (!nothing).then_some((rooms, writers))
    }
}

#[cfg(test)]
mod tests;
