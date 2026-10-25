# Domain model: twenty-fourth round of blind reviews, merged

Five domain-model agents reviewed the model after the twenty-third-round
decisions were applied (commit c4ea643): one across the whole model (X)
and four focused, as in rounds 22 and 23 (W, T, G, R). They read every
file from disk. This note keeps the needs-fix findings, each checked
against the files. Each question takes its recommended option, as the
stakeholder asked. From round 25, three blind reviewers run per round,
as the stakeholder set.

## 1. Questions and decisions

| #   | Question                                                               | Found | Decision                                                                                                                                          |
| --- | ---------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Which MCP server holds a subagent's run-seat key?                      | 1/5   | The harness session's MCP server, holding the main run's and its subagents' run-seat keys; spike S4 checks it beside ASM-21.                      |
| Q2  | Which writer does `parinn ingest` use while the MCP server still runs? | 1/5   | Always a new seat and writer the core seals, taking over the run seat's add and role.                                                             |
| Q3  | Who may revoke a service account's certificate?                        | 1/5   | Only its certifier; managed policy, for one it lists, by no longer listing it.                                                                    |
| Q4  | How are the harness's resume and clear named?                          | 2/5   | **Harness resume** and **harness clear**; bare "resume" stays the run control.                                                                    |
| Q5  | Where network cannot be denied, does a witness check run?              | 1/5   | No: ParleyInn refuses it; I4 stands.                                                                                                              |
| Q6  | What origin, recorder and provenance do bridge events carry?           | 1/5   | `witnessed`, the bridge component, `web`, always untrusted.                                                                                       |
| Q7  | Where do runless events about a room go?                               | 2/5   | A tombstone, a retention erasure request and a bridge's or launcher's event about a room's branch go to the recording device's seat in that room. |
| Q8  | May a criterion name a check, and is a forge-reported check a result?  | 1/5   | A criterion may name a check's command (LANE-20); a forge-reported check is a forge report, `asserted`, not a result.                             |
| Q9  | Which checkouts may the launcher write (ADM-13)?                       | 1/5   | Fresh checkouts and new worktrees outside the run's worktree, adding no ref to the run's repository.                                              |
| Q10 | Is writing a room summary a capability?                                | 2/5   | Yes, in LANE-16's closed set, carried only by the facilitator's appointment.                                                                      |
| Q11 | May a principal unpin its own agent's run-seat `stake`?                | 2/5   | Yes: Unpin governs it.                                                                                                                            |
| Q12 | What do run seats get from an invite naming moderator?                 | 1/5   | Contributor; a run seat is a moderator only by appointment.                                                                                       |
| Q13 | Can a trust grant cover a cross-room post shown in another room?       | 1/5   | No trust grant ever covers it.                                                                                                                    |
| Q14 | What class is confirming a steer for a later turn (OWN-13)?            | 1/5   | Widening.                                                                                                                                         |
| Q15 | Does cut cover undoing a widening?                                     | 1/5   | Yes: a cut act stops, narrows or undoes a widening.                                                                                               |
| Q16 | How are writer labels assigned?                                        | 1/5   | Unique per node, each assignment a structural event in the device seat's personal-room writer, so I10 and INJ-06 hold.                            |
| Q17 | Do proof classes stay inside I10?                                      | 1/5   | Yes: the clone facts LANE-06 reads are recorded as structural events, and LANE-06 derives from the record alone.                                  |
| Q18 | Can natural-language headlines stay derived artifacts?                 | 1/5   | Yes, made only by a deterministic template over trusted fields.                                                                                   |
| Q19 | Should I10 say "run and integrity statuses"?                           | 1/5   | No: the model's Derived artifact entry narrows it.                                                                                                |
| Q20 | How is a multi-word concept spelled in names?                          | 1/5   | Snake case in MCP tools and settings keys, kebab case in CLI commands.                                                                            |

## 2. Fixes that need no decision

- **Model:** a re-minted personal-room device seat needs no add;
  principals of agents whose runs a node records; Device certifies once
  PRV-10 ships; the personal room's clone case; the kernel worker
  exception to the launcher; pin priority's values and a Budget entry;
  rule levels ordered; delegated recall taint; event kinds listed;
  links' ends named; `claim` covers tool output alone; the outcome
  window shows a presentation, never the outcome.
- **SRS and scenarios:** OWN-03 allows OWN-24's and OWN-29's template
  text; OWN-10 tightens a tainted run; LANE-31 names leave, kick and
  bar; PEER-06 for the access token; §9.5 unbar where SEC-32 permits;
  §6.3 row 23 rejected (NG9), row 1 names the CLI; INJ-02 and LANE-30
  say harness resume; the restore and record scenarios' wording.

## 3. Optional, not chased

§9.5 rows that leave their class unstated; `successor withdraw` and
`needs-you dismiss` carrying two acts; "Core library" in §4; PIN-06's
"in v1".

## 4. Invariants

None changes. The I4 commit-trailer question of round 23 still waits for
the stakeholder.
