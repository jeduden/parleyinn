# Domain model: twenty-sixth round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
twenty-fifth-round decisions were applied (commit d50b209): one across
the whole model (X), one on principals, harness facts, places, seats,
the record and pins (A), and one on acts, trust, git and components (B).
They read every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option.

## 1. Questions and decisions

| #   | Question                                                         | Found | Decision                                                                                                                                                 |
| --- | ---------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | What add and role has the seat `parinn ingest` starts?           | 1/3   | It takes over the run seat's: one add stands for both, so a kick, leave or bar of either ends it for both.                                               |
| Q2  | Is `room_summary_get` a recall call?                             | 1/3   | Yes: a **recall tool** records a recall event and writes no room summary; `event_search` leaves out `summary` events.                                    |
| Q3  | Is launcher-carried text a trusted user turn?                    | 1/3   | No: the launcher records the carried text's commitment as its turn trigger, and ingest marks the matching line untrusted (LANE-14); S9 and S10 check it. |
| Q4  | Where does a tombstone go when its principal has no member seat? | 1/3   | To the recording device's seat in the personal room, naming the room, as for a principal act.                                                            |
| Q5  | Where does a seat key's rotation go?                             | 1/3   | To that seat's own writer (SEC-27).                                                                                                                      |
| Q6  | Who records a principal act taken in the browser room view?      | 2/3   | The room-view component, marked with its surface; how it signs waits for a security review (section 4).                                                  |
| Q7  | Does a device seat's join to a room with no member seat widen?   | 1/3   | Yes: asking for it is a widening principal act.                                                                                                          |
| Q8  | Which agents does a trust-granted post reach?                    | 1/3   | Only the grantor's agents whose run has a seat in the post's room.                                                                                       |
| Q9  | How does ParleyInn know a service account relays others' text?   | 1/3   | Its certifier marks it in the service-account certificate; a grant naming it is refused; any other service account's grant shows a warning.              |
| Q10 | Does an appointed moderator keep its own pins?                   | 1/3   | Yes: it changes no other seat's pin (SEC-32).                                                                                                            |
| Q11 | Is PEER-01's environment setting a principal act?                | 1/3   | No: a structural event, audited, standing in for turning the peer component on.                                                                          |
| Q12 | Is "holds" only a node's verb?                                   | 1/3   | Yes: "keeps" for keys, "contains" for rooms, records and transcripts.                                                                                    |
| Q13 | Add `parinn quarantine-request apply\|refuse`?                   | 1/3   | Yes, as for erasure requests.                                                                                                                            |
| Q14 | Qualify a cloned node?                                           | 1/3   | Yes: "node clone"; bare "clone" stays git's.                                                                                                             |

## 2. Fixes that need no decision

- **Model:** the room merge covers the room's CI keys and its git-carrier
  switch; the seat certificate entry absorbs the hub's naming bullet;
  Device certifies its seats; forge reports bolded; the forge bridge
  reads the forge's checks; the MCP server serves §9.2's tools; the CI
  key is the owner's, in the room.
- **SRS and scenarios:** RCL-01's scenario lists the six P0 recall tools
  among others; NFR-02 and its scenario say hook budget; OQ-11 "recall
  events"; OWN-27 "that person's newer one"; OWN-29 "neither MAY";
  "runner hosts"; ASM-08 "harness session"; the `hook_timeout` counter;
  PIN-07's trusted `user` events.

## 3. Optional, not chased

`parinn receipt` unqualified; `--needs`; `pin.max_restore_model_tokens`;
merging the OWN-18 confirmation with starting a witness check.

## 4. Invariants and reviews

No invariant changes. Two items wait for the stakeholder:

- **I4, commit trailers** (round 23), unchanged.
- **Signing browser principal acts.** Q6 names the room-view component,
  a B1 component, as the recorder of a principal act taken in the
  browser room view. Whether it may use the device key, or how else the
  act is signed, needs the security review SEC-10 and SEC-20 call for.
  The model says so and leaves the signing open.
