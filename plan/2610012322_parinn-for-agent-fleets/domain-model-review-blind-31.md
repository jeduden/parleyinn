# Domain model: thirty-first round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
thirtieth-round decisions were applied (commit fc264fd), in the review
workflow: two skeptics, on text accuracy and on whether the defect is
real, checked every needs-fix finding. All 13 survived. Each question
takes its recommended option, but the two in section 4 and Q3, where
round 21's ruling stands.

## 1. Questions and decisions

| #   | Question                                                                    | Found | Decision                                                                                                                                          |
| --- | --------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Which room does `room_pin` without a room use, and whence does `room_post`? | 1/3   | `room_pin` naming no room pins in the run's personal room (PIN-10); `room_post` posts from the working room, its `room` the target.               |
| Q2  | Does "no tool takes or returns a key" hold beside `room_bar`'s key?         | 1/3   | It means a private key: "No tool takes or returns a private key".                                                                                 |
| Q3  | Does recall return a foreign room's own pins as trusted?                    | 2/3   | No, as round 21 ruled: recall marks every item from a foreign room untrusted; those pins stay trusted for restore (RCL-10, PRV-02).               |
| Q4  | Does a succession stop the intent restoring as a handover does?             | 1/3   | Yes: after a change of ownership, a handover or a succession, it restores only to its stampers' agents until the new owner acts.                  |
| Q5  | What is a pin candidate ParleyInn detects?                                  | 1/3   | A derived artifact over its creating `user` event, its text held by address, with no provenance of its own (I10).                                 |
| Q6  | How many seats does `parinn ingest` start beside one run seat?              | 1/3   | One, which every later `parinn ingest` extends (Run seat, REC-19).                                                                                |
| Q7  | How does a delegation or acceptance grant's expiry take effect?             | 1/3   | By an expire act the granting principal's node records; the delegating node also refuses a delegation past it, the only effect before PRV-10.     |
| Q8  | Should the launcher's loopback link be named otherwise?                     | 1/3   | Yes: "loopback connection", so "link" keeps its one meaning.                                                                                      |
| Q9  | Fix Setup's "nothing leaves the machine" now?                               | 1/3   | Yes: no ParleyInn component sends anything off the machine while no B2 or B3 component is on, and the harness alone carries content to the model. |
| Q10 | May a confirmed command from an untrusted event reach an agent?             | 1/3   | No: OWN-18 covers only what the launcher runs; what reaches an agent stays under I2 and OWN-03.                                                   |

## 2. Fixes that need no decision

- **Model:** recall marks a foreign room's items untrusted; a personal room
  holds a seat for every run its node records; sync exchanges the sealed
  ranges each peer may receive; a blind peer cites PEER-12; the unpin and
  list-removal entries say who gets the Needs you item; the step budget
  is no limit; the recorder list names the room-view component; an
  invite's review step; a seat key minted on a node identity change, a
  clone or a restore; the token key's clause order; a shrunk transcript
  starts a new generation; the owner's device seats; a document's single
  source; the trust grant's refusals in their own sentence; I2's "at that
  time" in Closed path; a principal surface is a client; a browser on the
  principal's phone; ready and abandoned marks named; Focus set moved to
  Needs you's file and its act called a neutral principal act; the link
  act's three links; a verdict is a neutral principal act; "one word" for
  node-wide utilities; the excluded "actor" and "poster" replaced by
  author.
- **SRS and scenarios:** "a room the run has had a seat in during the
  run" for "the run's rooms" (PIN-10, INJ-02); ENG-29's "B1, B2 or B3
  component"; PRV-10's rule levels by name; ASM-03's transcript;
  RCL-10's recall results; OWN-07's "allowed"; "result" only in the
  model's sense; NFR-15 and NFR-05's ingest wording; M8's sync; `parinn
  ui --phone`; the README's "until you turn it on".

The adversarial check found one problem, fixed: a budget cut in LANE-11
had made "its stamp" ambiguous, now "the new owner's stamp" again.
LANE-25 drops the undefined "grantor's node", which OWN-23 and OWN-26
already name; RCL-10's scenario lists the foreign room's pin through
`pin_list`; and "an error" and "what the tool returns" replace two
phrases the model does not define.

## 3. Optional, not chased

The CLI names `parinn bundle unpublish`, `parinn role assign|revoke` and
`parinn risk-acceptance show`; listing the remaining closed status-word
sets (handover, directed post, delegation, worktree fidelity, peer
status) in §9.7.

## 4. Invariants and reviews

No invariant changes. Waiting for the stakeholder:

- the I4 commit-trailer wording;
- the signing and sealing of what the room-view component, the launcher
  and the bridge component record (SEC-10, SEC-20);
- I10's "statuses";
- whether I4's last sentence allows §6.3 row 11;
- new, marked hard to revert: a paired phone keeps a device key, its
  device seat's key and an unsent writer outside any home, while I8
  binds all local state to one principal's home. A reviewer recommends
  rewording I8's first sentence by ADR with a security review, for
  example "All of a node's local state is bound to one principal's home
  with strict permissions; a paired phone keeps only its device key, its
  device seat's key and the part of that seat's writer its node does not
  yet hold, for one principal", and extending SEC-10 to keep the phone's
  keys in its platform key store.
