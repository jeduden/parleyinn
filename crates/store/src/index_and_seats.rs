//! The store's search index, seats, writers, meta and closing appends.

use super::*;

impl Store {
    /// Derives the search index again from the record alone, in one
    /// transaction, and returns how many events it read (I10).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when an event cannot be read, or when
    /// the store has no search index.
    pub fn rebuild_search_index(&mut self) -> Result<u64, StoreError> {
        if !search_index::exists(&self.conn)? {
            return Err(StoreError::NoIndex);
        }
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        let done = self.reindex();
        self.finish(done)
    }

    /// Empties the index and fills it from every writer's log, inside a
    /// transaction already begun.
    pub(crate) fn reindex(&self) -> Result<u64, StoreError> {
        self.reindex_into(&self.conn, &|| true)
    }

    /// Derives the search index from the record into the index tables
    /// `index` holds, cleared first, while `time_left` says time remains
    /// before each chunk of events: [`StoreError::Cancelled`] once it no
    /// longer does (ENG-03). Returns how many events it read.
    fn reindex_into(
        &self,
        index: &Connection,
        time_left: &dyn Fn() -> bool,
    ) -> Result<u64, StoreError> {
        search_index::clear(index)?;
        let mut count = 0;
        for writer in self.writers()? {
            // A writer with no event holds no slot, as when it was first
            // indexed (I10).
            let mut indexing = None;
            let mut after = 0;
            loop {
                if !time_left() {
                    return Err(StoreError::Cancelled);
                }
                let chunk =
                    self.rows_limited(writer, "AND seq > ?2", after, search_index::CHUNK)?;
                let Some(last) = chunk.last() else { break };
                after = sql(last.link.header.address.seq);
                let open = match indexing.as_mut() {
                    Some(open) => open,
                    None => indexing.insert(search_index::Indexing::open(index, writer)?),
                };
                index_chunk(open, &chunk)?;
                count += chunk.len() as u64;
            }
            if let Some(open) = indexing {
                open.done_to(u64::try_from(after).unwrap_or(0))?;
            }
        }

        Ok(count)
    }

    /// Searches the index (RCL-02): the hits `search` asks for, best
    /// first, under its deadline (SEC-04).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when the store has no search index, or
    /// with [`StoreError::Cancelled`] when the deadline passes.
    pub fn search(&self, search: &Search<'_>) -> Result<Vec<Hit>, StoreError> {
        search_index::run(&self.conn, search, None)
    }

    /// [`search`](Self::search) over the matches `keep` keeps, chosen
    /// before the ranking cuts them (RCL-02).
    ///
    /// # Errors
    ///
    /// As [`search`](Self::search).
    pub fn search_where(
        &self,
        search: &Search<'_>,
        keep: &dyn Fn(Address) -> bool,
    ) -> Result<Vec<Hit>, StoreError> {
        search_index::run(&self.conn, search, Some(keep))
    }

    /// The writer labels `device`, a device seat's writer, assigned
    /// (RCL-08): derived from its assignment events alone.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn writer_labels(
        &self,
        device: WriterId,
    ) -> Result<BTreeMap<WriterId, record::range::Label>, StoreError> {
        recall::labels(&self.conn, device)
    }

    /// The writers `device`'s certificate records name, in the order it
    /// recorded them.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn certified_writers(&self, device: WriterId) -> Result<Vec<WriterId>, StoreError> {
        recall::certified(&self.conn, device)
    }

    /// The content of each event of kind `kind` in `writer`'s log, in seq
    /// order, with its seq: what a derivation over one kind of event
    /// reads without parsing the whole log. An event whose content was
    /// erased by a purge is left out.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn contents(
        &self,
        writer: WriterId,
        kind: EventKind,
    ) -> Result<Vec<(Seq, Value)>, StoreError> {
        recall::contents(&self.conn, writer, kind)
    }

    /// The events of `writer` from `first` to `last`, in seq order, as
    /// the log holds them, none of their content read: a purge's plan
    /// and its tombstone read them.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn range_events(
        &self,
        writer: WriterId,
        first: Seq,
        last: Seq,
    ) -> Result<Vec<RangeEvent>, StoreError> {
        recall::range_events(&self.conn, writer, first, last)
    }

    /// The heads of `writer` that events of `device` cited, as pairs of
    /// the citing event's seq and the seq of the cited event of `writer`,
    /// in the order they were cited.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn citations(
        &self,
        device: WriterId,
        writer: WriterId,
    ) -> Result<Vec<(Seq, Seq)>, StoreError> {
        recall::citations(&self.conn, device, writer)
    }

    /// The kind, provenance class and transcript timestamp of the event
    /// at `address`, and whether a purge erased it.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn event_facts(&self, address: Address) -> Result<Option<EventFacts>, StoreError> {
        recall::event_facts(&self.conn, address)
    }

    /// The counts of the events `reach` reaches (`stat_list`).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn stats(&self, reach: &RecallReach) -> Result<Stats, StoreError> {
        recall::stats(&self.conn, reach)
    }

    /// The search index's canonical digest (§8.3): the same for any two
    /// stores that hold the same writer logs, however they came to
    /// hold them.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn search_index_digest(&self) -> Result<Digest, StoreError> {
        if !search_index::exists(&self.conn)? {
            return Err(StoreError::NoIndex);
        }

        search_index::digest(&self.conn)
    }

    /// The schema objects the store holds that this build does not
    /// ship, as `schema.name` (ADM-09).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn foreign_schema_objects(&self) -> Result<Vec<String>, StoreError> {
        integrity::foreign(&self.conn)
    }

    /// Whether the search index ranks as a new index does (ADM-09): no
    /// ranking function was set in its configuration.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn search_index_ranking_is_shipped(&self) -> Result<bool, StoreError> {
        if !search_index::exists(&self.conn)? {
            return Err(StoreError::NoIndex);
        }

        integrity::ranking_is_shipped(&self.conn)
    }

    /// Whether the search index holds what the record derives (ADM-09,
    /// I10): the index derived again from the record alone, into an
    /// index in memory, has the canonical digest of the one the store
    /// keeps. The entries of the events inside a pending purge's ranges
    /// are left out of both digests: the purge's residue stays in the
    /// index until it is finished ([`Store::unfinished_purges`] names
    /// those ranges). Everything is read in one read transaction, which
    /// takes no write lock: live ingest and the hook handlers append
    /// meanwhile (I9). `time_left` is asked before each chunk of events.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when an event cannot be read, when the
    /// store has no search index, or with [`StoreError::Cancelled`] once
    /// `time_left` says the time is up (ENG-03).
    pub fn search_index_is_derived(
        &self,
        time_left: &dyn Fn() -> bool,
    ) -> Result<bool, StoreError> {
        if !search_index::exists(&self.conn)? {
            return Err(StoreError::NoIndex);
        }
        self.conn.execute_batch("BEGIN DEFERRED")?;
        let compared = purge::pending_ranges(&self.conn).and_then(|pending| {
            let kept = search_index::digest_outside(&self.conn, &pending)?;
            let index = search_index::in_memory()?;
            self.reindex_into(&index, time_left)?;
            Ok(search_index::digest_outside(&index, &pending)? == kept)
        });
        let _ = self.conn.execute_batch("ROLLBACK");

        compared
    }

    /// The writers whose events the search index leaves out, because
    /// another writer's id holds their slot.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn search_index_unindexed_writers(&self) -> Result<Vec<WriterId>, StoreError> {
        search_index::unindexed_writers(&self.conn)?
            .iter()
            .map(|text| {
                WriterId::parse(text).map_err(|_| StoreError::Corrupt {
                    address: None,
                    what: format!("writer id {text:?}"),
                })
            })
            .collect()
    }

    /// The events the search index leaves out (REC-11, I6): a writer
    /// whose slot another holds, and a writer from seq 2^28 on.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn search_index_gaps(&self) -> Result<Vec<IndexGap>, StoreError> {
        search_index::gaps(&self.conn, &self.newest_seqs()?)
    }

    /// The gaps of the search index not yet reported.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, an id does not parse or the store has no
    /// search index.
    pub fn unreported_search_index_gaps(&self) -> Result<Vec<IndexGap>, StoreError> {
        search_index::unreported(&self.conn, self.search_index_gaps()?)
    }

    /// Notes that `gaps` were reported, so the next ingest does not
    /// report them again.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn mark_search_index_gaps_reported(&self, gaps: &[IndexGap]) -> Result<(), StoreError> {
        search_index::mark_reported(&self.conn, gaps)
    }

    /// Takes back the note that `gaps` were reported, for the ones whose
    /// report did not reach the audit log.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the store has no search index.
    pub fn unmark_search_index_gaps_reported(&self, gaps: &[IndexGap]) -> Result<(), StoreError> {
        search_index::unmark_reported(&self.conn, gaps)
    }

    /// The schema version this store was made with (ADM-05, ADM-11).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the version does not parse.
    pub fn schema_version(&self) -> Result<u32, StoreError> {
        let version: String = self.conn.query_row(VERSION, [], |r| r.get(0))?;

        version.parse().map_err(|_| StoreError::Corrupt {
            address: None,
            what: format!("schema_version {version:?}"),
        })
    }

    /// Checks the store's schema version, header field set and encoding
    /// against this parinn's (§8.3, ADM-05).
    pub(crate) fn check_format(&self) -> Result<(), StoreError> {
        let found = self.schema_version()?;
        if found > SCHEMA_VERSION {
            return Err(StoreError::Newer { found });
        }
        for (key, want) in [
            ("header_field_set", FIELD_SET),
            ("header_encoding", ENCODING),
        ] {
            let found = self.meta(key)?.unwrap_or_default();
            if found != want {
                return Err(StoreError::Format {
                    key: key.to_owned(),
                    found,
                });
            }
        }

        Ok(())
    }

    /// Runs `read` on one snapshot of the store: every read it makes
    /// sees the store as it stood when the snapshot began, whatever
    /// another connection appends meanwhile.
    ///
    /// # Errors
    ///
    /// Fails when SQLite cannot begin or end the snapshot, or with what
    /// `read` fails with.
    pub fn snapshot<T>(
        &self,
        read: impl FnOnce(&Self) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.conn.execute_batch("BEGIN DEFERRED")?;
        // The snapshot is fixed by the first read.
        let begun = self.conn.query_row(HAS_META, [], |r| r.get::<_, bool>(0));
        let out = begun.map_err(StoreError::from).and_then(|_| read(self));
        self.conn.execute_batch("COMMIT")?;

        out
    }

    /// The value `meta` holds under `key`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn meta(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?)
    }

    /// Records a seat.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, its writer or seat already recorded.
    pub fn add_seat(&mut self, seat: &StoredSeat) -> Result<(), StoreError> {
        insert_seat(&self.conn, seat)
    }

    /// Records `seat` and appends `new_events` to `writer`, in one
    /// immediate transaction: both land, or neither does.
    ///
    /// # Errors
    ///
    /// Fails, recording nothing, when SQLite does, when the seat is
    /// already recorded, or when a new event's content has no canonical
    /// encoding.
    pub fn add_seat_and_append(
        &mut self,
        seat: &StoredSeat,
        writer: WriterId,
        new_events: &[NewEvent],
    ) -> Result<Vec<Address>, StoreError> {
        let out = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        insert_seat(&tx, seat)?;
        let out = out.append_chunk(&tx, writer, new_events, true)?;
        tx.commit()?;

        Ok(out)
    }

    /// The seat `writer` belongs to.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn seat(&self, writer: WriterId) -> Result<Option<StoredSeat>, StoreError> {
        Ok(self
            .seat_rows("WHERE writer = ?1", &writer.to_string())?
            .pop())
    }

    /// Every seat, by writer id.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn seats(&self) -> Result<Vec<StoredSeat>, StoreError> {
        self.seat_rows("WHERE ?1 = ?1", "")
    }

    fn seat_rows(&self, filter: &str, arg: &str) -> Result<Vec<StoredSeat>, StoreError> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT writer, seat, room, certificate FROM seats {filter} ORDER BY writer"
        ))?;
        let rows = statement.query_map([arg], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        rows.map(|row| {
            let (writer, seat, room, certificate) = row?;
            let corrupt = |what: &str| StoreError::Corrupt {
                address: None,
                what: format!("seat {what}"),
            };
            Ok(StoredSeat {
                writer: WriterId::parse(&writer).map_err(|_| corrupt("writer"))?,
                seat: SeatId::parse(&seat).map_err(|_| corrupt("seat"))?,
                room: RoomId::parse(&room).map_err(|_| corrupt("room"))?,
                certificate: serde_json::from_str(&certificate)
                    .map_err(|_| corrupt("certificate"))?,
            })
        })
        .collect()
    }

    /// Every writer with an event or a seat, by writer id.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or an id does not parse.
    pub fn writers(&self) -> Result<Vec<WriterId>, StoreError> {
        let mut statement = self
            .conn
            .prepare("SELECT writer FROM events UNION SELECT writer FROM seats ORDER BY 1")?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|row| {
            let text = row?;
            WriterId::parse(&text).map_err(|_| StoreError::Corrupt {
                address: None,
                what: format!("writer id {text:?}"),
            })
        })
        .collect()
    }

    /// Appends `new_events` to `writer`'s log, a hundred to an immediate
    /// transaction, and closes the segment they end in the last: the
    /// core's append. Each transaction reads the writer's newest event,
    /// numbers and chains its events after it, and has its first event
    /// cite every other writer's head that advanced since the writer
    /// last cited it. Content above the payload threshold goes to the
    /// payload store first. Returns the new events' addresses.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, when a new event's content has no canonical
    /// encoding, when a payload cannot be written, when the writer runs
    /// out of seqs, or when it holds a segment of a newer format version.
    /// The transaction the failure falls in is rolled back; those before
    /// it stay.
    pub fn append(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
    ) -> Result<Vec<Address>, StoreError> {
        self.append_closing(writer, new_events, true)
    }

    /// Appends `new_events` to `writer`'s log as [`append`](Self::append)
    /// does, leaving the segment they end in open, for the writer to
    /// close later (REC-19).
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append).
    pub fn append_open(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
    ) -> Result<Vec<Address>, StoreError> {
        self.append_closing(writer, new_events, false)
    }

    fn append_closing(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
        close: bool,
    ) -> Result<Vec<Address>, StoreError> {
        let mut out = Vec::with_capacity(new_events.len());
        let chunks = new_events.chunks(BATCH).count();
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        for (n, chunk) in new_events.chunks(BATCH).enumerate() {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            out.extend(to.append_chunk(&tx, writer, chunk, close && n + 1 == chunks)?);
            tx.commit()?;
        }

        Ok(out)
    }

    /// Refuses the part of `offered`, a history brought in from elsewhere,
    /// that differs from an event this store holds at the same (writer,
    /// seq): the first head seen wins, so the held events stay as they
    /// are and the offered ones are never stored as events. Each fork is
    /// recorded, by writer, seq range and the two digests, and the new
    /// ones returned for the caller to count and audit (PEER-10, REC-21,
    /// I1, I6). A fork recorded already is not new.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, a held writer holds a segment of a newer
    /// format version, or a held row does not parse.
    pub fn refuse_forks(&mut self, offered: &[Link]) -> Result<Vec<Fork>, StoreError> {
        let mut writers: Vec<WriterId> = offered.iter().map(|l| l.header.address.writer).collect();
        writers.sort();
        writers.dedup();
        let mut held = Vec::new();
        for writer in writers {
            held.extend(self.log(writer)?.into_iter().map(|s| s.link));
        }
        let forks = record::fork::find(&held, offered);
        if forks.is_empty() {
            return Ok(forks);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(refusal::SCHEMA)?;
        let new = refusal::record(&tx, &forks)?;
        tx.commit()?;

        Ok(new)
    }

    /// Every fork this store refused, by writer then first seq (PEER-10).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn refused_forks(&self) -> Result<Vec<Fork>, StoreError> {
        refusal::all(&self.conn)
    }
}
