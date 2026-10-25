# Domain model: eighteenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the seventeenth-round decisions were applied (commit 954ad59),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                  | Found | Decision                                                                                                                         |
| --- | ------------------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is room status a derived artifact, given I10's "statuses"?                | 3/3   | No: I10's statuses are run statuses, but for their time-relative freshness marks, and integrity statuses; I10 keeps its words.   |
| Q2  | What should the hook handlers' narrow sense of "witnessed" be called?     | 1/3   | "Recorded" by the hook handlers; the origin keeps `witnessed` (I2, ADR-2610063200).                                              |
| Q3  | Does a neutral unpin need the seat's pin capability?                      | 1/3   | No, as PIN-01 says; LANE-26 and the model add the exception.                                                                     |
| Q4  | Should the trusted-only export be a concept, under SEC-26?                | 1/3   | Yes: defined in the model, and an export under SEC-26 and §6.3 row 7.                                                            |
| Q5  | Is `ingested` a freshness mark?                                           | 2/3   | Yes: a freshness mark says how current a run's events are, or that they came by ingest.                                          |
| Q6  | Should the envelope keep one name?                                        | 1/3   | "Envelope", with I2's "untrusted-data envelope" as its one full name; "untrusted envelope" goes.                                 |
| Q7  | Is `parinn import` a principal act?                                       | 1/3   | No: it records no principal act (§9.5).                                                                                          |
| Q8  | Does the review-gate exemption cover ENG-28's `verdict` field?            | 1/3   | Yes: the exemption names the review gate and the requirement naming its fields.                                                  |
| Q9  | Before PRV-10, what signs a principal act?                                | 1/3   | No device key: device keys sign principal acts once PRV-10 ships, as I2 and Principal act say; the model's other entries add it. |
| Q10 | What CLI verb carries confirming a command taken from an untrusted event? | 1/3   | The verb that carries the command; `parinn check witness` becomes `parinn witness-check start`, which shows and confirms it.     |
| Q11 | Should `parinn pin remove` use the act's name?                            | 2/3   | Yes: `parinn list-removal make <room> <pin>`.                                                                                    |

## 2. Fixes that need no decision

- **Model:** of ParleyInn's components, only the launcher starts and
  controls runs in the harness; the facilitator's device seat is the
  appointed moderator; "writer id", "event kind", `harness_meta`, the
  hook and step budgets, the room settings, the pin capability and
  causal order defined; a capture gap is part of a run its capture did
  not record; a device-seat pin is edited, or a version stamped, from a
  device whose device scope allows it; Trusted sources say "once PRV-10
  ships".
- **SRS and scenarios:** "peering" → the peer component or sync (NG4,
  PEER-01, OQ-30, SEC-26, peer.feature); VIEW-08 "a writer a new seat
  key started"; NFR-08 "concurrent appends"; ADR-06 "harness-configuration
  change"; lane.feature "a post" for a message; OWN-21 adds freshness
  marks; LANE-26, §9.5, PEER-06 and scenarios take the device scope;
  VIEW-12 "the room's events, its conversation included"; SEC-01 and
  §4.2 except the core's kernel worker (CMP-05); appendix A D4 "the
  harness's managed settings"; §4.3 "run and integrity statuses".

## 3. Optional, not chased

The fork "kept" wording; the harness adapter inside Component's list;
"room bundle"; "config"; "the view" and bare "policy"; "LLM headlines";
"abandoned run"; "user-stated constraints"; "operator event" without
backticks; `room_get` beside the `room_<act>` names; "review step",
"controlling terminal" and "secret reference"; I4's core list; I8's
"attributed to its seat key"; the pin types' meanings; "home id"; T27's
"steers"; SEC-32's pin reference; §6.3's short component names;
"read-only client"; CI attested's wording; the git carrier's component.

## 4. Invariants

I2 changes, recorded in ADR-2610063200.
