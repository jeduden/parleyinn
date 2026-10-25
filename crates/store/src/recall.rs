//! What recall reads of the record beyond one event (RCL-05, RCL-08,
//! RCL-01): the writer labels a node's device seat assigned, and the
//! counts `stat_list` gives within a recall scope. Both are functions of
//! the record alone (I10): nothing here is stored.

use std::collections::BTreeMap;

use record::commitment::Commitment;
use record::event::{EventKind, Provenance};
use record::id::{Address, RoomId, Seq, WriterId};
use record::range::Label;
use rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

use crate::StoreError;

/// The events a recall scope reaches: the writers of some rooms, or
/// some writers whatever their rooms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecallReach {
    Writers(Vec<WriterId>),
    Rooms(Vec<RoomId>),
}

/// The counts within a recall scope (`stat_list`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    pub events: u64,
    pub recalls: u64,
    pub compactions: u64,
}

/// The writer labels the assignment events of `device`, a device
/// seat's writer, give: an assignment's label is `A` and its rank among
/// that writer's assignment events, so no two writers share one. A
/// writer assigned twice keeps its first. An assignment whose content
/// was removed still takes its rank.
pub(crate) fn labels(
    conn: &Connection,
    device: WriterId,
) -> Result<BTreeMap<WriterId, Label>, StoreError> {
    let mut statement =
        conn.prepare("SELECT content FROM events WHERE writer = ?1 AND kind = ?2 ORDER BY seq")?;
    let args = [
        device.to_string(),
        EventKind::WriterLabel.as_str().to_owned(),
    ];
    let rows = statement.query_map(args, |r| r.get::<_, Option<String>>(0))?;
    let mut out = BTreeMap::new();
    for (rank, row) in (1u32..).zip(rows) {
        let writer = row?
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
            .and_then(|content| WriterId::parse(content["writer"].as_str()?).ok());
        if let (Some(writer), Some(label)) = (writer, Label::new(rank)) {
            out.entry(writer).or_insert(label);
        }
    }

    Ok(out)
}

/// The writers the certificate records of `device` name, in the order
/// the device seat recorded them (PRV-11).
pub(crate) fn certified(conn: &Connection, device: WriterId) -> Result<Vec<WriterId>, StoreError> {
    let mut statement =
        conn.prepare("SELECT content FROM events WHERE writer = ?1 AND kind = ?2 ORDER BY seq")?;
    let args = [
        device.to_string(),
        EventKind::CertificateRecord.as_str().to_owned(),
    ];
    let rows = statement.query_map(args, |r| r.get::<_, Option<String>>(0))?;
    let mut out = Vec::new();
    for row in rows {
        let writer = row?
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
            .and_then(|content| WriterId::parse(content["writer"].as_str()?).ok());
        out.extend(writer);
    }

    Ok(out)
}

/// The content of each event of kind `kind` in `writer`'s log, in seq
/// order, with its seq. An event whose content a purge erased, or that
/// does not parse, is left out.
pub(crate) fn contents(
    conn: &Connection,
    writer: WriterId,
    kind: EventKind,
) -> Result<Vec<(Seq, Value)>, StoreError> {
    let mut statement = conn.prepare(
        "SELECT seq, content FROM events WHERE writer = ?1 AND kind = ?2 AND content IS NOT NULL
         ORDER BY seq",
    )?;
    let args = [writer.to_string(), kind.as_str().to_owned()];
    let rows = statement.query_map(args, |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    let mut out = Vec::new();
    for row in rows {
        let (seq, content) = row?;
        let parsed = serde_json::from_str::<Value>(&content).ok();
        out.extend(crate::seq_of(seq).zip(parsed));
    }

    Ok(out)
}

/// What a range of one writer's log shows without its content: each
/// event's kind, provenance class, commitment and whether a purge erased
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeEvent {
    pub seq: Seq,
    pub kind: EventKind,
    pub provenance: Provenance,
    pub commitment: Commitment,
    /// Its content, commitment key and payload are all gone.
    pub erased: bool,
}

/// The events of `writer` from `first` to `last`, in seq order, as the
/// log holds them: none of their content is read.
pub(crate) fn range_events(
    conn: &Connection,
    writer: WriterId,
    first: Seq,
    last: Seq,
) -> Result<Vec<RangeEvent>, StoreError> {
    let mut statement = conn.prepare(
        "SELECT seq, kind, provenance, commitment,
                content IS NULL AND commitment_key IS NULL AND payload IS NULL
         FROM events WHERE writer = ?1 AND seq BETWEEN ?2 AND ?3 ORDER BY seq",
    )?;
    let rows = statement.query_map(
        rusqlite::params![writer.to_string(), crate::sql(first), crate::sql(last)],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, bool>(4)?,
            ))
        },
    )?;
    let corrupt = |what: &str| StoreError::Corrupt {
        address: None,
        what: format!("{what} of an event of writer {writer}"),
    };
    rows.map(|row| {
        let (seq, kind, provenance, commitment, erased) = row?;
        Ok(RangeEvent {
            seq: crate::seq_of(seq).ok_or_else(|| corrupt("seq"))?,
            kind: EventKind::parse(&kind).ok_or_else(|| corrupt("kind"))?,
            provenance: Provenance::parse(&provenance).ok_or_else(|| corrupt("provenance"))?,
            commitment: Commitment::from_hex(&commitment).map_err(|_| corrupt("commitment"))?,
            erased,
        })
    })
    .collect()
}

/// What an event shows of itself that a quarantine selector reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventFacts {
    pub kind: EventKind,
    pub provenance: Provenance,
    /// The transcript timestamp, kept as metadata (REC-12); none when the
    /// line carried none or a purge erased it.
    pub timestamp: Option<String>,
    /// Its content, commitment key and payload are all gone.
    pub erased: bool,
}

/// The facts of the event at `address`; none when it is not there.
pub(crate) fn event_facts(
    conn: &Connection,
    address: Address,
) -> Result<Option<EventFacts>, StoreError> {
    let row = conn
        .query_row(
            "SELECT kind, provenance, transcript_timestamp,
                    content IS NULL AND commitment_key IS NULL AND payload IS NULL
             FROM events WHERE writer = ?1 AND seq = ?2",
            rusqlite::params![address.writer.to_string(), crate::sql(address.seq)],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, bool>(3)?,
                ))
            },
        )
        .optional()?;
    let corrupt = |what: &str| StoreError::Corrupt {
        address: Some(address),
        what: what.to_owned(),
    };

    row.map(|(kind, provenance, timestamp, erased)| {
        Ok(EventFacts {
            kind: EventKind::parse(&kind).ok_or_else(|| corrupt("kind"))?,
            provenance: Provenance::parse(&provenance).ok_or_else(|| corrupt("provenance"))?,
            timestamp,
            erased,
        })
    })
    .transpose()
}

/// The heads of `writer` that events of `device` cited, as pairs of the
/// citing event's seq and the seq of `writer`'s event it cited, in the
/// order they were cited: the causal order of the two logs (PRV-12).
pub(crate) fn citations(
    conn: &Connection,
    device: WriterId,
    writer: WriterId,
) -> Result<Vec<(Seq, Seq)>, StoreError> {
    let mut statement = conn.prepare(
        "SELECT seq, other_seq FROM event_heads WHERE writer = ?1 AND other_writer = ?2
         ORDER BY seq",
    )?;
    let args = [device.to_string(), writer.to_string()];
    let rows = statement.query_map(args, |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
    let mut out = Vec::new();
    for row in rows {
        let (citing, cited) = row?;
        out.extend(crate::seq_of(citing).zip(crate::seq_of(cited)));
    }

    Ok(out)
}

/// The seq of the last event of `writer`'s log; none for an empty log.
pub(crate) fn head(conn: &Connection, writer: WriterId) -> Result<Option<Seq>, StoreError> {
    const LAST: &str = "SELECT max(seq) FROM events WHERE writer = ?1";
    let last: Option<i64> = conn.query_row(LAST, [writer.to_string()], |r| r.get(0))?;

    Ok(last.and_then(|n| u64::try_from(n).ok()).and_then(Seq::new))
}

/// The `WHERE` clause and argument selecting the events `reach`
/// reaches, over `events e JOIN seats s`.
fn clause(reach: &RecallReach) -> (&'static str, String) {
    match reach {
        RecallReach::Writers(writers) => (
            "e.writer IN (SELECT value FROM json_each(?1))",
            Value::from(writers.iter().map(ToString::to_string).collect::<Vec<_>>()).to_string(),
        ),
        RecallReach::Rooms(rooms) => (
            "s.room IN (SELECT value FROM json_each(?1))",
            Value::from(rooms.iter().map(ToString::to_string).collect::<Vec<_>>()).to_string(),
        ),
    }
}

/// The counts of the events `reach` reaches. A compaction is a `system`
/// line of subtype `compact_boundary` the harness wrote.
pub(crate) fn stats(conn: &Connection, reach: &RecallReach) -> Result<Stats, StoreError> {
    let (filter, argument) = clause(reach);
    let sql = format!(
        "SELECT count(*), coalesce(sum(e.kind = 'recall'), 0)
         FROM events e JOIN seats s ON s.writer = e.writer WHERE {filter}"
    );
    let both = |r: &rusqlite::Row<'_>| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?));
    let (events, recalls) = conn.query_row(&sql, [&argument], both)?;
    let mut statement = conn.prepare(&format!(
        "SELECT e.content FROM events e JOIN seats s ON s.writer = e.writer
         WHERE {filter} AND e.provenance = 'harness_text' AND e.content LIKE '%compact_boundary%'"
    ))?;
    let rows = statement.query_map([&argument], |r| r.get::<_, Option<String>>(0))?;
    let mut compactions = 0;
    for row in rows {
        compactions += u64::from(row?.is_some_and(|content| is_compact_boundary(&content)));
    }

    Ok(Stats {
        events: u64::try_from(events).unwrap_or(0),
        recalls: u64::try_from(recalls).unwrap_or(0),
        compactions,
    })
}

/// Whether a stored line's content is a `system` line of subtype
/// `compact_boundary`.
fn is_compact_boundary(content: &str) -> bool {
    let line = serde_json::from_str::<Value>(content)
        .ok()
        .and_then(|c| c["text"].as_str().map(str::to_owned))
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());

    line.is_some_and(|l| l["type"] == "system" && l["subtype"] == "compact_boundary")
}

#[cfg(test)]
mod tests;
