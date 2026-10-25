//! Ingest by the harness session's MCP server as the harness appends
//! (REC-13): each complete line past a transcript's ingest position goes
//! to its run's run seat, whose key the server mints and keeps in its
//! memory alone (SEC-10), certified by the device key as a run seat and
//! no ingest seat, so its events derive origin `witnessed` (PRV-13).
//!
//! - A line still being written, with no line break yet, waits for the
//!   next read.
//! - A held hook observation of the transcript's harness session goes to
//!   the main run's run seat at the offset where its hook fired (REC-19).
//! - A witnessed line is never a repeat reference, and never held: one
//!   whose `uuid` another transcript's event recorded is its own event,
//!   counted for the caller to audit (REC-04).
//! - An append another ingest got ahead of is refused in its transaction
//!   ([`store::StoreError::PositionMoved`]); the next read goes on from
//!   where that one left off (REC-06).

use std::collections::BTreeMap;
use std::path::Path;

use administration::home::Home;
use administration::node::Node;
use key::random::Entropy;
use key::signing::SeatKey;
use record::digest::Digest;
use record::id::WriterId;
use redaction::Redactor;
use store::{Position, Store};

use crate::observed::Observed;
use crate::run::{Defaults, RunKey};
use crate::{IngestError, Ingested, Inputs, read, reading};

/// The run seats of the runs one MCP server serves, by run, each key
/// held here alone: never written to the key store, a file, a log or any
/// output (SEC-10). A key is lost with the server, so a run it serves
/// again after a restart goes on in a new run seat.
#[derive(Default)]
pub struct RunSeats {
    seats: BTreeMap<RunKey, SeatKey>,
}

impl std::fmt::Debug for RunSeats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The runs alone; a key never reaches any output.
        f.debug_list().entries(self.seats.keys()).finish()
    }
}

impl RunSeats {
    /// The writer of the run seat `run` holds here, if it holds one.
    #[must_use]
    pub fn writer(&self, run: &RunKey) -> Option<WriterId> {
        self.seats
            .get(run)
            .map(|key| WriterId::derive(key.public().as_bytes()))
    }

    /// Each run with a run seat here, and its writer.
    #[must_use]
    pub fn runs(&self) -> Vec<(RunKey, WriterId)> {
        self.seats
            .keys()
            .filter_map(|run| Some((run.clone(), self.writer(run)?)))
            .collect()
    }

    /// The writer of `run`'s run seat: one held here, or a new one
    /// minted in memory and certified by `node`'s device key as a run
    /// seat and no ingest seat, recorded in `store`.
    ///
    /// # Errors
    ///
    /// Fails when the key cannot be minted or the store fails.
    pub(crate) fn ensure(
        &mut self,
        store: &mut Store,
        node: &Node,
        run: &RunKey,
        entropy: &mut dyn Entropy,
    ) -> Result<WriterId, IngestError> {
        if let Some(writer) = self.writer(run) {
            return Ok(writer);
        }
        let key = SeatKey::mint(entropy).map_err(IngestError::Key)?;
        let seat = node
            .certify_run_seat(store, &key, entropy)
            .map_err(IngestError::Admin)?;
        self.seats.insert(run.clone(), key);

        Ok(seat.writer)
    }
}

/// Ingests what the transcript source at `path`, a path resolved within
/// the allowed roots, gained past its ingest position, into the node in
/// `home`, each run in its run seat in `seats`: each complete line,
/// redacted by `redactor` and the harness account identifiers the
/// transcript names, and the held hook observations of its harness
/// session, at the offset where each hook fired. `limit`, when given,
/// stops it at that many lines of the transcript: a development build's
/// seam for the tests, never set in a release build.
///
/// None when it gained nothing: no complete line past its position and
/// no hook observation held for it, so no run seat starts for nothing.
///
/// # Errors
///
/// As [`crate::ingest_with`], and [`IngestError::Partial`] holding
/// [`store::StoreError::PositionMoved`] when another ingest got ahead.
pub fn ingest_live(
    home: &Home,
    path: &Path,
    redactor: &Redactor,
    inputs: &Inputs,
    seats: &mut RunSeats,
    limit: Option<usize>,
    entropy: &mut dyn Entropy,
) -> Result<Option<Ingested>, IngestError> {
    let bytes = read::transcript(path).map_err(|source| IngestError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let complete = complete_lines(&bytes, limit);
    if !gained(home, path, &bytes, complete as u64)? {
        return Ok(None);
    }
    let text = String::from_utf8(bytes[..complete].to_vec())
        .map_err(|_| IngestError::NotUtf8(path.to_path_buf()))?;

    reading::ingest(
        home,
        path,
        &text,
        redactor,
        inputs,
        &mut reading::Recorder::Live(seats),
        entropy,
    )
    .map(Some)
}

/// Whether the transcript source at `path`, whose bytes are `bytes` and
/// whose complete lines take `complete` of them, gained a line past its
/// ingest position, was rewritten, or holds a hook observation not yet
/// in the record.
fn gained(home: &Home, path: &Path, bytes: &[u8], complete: u64) -> Result<bool, IngestError> {
    home.create().map_err(IngestError::Admin)?;
    let store = Store::open(&home.store())?;
    let position = store
        .source(&path.to_string_lossy())?
        .map_or_else(Position::start, |s| s.position);
    let session = Defaults::of(path).own();
    // An observation whose hook fired past the complete lines waits.
    let held = session.agent_id.is_none()
        && Observed::load(&store)?
            .pending(&session.harness_session)
            .iter()
            .any(|h| h.observation.offset.is_none_or(|o| o <= complete));
    // A position past the complete lines is another ingest's, ahead,
    // unless what it consumed changed: a new transcript generation.
    let consumed = bytes
        .get(..usize::try_from(position.offset).unwrap_or(usize::MAX))
        .map(|prefix| Digest::of(&[prefix]));
    let rewritten = consumed != Some(position.prefix);

    Ok(position.offset < complete || rewritten || (held && position.offset == complete))
}

/// How many of `bytes` its complete lines take: up to the last line
/// break, and no further than the end of line `limit` when given.
fn complete_lines(bytes: &[u8], limit: Option<usize>) -> usize {
    let ends = bytes
        .iter()
        .enumerate()
        .filter(|(_, b)| **b == b'\n')
        .map(|(i, _)| i + 1);
    match limit {
        Some(n) => ends.take(n).fold(None, |_, end| Some(end)),
        None => bytes.iter().rposition(|b| *b == b'\n').map(|i| i + 1),
    }
    .unwrap_or(0)
}

#[cfg(test)]
mod tests;
