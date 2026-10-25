//! The forms an address and an address range are written in, and read
//! from (RCL-08): the short form `A2·4812` and the range `A2·4812–5025`
//! ParleyInn shows, the full form
//! `parinn:room/<room id>/w/<writer id>/<seq>`, and the ASCII input
//! form `<label>:<seq>` or `<label>:<from>-<to>`, the writer id allowed
//! in place of the label. A writer label is a writer's short display
//! alias, `A` and a number, unique per node (the store derives it from
//! the record). Nothing here reads a file, a clock or randomness.

use std::fmt;

use crate::id::{RoomId, Seq, WriterId};

/// The longest part of a refused range an error repeats.
const SHOWN: usize = 80;

/// How many digits a label's number holds at most.
const LABEL_DIGITS: usize = 9;

/// A writer's short display alias: `A` and a number from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(u32);

impl Label {
    /// The label of the writer whose assignment is the `rank`th, from 1.
    #[must_use]
    pub fn new(rank: u32) -> Option<Self> {
        (rank >= 1).then_some(Self(rank))
    }

    /// The label `text` spells: `A` and one to nine digits, no leading
    /// zero.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('A')?;
        let plain = !digits.starts_with('0') && digits.len() <= LABEL_DIGITS;
        if !plain || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }

        digits.parse().ok().and_then(Self::new)
    }

    /// The rank the label spells.
    #[must_use]
    pub fn rank(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A{}", self.0)
    }
}

/// How a range names its writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Writer {
    Label(Label),
    Id(WriterId),
}

/// An address range: events `from` to `to` of one writer, a single
/// address when they are one (RCL-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub writer: Writer,
    pub from: Seq,
    pub to: Seq,
    /// The room the full form names.
    pub room: Option<RoomId>,
}

impl Range {
    /// The range `text` writes, in any form ParleyInn shows or takes.
    ///
    /// # Errors
    ///
    /// Fails on any other text, and on a range that runs backwards.
    pub fn parse(text: &str) -> Result<Self, RangeError> {
        let refused = || RangeError::new(text);
        let (room, rest) = match text.strip_prefix("parinn:room/") {
            Some(full) => {
                let parts: Vec<&str> = full.split('/').collect();
                let [room, "w", writer, seqs] = parts.as_slice() else {
                    return Err(refused());
                };
                let room = RoomId::parse(room).map_err(|_| refused())?;
                (Some(room), (*writer, *seqs))
            }
            None => (None, split_short(text).ok_or_else(refused)?),
        };
        let (writer, seqs) = rest;
        let writer = match (Label::parse(writer), WriterId::parse(writer)) {
            (Some(label), _) => Writer::Label(label),
            (None, Ok(id)) => Writer::Id(id),
            (None, Err(_)) => return Err(refused()),
        };
        let (from, to) = seqs_of(seqs).ok_or_else(refused)?;

        Ok(Self {
            writer,
            from,
            to,
            room,
        })
    }

    /// Whether the range names one address.
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.from == self.to
    }
}

/// The writer and the seqs of a short or ASCII form: `A2·4812`,
/// `A2·4812–5025`, `A2:4812` or `A2:4812-5025`.
fn split_short(text: &str) -> Option<(&str, &str)> {
    text.split_once('·').or_else(|| text.split_once(':'))
}

/// The seqs of a single address or of a range written with a hyphen or
/// an en dash.
fn seqs_of(text: &str) -> Option<(Seq, Seq)> {
    let seq = |digits: &str| {
        let plain = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
        plain
            .then(|| digits.parse().ok().and_then(Seq::new))
            .flatten()
    };
    match text.split_once(['-', '–']) {
        None => seq(text).map(|s| (s, s)),
        Some((from, to)) => {
            let (from, to) = (seq(from)?, seq(to)?);
            (from <= to).then_some((from, to))
        }
    }
}

/// The short form of the address (`label`, `seq`): `A2·4812`.
#[must_use]
pub fn short(label: Label, seq: Seq) -> String {
    format!("{label}·{seq}")
}

/// The range form of events `from` to `to` of `label`: `A2·4812–5025`.
#[must_use]
pub fn range(label: Label, from: Seq, to: Seq) -> String {
    format!("{label}·{from}–{to}")
}

/// Text that writes no address range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeError {
    shown: String,
}

impl RangeError {
    fn new(text: &str) -> Self {
        let mut shown: String = text.chars().take(SHOWN).collect();
        if text.chars().count() > SHOWN {
            shown.push('…');
        }

        Self { shown }
    }
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "not an address or range: {:?}; want A1:7, A1:3-9 or a full address",
            self.shown
        )
    }
}

impl std::error::Error for RangeError {}

#[cfg(test)]
mod tests;
