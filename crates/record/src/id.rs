//! The ids the record names writers, rooms and seats by, `seq`, and
//! the address (writer, `seq`) every event is found by (REC-06). Each
//! id derives from a key, never chosen: SHA-256 over a label of its own
//! and the key, so one key's writer id and seat id never coincide.

use std::fmt;

pub use crate::canonical::MAX_SAFE;
pub use crate::digest::{Digest, HexError};

/// The label a writer id is derived under.
const WRITER_LABEL: &[u8] = b"parinn writer id v1\0";
/// The label a room id is derived under.
const ROOM_LABEL: &[u8] = b"parinn room id v1\0";
/// The label a seat id is derived under.
const SEAT_LABEL: &[u8] = b"parinn seat id v1\0";

/// The longest part of a refused address an error repeats.
const SHOWN: usize = 80;

macro_rules! id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Digest);

        impl $name {
            /// The digest the id is.
            #[must_use]
            pub fn digest(&self) -> &Digest {
                &self.0
            }

            /// The id its 64 lowercase hex digits spell.
            ///
            /// # Errors
            ///
            /// Fails on any other text.
            pub fn parse(text: &str) -> Result<Self, HexError> {
                Digest::from_hex(text).map(Self)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

id!(
    /// A writer's id, derived from its seat's first key.
    WriterId
);
id!(
    /// A room's id, derived from the creating seat's first key and 128
    /// random bits its create room act fixes (LANE-01).
    RoomId
);
id!(
    /// A seat's id, derived from its room's id and its first key.
    SeatId
);

impl WriterId {
    /// The writer id of the seat whose first key is `first_key`.
    #[must_use]
    pub fn derive(first_key: &[u8; 32]) -> Self {
        Self(Digest::of(&[WRITER_LABEL, first_key]))
    }
}

impl RoomId {
    /// The room id a create room act fixes from its seat's first key
    /// and `nonce`.
    #[must_use]
    pub fn derive(first_key: &[u8; 32], nonce: &[u8; 16]) -> Self {
        Self(Digest::of(&[ROOM_LABEL, first_key, nonce]))
    }
}

impl SeatId {
    /// The id of the seat in `room` whose first key is `first_key`.
    #[must_use]
    pub fn derive(room: &RoomId, first_key: &[u8; 32]) -> Self {
        Self(Digest::of(&[SEAT_LABEL, room.0.as_bytes(), first_key]))
    }
}

/// An event's position in its writer: from 1, strictly increasing,
/// gap-free and never reused (REC-06), and an integer the canonical
/// encoding writes exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seq(u64);

impl Seq {
    /// A writer's first seq.
    pub const FIRST: Self = Self(1);

    /// `n` as a seq, if it is one: from 1 to 2^53 - 1.
    #[must_use]
    pub fn new(n: u64) -> Option<Self> {
        (1..=MAX_SAFE).contains(&n).then_some(Self(n))
    }

    /// The number.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }

    /// The seq after this one, if there is one.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        Self::new(self.0 + 1)
    }
}

impl fmt::Display for Seq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An event's address: (writer, seq), the only way the record names
/// an event (REC-06).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    pub writer: WriterId,
    pub seq: Seq,
}

impl Address {
    /// The address of `seq` in `writer`.
    #[must_use]
    pub fn new(writer: WriterId, seq: Seq) -> Self {
        Self { writer, seq }
    }

    /// Its full form, `parinn:room/<room id>/w/<writer id>/<seq>`, for a
    /// writer whose seat is in `room`.
    #[must_use]
    pub fn full(&self, room: &RoomId) -> String {
        format!("parinn:room/{room}/w/{}/{}", self.writer, self.seq)
    }

    /// The address `text` names, in the ASCII input form `<writer
    /// id>:<seq>` or the full form, with the room the full form names.
    ///
    /// # Errors
    ///
    /// Fails on any other text.
    pub fn parse(text: &str) -> Result<(Self, Option<RoomId>), AddressError> {
        let refused = || AddressError::new(text);
        if let Some(rest) = text.strip_prefix("parinn:room/") {
            let parts: Vec<&str> = rest.split('/').collect();
            let [room, "w", writer, seq] = parts.as_slice() else {
                return Err(refused());
            };
            let room = RoomId::parse(room).map_err(|_| refused())?;
            return Ok((ascii(writer, seq).ok_or_else(refused)?, Some(room)));
        }
        let (writer, seq) = text.split_once(':').ok_or_else(refused)?;

        Ok((ascii(writer, seq).ok_or_else(refused)?, None))
    }
}

/// The address a writer id and a seq, each as text, name.
fn ascii(writer: &str, seq: &str) -> Option<Address> {
    let writer = WriterId::parse(writer).ok()?;
    let seq = seq.parse().ok().and_then(Seq::new)?;

    Some(Address::new(writer, seq))
}

impl fmt::Display for Address {
    /// The ASCII input form, `<writer id>:<seq>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.writer, self.seq)
    }
}

/// Text that names no address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressError {
    shown: String,
}

impl AddressError {
    /// The error for `text`, which repeats at most its first 80
    /// characters.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut shown: String = text.chars().take(SHOWN).collect();
        if text.chars().count() > SHOWN {
            shown.push('…');
        }

        Self { shown }
    }
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "not an address: {:?}; want <writer id>:<seq> or parinn:room/<room id>/w/<writer id>/<seq>",
            self.shown
        )
    }
}

impl std::error::Error for AddressError {}

#[cfg(test)]
mod tests;
