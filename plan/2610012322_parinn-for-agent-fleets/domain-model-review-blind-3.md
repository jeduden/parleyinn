# Domain model: third round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the second-round decisions were applied (commit ac9491d),
against itself, all of `docs/srs`, the invariants and `features/`,
with identical prompts and no shared context. Two claims were checked
and dropped: CLAUDE.md's included invariants and agent catalog are
current.

## 1. Questions for the stakeholder

| #   | Question                                                                                                                                                                                                                           | Found |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| Q1  | **Stamped run-seat pins.** A run seat's edit or unpin, or a moderator's unpin, ends a stamp and so takes a version out of a restore block, which the model calls widening while room acts never widen (LANE-24, LANE-26, LANE-32). | 3/3   |
| Q2  | **Stamped versions and I2.** A stamped version restores, yet the trusted sources and I2 do not list stamps; I3's "pinned constraints" is wider than PIN-10's qualifying pins.                                                      | 3/3   |
| Q3  | **A service account's two seats.** Its device seat and its service-account seat are both "used without a run".                                                                                                                     | 3/3   |
| Q4  | **Rooms without a seat.** Where a principal act on a room the principal has no seat in is recorded (rejecting a foreign room), and whether a room its principal owns without a seat is foreign.                                    | 3/3   |
| Q5  | **Member and seat** are one concept under two names.                                                                                                                                                                               | 3/3   |
| Q6  | **"Import"** means transcripts (origin `imported`) and also bundles and peers (REC-23, SEC-25, VIEW-18).                                                                                                                           | 3/3   |
| Q7  | **Notification bridge** sends off the device, while a notification is same-device only.                                                                                                                                            | 2/3   |
| Q8  | **Trust per event or per agent.** Trust is stored per event, yet stamps and trust grants make it depend on whose agents read it.                                                                                                   | 1/3   |
| Q9  | **Adapter** is undefined, and sits in the core in §4.2 but starts runs "never the core" in OWN-23; a token-key sandbox has no device key for principal acts.                                                                       | 2/3   |
| Q10 | **Outside tools' words.** The frit plan skills use "lane" and "session" in frit's own vocabulary.                                                                                                                                  | 1/3   |

## 2. Fixes that need no decision

- Handover: succession is a second path to ownership.
- Principal acts go to the device seat of the device that signs them
  (OWN-02's wording) in the model, LANE-01 and §4.3.
- Structural event: an act's structural fields only, never the text
  an act carries.
- Role capabilities cover every room act: role request (viewer),
  summary request (any role), room summary (the facilitator's seat).
- Verb rule: "owns" only for rooms (a home belongs to an OS user; the
  harness keeps transcripts); "holds" only for nodes.
- `transcript_roots` → `transcript.roots` in the model.
- "Active pins" → qualifying pins (PIN-10) in INJ, PIN, §8 and §9.4;
  the §9.4 header names trust-granted pins and room ids.
- Derived artifact matches I10's list; quarantine covers I5's list.
- Paired phone: held permission requests.
- `parinn sandbox check` reports recall taint → `parinn recall-taint
  show`; "recall-taint flag" → recall taint.
- `parinn purge` joins the one-verb utilities; `room_summary_get`
  stops writing (the summary request is `room_summary_request`).
- `parinn pin` gains `edit`; "a change is an unpin and an add" goes.
- "hooks" in the core and I4 → hook handlers (I4 by ADR).
- "assignment" in "by assignment" → role assignment.
- Define: certifier, holder order, overlap, fork, marked range,
  harness strip, context lens, counter, stat, landmark index, recall
  hint, room state, run status, freshness mark, queue class, trust
  mark, presence hint, watchdog observation.
- VIEW-20: a delegation grant only when one applies; OWN-01 names
  OWN-26 and OWN-29; ENG-28's agent "verdict" → review outcome;
  qualify bare "grant" and "source"; `allow-session` listed as a
  harness name; "enroll" spelling throughout.

## 3. Invariants that would need new wording

- I2: stamped versions as trusted sources (Q2); "another writer"
  given runs spanning several writers.
- I3: "pins that restore" instead of "pinned constraints".
- I4: "hook handlers".
- I8: "own" and trust-grant exceptions to "untrusted".

## 4. Stakeholder decisions, 6 October 2026

1. **Stamps stick (Q1):** a room act never changes any restore block.
   An edit adds an unstamped version while the stamped version keeps
   restoring to the stamper's agents; an unpin, by the author or a
   moderator, takes the pin off the room's pin list, and the stamped
   version keeps restoring until the stamper unstamps it, a cut
   principal act. Either raises a Needs you item for the stamper.
2. **I2 and I3 (Q2):** I2's trusted sources gain a pin version a
   principal stamped, for that principal's own agents; I3 reads "every
   pin that restores is stored verbatim and restored verbatim after
   every compaction, or named by id and count when the budget omits
   it". Both by ADR.
3. **One seat kind for acting without a run (Q3):** the service-account
   seat goes; every principal acts without a run through a device seat,
   one per device, and the facilitator acts through its device seat.
4. **Rooms without a seat (Q4):** a principal act on a room where the
   principal has no seat goes to its device seat in its personal room,
   naming the room; a foreign room is one its principal neither owns
   nor has a seat in.
5. **Member (Q5):** a seat whose add stands, or a run's personal-room
   seat; a kicked, departed or barred seat stays a seat but is no
   longer a member.
6. **Import (Q6):** "import" means reading a bundle; an unwatched
   transcript is ingested, an ingested run with origin `ingested`;
   peer segments are received. Origins: `witnessed`, `ingested`,
   `bundle`, `peer`.
7. **Notification (Q7):** a signal to a person on the same device or
   through the notification bridge; it carries only the room's petname,
   the queue class and a count, never room text, never reaches a model
   and never accepts answers.
8. **Trust level (Q8):** per event, for one principal's agents, from
   provenance, writer, the deployment mode recorded with the event, and
   that principal's stamps and trust grants as its writer logs carry
   them; every node of that principal holding the same logs derives the
   same level.
9. **Adapter and sandbox keys (Q9):** a harness adapter is the
   harness-specific code for one harness, its transcript and hook part
   in the core and its part that starts, hosts and controls runs in the
   launcher. A node holding only a token key signs no principal acts;
   its runs' pins restore only once a principal stamps them from one of
   its devices.
10. **Outside tools (Q10):** excluded words may appear in files an
    outside tool writes and owns, in that tool's meaning, such as
    frit's plan skills; the domain-model agent skips them.
