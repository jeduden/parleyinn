//! What recall reads of the record beyond one event (RCL-05, RCL-08,
//! RCL-01): the writer labels a node's device seat assigned, and the
//! counts `stat_list` gives within a recall scope. Both are functions of
//! the record alone (I10): nothing here is stored.

use std::collections::BTreeMap;

use record::event::EventKind;
use record::id::{RoomId, WriterId};
use record::range::Label;
use rusqlite::Connection;
use serde_json::Value;

use crate::StoreError;

/// The events a recall scope reaches: the writers of some rooms, or
/// some writers whatever their rooms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reach {
    Writers(Vec<WriterId>),
    Rooms(Vec<RoomId>),
}

/// The counts within a recall scope (`stat_list`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
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

/// The `WHERE` clause and argument selecting the events `reach`
/// reaches, over `events e JOIN seats s`.
fn clause(reach: &Reach) -> (&'static str, String) {
    match reach {
        Reach::Writers(writers) => (
            "e.writer IN (SELECT value FROM json_each(?1))",
            Value::from(writers.iter().map(ToString::to_string).collect::<Vec<_>>()).to_string(),
        ),
        Reach::Rooms(rooms) => (
            "s.room IN (SELECT value FROM json_each(?1))",
            Value::from(rooms.iter().map(ToString::to_string).collect::<Vec<_>>()).to_string(),
        ),
    }
}

/// The counts of the events `reach` reaches. A compaction is a `system`
/// line of subtype `compact_boundary` the harness wrote.
pub(crate) fn counts(conn: &Connection, reach: &Reach) -> Result<Counts, StoreError> {
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

    Ok(Counts {
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
