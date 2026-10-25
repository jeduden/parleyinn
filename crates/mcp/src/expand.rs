//! `event_expand` and `event_get` (RCL-03, RCL-08): the full events of
//! an address range of one writer, exactly as recorded after redaction,
//! payloads resolved, inside the envelope.
//!
//! A call returns events whole while their estimated model tokens
//! (§8.4) stay within [`CAP`]. An event that does not fit is cut at a
//! character boundary; the call then returns a continuation cursor,
//! which `event_expand` takes as `cursor` and continues from, without
//! gap or overlap. A range is read in the order of its seqs, and an
//! event is served only once it checks against the record. Every
//! refusal is fixed text, naming at most an address it parsed and
//! renders anew (RCL-04).

use administration::purge::{Reason, Tombstone as Recorded};
use administration::quarantine::Quarantine;
use envelope::{
    Continuation, Envelope, Item, QuarantineMarker, Tombstone, TombstoneReason, tokens,
};
use record::id::{Address, RoomId, Seq, WriterId};
use record::range::{self, Label, Range, Writer};
use serde_json::Value;
use store::{Stored, StoredSeat};

use crate::cursor::Cursor;
use crate::scope::{self, Call, RecallScope};
use crate::{Note, Served, Server, UNREADABLE};

/// The most estimated model tokens one call returns (RCL-03).
pub const CAP: u64 = 8_000;

const NO_RANGE: &str = "event_expand needs a range or a cursor";
pub(crate) const BAD_RANGE: &str =
    "the range is not an address range such as A1:3-9, nor two addresses from and to of one writer";
const NOT_ONE: &str = "event_get takes one address, not a range";
const NO_ADDRESS: &str = "event_get needs an address or a cursor";
const NO_LABEL: &str = "no writer has that label";
pub(crate) const OUTSIDE_SCOPE: &str =
    "the address lies outside the current scope: pass scope \"room\" or \"rooms\" to extend it";
const OTHER_ROOM: &str = "the event lies in another room";
const NO_SEAT: &str = "the event's writer has no seat";
const CURSOR_MISMATCH: &str = "the cursor belongs to another writer than the range";

/// What a call asks for, once its range and cursor are read.
struct Asked {
    writer: Writer,
    from: Seq,
    to: Seq,
    /// Characters of the first event already returned.
    offset: usize,
    /// The room an address names.
    named: Option<RoomId>,
}

impl Server {
    /// `event_get`: the event at an address, or the rest of one a cursor
    /// continues, in the envelope.
    pub(crate) fn event_get(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        let given = match arguments["address"].as_str() {
            Some(text) => Some(Range::parse(text).map_err(|_| crate::BAD_ADDRESS)?),
            None => None,
        };
        if given.is_some_and(|range| !range.is_single()) {
            return Err(NOT_ONE.to_owned());
        }
        let asked = match arguments["cursor"].as_str() {
            Some(text) => self.continuing(given, text, "event_get")?,
            None => asked_of(given.ok_or(NO_ADDRESS)?),
        };

        self.read(arguments, run, "event_get", asked)
    }

    /// `event_expand`: the events of a range, or of the rest of one a
    /// cursor continues, in the envelope.
    pub(crate) fn event_expand(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        let given = optional_range(arguments)?;
        let asked = match arguments["cursor"].as_str() {
            Some(text) => self.continuing(given, text, "event_expand")?,
            None => asked_of(given.ok_or(NO_RANGE)?),
        };

        self.read(arguments, run, "event_expand", asked)
    }

    /// The position a cursor continues, which a range or an address the
    /// call also names must agree with in its writer.
    fn continuing(
        &mut self,
        given: Option<Range>,
        text: &str,
        tool: &'static str,
    ) -> Result<Asked, String> {
        let cursor = Cursor::decode(text)?;
        if let Some(range) = given {
            let writer = self.resolve_writer(range.writer, tool)?;
            if writer != cursor.writer {
                return Err(CURSOR_MISMATCH.to_owned());
            }
        }

        Ok(Asked {
            writer: Writer::Id(cursor.writer),
            from: cursor.seq,
            to: cursor.to,
            offset: cursor.offset,
            named: None,
        })
    }

    /// Serves `asked` within the scope `arguments` name.
    fn read(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
        tool: &'static str,
        asked: Asked,
    ) -> Result<Served, String> {
        let writer = self.resolve_writer(asked.writer, tool)?;
        let (scope, seat) = {
            let call = Call {
                store: &self.store,
                personal_room: self.personal_room,
                run,
            };
            let scope = scope::resolve(&call, arguments, asked.named, self.no_run(tool))
                .map_err(|r| self.refusal(tool, r))?;
            let seat = self
                .store
                .seat(writer)
                .map_err(|_| UNREADABLE)?
                .ok_or(NO_SEAT)?;
            (scope, seat)
        };
        if !scope.covers(writer, seat.room) {
            self.notes.borrow_mut().push(Note::ScopeRefused(tool));
            return Err(match scope.name {
                RecallScope::Run => OUTSIDE_SCOPE,
                _ => OTHER_ROOM,
            }
            .to_owned());
        }
        self.note_extended(tool, &scope);
        let (envelope, cut) = self.read_events(&scope, &seat, &asked, tool)?;
        if cut {
            self.notes.borrow_mut().push(Note::Truncated(tool));
        }

        Ok(Served {
            envelope,
            scope: Some(scope.name),
        })
    }

    /// The writer a range names, by id or by label.
    fn resolve_writer(&mut self, writer: Writer, tool: &'static str) -> Result<WriterId, String> {
        let label = match writer {
            Writer::Id(id) => return Ok(id),
            Writer::Label(label) => label,
        };
        let find = |server: &Server| -> Result<Option<WriterId>, String> {
            let labels = server.labels()?;

            Ok(labels
                .into_iter()
                .find(|(_, l)| *l == label)
                .map(|(w, _)| w))
        };
        if let Some(found) = find(self)? {
            return Ok(found);
        }
        self.assign_labels(tool)?;

        find(self)?.ok_or_else(|| NO_LABEL.to_owned())
    }

    /// The events from `asked.from` on, within the cap. An event a purge
    /// erased, or a quarantine holds, is not returned: its address shows
    /// as a tombstone of its reason (RCL-06).
    fn read_events(
        &self,
        scope: &scope::Scope,
        seat: &StoredSeat,
        asked: &Asked,
        tool: &'static str,
    ) -> Result<(Envelope, bool), String> {
        let label = self.labels()?.get(&seat.writer).copied();
        let mut reading = Reading::new(self, scope, seat, label)?;
        let (mut seq, mut offset) = (asked.from, asked.offset);
        let mut next = None;
        while seq <= asked.to {
            let address = Address::new(seat.writer, seq);
            let Some(stored) = self.store.event(address).map_err(unreadable)? else {
                self.end_of_log(address, tool, reading.is_empty())?;
                break;
            };
            let at = |offset| Cursor {
                writer: seat.writer,
                seq,
                offset,
                to: asked.to,
            };
            match reading.step(self, &stored, offset)? {
                Step::Went => {}
                Step::Cut(chars) => {
                    next = Some(at(offset + chars));
                    break;
                }
                Step::Stopped => {
                    next = Some(at(offset));
                    break;
                }
            }
            offset = 0;
            let Some(after) = seq.next() else { break };
            seq = after;
        }

        let (tombstones, quarantine_markers) = reading.marks.finish();

        Ok((
            Envelope {
                items: reading.items,
                tombstones,
                quarantine_markers,
                truncated: next.is_some(),
                next_cursor: next.map(|c| c.encode()),
                structural: None,
            },
            next.is_some(),
        ))
    }

    /// Ends a read at `address`, where the log holds no event: an error
    /// when nothing was read before it, or when later events exist; else
    /// the log ends inside the range, which is no gap (I6).
    fn end_of_log(
        &self,
        address: Address,
        tool: &'static str,
        nothing: bool,
    ) -> Result<(), String> {
        if nothing {
            return Err(format!("no event at {address}"));
        }
        // An event missing before later ones is a gap, and is told, not
        // passed over as if the log ended there.
        let head = self.store.head(address.writer).map_err(unreadable)?;
        if head.is_some_and(|head| head > address.seq) {
            self.notes.borrow_mut().push(Note::RecordGap(tool));
            return Err(format!(
                "the record has no event at {address} though later ones exist: run parinn verify"
            ));
        }

        Ok(())
    }
}

/// What reading one event came to.
enum Step {
    /// The event, or its tombstone, is in; the read goes on.
    Went,
    /// The event's content was cut after this many characters.
    Cut(usize),
    /// The event does not fit; the next call begins with it.
    Stopped,
}

/// What a read of one writer's range has gathered, and reads it from.
struct Reading<'a> {
    scope: &'a scope::Scope,
    seat: &'a StoredSeat,
    keys: administration::origin::KeySet,
    quarantine: Quarantine<'a>,
    purged: Vec<Recorded>,
    budget: Budget,
    items: Vec<Item>,
    marks: Marks,
}

impl<'a> Reading<'a> {
    fn new(
        server: &'a Server,
        scope: &'a scope::Scope,
        seat: &'a StoredSeat,
        label: Option<Label>,
    ) -> Result<Self, String> {
        Ok(Self {
            scope,
            seat,
            keys: administration::origin::KeySet::load(&server.store, &server.device_key)
                .map_err(unreadable)?,
            quarantine: Quarantine::load(&server.store, server.personal_room)
                .map_err(unreadable)?,
            purged: administration::purge::recorded(&server.store, server.personal_room)
                .map_err(unreadable)?,
            budget: Budget::new(),
            items: Vec::new(),
            marks: Marks::new(seat, label),
        })
    }

    /// Whether nothing was gathered yet.
    fn is_empty(&self) -> bool {
        self.items.is_empty() && self.marks.is_empty()
    }

    /// Reads `stored`, its content from the character `offset` on.
    fn step(&mut self, server: &Server, stored: &Stored, offset: usize) -> Result<Step, String> {
        let address = stored.link.header.address;
        let reason = held_back(stored, &self.purged, &self.quarantine).map_err(unreadable)?;
        if reason.is_none() && erased(stored) {
            // Content gone with no tombstone is a loss, not a purge.
            return Err(format!(
                "the content at {} was removed, and no tombstone records it: run parinn verify",
                address.full(&self.seat.room)
            ));
        }
        let Some(reason) = reason else {
            let item = server.event_item(stored, self.seat, &self.keys, self.scope, offset)?;

            return Ok(match self.budget.take(item) {
                Taken::Whole(item) => {
                    self.items.push(item);
                    Step::Went
                }
                Taken::Cut(item, chars) => {
                    self.items.push(item);
                    Step::Cut(chars)
                }
                Taken::Stop => Step::Stopped,
            });
        };
        let begins = self.marks.begins(address.seq, reason);
        if !self.budget.mark(begins, self.marks.cost()) {
            return Ok(Step::Stopped);
        }
        self.marks.add(address.seq, reason);

        Ok(Step::Went)
    }
}

/// Whether a purge erased the event's content: all of it is gone.
fn erased(stored: &Stored) -> bool {
    stored.content.is_none() && stored.commitment_key.is_none() && stored.payload.is_none()
}

/// Why the event is not returned, when it is not: a tombstone records
/// that a purge erased its content, or a quarantine holds it (RCL-06). A
/// purge wins, since nothing is left to hold. Content gone with no
/// tombstone is not a purge: it is returned as neither, and the caller
/// refuses it.
fn held_back(
    stored: &Stored,
    purged: &[Recorded],
    quarantine: &Quarantine<'_>,
) -> Result<Option<Held>, administration::AdminError> {
    let address = stored.link.header.address;
    if erased(stored) {
        return purged
            .iter()
            .find(|t| t.covers(address))
            .map(|t| reason_of(t).map(Held::Purged))
            .transpose();
    }

    Ok(quarantine.holds(address)?.then_some(Held::Quarantined))
}

/// Why a tombstone purged its range, by the words of
/// [`administration::purge::Reason`]: the principal, or a retention
/// policy (ADM-07, REC-15). A tombstone naming any other word is
/// refused, not shown as one of these.
fn reason_of(tombstone: &Recorded) -> Result<TombstoneReason, administration::AdminError> {
    let retention = Reason::Retention {
        policy: String::new(),
    };
    match tombstone.reason.as_str() {
        word if word == Reason::Purged.as_str() => Ok(TombstoneReason::Purged),
        word if word == retention.as_str() => Ok(TombstoneReason::Retention),
        _ => Err(administration::AdminError::Node(
            "a tombstone names a reason this parinn does not know",
        )),
    }
}

/// How the envelope holds an address back: a purged range as a
/// tombstone with its reason, a quarantined one as a marker with none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Purged(TombstoneReason),
    Quarantined,
}

/// The tombstones and quarantine markers a call has gathered:
/// consecutive seqs held back the same way share one.
struct Marks {
    room: RoomId,
    writer: WriterId,
    label: Option<Label>,
    ranges: Vec<(Seq, Seq, Held)>,
}

/// The widest seq: sixteen digits, the most a seq can hold.
const WIDEST_SEQ: &str = "9007199254740991";

impl Marks {
    fn new(seat: &StoredSeat, label: Option<Label>) -> Self {
        Self {
            room: seat.room,
            writer: seat.writer,
            label,
            ranges: Vec::new(),
        }
    }

    /// The estimated model tokens one tombstone of this writer takes at
    /// its longest, with the comma that joins it: its range at the
    /// widest seqs. A marker takes less, so the cap holds for both.
    fn cost(&self) -> u64 {
        let range = match self.label {
            Some(_) => format!("A999999999·{WIDEST_SEQ}–{WIDEST_SEQ}"),
            None => format!(
                "parinn:room/{}/w/{}/{WIDEST_SEQ}-{WIDEST_SEQ}",
                self.room, self.writer
            ),
        };
        let longest = Tombstone {
            range,
            reason: TombstoneReason::Retention,
        };
        let with = Envelope {
            tombstones: vec![longest],
            ..Envelope::default()
        };

        tokens::estimate(&with.render()) - tokens::estimate(&Envelope::default().render()) + 1
    }

    fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Whether `seq`, which the call reaches after the seqs before it,
    /// begins a tombstone of its own rather than extending the last.
    fn begins(&self, seq: Seq, reason: Held) -> bool {
        !self
            .ranges
            .last()
            .is_some_and(|(_, last, held)| *held == reason && last.next() == Some(seq))
    }

    /// Adds `seq` to the last tombstone, or begins one.
    fn add(&mut self, seq: Seq, reason: Held) {
        if self.begins(seq, reason) {
            self.ranges.push((seq, seq, reason));
        } else if let Some((_, last, _)) = self.ranges.last_mut() {
            *last = seq;
        }
    }

    /// A range as ParleyInn renders it: with its writer's label when it
    /// has one, else in the full form.
    fn render(&self, first: Seq, last: Seq) -> String {
        match (self.label, first == last) {
            (Some(label), true) => range::short(label, first),
            (Some(label), false) => range::range(label, first, last),
            (None, _) => format!("parinn:room/{}/w/{}/{first}-{last}", self.room, self.writer),
        }
    }

    /// The tombstones and the quarantine markers, in the order the call
    /// reached them.
    fn finish(self) -> (Vec<Tombstone>, Vec<QuarantineMarker>) {
        let mut tombstones = Vec::new();
        let mut markers = Vec::new();
        for (first, last, held) in &self.ranges {
            let range = self.render(*first, *last);
            match held {
                Held::Purged(reason) => tombstones.push(Tombstone {
                    range,
                    reason: *reason,
                }),
                Held::Quarantined => markers.push(QuarantineMarker { range }),
            }
        }

        (tombstones, markers)
    }
}

/// The refusal for a record that cannot be read, whatever failed.
fn unreadable<E>(_: E) -> String {
    UNREADABLE.to_owned()
}

/// What a range asks for, from its first event.
fn asked_of(range: Range) -> Asked {
    Asked {
        writer: range.writer,
        from: range.from,
        to: range.to,
        offset: 0,
        named: range.room,
    }
}

/// The range the `range` argument names, as one string or as `from`
/// and `to` addresses of one writer; none when the call has none.
fn optional_range(arguments: &Value) -> Result<Option<Range>, String> {
    let parse = |text: &Value| {
        text.as_str()
            .and_then(|t| Range::parse(t).ok())
            .ok_or_else(|| BAD_RANGE.to_owned())
    };
    match &arguments["range"] {
        Value::Null => Ok(None),
        text @ Value::String(_) => parse(text).map(Some),
        Value::Object(ends) => {
            let end = |name: &str| parse(ends.get(name).unwrap_or(&Value::Null));
            let (from, to) = (end("from")?, end("to")?);
            let same = from.writer == to.writer && from.room == to.room;
            let ordered = from.is_single() && to.is_single() && from.from <= to.from;
            if !same || !ordered {
                return Err(BAD_RANGE.to_owned());
            }

            Ok(Some(Range { to: to.to, ..from }))
        }
        _ => Err(BAD_RANGE.to_owned()),
    }
}

/// What taking one more item of the call's output came to.
enum Taken {
    /// The item fits whole.
    Whole(Item),
    /// The item's content was cut after this many characters.
    Cut(Item, usize),
    /// The item does not fit, and others are already in.
    Stop,
}

/// The estimated model tokens the call may still spend (RCL-03).
struct Budget {
    left: u64,
    holds: bool,
}

impl Budget {
    /// A budget for one call: the cap less the envelope around the
    /// items, at its longest cursor and the one that costs most, whose
    /// characters alternate between digits and letters.
    fn new() -> Self {
        let around = Envelope {
            items: vec![],
            tombstones: vec![],
            quarantine_markers: vec![],
            truncated: true,
            next_cursor: Some("0a".repeat(Cursor::LONGEST)),
            structural: None,
        };

        Self {
            left: CAP.saturating_sub(tokens::estimate(&around.render())),
            holds: false,
        }
    }

    /// Takes `item` whole if it fits. One that does not goes whole into
    /// the next call when others are already in; the first item of a
    /// call that does not fit even alone is cut so it does, and always
    /// keeps at least one character, so a call makes progress.
    fn take(&mut self, item: Item) -> Taken {
        let cost = item_cost(&item);
        if cost <= self.left {
            self.left -= cost;
            self.holds = true;

            return Taken::Whole(item);
        }
        if self.holds {
            return Taken::Stop;
        }
        let chars = self.fitting(&item).max(1);
        let mut cut = item;
        cut.content = cut.content.chars().take(chars).collect();
        cut.continuation.push(Continuation::Cut);
        self.left = self.left.saturating_sub(item_cost(&cut));
        self.holds = true;

        Taken::Cut(cut, chars)
    }

    /// Takes the `cost` of a new tombstone, when `new`; whether the call
    /// goes on. A tombstone that does not fit after others stops it,
    /// for the next call to begin with.
    fn mark(&mut self, new: bool, cost: u64) -> bool {
        if !new {
            return true;
        }
        if cost > self.left && self.holds {
            return false;
        }
        self.left = self.left.saturating_sub(cost);
        self.holds = true;

        true
    }

    /// How many characters of `item`'s content fit in what is left.
    fn fitting(&self, item: &Item) -> usize {
        // No character costs under a tenth of a model token.
        let most = usize::try_from(self.left.saturating_mul(10)).unwrap_or(usize::MAX);
        let text: Vec<char> = item.content.chars().take(most).collect();
        let (mut low, mut high) = (0, text.len());
        while low < high {
            let middle = (low + high).div_ceil(2);
            let probe = Item {
                content: text[..middle].iter().collect(),
                continuation: [item.continuation.clone(), vec![Continuation::Cut]].concat(),
                ..item.clone()
            };
            if item_cost(&probe) <= self.left {
                low = middle;
            } else {
                high = middle - 1;
            }
        }

        low
    }
}

/// An item's estimated model tokens, with the comma that joins it.
fn item_cost(item: &Item) -> u64 {
    item.tokens() + 1
}

#[cfg(test)]
mod tests;
