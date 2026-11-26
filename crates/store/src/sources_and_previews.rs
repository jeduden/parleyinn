//! The store's origins, worktree identities, sources, ingested runs and
//! previews.

use super::*;

impl Store {
    /// Every writer's kept origin, by writer: a derived artifact (PRV-13,
    /// I10) that decides nothing, kept so `parinn verify` can compare it
    /// to a fresh derivation. None while `parinn rebuild` has not written
    /// it.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a writer id does not parse.
    pub fn origins(&self) -> Result<BTreeMap<WriterId, String>, StoreError> {
        origin::all(&self.conn)
    }

    /// Replaces the kept origins with `origins`, in one transaction.
    ///
    /// # Errors
    ///
    /// Fails, changing nothing, when SQLite does.
    pub fn replace_origins(
        &mut self,
        origins: &BTreeMap<WriterId, String>,
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        origin::replace(&tx, origins)?;
        tx.commit()?;

        Ok(())
    }

    /// The identity, as its caller named it, the worktree `worktree`
    /// last resolved to (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn worktree_identity(&self, worktree: &str) -> Result<Option<String>, StoreError> {
        repository::identity(&self.conn, worktree)
    }

    /// Keeps `identity` as what `worktree` resolves to (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn set_worktree_identity(
        &mut self,
        worktree: &str,
        identity: &str,
    ) -> Result<(), StoreError> {
        repository::set_identity(&self.conn, worktree, identity)?;

        Ok(())
    }

    /// What the mark `mark` holds: a node-local note of what this node
    /// already reported or looked at, not derived from the record
    /// (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn worktree_mark(&self, mark: &str) -> Result<Option<String>, StoreError> {
        repository::mark(&self.conn, mark)
    }

    /// Makes the mark `mark` hold `value` (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn set_worktree_mark(&mut self, mark: &str, value: &str) -> Result<(), StoreError> {
        repository::set_mark(&self.conn, mark, value)?;

        Ok(())
    }

    /// Clears the mark `mark`; nothing, when it was never made (LANE-02).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn forget_worktree_mark(&mut self, mark: &str) -> Result<(), StoreError> {
        repository::forget_mark(&self.conn, mark)?;

        Ok(())
    }

    /// Appends the bind `event` to `writer`'s log and keeps `identity` as
    /// what `worktree` resolves to, in one transaction, provided the
    /// worktree still resolves to `expected`: a bind another process
    /// made since the caller looked changes nothing and returns `None`
    /// (LANE-02).
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append); nothing changes then.
    pub fn bind_worktree(
        &mut self,
        writer: WriterId,
        event: &NewEvent,
        worktree: &str,
        expected: Option<&str>,
        identity: &str,
    ) -> Result<Option<Address>, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if repository::identity(&tx, worktree)?.as_deref() != expected {
            return Ok(None);
        }
        let address = to.append_chunk(&tx, writer, std::slice::from_ref(event), true)?;
        repository::set_identity(&tx, worktree, identity)?;
        tx.commit()?;

        Ok(address.into_iter().next())
    }

    /// The identity each worktree resolves to, as derived from the
    /// record (LANE-02, I10).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn worktree_identities(&self) -> Result<BTreeMap<String, String>, StoreError> {
        repository::identities(&self.conn)
    }

    /// Replaces the worktree identities with `identities` in one
    /// transaction, leaving the marks alone (LANE-02, I10).
    ///
    /// # Errors
    ///
    /// Fails, changing nothing, when SQLite does.
    pub fn replace_worktree_identities(
        &mut self,
        identities: &BTreeMap<String, String>,
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        repository::replace(&tx, identities)?;
        tx.commit()?;

        Ok(())
    }

    /// Closes `writer`'s open segment, if it has one (REC-19).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, or when the writer holds a segment of a
    /// newer format version.
    pub fn close_segment(&mut self, writer: WriterId) -> Result<(), StoreError> {
        refuse_newer(&self.conn, writer)?;

        Ok(segment::close(&self.conn, writer)?)
    }

    /// `writer`'s segments, in seq order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn segments(&self, writer: WriterId) -> Result<Vec<Segment>, StoreError> {
        self.select_segments("WHERE writer = ?1 AND ?2 = ?2", &writer.to_string())
    }

    /// Every open segment, by writer: each writer chain that ended
    /// without a closed segment, unless its writer still appends.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn open_segments(&self) -> Result<Vec<Segment>, StoreError> {
        self.select_segments("WHERE closed = 0 AND ?1 = ?1 AND ?2 = ?2", "")
    }

    /// Every segment of a newer format version than this parinn's, by
    /// writer (ADM-05).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn newer_segments(&self) -> Result<Vec<Segment>, StoreError> {
        self.select_segments("WHERE format_version > ?2 AND ?1 = ?1", "")
    }

    fn select_segments(&self, filter: &str, arg: &str) -> Result<Vec<Segment>, StoreError> {
        segment::select(&self.conn, filter, arg)?
            .into_iter()
            .map(|row| {
                row.map_err(|bad| StoreError::Corrupt {
                    address: None,
                    what: bad.0,
                })
            })
            .collect()
    }

    /// The newest generation of the transcript source at `path`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or its row does not parse.
    pub fn source(&self, path: &str) -> Result<Option<Source>, StoreError> {
        let found = source::select(
            &self.conn,
            &format!("{} AND path = ?1", source::NEWEST),
            path,
        );

        found.map(|mut s| s.pop())
    }

    /// Every generation of the transcript source at `path`, oldest
    /// first.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn generations(&self, path: &str) -> Result<Vec<Source>, StoreError> {
        source::select(&self.conn, "WHERE path = ?1 ORDER BY generation", path)
    }

    /// The newest generation of every transcript source, by path.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn sources(&self) -> Result<Vec<Source>, StoreError> {
        source::select(
            &self.conn,
            &format!("{} AND ?1 = ?1 ORDER BY path", source::NEWEST),
            "",
        )
    }

    /// Starts the next generation of the transcript source at `path`,
    /// which lies in the transcript directory `directory`, its first
    /// when none was read (REC-07).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn start_generation(&mut self, path: &str, directory: &str) -> Result<Source, StoreError> {
        source::start(&self.conn, path, directory, self.marker_time)
    }

    /// Moves the position of the transcript source generation `id` to
    /// `position`, appending nothing; never back from where another
    /// ingest moved it.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn advance(&mut self, id: i64, position: &Position) -> Result<(), StoreError> {
        source::advance(&self.conn, id, position, self.marker_time)
    }

    /// Marks the transcript source generation `id` held at its ingest
    /// marker since `since`, or, with none, no longer held (REC-04).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn hold(&mut self, id: i64, since: Option<u64>) -> Result<(), StoreError> {
        source::hold(&self.conn, id, since)
    }

    /// Marks the transcript source generation `id` gone before it was
    /// fully ingested (REC-08).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn lose(&mut self, id: i64) -> Result<(), StoreError> {
        source::lose(&self.conn, id)
    }

    /// Appends `new_events` to `writer`'s log in one transaction, as
    /// [`append`](Self::append) does, keeping each one's place in the
    /// transcript source generation `id` and moving its position to
    /// `position` in the same transaction (REC-03); the writer's segment
    /// closes after each event `closes` marks `true` (REC-19).
    ///
    /// # Errors
    ///
    /// As [`append`](Self::append), and [`StoreError::PositionMoved`]
    /// when another ingest moved the position past the first line;
    /// nothing is appended or moved then.
    pub fn append_lines(
        &mut self,
        writer: WriterId,
        new_events: &[NewEvent],
        places: &[Place],
        closes: &[bool],
        id: i64,
        position: &Position,
    ) -> Result<Vec<Address>, StoreError> {
        let to = Out {
            payloads: &self.payloads,
            threshold: self.threshold,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        // An ingest that read the source before another appended starts
        // behind the position that one moved: its lines are recorded.
        let first = places.first().map_or(position.offset, |p| p.offset);
        if source::offset(&tx, id)? > first {
            return Err(StoreError::PositionMoved { source: id });
        }
        let mut out = Vec::with_capacity(new_events.len());
        let mut start = 0;
        for i in 0..new_events.len() {
            let close = closes.get(i).copied().unwrap_or(false);
            if close || i + 1 == new_events.len() {
                out.extend(to.append_chunk(&tx, writer, &new_events[start..=i], close)?);
                start = i + 1;
            }
        }
        source::place(&tx, id, &out, places)?;
        source::advance(&tx, id, position, self.marker_time)?;
        tx.commit()?;

        Ok(out)
    }

    /// The event that first recorded the line `uuid` names in the
    /// transcript directory `directory`, read from a transcript source
    /// other than `path`; never a repeat reference (REC-04).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the stored address does not parse.
    pub fn line_owner(
        &self,
        directory: &str,
        uuid: &str,
        path: &str,
    ) -> Result<Option<Address>, StoreError> {
        source::owner(&self.conn, directory, uuid, path)
    }

    /// The events of the transcript source generation `id`, in line
    /// order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn placed(&self, id: i64) -> Result<Vec<Placed>, StoreError> {
        source::placed(&self.conn, id)
    }

    /// Keeps `run`, an ingested run (REC-02): for its transcript source
    /// and agent id, it takes over from any kept before, whose writer
    /// gets nothing more (REC-19).
    ///
    /// # Errors
    ///
    /// Fails when SQLite does, or when its writer is already kept.
    pub fn add_run(&mut self, run: &IngestedRun) -> Result<(), StoreError> {
        source::add_run(&self.conn, run)
    }

    /// The ingested run of the transcript source at `path` and the
    /// harness's agent id `agent_id`, none for the main run: the one
    /// kept last.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn ingested_run(
        &self,
        path: &str,
        agent_id: Option<&str>,
    ) -> Result<Option<IngestedRun>, StoreError> {
        let found = source::runs(
            &self.conn,
            "WHERE path = ?1 AND ifnull(agent_id, '') = ?2",
            path,
            agent_id.unwrap_or_default(),
        );

        found.map(|mut r| r.pop())
    }

    /// Every ingested run, in the order they were kept.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn ingested_runs(&self) -> Result<Vec<IngestedRun>, StoreError> {
        source::runs(&self.conn, "WHERE ?1 = ?2", "", "")
    }

    /// The seq of the last event of `writer`'s log; none for an empty
    /// log.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does.
    pub fn head(&self, writer: WriterId) -> Result<Option<Seq>, StoreError> {
        recall::head(&self.conn, writer)
    }

    /// The event at `address`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or its row does not parse.
    pub fn event(&self, address: Address) -> Result<Option<Stored>, StoreError> {
        Ok(self
            .rows(address.writer, "AND seq = ?2", sql(address.seq))?
            .pop())
    }

    /// The preview of the event at `address` (REC-09): the first bytes
    /// of its content's text, at most [`payload::PREVIEW_BYTES`], cut at
    /// a character boundary. It is read from the row alone, so an event
    /// whose content went to the payload store shows the preview its
    /// row keeps; none for an event that is not there.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn preview(&self, address: Address) -> Result<Option<String>, StoreError> {
        let row = self
            .conn
            .query_row(
                PREVIEW_ROW,
                params![address.writer.to_string(), sql(address.seq)],
                |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, Option<String>>(1)?,
                    ))
                },
            )
            .optional()?;
        let Some((content, preview)) = row else {
            return Ok(None);
        };
        let Some(content) = content else {
            return Ok(Some(preview.unwrap_or_default()));
        };
        let value: Value = serde_json::from_str(&content).map_err(|_| StoreError::Corrupt {
            address: Some(address),
            what: "content".to_owned(),
        })?;

        Ok(Some(payload::preview(&value, &content)))
    }

    /// The kind and provenance class of the event at `address`.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or the row does not parse.
    pub fn kind_and_provenance(
        &self,
        address: Address,
    ) -> Result<Option<(EventKind, Provenance)>, StoreError> {
        let row = self
            .conn
            .query_row(
                CLASS_ROW,
                params![address.writer.to_string(), sql(address.seq)],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;
        let corrupt = |what: &str| StoreError::Corrupt {
            address: Some(address),
            what: what.to_owned(),
        };

        row.map(|(kind, provenance)| {
            Ok((
                EventKind::parse(&kind).ok_or_else(|| corrupt("kind"))?,
                Provenance::parse(&provenance).ok_or_else(|| corrupt("provenance"))?,
            ))
        })
        .transpose()
    }

    /// `writer`'s log, in seq order.
    ///
    /// # Errors
    ///
    /// Fails when SQLite does or a row does not parse.
    pub fn log(&self, writer: WriterId) -> Result<Vec<Stored>, StoreError> {
        self.rows(writer, "AND ?2 = ?2", 0)
    }

    fn rows(&self, writer: WriterId, filter: &str, seq: i64) -> Result<Vec<Stored>, StoreError> {
        self.rows_limited(writer, filter, seq, i64::MAX)
    }

    /// At most `limit` of the rows `rows` reads.
    pub(crate) fn rows_limited(
        &self,
        writer: WriterId,
        filter: &str,
        seq: i64,
        limit: i64,
    ) -> Result<Vec<Stored>, StoreError> {
        refuse_newer(&self.conn, writer)?;
        let mut statement = self.conn.prepare(&format!(
            "SELECT seq, kind, provenance, prev_hash, commitment, hash, content, commitment_key,
                    preview, payload, payload_size, transcript_timestamp
             FROM events WHERE writer = ?1 {filter} ORDER BY seq LIMIT ?3"
        ))?;
        let rows = statement.query_map(params![writer.to_string(), seq, limit], |r| {
            Ok(Row {
                seq: r.get(0)?,
                kind: r.get(1)?,
                provenance: r.get(2)?,
                prev_hash: r.get(3)?,
                commitment: r.get(4)?,
                hash: r.get(5)?,
                content: r.get(6)?,
                commitment_key: r.get(7)?,
                preview: r.get(8)?,
                payload: r.get(9)?,
                payload_size: r.get(10)?,
                transcript_timestamp: r.get(11)?,
            })
        })?;
        let rows: Vec<Row> = rows.collect::<Result<_, _>>()?;
        rows.into_iter()
            .map(|row| self.stored(writer, row))
            .collect()
    }

    fn stored(&self, writer: WriterId, mut row: Row) -> Result<Stored, StoreError> {
        let seq = seq_of(row.seq).ok_or_else(|| StoreError::Corrupt {
            address: None,
            what: format!("seq {} of writer {writer}", row.seq),
        })?;
        let address = Address::new(writer, seq);
        let corrupt = |what: &str| StoreError::Corrupt {
            address: Some(address),
            what: what.to_owned(),
        };
        let digest = |text: &str, what: &str| {
            Digest::from_hex(text).map_err(|e| corrupt(&format!("{what}: {e}")))
        };
        let header = EventHeader {
            address,
            kind: EventKind::parse(&row.kind).ok_or_else(|| corrupt("kind"))?,
            provenance: Provenance::parse(&row.provenance).ok_or_else(|| corrupt("provenance"))?,
            prev_hash: digest(&row.prev_hash, "prev_hash")?,
            heads: self.heads(address)?,
            commitment: Commitment::from_hex(&row.commitment)
                .map_err(|e| corrupt(&format!("commitment: {e}")))?,
        };
        let payload = payload_reference(&mut row).map_err(corrupt)?;
        let text = match &payload {
            Some(p) => Some(self.read_payload(address, &p.name)?),
            None => row.content.take(),
        };
        let content = text
            .map(|text| serde_json::from_str(&text).map_err(|e| corrupt(&format!("content: {e}"))))
            .transpose()?;
        let commitment_key = row
            .commitment_key
            .map(|text| {
                CommitmentKey::from_hex(&text).map_err(|e| corrupt(&format!("commitment_key: {e}")))
            })
            .transpose()?;

        Ok(Stored {
            link: Link {
                header,
                hash: digest(&row.hash, "hash")?,
            },
            content,
            commitment_key,
            payload,
            transcript_timestamp: row.transcript_timestamp,
        })
    }

    /// The text of the payload `name` of the event at `address`.
    fn read_payload(&self, address: Address, name: &str) -> Result<String, StoreError> {
        let bytes = self
            .payloads
            .read(name)
            .map_err(|source| StoreError::Payload { address, source })?;

        String::from_utf8(bytes).map_err(|_| StoreError::Payload {
            address,
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "its payload is not UTF-8",
            ),
        })
    }

    /// The heads the event at `address` cites.
    fn heads(&self, address: Address) -> Result<Vec<SeenHead>, StoreError> {
        let mut statement = self.conn.prepare(CITED)?;
        let rows =
            statement.query_map(params![address.writer.to_string(), sql(address.seq)], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
        rows.map(|row| {
            let (writer, seq, hash) = row?;
            let corrupt = |what: &str| StoreError::Corrupt {
                address: Some(address),
                what: format!("cited head {what}"),
            };
            let writer = WriterId::parse(&writer).map_err(|_| corrupt("writer"))?;
            let seq = seq_of(seq).ok_or_else(|| corrupt("seq"))?;
            let head = Digest::from_hex(&hash).map_err(|_| corrupt("hash"))?;
            Ok(SeenHead::new(Address::new(writer, seq), head))
        })
        .collect()
    }
}
