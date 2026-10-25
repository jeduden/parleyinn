# Domain model: second round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the adopted model v2 was applied (commit 5ab372f). They checked
it against itself and against all of `docs/srs`, the invariants and
`features/`, with identical prompts and no shared context. The count
after each finding says how many found it on their own.

The first round's 32 findings are gone. What remains is narrower:
gaps where the model is silent, and a few rules that the v2 split
(room acts against principal acts) left in tension.

## 1. Questions for the stakeholder

| #   | Question                                                                                                                                                                                                                                                                                                                                                                                | Found |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| Q1  | **Pins and widening.** `pin`, `edit` and `unpin` are room acts, and room acts "never widen". But a device seat's pin restores to its principal's agents, and a moderator's unpin empties other principals' restore blocks, which the model calls widening (ADM-04, OWN-11, PIN-01, LANE-26/27).                                                                                         | 3/3   |
| Q2  | **Trusted sources.** PRV-02 trusts interactive `user` text, and LMK-02 and PIN-05 depend on it; trust grants carry posts *and pins* (OWN-29, LANE-27). I2 and the model's trusted sources list neither. Each change rewords I2 (ADR and security review).                                                                                                                               | 3/3   |
| Q3  | **Unclassified acts.** Several acts have no kind or class: title, labels, assignment, visibility, invite and add, a room summary and a summary request, confirming a pin candidate, choosing a branch or a fork, declining a handover, withdrawing a successor, dismissing a directed post, acknowledging counters. The OWN-11 gate fails on any principal act the model does not list. | 3/3   |
| Q4  | **Member.** "A seat is a member" (the verb rule) against "Member: anyone with a seat" (a principal). The SRS uses both.                                                                                                                                                                                                                                                                 | 2/3   |
| Q5  | **Seat keys.** One key per seat, the seat id derived from it, the writer named by it; yet SEC-27 rotates seat keys, REC-24 mints new ones, ADM-06 starts new writers on restore, and PRV-10 puts a "sandbox token key" between device key and seat key.                                                                                                                                 | 3/3   |
| Q6  | **Room surfaces.** The closed set of room-view surfaces leaves out the room's own page (timeline, review, replay tabs, context lens, verify and why panels), the foreign room view, the comparison and the paired phone.                                                                                                                                                                | 3/3   |
| Q7  | **Facilitator and appointment.** The facilitator is a service account, yet SEC-32 appoints a seat; an appointment is "a principal act of the owner or a moderator", but a moderator is a seat, and a run-seat moderator cannot sign principal acts.                                                                                                                                     | 3/3   |
| Q8  | **Names outside the rule.** Config keys `mode`, `transcript_roots`, `home_id`, `payload_threshold_bytes` and the prefixes `inject`, `budget`, `redaction`, `flagging`, `retention`, `quota`; MCP `event_expand`, `record_stats`, `kernel_vars`, `room_show`, `room_get`; CLI `parinn ack`, `parinn policy`. Rename them, or widen the rule?                                             | 3/3   |
| Q9  | **File and persona names.** `05c-principal-and-peer-requirements.md`, `principal-acts.feature`, `persona-platform-engineer` and the "Returning owner" persona use excluded words; the model excepts only LANE and VIEW names.                                                                                                                                                           | 3/3   |
| Q10 | **Erasure request.** The model sends it to peers; SEC-30 sends it from a member to the room's owner.                                                                                                                                                                                                                                                                                    | 2/3   |
| Q11 | **Personal room creation.** "ParleyInn never creates a room", yet every node has a personal room. Who creates it?                                                                                                                                                                                                                                                                       | 1/3   |
| Q12 | **Paired phone.** It has no home, yet §6.3 gives it segments and a device seat joins with it; a writer lives "on one node".                                                                                                                                                                                                                                                             | 1/3   |
| Q13 | **Cross-room post.** LANE-29 records room A's seat's post in room B, while every event belongs to its writer's seat's room.                                                                                                                                                                                                                                                             | 1/3   |

## 2. Fixes that need no decision

| #   | Fix                                                                                                                                                                                                                           | Found |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| F1  | **Device seat** is "a principal's seat for one device", so a service account's node has one too; the routing of runless events and principal acts then holds.                                                                 | 3/3   |
| F2  | **Envelope:** "the only way *recalled* content reaches a model" (restore blocks, endorsements and trust-granted text also reach it).                                                                                          | 3/3   |
| F3  | **"own run"** used in the model (recall scope) and in RCL texts while it is excluded: narrow the exclusion to "own run, as an evidence class".                                                                                | 3/3   |
| F4  | **Bridge component:** "hosts the node's principal names", not "the owner".                                                                                                                                                    | 3/3   |
| F5  | **Integrity status:** RCL-09 and the envelope's `chain` field say "chain status" with four values, VIEW-10 six; use integrity status with §9.7.5's seven.                                                                     | 3/3   |
| F6  | **"run component"** left in ENG-12 and ENG-16's pending scenarios, and "user-run component" in three scenario files: launcher.                                                                                                | 3/3   |
| F7  | **§6.3 register:** its Component column names "browser", "the principal's own tunnel", "any static host" and "child of the launcher"; move those to the Reach column under the closed set of components.                      | 3/3   |
| F8  | **Room status:** "no act sets it directly" (OWN-21's ready and abandoned marks feed the derivation).                                                                                                                          | 2/3   |
| F9  | **`launch`** joins the list of one-verb commands.                                                                                                                                                                             | 2/3   |
| F10 | **"link"** as an act name: the act adds one qualified link; say so where the model says "link" never stands alone. Add "permission request" to the qualified requests.                                                        | 2/3   |
| F11 | **Room merge** gains LANE-31's pick order and LANE-01's first-branch-link rule; **mute** keeps LANE-16's posting exception; **appointment limits** gain SEC-32's "no act on the owner or another moderator".                  | 3/3   |
| F12 | **"viewer"** for a person looking: "the person viewing"; "room pin" → pin; "boundary kind" → span boundary kind; "an agent joining" → a run joining; "judge" (model text) reworded; SECURITY.md "per tenant" → per principal. | 2/3   |
| F13 | **The domain-model agent's own instructions** name "agents" and "pull request", both model concepts (step 5).                                                                                                                 | 1/3   |

## 3. Undefined terms

Found by all three: **add** (a seat's add), **token** and enrolment
token, **sandbox token key**, **authenticator**, **trust policy**,
**administrative act** and "ParleyInn's own acts", **CI carrier**, **forge
bridge**, **notification bridge**, **deployment mode** (`automation`,
`interactive`).

Found by one or two: action class, relaying service account, recall
hint, presence hint, chain and chain head, derived artifact, agent
turn, invite and handover offer, capture, the keymap's fork room,
snooze, redirect, take over and point an agent.

## 4. Invariants that would need new wording

- **I2:** Q2 (interactive `user` text, trust grants covering pins).
- **I1:** "expires by policy" collides with the expire act.
- **I3:** "re-injected" where the model says restore; Q1.
- **I5:** "derived artifact" is undefined.

## 5. Stakeholder decisions, 6 October 2026

1. **Pins (Q1):** adding, editing or unpinning a pin that restores (a
   device-seat or service-account-seat pin, or a stamped version) is a
   widening principal act of that pin's author's principal. Run-seat
   pins, which never restore unstamped, stay room acts. A moderator's
   unpin hides the pin in the room and raises a Needs you item; its
   author's agents keep restoring it until their own principal unpins
   it.
2. **Trusted sources (Q2):** I2 gains this node's `user` turns in
   interactive mode and `harness_meta` (PRV-02), and a trust grant
   covers the trusted principal's posts and pins written from its
   device or service-account seats (OWN-29). Both reword I2 by ADR.
3. **Unclassified acts (Q3):** title, labels and assignment are room
   acts of the owner's seat or a moderator. Changing visibility, an
   invite and an invite link are widening principal acts of the owner.
   An add is a state: the join that placed a seat, accepted under the
   room's admission. A room summary is a room act of the facilitator's
   service-account seat, and a summary request a room act of any
   member. Confirming a pin candidate, choosing a fork and accepting a
   handover or succession are widening; choosing a branch and
   acknowledging counters (`parinn counter ack`) are neutral; declining
   a handover, withdrawing a successor and dismissing a directed post
   are cut.
4. **Member (Q4):** a seat. A principal has a seat in a room;
   people-facing text speaks of the room's principals.
5. **Seat keys (Q5):** a seat has one current key. A rotation is a
   signed event in the seat's writer, signed by the old key and the new
   key; the seat keeps its id and writer. A key minted because the node
   changed (a clone or a restore) starts a new seat and writer, linked
   to the old one. The model adds the token key, a device-certified key
   limited to a token's rooms and expiry, which may stand between a
   device key and a seat key (PRV-10).
6. **Surfaces (Q6):** Fleet, Room page, Catch up, Needs you, Health,
   Setup and Peers. The Room page holds the Timeline, Review and Replay
   tabs (with the context lens); the verify and why panels, the
   comparison and the quarantine list open from it, and a foreign room
   opens in it, marked foreign. The TUI, the paired phone and the
   harness strip are reduced clients.
7. **Appointment (Q7):** the facilitator is a service account whose
   service-account seat is an appointed moderator. An appointment is a
   principal act of the owner, or of a principal whose device seat
   holds the moderator role by assignment, making a run seat or a
   service-account seat a moderator. An appointed moderator never
   appoints.
8. **Names (Q8):** renamed as proposed: `deployment.mode`,
   `transcript.roots`, `home.id`, `payload.threshold_bytes`,
   `restore_block.*`, `pin.max_restore_tokens`,
   `room_summary.max_tokens`, `flag.enabled`, `stat_list`,
   `kernel_var_list`, one `room_get`, and `parinn sandbox check`. The
   model defines deployment mode, flag, redaction, retention policy and
   quota, and "expand" joins the reading verbs.
9. **File names (Q9):** `05c-principal-and-peer-requirements.md`,
   `principal-acts.feature` and `persona-platform-engineer`;
   `persona-returning-owner` stays.
10. **Erasure (Q10):** an erasure request carries a node's purge to its
    peers (PEER-11); a purge request asks the room's owner to purge the
    events one's seats wrote (SEC-30).
11. **Smaller gaps (Q11):** `parinn install` creates the node's personal
    room as the device seat's first act, so ParleyInn never creates a room
    on its own initiative; a paired phone signs with its device key
    while the node it pairs with holds and seals that seat's writer; a
    cross-room post is an event in the sending seat's writer, shown in
    the target room and pulled there by address, enveloped.
