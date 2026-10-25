# Domain model: sixteenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the fifteenth-round decisions were applied (commit 122541f),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                       | Found | Decision                                                                                                             |
| --- | ------------------------------------------------------------------------------ | ----- | -------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is on-prompt injection (INJ-04) a restore block or a new path?                 | 1/3   | A restore block; I2 cites INJ-04 (ADR-2610063100).                                                                   |
| Q2  | Does adding a restoring pin by a principal act need the seat's pin capability? | 2/3   | Yes: every pin act still needs its seat's pin capability.                                                            |
| Q3  | Are a token-key-only node's own events trusted on that node?                   | 3/3   | Yes, as that node's trusted sources; its pins still restore only once stamped.                                       |
| Q4  | Do node-wide utilities record principal acts?                                  | 1/3   | No: only verbs that record an act take one; ingest, rebuild, migrate, canary, install and doctor record none (§9.5). |
| Q5  | Who may publish a room or enroll a blind peer for it?                          | 1/3   | The node's principal, only as the visibility the owner set allows.                                                   |
| Q6  | Which principal keys may a node accept segments under?                         | 1/3   | Keys that chain to a principal key in its key set (PEER-07, SEC-25).                                                 |
| Q7  | What origin do the git carrier's segments take?                                | 1/3   | `peer`; segments are what peers and the git carrier exchange.                                                        |
| Q8  | May the owner edit its pins from another of its devices?                       | 1/3   | Yes: it edits and unpins pins its principal wrote from a device seat.                                                |

## 2. Fixes that need no decision

- **Model:** "witnessed run" and "exporter" defined; managed policy
  "alone overrides the principal's settings"; a node joins before
  recording a principal or expire act; a directed post is "directed";
  the Token rule names token certificate and token-key-only node;
  delegation budget named in both grants, and I9's budgets are the named
  ones; time-relative freshness marks are computed where shown; Setup
  covers harness configuration; Stake lets its author's principal write
  a device-seat stake.
- **SRS and scenarios:** LANE-30 excepts qualifying pins, endorsements
  and trust-granted posts; PEER-13 "every missing range in a writer it
  holds"; LANE-16 and §9.5 "pins its principal wrote from a device seat";
  OQ-33's "room automations" in model words; SEC-17's scenario says
  threat sources; §9.5 "every other verb that records an act" and the
  utilities record none; appendix A "the harness's managed settings";
  REC-18 "every writer MUST be sealed"; ADR-07 "50 concurrent
  connections"; §4.2 `parinn peer-component on|off`; "another machine"
  for a moved home (record.feature, security.feature); "carol's posts in
  a foreign room"; LANE-23 "joining any room"; landmarks.feature
  "sanitized structural fields".

## 3. Optional, not chased

"surface" in two senses; "taint" and "recall taint"; ENG-02 crate
names; the excluded-term table's pairing; `parinn role request` naming;
the CI and git carriers; "secure purge"; numeric rule levels in
scenarios; `room_` prefix on reading tools; "attempted"; "config" in
personas; OWN-11's foreign-room exception; freshness mark `ingested`;
T28's "trusted service account"; RCL-01's tool count.

## 4. Invariants

I2 changes, recorded in ADR-2610063100.
