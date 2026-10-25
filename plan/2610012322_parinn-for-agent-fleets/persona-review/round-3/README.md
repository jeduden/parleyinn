# Persona review, round 3

The persona-review skill ran all nine persona agents a third time, in
parallel, on [proposal.md](../../proposal.md) at commit 77ea1d9, the
revision that answered [round 2](../round-2/README.md). Each persona's
full report is beside this file.

## Verdicts

| Persona                 | Verdict                                     | Still blocking                                  |
| ----------------------- | ------------------------------------------- | ----------------------------------------------- |
| fleet developer         | most of round 2 fixed                       | no owner act works with only the core installed |
| multi-machine developer | much closer                                 | a sandbox needs its issuing peer online         |
| reviewer                | serves journeys 1 to 3 and the landed trace | none                                            |
| OSS maintainer          | the trust model mostly serves               | ingest trusts a stranger; fabrication; exports  |
| live collaborator       | serves in contract terms                    | none; role rights and handover need rules       |
| returning owner         | serves                                      | none; removals and baselines need rules         |
| platform operator       | serves in structure                         | none; retention and backups need rules          |
| security officer        | serves                                      | none; erasure copies and the confirmation code  |
| Claude, the agent       | mostly serves                               | a branch switch drops lane pins                 |

Five of nine report no blocker. The six blockers left are narrow and
each has a drafted fix.

## Merged findings

R3-1 to R3-6 are blocking; the rest are important or minor.

| #     | Finding                                                                                                                                                         | Raised by                                                                          | Action                                                                                                                                     |
| ----- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| R3-1  | No owner act works with only the core: OWN-12's code comes only from `parinn-ui` or `parinn-run`, so `parinn pin add` needs a B1 binary; the code is recordable | fleet, security                                                                    | Terminal confirmation naming the verb in core-only mode; code never stored; tightening acts need none; status says when OWN-11 rests on it |
| R3-2  | A branch switch or lane merge drops lane pins from the restore block                                                                                            | agent, multi-machine                                                               | Restore every lane's pins the session has belonged to; merged lanes' pins carry over; references to a merged id resolve                    |
| R3-3  | `parinn ingest --path` stores a stranger's transcript under the local writer, so its `user` lines are trusted                                                   | maintainer                                                                         | Events from a transcript the hooks did not report are untrusted whatever their class; outside the harness's own directory, a foreign lane  |
| R3-4  | The git-key cross-signature runs the wrong way, and foreign writer keys are not bound to their owner key                                                        | maintainer                                                                         | A statement signed by the contributor's git signing key binds their owner and writer keys; LANE-15 matches only on it                      |
| R3-5  | `parinn export` bypasses the review step and the owner-act rules                                                                                                | maintainer, security                                                               | Export is an owner act under OWN-12, passes SEC-26 for every bundle or report, and is audited                                              |
| R3-6  | A sandbox cannot certify its writer or deliver segments while its issuing peer is offline; nothing starts `parinn-peer` in it                                   | multi-machine                                                                      | The token carries a scoped delegation, the bound project identity and every delivery address; the sandbox entrypoint starts `parinn-peer`  |
| R3-7  | Erasure copies: backups, peers' own copies, SEC-31 below ADM-07's priority, no purge receipt                                                                    | security, operator                                                                 | Purge covers or lists backups; restore reapplies purges; every purge replicates and peers run SEC-31; SEC-31 P0; a per-node purge receipt  |
| R3-8  | Keys and identity: revocation misses issued certificates; node identity undefined; retired and never-synced writers                                             | security, operator, multi-machine                                                  | Revocation covers what the key issued; node identity from a value clones cannot carry; retired writers; "enrolled, never synced"           |
| R3-9  | Managed policy and resources: invalid policy, no retention window, unbounded local writer, the `os/exec` contradiction                                          | operator                                                                           | Fail closed on an invalid policy; managed retention; one `os/exec` list                                                                    |
| R3-10 | Lane membership: lanes served to any enrolled peer; role rights not normative; former owner's agents refused; handover never expires                            | security, collaborator                                                             | Serve only members; a normative role table; the former owner becomes a co-author; offers expire; requests go to the agent's principal      |
| R3-11 | Review gate: forge hand-off has no derived state; OQ-22; changes requested does not block; authorship of checkpoint hunks; three key names                      | reviewer                                                                           | Forge approvals `asserted` and not flagged; agent verdicts never count; a request for changes blocks; one signing key                      |
| R3-12 | Catch up: baseline heads, legitimate removals, `unrecorded` source, line order, §8 not normative, time filter                                                   | returning                                                                          | Heads from the newest receipt; list removals; define `unrecorded`; order and cap; §8 into the SRS; writer time marked asserted             |
| R3-13 | Delivery and phase tests: OWN-08 after M8's exit needs it; weekend fixture contents; RED tests miss evidence, recall-only, policy and lane merge                | reviewer, collaborator, returning, agent, security, operator, multi-machine, fleet | OWN-08 into M8; fixture contents; four RED-test additions; phase wording in §8's vocabulary                                                |
| R3-14 | Hooks and holds: `Notification` has no budget; OWN-15 misses OQ-17; holds with nobody connected; phone allow for session; terminal signals                      | fleet, security                                                                    | A budget row; OWN-15 covers OQ-17; hold only while connected; phone allows once only; signals never in context                             |
| R3-15 | Agent-facing details: request ids, petnames on pushed paths, chain words, §9.3 fields                                                                           | agent                                                                              | Request ids resolve through `get`; agent-facing paths use fingerprints; `unverified` in §8.5; `origin` in §9.3                             |
| R3-16 | Peer details: relay hop, tail claim, publish bind, rows 25 and 26, token minting, recall display, presence display, first-view budget                           | multi-machine, security, collaborator                                              | One MUST each                                                                                                                              |
| R3-17 | Document defects: rows outside tables, SEC-30's columns, a stray heading, D6 for D7, pitch leftovers                                                            | all                                                                                | Fix the text                                                                                                                               |

## Next

Revise proposal.md against R3-1 to R3-17, then run a fourth round.
