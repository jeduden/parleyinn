# Domain model: seventh round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the sixth-round decisions were applied (commit e7a36c8), against
itself, all of `docs/srs`, the invariants and `features/`, reading
every file from disk. All three found the invariant copies identical
and no excluded term outside where the model allows it. Each question
takes its recommended option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                                                                        | Found | Decision                                                                                                                                                                                                                                                                      |
| --- | ------------------------------------------------------------------------------------------------------------------------------- | ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | The harness adapter is listed as a component, yet spans B0 and B1.                                                              | 3/3   | Not a component: ParleyInn's code for one harness, split between the core and the launcher.                                                                                                                                                                                   |
| Q2  | How do a room's seats receive a principal act recorded in the signing device's personal room?                                   | 1/3   | A principal act on a room is recorded on the signing device's seat there; a device of a principal that owns the room or has a member seat in it joins without admission. Only acts on a room the principal has no seat in (rejecting a foreign room) go to the personal room. |
| Q3  | I4 leaves recalled content out of what leaves the machine; I2 and I4 say "Claude" for the agent.                                | 3/3   | Reword by ADR: data leaves as recalled content or as what ParleyInn writes through I2's closed paths; "the agent" calls recall tools; I2's parenthetical excepts what a trust grant covers and says "the model's replies".                                                    |
| Q4  | Can a room have several facilitators?                                                                                           | 2/3   | At most one: the service account whose device seat the owner appoints as the room's facilitator.                                                                                                                                                                              |
| Q5  | Who enables the git carrier?                                                                                                    | 1/3   | Both: the node's principal per remote, and the room's owner per room (PEER-08).                                                                                                                                                                                               |
| Q6  | Is the forensic view a surface?                                                                                                 | 3/3   | No: a reading of quarantined content opened from the Room page's quarantine list.                                                                                                                                                                                             |
| Q7  | Is a subagent that receives a delegated task a delegate?                                                                        | 1/3   | Yes: a delegate is any agent that receives a delegation, a subagent included.                                                                                                                                                                                                 |
| Q8  | Which provenance do expire acts, retention tombstones, requests and token-carried pins get?                                     | 1/3   | `operator` covers principal acts, expire acts and device-seat pins; a retention purge's tombstone is structural; run-seat pins are `assistant` wherever they travel.                                                                                                          |
| Q9  | Do a token-key-only node's device-seat pins restore without a stamp?                                                            | 1/3   | No: every pin such a node writes restores only once stamped.                                                                                                                                                                                                                  |
| Q10 | May a cross-room post's target room recall it when the agent has no seat in the sending room?                                   | 1/3   | Yes, that one event: the target room shows it, and recall extends to the cross-room posts a room shows.                                                                                                                                                                       |
| Q11 | Do the one-meaning and verb rules cover outside senses (network and email addresses, CLI flags, code owners, "held in memory")? | 3/3   | No: the rules bind ParleyInn's concepts; an outside thing keeps its name when its domain qualifies it. Persona names are allowed as names.                                                                                                                                    |
| Q12 | Sample room in VIEW-18?                                                                                                         | 1/3   | Dropped: ParleyInn never creates a room.                                                                                                                                                                                                                                      |

## 2. Fixes that need no decision

- **Model:** membership verbs add "no bar covers it" and the paired
  phone's seat; qualifying pin defined once; trust grant sentence
  fixed; structural event's example is a hook observation; Pin's
  non-restoring types restore never; Event's message is a person's
  input or the model's reply; Principal signs with a device key,
  recorded on a device seat; delegation link "under a delegation
  grant"; "no check can prove" → nothing can prove; "History" →
  historical records; retire a writer also on access-token expiry;
  segment and fork wording; Successor's freeze ends by handover,
  succession or the owner's rejoin; room merge's branch-link
  tie-break; "Flags name concepts" → command-line options; "source of
  truth"/"never its source" reworded.
- **SRS:** PIN-11's scope → seat certificate or token key; `room_create`
  adds bars; LANE-21 "concerns"; "within the core's boundary: no
  program start" → inside the core (no socket, no program start);
  "same local user" → same OS user; 12's "invited contributor" → a
  seat with the contributor role; index.md "Owners" → Stakeholder;
  CLAUDE.md "projection code"; crate names (`payload`, `restore_block`,
  `redaction`, `configuration`, `search`, `administration`);
  "loosening configuration" → a configuration change accepted only by
  a widening principal act; node and agent identity, live co-editing,
  canary described; stage one and tunnel per §1.4.
- **Names:** `pin_candidate_propose`, `parinn pin-candidate confirm`,
  `away_policy.<room>.mode`, `notice_opt_in.<room>.enabled`;
  `room_get` and `room_summary_*` read a room, as the naming rule
  allows; §9.5 gains verbs for the acts VIEW-03 needs (visibility,
  title, labels, assignment, grants, notices, focus set, fork, purge
  request, directed post, CI key, device key, writer, capture).
- **Scenarios:** "the criterion link"; "contains a copy"; "an earlier
  ingest"; "the nightly fuzz job"; "rooms with no activity"; "each
  branch's exposure"; LANE-14 "its principal's message".

## 3. Invariants

I2 and I4 change, recorded in ADR-2610062200.
