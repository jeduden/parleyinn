# Domain model: eleventh round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the tenth-round decisions were applied (commit cf9ec11), against
itself, all of `docs/srs`, the invariants and `features/`, reading
every file from disk. This note keeps the needs-fix findings, each
checked against the files. Each question takes its recommended option,
as the stakeholder asked. The stakeholder raised the model's token
budget from 12,000 to 14,000 so its definitions need not be cut.

## 1. Questions and decisions

| #   | Question                                                                    | Found | Decision                                                                                                                                                   |
| --- | --------------------------------------------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | May restore blocks, notices and templates carry fixed text ParleyInn ships? | 2/3   | Yes: I2, INJ-03, SEC-07 and the restore block name fixed text ParleyInn ships (ADR-2610062600).                                                            |
| Q2  | Are this node's ingested `harness_meta` events and `user` turns trusted?    | 1/3   | No: only those its hook handlers witnessed (I2, PRV-02, VIEW-07, ADR-2610062600).                                                                          |
| Q3  | What names a moderator's or the owner's taking a pin off the list?          | 2/3   | A **list removal**, a room act; **unpin** is the author's or its principal's act that ends the pin. A device-seat pin a list removal took keeps restoring. |
| Q4  | Which key signs a bundle?                                                   | 2/3   | The exporter's device key, which chains to the bundle's principal key; the core never handles a principal key.                                             |
| Q5  | Does a key chain pass through another principal key?                        | 1/3   | No: it runs through device, token or seat certificates only, so a bar or trust grant on a certifier never covers the service account it certified.         |
| Q6  | Where does an event with no run go?                                         | 2/3   | To the recording device's seat in its personal room.                                                                                                       |
| Q7  | Does an invite's role count as a role assignment?                           | 1/3   | Yes: an invite or invite link records one; a seat with none is a viewer, and a run's personal-room seat a contributor.                                     |
| Q8  | Is the notice opt-in configuration or a principal act?                      | 1/3   | A principal act; `notice_opt_in.<room>.enabled` goes.                                                                                                      |
| Q9  | Does the command-line options rule cover every option?                      | 1/3   | Only an option that selects a concept is named for it.                                                                                                     |
| Q10 | Who seals a node's device-seat writer?                                      | 1/3   | The core, as for every writer but a run seat's.                                                                                                            |

## 2. Fixes that need no decision

- **Model:** harness configuration, fixed template, principal-typed
  text, payload store and sync defined; a principal key may certify a
  service account's principal key; the token-key-only node, not its
  seat, signs no principal acts; the pin list holds pins not unpinned;
  "assignment" alone is the branch sense; the domain-model agent's
  instructions may say "the model" for the domain model.
- **SRS and scenarios:** `room_role_request` and a CLI verb (LANE-10);
  "boundary's sandbox" → confined to its boundary (SEC-01, SEC-19,
  security.feature, engineering.feature); "foreign writer", "memory
  store", "record state", "public host", "managed lock", "room-view
  page" reworded; §8.1 and §4.1 put the payload store inside the store;
  pins.feature PIN-03 names its room; security.feature bars principal
  keys; LANE-32 "a principal with a seat in the room"; lane.feature's
  overlap names runs; CLAUDE.md says the domain model's concepts.

## 3. Optional, not chased

Run and room statuses sharing Asking and Blocked; "holds" for clones
and key sets; "writer log"; the excluded-term row's pairing; layout
words such as "pill"; the facilitator and relayed text; Kernel's
"Claude runs code"; I1's "removes under a retention policy"; I4's core
list.

## 4. Invariants

I2 changes, recorded in ADR-2610062600.
