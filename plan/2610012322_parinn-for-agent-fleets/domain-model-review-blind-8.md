# Domain model: eighth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the seventh-round decisions were applied (commit 224e32c),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. Each marked its findings "needs fix" or
"optional"; this note keeps the needs-fix ones, each checked against
the files. All three found the invariant copies identical and no
excluded term outside where the model allows it. Each question takes
its recommended option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                                                 | Found | Decision                                                                                                                                                                                         |
| --- | -------------------------------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Q1  | Do a token-key-only node's device-seat pins restore unstamped on that node?                              | 3/3   | No: only a pin from a device seat certified through a device key restores unstamped; a token-key-only node's pins change by room acts and restore only once stamped (Pin, PIN-10). I2 unchanged. |
| Q2  | Where do a paired phone's principal acts go, when the routing relation says the device first joins?      | 3/3   | To its device seat in the personal room, naming the room.                                                                                                                                        |
| Q3  | Are pins frozen while a named successor has not yet accepted?                                            | 3/3   | Yes: a room whose owner's seats have all left keeps its pins as they were until a handover, a succession or the owner's rejoin, successor named or not (LANE-11 follows the model).              |
| Q4  | Which provenance class do tombstones, hook observations and room acts carry?                             | 3/3   | PRV-01 gains `structural`: hook observations, key rotations and tombstones; worktree checkpoints stay `file`. A room act takes the class of its seat's pins (`operator` or `assistant`).         |
| Q5  | How does an act recorded in a personal room, naming a room, reach that room's other nodes?               | 1/3   | That room shows it by address, as it shows a cross-room post (LANE-29).                                                                                                                          |
| Q6  | Is a confirmed intent or criterion candidate a new pin or a new intent version?                          | 2/3   | A new version of the room's intent pin, authored by the owner's device seat.                                                                                                                     |
| Q7  | Does taint make the landmark index untrusted, though a restore block carries it?                         | 1/3   | Sanitized structural fields are trusted whatever the event's trust level; taint excepts them (INJ-03).                                                                                           |
| Q8  | May a landmark carry 80 characters of the user turn, when INJ-03 allows only pins and structural fields? | 1/3   | No: LMK-02 drops the excerpt (safety first).                                                                                                                                                     |
| Q9  | Is a device seat's join without admission an add?                                                        | 1/3   | Yes (Seat, LANE-25).                                                                                                                                                                             |
| Q10 | Where do a run's events go when its seat's role does not permit them?                                    | 1/3   | To its personal-room seat, so nothing is refused (I1).                                                                                                                                           |
| Q11 | Is a stage-one phone a paired phone?                                                                     | 1/3   | No: before B2, a phone reaches the room view only as a principal surface through the principal's own tunnel. No I4 change.                                                                       |
| Q12 | Does admission apply when an owner whose seats have all left rejoins?                                    | 1/3   | Yes; the owner's own invite satisfies it.                                                                                                                                                        |
| Q13 | Keep "evidence" for what results rest on, rewording I5?                                                  | 1/3   | Yes: I5 reads "without destroying the record" (ADR-2610062300); Fork and PEER-10 say "for forensics".                                                                                            |
| Q14 | May the bound ENG-28 scenario keep "verdict", "finding" and "review gate"?                               | 1/3   | Yes: the repository's own engineering tooling keeps its terms, like an outside tool.                                                                                                             |

## 2. Fixes that need no decision

- **Model:** restore block also at a run's start, resume or clear, with
  room ids, omitted pins' names and the room summary pointer; backup
  holds no seat or device keys; notification carries the petname, else
  the id; an ingested run's writer is sealed by the core; opt-in notice
  covers a waiting delegate report; user turn covers `automation`;
  device scope covers posts and pins; configuration, node identity,
  binding statement, terminal takeover and an envelope marked
  structural defined; "findings" of the facilitator dropped; token key
  bolded once; Event's list "such as"; "Owner" allows an outside
  domain's code owner; historical records named once.
- **SRS:** core starts no program but its kernel worker (§4.2); PEER-02
  also serves blind peers (PEER-12); LANE-11's former owner's pins
  reach the new owner by stamp or trust grant; PRV-10 "its token key";
  LANE-15 "shown as an unknown key"; LANE-21 and VIEW-21 "reads as a
  `claim`"; "peer address" → peer network address; "conversation"
  only for a room's posts; canary described.
- **Names:** `parinn room join` joins with a device seat only, and
  asking for a run's join is `parinn join-request make`; `parinn room
  set` keeps room acts, and the owner's widening settings move to their
  own verbs; `parinn peer invite` gives way to `parinn access-token
  mint`.

## 3. Optional, not chased

"result", "store", "check" and "run" in plain senses; "signed with
that device seat"; "assignment" beside role assignment; layout words
"pill" and "composer"; "user-stated constraints"; I4's core list
leaves out the harness adapter's transcript and hook part.

## 4. Invariants

I5's title changes, recorded in ADR-2610062300.
