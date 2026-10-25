---
n: 14
title: "Quarantine and parinn purge --range"
status: "🔲"
result: false
---
# Phase 14: quarantine and purge

Requirements. SEC-12, RCL-06, REC-17, SEC-22's retention row and
ADM-07 in part (ruling 4), on what phases 8 to 13 record. RCL-08's
clause on a purged or quarantined address closes. ENG-06: the purge
is crash consistent. I1, I5, I6: the persona-security-officer reviews
the purge.

Purge, `parinn purge --range A:from-to [--reason R]`, a widening
principal act at a terminal:

- One transaction appends the act, a tombstone per range (a structural
  event of the device seat: addresses, count, reason, commitments), and
  erases each event's content, commitment key, preview, payload
  reference and index postings. Payload files go after the commit, from
  a journal the next opener finishes. SQLite's and FTS5's secure delete
  are on, the WAL is checkpointed, both files vacuumed.
- A range is refused, audited and counted, naming each cause, when it
  holds an event the key set or room state derives from, or a writer
  with a pending ingest marker. The audit entry names the range, the
  reason and the act, never content.
- `parinn verify` accepts missing content only under a tombstone that
  holds its commitment, and rejects a tombstoned event that keeps a key.

Quarantine, `parinn quarantine add|release|list`: an `operator`
principal act holding a selector (room, run, author, writer, range,
provenance, flag, since) and a reason. `add` is a cut act, `release`
widening. The quarantine set derives from the acts alone (I10) and
matches events recorded later too. `event_get`, `event_expand` and
`event_search` leave a quarantined event out; an address names a
`quarantined` marker, a purged one a `purged` tombstone, in
`tombstones`, which phase 13 left empty. No origin changes.

Retention, a narrowing of REC-15: a time mark, a structural event of
the device seat, is recorded by `parinn ingest` while a retention
policy is set; an event ages from the first mark after it in causal
order. Ingest then purges what a policy's window has passed, reason
`retention`, audited and counted.

Where scenarios go live: `@SEC-12`, `@RCL-06`, `@REC-17`, `@SEC-22` and
`@RCL-08` in worlds the steps build, through the built core. A row
that cannot pass as written stays `@pending` and is raised; SEC-12's
`--flag` row needs PRV-07 (M5).

Gate: `@SEC-12`, `@RCL-06`, `@REC-17` and `@SEC-22` pass or are
raised; a purged event leaves nothing confirming a guess.
