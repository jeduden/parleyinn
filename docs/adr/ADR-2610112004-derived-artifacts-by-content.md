---
id: ADR-2610112004
title: "Derived artifacts are compared by content, not by storage layout"
status: accepted
summary: >-
  I10 now says a rebuild reproduces derived artifacts' content exactly,
  whatever their storage layout, and ADM-08's verify compares each one's
  canonical digest. Removing stored content must leave no residue in any
  file. Spike S12 found that a search index kept per append never matches
  a rebuild byte for byte. The security reviewer approved it on 13
  October 2026.
---
# ADR-2610112004: Derived artifacts are compared by content

## Context

I10 said a rebuild reproduces every derived artifact "exactly", and
ADM-08 had `parinn verify` confirm that a rebuild is byte-identical to the
state before it. PEER-03 asked two peers holding the same segments for
byte-identical room state and index.

Spike S12 is the gate of
[plan 2610012322](../../plan/2610012322_parinn-for-agent-fleets/plan.md)'s
phase 1. It built two writers' record in SQLite with FTS5, as ADR-07
and [ADR-2610110553](ADR-2610110553-sqlite-rusqlite.md) propose. From
it, it derived the search index and room state in one file, the spike's
index file. Its [report][s12] found:

- An index file rebuilt from nothing is byte-identical whatever order
  the segments arrived in.
- A search index kept per append, which NFR-15's latencies need, holds
  the same rows as a rebuild but never the same bytes. That stays true
  after FTS5's `optimize` and `VACUUM`, and with row ids taken from the
  event hash. FTS5's structure record keeps an FTS5 segment id and a
  write counter from its history.
- A full rebuild runs at about 35,000 events/s on a 4 vCPU host, so it
  cannot run on every append.
- Two peers' index files differ in each event's trust level and origin,
  which each node keeps for itself (PRV-02, RCL-09). PEER-03's
  byte-identity across peers could not hold for them.
- An in-place purge left the purged text in the file for 18 of 40
  events without SQLite's `secure_delete`. It left the FTS5 postings in
  every case without FTS5's own `secure-delete`.

The stakeholder chose, on 11 October 2026, to compare derived artifacts
by content, and to narrow PEER-03 to what is the same on every node.

[s12]: https://github.com/jeduden/parleyinn/pull/35
[keri]: https://trustoverip.github.io/kswg-keri-specification/
[ik]: ../../research/notes/identity-keys/identity-keys.md

## Decision

I10's list names the search index where it said "indexes" and adds origins. Its
last sentence becomes: "Rebuilding reproduces their content exactly, whatever
their storage layout." The rest of I10 is unchanged. Derived artifacts stay a
deterministic function of the writer logs a node holds and its own key set.
Arrival order still plays no part.

With it, SRS 3.0-draft changes these requirements:

- **§8.3** defines a derived artifact's canonical digest. It is SHA-256
  over the canonical encoding of every value any read of the artifact
  can return, row by row in key order. Its schema and every stored
  setting that decides what a read returns, such as the search index's
  tokenizer and ranking configuration, are part of it.
- **ADM-08.** `parinn verify` re-derives every derived artifact and
  confirms it is identical in content, by its canonical digest. It also
  fails on the store's own integrity check, on a search index that
  disagrees with the event text it indexes, on a schema object ParleyInn
  does not ship, and while a setting the no-residue rule needs is off.
  No canonical digest, and no digest of any part of an artifact, is
  written to the audit log, a log, a receipt, a backup or an export, or
  leaves the node. One kept between rebuild and verify is erased when
  verify completes and by any purge it covers, and verify then says so.
  Every read that builds output orders it by values, never by storage
  order.
- **SEC-31.** However stored content is removed, by purge, retention,
  an applied erasure request or a backup restore that reapplies a
  purge, no file ParleyInn or its storage engine writes may afterwards
  contain the removed events' content, their commitment keys or a
  search index entry naming their addresses. No audit entry or log line
  carries event content. A purge that cannot reach that state in time
  does not report the range erased: it audits, counts and names each
  residue, stays pending and is retried outside every hook budget, and
  `parinn doctor`, the Needs you queue and Catch up show it until a
  further purge receipt says it is gone. ADM-07's vacuum must reach
  that state.
- **§8.1.** Every temporary file ParleyInn or its storage engine writes stays
  in memory or in the home, and, with encryption at rest on, in memory
  or encrypted.
- **ENG-08.** Its property test compares canonical digests with a
  rebuild from nothing after any sequence of appends and lifecycle
  events.
- **PRV-13, new.** The first time a node accepts a sealed range by a way,
  from a peer, a paired phone, a bundle or a backup restore, it records
  an arrival event in its device seat in its personal room: writer id,
  seq range, a digest of the chained headers, never of content, and how
  it arrived, in the same transaction as the range. An event's origin
  derives from the record and the key set alone, in this order:
  `bundle` or `peer` from this node's arrival events, `ingested` for an
  ingest seat's writer, `witnessed`, else `peer`. Only arrival events,
  ingest seats and writers whose seats a key of this node's own
  certified count: a key this node's key events name and no node
  identity change event it recorded replaced. So another node's arrival
  events, a bundle's claimed ingest seats and a clone's carried-over
  writers decide nothing here. A range holding only another node's
  arrival events causes none, so two nodes of one principal stop
  exchanging them. No stored origin value, no origin another node
  derived and no quarantine decides it. PRV-13 passes an I2 security
  review before it ships (ENG-29).
- **SEC-10 and REC-24** record a key event for every device key or token
  key a node starts using, carrying the key's fingerprint, its kind and
  the key it replaces. A device key lost and minted anew keeps the
  earlier writers `witnessed`; a node identity change does not.
- **PRV-02** trusts this node's `operator` and structural events only in
  writers of origin `witnessed`.
- **PRV-11.** A seat certificate also names whether its seat is an
  ingest seat, which PRV-13 reads for origin `ingested`. PRV-11's I2
  security review is part of this record's.
- **REC-17, ADM-07 and §8.2.** A tombstone no longer keeps a purged
  event's origin; each node derives it from the writer's seat
  certificate, the arrival events, the key events and the node identity
  change events, none of which a purge covers.
- **PEER-03.** Two peers holding the same segments derive room state and
  a search index identical in content, by a canonical digest over every
  value but each event's trust level and origin and what derives from
  them. Each node derives those for itself and never takes them from
  another node.
- **LMK-05** gets back its bound on the landmark index, which a
  rewording dropped without a decision, now stated for `k` ≥ 3.

The domain model gains the canonical digest, the arrival event and the
key event. A derived artifact is its content, never its storage layout.
Two copies are the same when their canonical digest matches.

[invariants.md](../srs/invariants.md) holds the text approved here. Its
SHA-256 is:

```text
4204e0b5e13b9e59467c5ee7963d53fd06fdca3a0fcf18a64cd3a0f0fff81fd7
```

That text also holds the changes to I2 and I3 that
[ADR-2610112209](ADR-2610112209-invariants-after-s1-s3.md) proposes.
The two are approved in a fixed order: ADR-2610112209 first, on its
own text, then this record. This record approves only I10, and its hash
names the text in force once both are accepted. Until ADR-2610112209 is
accepted, this record cannot be.

The stakeholder, `@jeduden`, is to approve this record as security
reviewer (ENG-29) by a comment on the pull request that adds it. The
comment names the record's id, the hash above and the pull request's
head commit. A squash or merge that lands the same `invariants.md`
carries the approval, since the hash, not the commit, binds it. The
approval covers I10 and the requirements listed above as worded there.
Weakening any of them reopens this record under ENG-29. On that approval
the record becomes accepted, and its hash replaces ADR-2610112209's as
the approved text, which in turn replaced
[ADR-2610091555](ADR-2610091555-initial-invariants.md)'s.

The stakeholder approved this record as security reviewer on 13 October
2026 by a [comment on jeduden/parleyinn#39][approval]. The comment
listed ADR-2610112209 first and then named the record's id. It
came at that pull request's head commit `de59777`, where invariants.md
has the hash above, and on a later pull request than the one that adds
this record. It names neither the hash nor the commit, so this record
binds the approval to both from the pull request's state at that time.

Approval: [given][approval].

[approval]: https://github.com/jeduden/parleyinn/pull/39#issuecomment-6149850310

### Security analysis (author)

This is the author's analysis for the security reviewer, not the review
itself. Each row names the scenario that must prove it, all still
`@pending`.

A byte comparison never held for a store kept per append. Part of what
it promised was never real. Here is what a canonical digest alone would
miss, and what answers each:

| A byte comparison would catch                                                   | What answers it now                                                                   |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| purged text or postings left in free pages or FTS5's index tables               | SEC-31's no-residue rule, checked on the decrypted raw files (ADM-07, SEC-31)         |
| residue from retention, applied erasure requests or a backup restore            | SEC-31 covers every way stored content is removed                                     |
| a plaintext copy in a temporary file, such as `VACUUM`'s                        | §8.1 keeps every temporary file in memory or in the home, encrypted when at rest is   |
| purged content copied into the audit log or a log                               | SEC-31: no audit entry or log line carries event content                              |
| a trigger, view or extra index, or a changed tokenizer or ranking configuration | the schema and settings are part of the digest; verify refuses unknown schema objects |
| a search index that disagrees with the event text it indexes                    | ADM-08's integrity checks of the store and the search index                           |
| output whose order follows storage                                              | ADM-08: every read orders its output by values                                        |
| a store with `secure_delete` turned off                                         | ADM-08: verify fails while a setting the no-residue rule needs is off                 |
| a stored origin changed from `peer` to `witnessed`, making the event trusted    | PRV-13 derives origin from the record; verify re-derives it                           |
| a node's own backup restored and read as `witnessed`                            | PRV-13 reads arrival events first                                                     |
| another node's arrival event relabelling this node's writers                    | PRV-13 counts only arrival events in seats a key of this node's own certified         |
| a purge of the events origin derives from, turning `peer` back into `witnessed` | ADM-07: no purge scope covers arrival, node identity change or key events             |
| a clone that keeps its source's writers `witnessed`                             | PRV-13, REC-24: keys a node identity change replaced are no longer this node's own    |

The digest is itself a risk: kept past a purge, it would let anyone
holding the other content confirm a guess at the purged content. ADM-08
keeps it on the node, out of every log, receipt, backup and export,
erases a kept one when verify completes and on purge, and allows no
digest of part of an artifact.

The ADM-07 scenario reads the store decrypted, with encryption at rest
on and off. It looks for a canary token only the purged events carried,
their commitment keys and search index entries naming them, never for
single shared words. It runs while another process holds a read
transaction, and the purge must not report the range erased before the
residue is gone.

What stays as strong as before:

- Every derived artifact is still a deterministic function of the
  writer logs and the key set, independent of arrival order (I10).
- Verify still fails on any difference in content, trust levels and
  quarantine set included.
- Events, commitments and chain heads are still compared exactly
  (REC-10, REC-18, REC-25).

PEER-03's narrowing adds no path from untrusted content to the model.
Each node derives trust levels and origin for itself, and never takes
either from another node.

Before this change, origin was a stored per-event value outside the
hash chain. Trust levels derive from it, so changing a peer event's
origin to `witnessed` would have made it trusted (I2), and nothing would
have noticed. PRV-13 makes origin derive from the record and the key
set, and verify re-derives it. A stored origin that M1 keeps for speed
can still be edited between rebuild and verify; that is residual risk
R3 (§6.1), caught by verify.

What stays open (OQ-52): a key event is signed only by the keys it
names, so a process that can append to the store can record one naming
its own key and make its writers read `witnessed`. That is residual risk
R3, as for any record before seals ship (REC-18). A key event log whose
keys commit to an offsite recovery key closes it; it is its own
decision, in a follow-up record, before M1 ships SEC-10's key events.

## Alternatives

- **Keep byte identity and compare only rebuilt snapshots.** Verify
  would then compare one rebuild with another, which proves little, and
  the store would need a second index lifecycle.
- **Keep byte identity and rebuild instead of indexing per append.** It
  breaks NFR-15: a full rebuild of 10M events takes about five minutes.
- **Keep PEER-03 byte-identical but require the same key set.** Each
  peer holds its own keys, so two peers' key sets seldom match, and the
  requirement would seldom apply.
- **Anchor key events here, with a key event log and recovery keys, as
  KERI does** ([KERI][keri], [identity-keys research][ik]). Reviews of
  that design raised enough questions (headless nodes, lost keys,
  duplicity) that it is decided on its own.

## Consequences

- An invariant changed, so the SRS moves to a new major version,
  3.0-draft, and the release that carries it is a new major version.
- M1 keeps its search index per append and implements the canonical
  digest as `canonical_digest`, replacing the spike's `logical_digest`.
- M1's store sets `PRAGMA secure_delete = ON`, FTS5's `secure-delete`
  and `PRAGMA temp_store = MEMORY`. Its purge checkpoints the
  write-ahead log with `TRUNCATE`, or rebuilds the search index.
- M1's schema may keep trust levels and origin apart from the parts
  every node shares, so PEER-03's shared parts compare directly.
- M1 derives origin from the record and writes key events and node
  identity change events. M3, M8 and M9, which bring backup restore,
  peers and paired phones, and bundles, write the arrival events they
  need.
