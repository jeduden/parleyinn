# Domain model: fifth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the fourth-round decisions were applied (commit c1054bb),
against itself, all of `docs/srs`, the invariants and `features/`,
with identical prompts and no shared context. All three found no
excluded term outside where the model allows it. Three claims were
checked and dropped: I2, I4, I7 and I8 and every included copy
already carry the fourth-round wording.

## 1. Questions and decisions

The stakeholder asked for the recommended option on every question
unless none exists or it is hard to reverse. Every question below has
one, and each is spec text, so each takes it.

| #   | Question                                                                                                                                                  | Found | Decision                                                                                                                                                                                                                                                                             |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Q1  | Does a principal's own device-seat post reach its agents other than by recall or endorsement, when OWN-29 pushes trust-granted posts through the harness? | 3/3   | No: own posts reach its agents only by recall or endorsement (OWN-08); OWN-29's push stays for trust-granted posts. No new path, so no invariant changes.                                                                                                                            |
| Q2  | Is revoking a trust grant cut, when the rule makes any act that removes a pin from a restore block widening?                                              | 2/3   | Cut: like an unstamp, it withdraws only the actor's own trust; the rule's exception names both.                                                                                                                                                                                      |
| Q3  | What class is ending a delegation grant or an acceptance grant, or turning an away policy off?                                                            | 3/3   | Cut.                                                                                                                                                                                                                                                                                 |
| Q4  | Is a token-key-only node a device, and what does a node's device key sign?                                                                                | 3/3   | A device is a node or a paired phone; a token-key-only node's token key certifies its device seat. A device key signs principal acts and expire acts; seat keys sign room acts and seal writers.                                                                                     |
| Q5  | What kind of act is a purge a retention policy makes?                                                                                                     | 1/3   | Not an act: the node records it as a purge naming the policy, whose setting was the act. Purge is "the only way content is destroyed".                                                                                                                                               |
| Q6  | Is an active pin a qualifying pin?                                                                                                                        | 2/3   | No: an active pin is one on its room's pin list (not unpinned, not a candidate); qualifying pins are the active pins that restore to one agent under PIN-10.                                                                                                                         |
| Q7  | Which keys are segments encrypted to, when run seat keys never persist (PEER-08, PEER-12, §6.3)?                                                          | 1/3   | The device keys of the principals with a member seat in the room (a token-key-only node's token key), never seat keys.                                                                                                                                                               |
| Q8  | Are live drafts, typing and the appointment rate room state?                                                                                              | 2/3   | Yes: room settings the owner's principal act sets, merged under LANE-31.                                                                                                                                                                                                             |
| Q9  | "Holder order" breaks the rule that "holds" is said of nodes, and lists the facilitator beside moderators.                                                | 3/3   | **Pick order**: the facilitator's seat, then any other moderator, then the owner.                                                                                                                                                                                                    |
| Q10 | With what key does managed policy certify a service account, and how does ParleyInn tell a person from a service account?                                 | 3/3   | Managed policy lists certified service-account principal keys in its root-owned configuration, no signature; a principal key that a person, a service account or managed policy certified is a service account's; an uncertified one counts as a person's, which no check can prove. |
| Q11 | How does a paired phone get a device seat in a room, when it may only read and answer held permission requests?                                           | 2/3   | It never joins: its acts go to its device seat in the personal room, naming the room (routing by the signing device).                                                                                                                                                                |
| Q12 | Does a stamped version of a type that does not restore (`fact`, `verdict`) restore?                                                                       | 1/3   | No: only versions of a type that restores.                                                                                                                                                                                                                                           |
| Q13 | May the owner write room summaries, given "every room capability"?                                                                                        | 1/3   | No: every room capability but writing a room summary.                                                                                                                                                                                                                                |
| Q14 | Is a subagent's parent link also a delegation link?                                                                                                       | 1/3   | No: a delegation link runs from the delegating run to the delegate's run, for a delegation under a grant; a subagent has a parent link.                                                                                                                                              |
| Q15 | Does a run's seat in a room it created route events like a joined one?                                                                                    | 1/3   | Yes: "joins or creates" everywhere (LANE-01, LANE-23, PIN-10).                                                                                                                                                                                                                       |

## 2. Fixes that need no decision

- **Routing:** a principal act goes to the personal room when the
  signing device has no seat in the room, not when its principal has
  none (model, LANE-01, OWN-02, §4.3, §9.5).
- **`operator`:** events a principal writes through a principal
  surface, not "this node's" principal; PRV-10 also trusts a post or
  pin a trust grant covers.
- **Model wording:** "widened recall scope" → extended; "source" in
  Sandbox state → from outside the sandbox; "work" in other senses
  (a room's work → what its runs did); "a hook recorded" → the hook
  handlers; "owner's principal" → every seat of the owner; "the
  agent's own act" → the agent's own tool call; "plus one run seat" →
  further; define hook observation, tombstone, message, finding,
  tree, attestation, leave, retire a writer, repository identity,
  the room's principals, harness configuration, commit author.
- **Room acts** are checked against the seat's role and the room
  state (LANE-16's "owner's configuration").
- **Search:** `parinn event search` covers the principal's rooms and a
  foreign room named in the call (VIEW-09).
- **Verdict pin:** LANE-26's author is the seat with the pin
  capability, except a verdict, recorded by its principal act.
- **Criterion confirmation** makes a new intent version (lane.feature).
- **REC-23:** imported events form a foreign room unless the principal
  owns or has a seat in it. **VIEW-18:** "with no run recorded".
- **Delegated task:** the harness delivers a subagent's; others come in
  OWN-24's template. **OWN-09's scenario:** a post no trust grant
  covers.
- **Evidence class** belongs to results, not criterion links (LANE-21,
  VIEW-21, §9.2).
- **Names:** `room_join` records a join request; `parinn appointment
  make|revoke`; `parinn admission set`; `parinn peer` enrolls; the peer
  component in §6.3 and CON-02; `*.max_model_tokens` keys;
  "structured query terms" for SEC-04's operators; "authored no
  commit" for commit authors.
- **Scenarios:** bare "request"; "user prompt" → user turn; "recording
  status" → capture status; "shows its author" → its principal;
  "every key bob's principal key certified" → chains to; "widening
  event" → widening principal act; REC-22's "`ingested` mark" → origin.
- **§9.7.2:** room status "Running" and the pull-request states "Draft"
  and "Landing" get distinct names.

## 3. Undefined terms

Surface, phone and login credentials (OWN-16, SEC-20), connection key
(§6.3), readable report (SEC-26), pin scope (PIN-11), "deployed ParleyInn
node's URL" (LANE-28), "laptop key" and "phone key" (provenance
scenarios), "Sandbox events" (§9.7.6).

## 4. Invariants

No invariant changes. I10's "active pins" is now defined in the model,
and I2's later sentence already names what a trust grant carries.
