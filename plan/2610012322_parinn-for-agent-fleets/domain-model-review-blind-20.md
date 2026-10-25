# Domain model: twentieth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the nineteenth-round decisions were applied (commit 0e7e931),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked, except the two left open in section
4, which change an invariant's wording and wait for the stakeholder.

## 1. Questions and decisions

| #   | Question                                                                                    | Found | Decision                                                                                                                                                   |
| --- | ------------------------------------------------------------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Does a leave, kick or bar drop a room's pins from a run's restore block?                    | 1/3   | No: the run keeps the qualifying pins of every room it had a seat in during the run, and LANE-30 keeps naming the room.                                    |
| Q2  | After a handover, whose agents does the intent restore to?                                  | 1/3   | The intent restores as its newest version's author wrote it, to that author's principal's agents.                                                          |
| Q3  | Before PRV-10 ships, what tells a principal act from a room act?                            | 2/3   | OWN-02's surface mark on an `operator` event its device seat's seal covers; no expiry can be set, and a node's own device-seat pin restores to its agents. |
| Q4  | What tells the trust policy who recorded an event?                                          | 1/3   | Each event records its **recorder**, an input to the trust policy beside the node's key set.                                                               |
| Q5  | Which origin do events `parinn ingest` appends past an ingest marker take?                  | 1/3   | `ingested`: whatever `parinn ingest` reads.                                                                                                                |
| Q6  | Does a room that names no branch count as Landed?                                           | 1/3   | No: Landed needs at least one branch, all landed; such a room stays open.                                                                                  |
| Q7  | Is "lock" a concept?                                                                        | 1/3   | No: managed policy's locked rows keep their SRS wording; a stake or an assignment makes ParleyInn refuse no other seat's act.                              |
| Q8  | Which class do declining a join request, revoking a CI key and stopping a publication take? | 1/3   | Cut. Revoking a device or an authenticator stays widening, since it can strand acts and pins.                                                              |
| Q9  | What happens to a held request when no away policy is on?                                   | 1/3   | It stays held (OWN-06).                                                                                                                                    |

## 2. Fixes that need no decision

- **Model:** the harness's facts change only in harness configuration;
  ingest defined once; model reply, export, succession, home id and
  settings key defined; the restore block is built from qualifying pins,
  sanitized structural fields and fixed text, not a fixed template; the
  owner's room acts edit only pins that do not restore; a paired phone's
  personal-room seat sits in its node's personal room; a foreign room
  keeps its principal's own signed acts, pins and stamps; a pin candidate
  has no pin author; the freshness mark covers `unrecorded` and
  `stuck?`; ingest splits a transcript by run.
- **SRS and scenarios:** LANE-01 "any other event that belongs to no
  run"; lane.feature's seat column; LANE-26 "pin, edit or unpin act";
  §9.5 narrows "room acts never change any restore block" to pin, edit,
  unpin and list removal acts; `parinn room show --address`, `parinn
  correction send --worktree-checkpoint`; LANE-25 expire acts once
  PRV-10 ships; PIN-10's own-device-seat rule before PRV-10; ADM-04 "a
  repository configuration key"; §9.7.2 Landed; RCL-09's `ingested`.

## 3. Optional, not chased

"ingestion"; "landmark block"; "carrier" in two senses; `parinn notice`,
`room_get` and `room_create` names; the "lane" allowance; §6.3 short
component names; "layer" in 01b and NG2; Work beside the routing
relation; LANE-23's parent link; "resume" as a run control and a
trigger; "unsandboxed agent"; `parinn successor withdraw` and the
dismiss verbs; "secure purge"; "granted roles"; OQ-33's wording;
"config"; "question" and "message"; "harness" for a running instance;
VIEW-16's "cost"; SEC-32's "claims"; T27's "steers"; ENG-02 crate
names; I8's "seat key"; §3's "Kernel (§5.8)".

## 4. Invariants

- **I2, "`user` turns".** A turn runs to the model reply that ends it,
  so "trusted `user` turns" read literally also trusts the reply. The
  stakeholder approved "the `user` events they recorded", recorded in
  ADR-2610063300; the model's Turn and Trusted sources follow.
- **I4, terminal takeover.** The launcher hosts the terminal a person
  types in, yet I4 lets it carry into the harness's input only text the
  core built. Of four options, the recommended one applies: takeover is
  passthrough, not carrying. The launcher relays the person's keystrokes
  unchanged, only from its own terminal and never over its listener, so
  the room view's `t` key cannot reach an agent (model, OWN-19, §9.7.7,
  OWN-19's scenario). I4 keeps its words.
