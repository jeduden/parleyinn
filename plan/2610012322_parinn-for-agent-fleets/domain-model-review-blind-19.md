# Domain model: nineteenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the eighteenth-round decisions were applied (commit c559a34),
against itself, all of `docs/srs`, the invariants and `features/`,
reading every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked.

## 1. Questions and decisions

| #   | Question                                                             | Found | Decision                                                                                                                                      |
| --- | -------------------------------------------------------------------- | ----- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Does "configuration" mean the principal's settings or every layer?   | 2/3   | The principal's; managed policy, configuration and repository configuration are the three **settings layers** (ADM-04, PRV-05, §9).           |
| Q2  | What does the appointment rate limit?                                | 1/3   | The kicks, bars and mutes an appointed moderator may set per period (SEC-32).                                                                 |
| Q3  | Which budgets are I9's?                                              | 2/3   | The named budgets and the named limits: restore block limit (INJ-07), CMP-05's limits, SEC-04's query deadline, the output caps.              |
| Q4  | Does "ingest" have two meanings?                                     | 1/3   | No: ingest is reading a transcript into the record; only an unwatched transcript's events take origin `ingested`.                             |
| Q5  | Is what `parinn ingest` appends past an ingest marker hook-recorded? | 1/3   | No: never recorded by the hook handlers, so never a trusted source.                                                                           |
| Q6  | Which class does quarantining content outside a restore block take?  | 1/3   | Cut: a quarantine that removes nothing from a restore block.                                                                                  |
| Q7  | Is turning an away policy off cut while changing one is widening?    | 1/3   | Yes: widening is turning on or changing an away policy other than turning it off.                                                             |
| Q8  | Is a paired phone's writer written on the phone or on its node?      | 1/3   | On the phone: a writer is written on one node or paired phone and held by any node.                                                           |
| Q9  | What is work: event routing or LANE-16's actions?                    | 1/3   | LANE-16's actions; a run's events go to its run seat while its role has work.                                                                 |
| Q10 | Where does the harness adapter's core part sit, given I4's list?     | 2/3   | Among the hook handlers and the CLI, which I4 names; I4 keeps its words.                                                                      |
| Q11 | What may `harness_meta` carry?                                       | 1/3   | What the harness reports, or the launcher records, about the harness's operation, never free text; terminal input only as that input arrived. |
| Q12 | Should accepting or asking for a join stay neutral?                  | 1/3   | Yes: a join brings in only pins the run's own principal already trusts, so OWN-11 keeps it neutral.                                           |

## 2. Fixes that need no decision

- **Model:** commitment key, seat kind, delegation budget, room summary
  pointer and landmark tiers defined; a backup holds the audit log; the
  fixed template may carry a post a trust grant covers; a trust grant
  covers an author's principal; the naming rule speaks of settings
  keys, as do the domain-model agent and CLAUDE.md.
- **SRS and scenarios:** PRV-02 "once PRV-10 ships"; OWN-28 "the
  earlier run"; SEC-32's audit post names the act's target; PEER-12
  "another principal"; VIEW-17 and §6.3 row 6 "terminal notification"
  and "in-page notification"; `parinn hook <hook>`; OWN-02's terminal
  input; the SRS status row names ADR-2610063200.

## 3. Optional, not chased

Layout words beyond the model's list (tile, pill, rail, composer,
banner); "recall address"; the verdict "stale" beside the Stale check
state; §6.3's short component names; ENG-02 crate names; "secure
purge"; "separate from the view"; "rules" in principal-acts.feature;
"view" alone; "source" in its everyday sense; "reply" in two senses;
"room bundle"; `pin.max_restore_model_tokens`; "was approved by your
principal"; "user-stated constraints"; PIN-10 "its principal's personal
room"; §9.7.2 "a draft"; I1's "writer's log" and I8's "seat key"; "chain
status" in the excluded-term table; "subagent identity"; "the run's
rooms"; "metadata lines" and "home id"; "attestation"; nodes starting
agents; revoking a device beside revoking a token.

## 4. Invariants

None changes.
