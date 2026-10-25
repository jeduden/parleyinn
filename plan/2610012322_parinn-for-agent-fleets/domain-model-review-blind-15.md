# Domain model: fifteenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the fourteenth-round decisions were applied (commit d0472e5),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                                            | Found | Decision                                                                                                                                         |
| --- | ----------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Q1  | Is PIN-07's compaction guidance a write outside I2's closed paths?                  | 1/3   | It becomes one: fixed text ParleyInn ships, named in I2 (ADR-2610063000) and defined in the model.                                               |
| Q2  | Is revoking a seat key, access token or certificate that stops a pin restoring cut? | 1/3   | No: such a revocation is widening, as the model's catch-all already says; SEC-27 and OWN-11 add the condition.                                   |
| Q3  | What may the owner's device seat do?                                                | 3/3   | Every room capability except writing a room summary; it edits and unpins only pins it wrote, and makes a list removal of any pin but the intent. |
| Q4  | Does a stamped version of a type that does not restore restore?                     | 1/3   | No.                                                                                                                                              |
| Q5  | Is confirming a candidate of a type that does not restore a principal act?          | 1/3   | Yes: confirming any pin candidate is its own principal act (PIN-05); a token-key-only node cannot confirm one.                                   |
| Q6  | Does the restore block count pins of types that do not restore?                     | 1/3   | No: it counts only pins of a type that restores that do not qualify.                                                                             |
| Q7  | Does a re-add after a kick or leave keep the seat?                                  | 1/3   | Yes: the same seat and writer; a different run joining gets its own seat (LANE-23).                                                              |
| Q8  | Does the Gap marker cover missing segments and capture gaps?                        | 1/3   | A **missing range** is part of a writer this node does not hold, such as a capture gap or a lost tail.                                           |

## 2. Fixes that need no decision

- **Model:** the member verb points to Member; the provenance classes
  listed; token-key-only node named under Node; the hide row says the
  UI does not show it.
- **SRS and scenarios:** §9.7.6's token-key-only events → "its pins
  restore only once a principal stamps a version"; LANE-27's join names
  the room in the next restore block and `pin_list` lists its pins;
  ADM-02 "every other file ParleyInn wrote"; NG9 and §6.3 row 23 "merging
  several seats' edits to one worktree"; §4 recall tools `event_search`
  and `event_expand`; SEC-32 names the pin by id and carries a range
  link to the marked range; 01b's web service in model terms; Gherkin
  `<outcome>` columns → `<expected>`; LANE-31 concurrent branch links by
  the lower commitment; VIEW-04 drops `silent 40 min` for §9.7.1's
  marks; LANE-15 quarantines a foreign room's writers.

## 3. Optional, not chased

"LLM headlines"; "structured operators"; bare "policy"; VIEW-21's
"each branch the outcome names"; personas' "config"; OWN-27's "the room
records"; LANE-15's "asserted by"; ENG-02's crate names; "concurrent
writers"; "approval" in OWN-07; "message" for a post; layout words;
`parinn check witness`; `parinn pin remove`; I3's title; I4's core list;
"untrusted-data envelope".

## 4. Invariants

I2 changes, recorded in ADR-2610063000.
