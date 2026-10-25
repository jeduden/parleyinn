# Pending decisions: the stakeholder's walkthrough

After the thirty-second round of blind reviews, seven items still waited
for the stakeholder ([round 32][blind32], section 4). One agent per item
wrote a brief from the files on disk: the options, what each changes and
what would be hard to undo once a release ships. A checker verified the
briefs. The stakeholder then took them one by one, in an order that put
each item after the ones it depends on.

## 1. Decided

| #   | Item                                  | Decision                                                                                                                                                                                                                                          |
| --- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Does I4's last sentence allow row 11? | I4's last sentence binds what ParleyInn's components send. A feature whose purpose is to reach off the machine through a carrier the principal runs gets its own §6.3 row, is off by default and is turned on by a widening act (ADR-2610072203). |
| 2   | The I4 commit-trailer wording         | Settled by item 1. The trailers get a §6.3 row for the principal's own push and a line in Setup (VIEW-18); LANE-28 traces I4. They stay always on (Q35); item 1's guard does not cover them.                                                      |
| 3   | I10's "statuses"                      | "room state, trust levels, run and integrity statuses" (ADR-2610072203). A refused segment is recorded as a structural event, so `refused` derives from the record.                                                                               |
| 6   | Where a seat's kind is recorded       | Seat certificates from the first release: the node's device key certifies each seat key, naming its kind (PRV-11, M1). PRV-10 keeps the principal-key chain, scopes, token keys and the trust they grant.                                         |

## 2. Deferred, explicitly

Each deferred item is an open question with its options, a
recommendation and a deadline. Every text that rests on it points to it.

| #   | Item                                                                         | Open question | Due | Recommended                                                  |
| --- | ---------------------------------------------------------------------------- | ------------- | --- | ------------------------------------------------------------ |
| 4   | A paired phone's keys and writer outside any home (I8)                       | OQ-39         | M8  | The phone keeps only its device key; no seat, no writer      |
| 5   | Signing and sealing what the room-view component, launcher and bridge record | OQ-40         | M7  | Only the core holds keys; components pass events to the core |
| 7   | An act on a room by a principal with no member seat there                    | OQ-41         | M7  | A non-member seat in the room, limited to ownership acts     |

OQ-39's part about a room-view secret held by a browser comes due by M7,
with §6.3 row 11. OQ-40's part for the bridge may wait until M9.

## 3. Found on the way

- `ParleyInn-Link:` trailers for posts and ranges carry full addresses, so
  pushed history shows writer ids and seq positions; the room id groups
  a room's commits across repositories. The stakeholder kept Q35 with
  this known.
- PRV-11's I2 security review is due before M1 ships.

[blind32]: domain-model-review-blind-32.md
