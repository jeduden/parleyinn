# Spike S12 report: two writers, one index file

Spike S12 asks whether two writers' closed segments, merged in either
order, rebuild a byte-identical index, with commitments surviving
purge (§12.1). It bears on REC-17, ADM-08 and PEER-03, and its exit
criterion is plan 2610012322 phase 1's gate. The slice is the
standalone crate beside this report, outside the parinn workspace, so
nothing in it enters parinn's module graph.

Here the index file is the one SQLite file the slice derives, holding
the search index and the room state. FTS5's index tables are the part
of it FTS5 keeps for the search index.

## Answer

Yes, on each node, for an index file rebuilt from nothing. The gate's
test passes:

- On each node, the segments received writer A's first, writer B's
  first, or reordered and duplicated rebuild a byte-identical index
  file. The two nodes' index files differ from each other, because
  trust levels and origin derive per node.
- Every event keeps its writer, address and provenance in the index
  file. The other writer's events read `peer` and untrusted, while the
  same classes of the node's own writer read trusted.
- Seats A and B each link one branch to the room. Either order derives
  one standing branch link: the lower commitment of two concurrent
  branch links, or the first in causal order. Each resolved conflict
  is recorded with both acts (LANE-31).
- After A's principal purges one of A's events, nothing A's node
  stores or serves confirms a guess at the purged content. That holds
  whether the purge arrives before or after a stale copy. B's node
  keeps its copy until its principal applies the erasure request,
  which the slice stands in for with A's tombstone (PEER-11). After
  that, the stored files confirm nothing, and a later stale copy does
  not bring the content back.

Two findings bear on M1:

- Byte-identity holds only for an index file rebuilt from nothing. An
  index file kept per append holds the same rows, but its bytes differ
  even after FTS5's `optimize` and `VACUUM`. ADM-08, PEER-03 and I10
  all speak of it.
- A purge in the index file needs both SQLite's `secure_delete` and
  FTS5's `secure-delete`, or a rebuild. Without them, the purged text
  or its postings stay in the file.

## What the slice builds

- **Writers and events.** A writer's id is the SHA-256 of its seat's
  first key. Its first event is structural and records the node's
  deployment mode. Each event's header holds only the address, event
  kind, provenance, `prev_hash`, the chain heads of other writers it
  had seen, and the commitment (REC-10). The event's hash is SHA-256
  over the header's canonical encoding (§8.3).
- **Commitments.** The commitment is HMAC-SHA256 under a 256-bit
  commitment key read from the OS's random source, one per event,
  over the canonical content (REC-17). Content and commitment key
  travel beside the header, never inside it.
- **Canonical encoding.** `serde_json` without `preserve_order` emits
  RFC 8785 for this field set. Every key is ASCII, where UTF-8 and
  UTF-16 key order agree, and every number is an integer.
- **Segments.** A segment is canonical JSON lines, with a format line
  first; an unknown format version is refused (REC-21).
- **Receiving a segment.** A node checks every hash, every commitment
  whose content it holds, and that each event chains to what it
  already holds. It refuses a second, different event at a seq it
  already holds; without seals this is no fork, and PEER-10 would keep
  both. It keeps an event's content while some copy carried it and no
  tombstone that applies on this node names it. So what a node holds
  depends on the set of segments it received, not on their order.
- **Origin and trust.** The slice reads origin from whether the writer
  is one of the node's own: `witnessed` for its own writers, `peer`
  for any other (RCL-09). Trust follows PRV-02 for the classes the
  slice records, reading the deployment mode recorded with them.
- **The index file.** The search index and the room state are built
  in one new SQLite file, in (writer, seq) order and one transaction,
  then renamed over the old file.
- **Branch links.** The room merge resolves branch links by causal
  order, from the chain heads each header names as seen, then by the
  lower commitment (LANE-31).
- **Purge.** A purge erases content and commitment key and appends a
  tombstone holding only addresses, count, reason and commitments
  (ADM-07). A tombstone in one of the node's own writers applies
  there. Any other tombstone applies only once the node's principal
  applies its erasure request.

The corpus is spike S2's, read where it lives in S2's Rust port:
writer A draws S2's stream 1 and writer B stream 2, both over S2's
vocabulary. A writer closes a segment every 250 events in the test,
and every 500 in the measurement. After each close, each writer
records the other's chain head at its newest closed segment.

## The gate's test

`cargo test` in this crate runs 21 unit tests and the four tests of
[proving_slice.rs](tests/proving_slice.rs), over 4,006 events in 18
segments:

| Test                                                                       | Requirements                    |
| -------------------------------------------------------------------------- | ------------------------------- |
| `either_order_rebuilds_a_byte_identical_index_on_each_node`                | ADM-08, I10                     |
| `every_event_keeps_its_writer_and_the_other_writers_events_read_untrusted` | REC-06, PRV-02, I2, I8          |
| `either_order_derives_one_branch_link_per_branch`                          | LANE-31                         |
| `a_purged_event_leaves_nothing_that_confirms_a_guess`                      | REC-17, ADM-07, SEC-31, PEER-11 |

The trust test also checks RCL-09's origin values. Two peers' index
files differ wherever they hold trust levels and origin, so PEER-03's
byte-identity across peers does not hold for them; see the questions
below.

The purge test first checks that the stored record confirms the guess
before the purge, so the scan can see what it looks for. The guess is
confirmed by any of:

- the text, or its opening 48 bytes;
- the canonical content;
- SHA-256 of the text or of the content, raw or hex;
- any 64-digit hex string stored anywhere that opens a stored
  commitment to the guess.

With the node's tombstone handling switched off, the purge test
fails.

## Measurements

The measurement binary is [measure.rs](src/bin/measure.rs). It ran
on a 4 vCPU, 15 GiB Linux host, not the reference hardware, from a
release build. Throughput, over 200,006 events in 402 segments:

| Measure                                  | Value                   |
| ---------------------------------------- | ----------------------- |
| append, a commitment key read per event  | 38,129 events/s         |
| receive and store, A then B              | 8.58 s, 23,316 events/s |
| full rebuild, A then B                   | 5.79 s, 34,564 events/s |
| full rebuild, B then A                   | 5.47 s, 36,558 events/s |
| index file                               | 219.3 MiB               |
| full rebuild, both orders byte-identical | yes                     |

At about 35,000 events/s, a full rebuild of 10M events would take
about five minutes on this host. That is fine for `parinn rebuild`,
but not on every append.

The index file kept per append, segment by segment in arrival order,
against a full rebuild:

| Row ids                   | Both orders identical | After `optimize` and `VACUUM` | Equal to a rebuild after both | Logical content equal |
| ------------------------- | --------------------- | ----------------------------- | ----------------------------- | --------------------- |
| assigned in arrival order | no                    | no                            | no                            | yes                   |
| taken from the event hash | no                    | no                            | no                            | yes                   |

Both measurements, at 20,006 and 200,006 events, gave the same
answers. Table by table, with row ids taken from the event hash, the
events table, the room state and FTS5's document sizes and
configuration come out identical. Only FTS5's index tables differ.
Their structure record holds the FTS5 segment id `optimize` assigned
and a write counter, and both depend on history.

In-place purges, each on a fresh copy of a 20,006-event rebuilt index
file, over A's first 40 user messages:

| Settings                                 | Time each | Text's opening left in the file | FTS5 data per purge |
| ---------------------------------------- | --------- | ------------------------------- | ------------------- |
| `secure_delete` off                      | 13.6 ms   | 18 of 40                        | +562 bytes          |
| `secure_delete` on                       | 14.7 ms   | 0 of 40                         | +562 bytes          |
| `secure_delete` on, FTS5 `secure-delete` | 21.1 ms   | 0 of 40                         | −200 bytes          |

Without FTS5's `secure-delete`, a delete only appends delete markers.
The purged event's postings stay in the file until an FTS5 merge.

## Findings for M1

1. **Byte-identity needs a decision.** ADM-08 asks `parinn verify` to
   confirm that a rebuild is byte-identical to the state before, and
   that derived artifacts never depend on the order in which logs or
   segments arrived. PEER-03 asks two peers for byte-identical room
   state and index, and I10 says a rebuild reproduces derived
   artifacts exactly. All of these hold for an index file rebuilt from
   nothing. An index file kept per append, which NFR-15's latencies
   need, holds the same rows but not the same bytes. The options are
   to compare a canonical digest per derived artifact (the slice's
   `logical_digest`), or to rebuild FTS5's index tables from nothing
   before comparing bytes. This spike recommends the digest. Then I10's
   "exactly" must mean logical content, an invariant wording change
   that needs an ADR and a security review. The wording is the
   stakeholder's call.
2. **Row ids from the address, not arrival.** With row ids assigned
   in arrival order, the same rows differ by id. Ids derived from the
   event's hash or its address make the events table order-free.
3. **Purge.** A purge must set `PRAGMA secure_delete`, and FTS5's
   `secure-delete` on the search index, or rebuild the index file. S2's
   store runs in WAL mode, where a page's old image stays in the WAL
   until a checkpoint. So a purge must also checkpoint with `TRUNCATE`.
   The slice runs without a journal and did not measure this; ADM-07
   and SEC-31 rest on it.
4. **Trust needs the recorded deployment mode.** PRV-02 trusts a
   `user` event only if the node's deployment mode was `interactive`
   when it was recorded. The slice records the mode once, in each
   writer's first event. An index file kept per append must re-derive
   trust levels when the event recording the mode arrives late.
5. **Applying an erasure request must be recorded.** The slice keeps
   the erasure requests a node's principal applied as node state.
   Under I10 they must be principal acts in the record (PEER-11,
   OWN-11), so a rebuild applies them again.
6. **The canonical encoding's limits.** It is RFC 8785 only while
   keys stay ASCII and numbers stay integers. Otherwise M1 needs a
   full RFC 8785 encoder.

## Questions for the stakeholder

- **PEER-03** asks two peers holding the same segments for
  byte-identical room state and index. Trust levels and origin derive
  per node, and I10 lets the node's own key set differ. Should PEER-03
  say "the same writer logs and key set", or leave trust levels and
  origin out?
- **PEER-03 and §12.1's S12 row** say "index" with no qualifier, while
  the domain model defines a search index and a landmark index.
  "Merged in either order" in §12.1 is a third sense of merge, beside
  the room merge and the segment merge (REC-25).

## Not covered

- seals and seat-key signatures (REC-18), and forks (PEER-10);
- segment merges (REC-25);
- the payload store (REC-09);
- open segments and sealed prefixes;
- ingested events and the recorder;
- quarantine;
- erasure requests as signed events (PEER-11);
- auditing a refused segment and recording it as a structural event
  (REC-21);
- purge receipts and a purged range's transcript positions (SEC-31);
- the structural fields a tombstone keeps when an act, a tombstone or
  an event the key set holds is purged (ADM-07);
- one branch linked in two rooms (LANE-01), and a branch's remote URL;
- missing ranges;
- an exchange between real peers (both nodes run in one process);
- WAL mode;
- Windows: the commitment key is read from `/dev/urandom`.

## Reproduce

```sh
cd plan/2610012322_parinn-for-agent-fleets/slice
cargo test
cargo run --release --bin measure -- 10000 500 40
cargo run --release --bin measure -- 100000 500 0
```
