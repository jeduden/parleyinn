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

use envelope::{Envelope, Item, tokens};
use record::id::{Address, RoomId, Seq, WriterId};
use record::range::{Range, Writer};
use serde_json::Value;
use store::StoredSeat;

use crate::cursor::Cursor;
use crate::scope::{self, Call, Name};
use crate::{Note, Served, Server, UNREADABLE};

/// The most estimated model tokens one call returns (RCL-03).
pub const CAP: u64 = 8_000;

/// The flag on an item whose content stops short of the event's: the
/// cursor continues it.
pub const CUT: &str = "cut";
/// The flag on an item whose content continues an earlier one.
pub const CONTINUED: &str = "continued";

const NO_RANGE: &str = "event_expand needs a range or a cursor";
pub(crate) const BAD_RANGE: &str =
    "the range is not an address range such as A1:3-9, nor two addresses from and to of one writer";
const NOT_ONE: &str = "event_get takes one address, not a range";
const NO_ADDRESS: &str = "event_get needs an address";
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
    /// `event_get`: the event at an address, in the envelope.
    pub(crate) fn event_get(
        &mut self,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        let text = arguments["address"].as_str().ok_or(NO_ADDRESS)?;
        let range = Range::parse(text).map_err(|_| crate::BAD_ADDRESS)?;
        if !range.is_single() {
            return Err(NOT_ONE.to_owned());
        }
        let asked = Asked {
            writer: range.writer,
            from: range.from,
            to: range.to,
            offset: 0,
            named: range.room,
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
        let asked = match arguments["cursor"].as_str() {
            Some(text) => self.continuing(arguments, text)?,
            None => ranged(arguments)?,
        };

        self.read(arguments, run, "event_expand", asked)
    }

    /// The position a cursor continues, which a range the call also
    /// names must agree with in its writer.
    fn continuing(&mut self, arguments: &Value, text: &str) -> Result<Asked, String> {
        let cursor = Cursor::decode(text)?;
        if let Some(range) = optional_range(arguments)? {
            let writer = self.resolve_writer(range.writer)?;
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
        let writer = self.resolve_writer(asked.writer)?;
        let (scope, seat) = {
            let call = Call {
                store: &self.store,
                personal_room: self.personal_room,
                run,
            };
            let scope = scope::resolve(&call, arguments, asked.named, self.no_run(tool))
                .map_err(|r| self.refusal(r))?;
            let seat = self
                .store
                .seat(writer)
                .map_err(|_| UNREADABLE)?
                .ok_or(NO_SEAT)?;
            (scope, seat)
        };
        if !scope.covers(writer, seat.room) {
            return Err(match scope.name {
                Name::Run => OUTSIDE_SCOPE,
                _ => OTHER_ROOM,
            }
            .to_owned());
        }
        self.note_extended(tool, &scope);
        let (envelope, cut) = self.read_events(&scope, &seat, &asked)?;
        if cut {
            self.notes.borrow_mut().push(Note::Truncated(tool));
        }

        Ok(Served {
            envelope,
            scope: Some(scope.name),
        })
    }

    /// The writer a range names, by id or by label.
    fn resolve_writer(&mut self, writer: Writer) -> Result<WriterId, String> {
        let Writer::Label(label) = writer else {
            return match writer {
                Writer::Id(id) => Ok(id),
                Writer::Label(_) => Err(NO_LABEL.to_owned()),
            };
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
        self.assign_labels();

        find(self)?.ok_or_else(|| NO_LABEL.to_owned())
    }

    /// The events from `asked.from` on, within the cap.
    fn read_events(
        &self,
        scope: &scope::Scope,
        seat: &StoredSeat,
        asked: &Asked,
    ) -> Result<(Envelope, bool), String> {
        let unreadable = |_| UNREADABLE.to_owned();
        let keys = administration::origin::KeySet::load(&self.store, &self.device_key)
            .map_err(unreadable)?;
        let mut budget = Budget::new();
        let mut items: Vec<Item> = Vec::new();
        let (mut seq, mut offset) = (asked.from, asked.offset);
        let mut next = None;
        let writer = seat.writer;
        while seq <= asked.to {
            let address = Address::new(writer, seq);
            let Some(stored) = self.store.event(address).map_err(unreadable)? else {
                if items.is_empty() {
                    return Err(format!("no event at {address}"));
                }
                break;
            };
            let item = self.event_item(&stored, seat, &keys, scope, offset)?;
            match budget.take(item) {
                Taken::Whole(item) => items.push(item),
                Taken::Cut(item, chars) => {
                    items.push(item);
                    next = Some(Cursor {
                        writer,
                        seq,
                        offset: offset + chars,
                        to: asked.to,
                    });
                    break;
                }
                Taken::Stop => {
                    next = Some(Cursor {
                        writer,
                        seq,
                        offset,
                        to: asked.to,
                    });
                    break;
                }
            }
            offset = 0;
            let Some(after) = seq.next() else { break };
            seq = after;
        }

        Ok((
            Envelope {
                items,
                truncated: next.is_some(),
                next_cursor: next.map(|c| c.encode()),
                structural: None,
            },
            next.is_some(),
        ))
    }
}

/// The range a call names, or a refusal; one is required.
fn ranged(arguments: &Value) -> Result<Asked, String> {
    let range = optional_range(arguments)?.ok_or(NO_RANGE)?;

    Ok(Asked {
        writer: range.writer,
        from: range.from,
        to: range.to,
        offset: 0,
        named: range.room,
    })
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
        cut.flags.push(CUT.to_owned());
        self.left = self.left.saturating_sub(item_cost(&cut));
        self.holds = true;

        Taken::Cut(cut, chars)
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
                flags: [item.flags.clone(), vec![CUT.to_owned()]].concat(),
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
