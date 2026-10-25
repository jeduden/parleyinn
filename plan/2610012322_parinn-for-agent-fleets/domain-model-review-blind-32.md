# Domain model: thirty-second round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
thirty-first-round decisions were applied (commit 2b41185), in the
review workflow: two skeptics, on text accuracy and on whether the
defect is real, checked every needs-fix finding. All 8 survived. Each
question takes its recommended option, but the two in section 4, which
reviewers mark hard to revert.

## 1. Questions and decisions

| #   | Question                                                                     | Found | Decision                                                                                                                                            |
| --- | ---------------------------------------------------------------------------- | ----- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is "lock a boundary off" a power apart from "disable"?                       | 1/3   | No: one power, "disable", as I4 names it; "lock" goes from SEC-22, §6.3, ADM-16, M7 and the scenarios, superseding round 20's wording.              |
| Q2  | What is the credential that makes the browser room view a principal surface? | 1/3   | A **room-view secret**: a **launch secret** exchanged once for an **origin secret**; a phone-scoped one serves a browser on a phone.                |
| Q3  | Is issuing the phone-scoped room-view secret a principal act?                | 1/3   | Yes, widening, at the terminal (`parinn ui --phone`); plain `parinn ui` still records none.                                                         |
| Q4  | What creates the personal room when `parinn install` does not run?           | 1/3   | The principal installing ParleyInn by any supported path; its device seat records the create room act at ParleyInn's first start there.             |
| Q5  | Are keys outside `PARINN_HOME` bound to the home under I8?                   | 1/3   | Yes: a platform key-store entry named per home, or a key in that home's MCP server's memory; I8 keeps its words.                                    |
| Q6  | Does a bar end the add the seat ingest starts shares?                        | 1/3   | No: a kick or leave of either seat ends that one add; a bar is the Bar entry's.                                                                     |
| Q7  | Who gets the Needs you items of a list removal?                              | 1/3   | Each stamper (LANE-32), and for a device-seat pin its author's principal (LANE-26).                                                                 |
| Q8  | What does `parinn uninstall` do when its capture-off act is refused?         | 1/3   | It changes nothing, names the open residual risks and the runs that leave them open, audits the refusal and points to the harness's plugin removal. |

## 2. Fixes that need no decision

- **Model:** **spend** defined on its own, and quota limits storage or
  events until OQ-26 closes; the **pin author** named in Pin; a harness
  resume and a harness clear apart; a member is a seat no bar covers; a
  trust grant's pins restore outside a foreign room; Trusted text is what
  the restore-block code constructs; the node's principal's Needs you
  queue; PEER-01's environment variable; the appointment's comma; "offer
  a handover"; the landing link qualified; Trust and flow's summary
  lists its terms.
- **SRS and scenarios:** M7's "other principal surfaces"; RCL-05's scope
  words; §9.6 "Settings keys (selected)"; §9.7.1's "Detail" column;
  `parinn peer enroll|revoke|list` and "each peer's state"; `--expiry`;
  `parinn room mute` takes a reason and an expiry; `room_get` records a
  recall event; OWN-10's rule level change "the principal makes";
  CMP-08's kernel worker inside the core.

The adversarial check found no unsafe cut. LANE-01 points to ADM-02
for the install paths, for its token budget; 09a says "turn off" for
the git carrier and bridges, so "disable" stays managed policy's power;
and OQ-26 says quotas limit only storage and events until it closes.
The stakeholder stopped the review rounds after this one.

## 3. Optional, not chased

One verb for each successor withdrawal (`parinn successor withdraw` now
carries two acts of one class); the internal/srs test fixture's
"Lanes." and "Owner acts.", which code follows separately.

## 4. Invariants and reviews

No invariant changes. Waiting for the stakeholder:

- the I4 commit-trailer wording;
- the signing and sealing of what the room-view component, the launcher
  and the bridge component record (SEC-10, SEC-20);
- I10's "statuses";
- whether I4's last sentence allows §6.3 row 11;
- a paired phone's keys and writer outside any home (I8);
- new, marked hard to revert: an act on a room recorded in a personal
  room, naming the room (an owner with no seat left offering a handover
  or naming a successor, or the owner's node recording an expire act).
  The room merge does not say whether it counts such an act, nor how
  other principals' nodes come to hold it while the personal room stays
  private (PEER-02). A reviewer recommends counting it and serving that
  act alone, by address, to nodes holding the named room, amending Room
  merge, the routing relation, LANE-31, LANE-29 and PEER-02 together;
- new, marked hard to revert: where a seat's kind is recorded before
  PRV-10 ships, when no seat certificate exists. A reviewer recommends a
  structural field of the seat's first act, which the seat certificate
  later repeats.
