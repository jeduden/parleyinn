# Persona review, round 8: blind review of pitch v10

Round 8 gave all nine personas the text of pitch v10 (commit dc9b0a7)
and nothing else: no repository, no SRS, no history. Each answered five
questions: first reaction, whether the problem is their day, the
promise that matters and the one that fails, what stops them
installing, and one change that would win them over.

## Verdicts

| Seat                    | Reaction                 | Serves them?                                           |
| ----------------------- | ------------------------ | ------------------------------------------------------ |
| Fleet developer         | Keep reading, not yet    | Core yes; written for one long task, not a fleet       |
| Returning owner         | Keep reading, not yet    | No; it is for someone at the screen, not coming back   |
| Reviewer                | Keep reading, not try    | No; the reviewer only appears under "next"             |
| Live collaborator       | Keep reading, not yet    | No; their journeys are a "next" footnote               |
| OSS maintainer          | Keep reading, not yet    | Author yes; receiving a lane is only a promise         |
| Multi-machine developer | Keep reading, not yet    | No; a single-machine pitch, peering only a teaser      |
| Platform operator       | Move on, check the docs  | No; browser and peering look like new attack surface   |
| Security officer        | Keep reading for threats | Core maybe; sharing and remote answering, no           |
| Agent                   | Keep reading, warily     | Restore yes; recall, provenance and notices unpromised |

Nobody would install today. Everyone kept reading.

## Findings across seats

| #     | Finding                                                                                                                                                 | Seats                                                         |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| R8-1  | "Nothing leaves your machine" contradicts peer sharing and a browser listener, and reads as a double negative; say what ParleyInn opens, off by default | security, operator, collaborator, multi-machine, fleet, agent |
| R8-2  | "Answer its prompt there" is a remote-control channel; say who can reach the lane view and how it is authenticated                                      | security, operator                                            |
| R8-3  | Does passed-on text keep its author's mark, or become the owner's own words?                                                                            | agent, security, collaborator, maintainer                     |
| R8-4  | "Rewind to an earlier point" is unclear: the chat, the git tree, or the record?                                                                         | fleet, returning owner, agent                                 |
| R8-5  | Silent failure: would a failed capture or missing segment show? Integrity is not promised                                                               | returning owner, security, operator                           |
| R8-6  | Catch up after time away is missing; the pitch assumes you watch live                                                                                   | returning owner                                               |
| R8-7  | Intent is a goal, not standing rules ("never push to main"); are rules pinned too?                                                                      | agent                                                         |
| R8-8  | Install contract missing: what changes in `~/.claude`, reversible, hooks never slow or block, browser optional                                          | fleet, operator                                               |
| R8-9  | Fleet framing: many short sessions in worktrees; two lanes touching one file; whose dev server and port                                                 | fleet                                                         |
| R8-10 | Reviewer: CI evidence for the exact commit, verdict versus forge approval, landed commit back to its lane, the author not approving their own lane      | reviewer                                                      |
| R8-11 | Receiving a lane as a file beside the pull request, redacted again and verified on import, never instructions                                           | maintainer                                                    |
| R8-12 | Machines, not terminals: dial-out-only sandboxes, offline merge, the record surviving a reclaimed sandbox                                               | multi-machine                                                 |
| R8-13 | Collaborator: see whether a correction was forwarded or declined, a side channel the agent never reads, handover                                        | collaborator                                                  |
| R8-14 | Operator: managed install with the lane view and peering off, disk caps, purge with audit, drops counted                                                | operator                                                      |
| R8-15 | Agent: an exact recall tool by turn or search, a count of waiting messages instead of their content, what is injected per turn                          | agent                                                         |
| R8-16 | Can content from the running app's preview reach the model?                                                                                             | security                                                      |

## Pitch gaps and SRS gaps

Most findings are pitch gaps: the SRS already holds the answer.

- **Pitch gaps (SRS covers them):**
  - R8-1 (I4, SEC-01).
  - R8-2 (SEC-20, OWN-02).
  - R8-3 (OWN-08's template names the writer).
  - R8-5 (I6, VIEW-10, VIEW-11).
  - R8-6 (VIEW-08).
  - R8-7 (PIN, I3).
  - R8-8 (ADM-02, I9, VIEW-03).
  - R8-9 (LANE-13).
  - R8-11 (LANE-15, bundles).
  - R8-12 (PEER).
  - R8-13 (LANE-12 states, LANE-11).
  - R8-14 (SEC-22, ADM-15, ADM-07).
  - R8-15 (RCL, VIEW-15).
- **SRS gaps:**
  - R8-4: rewind's effect on chat, tree and record.
  - R8-16: preview isolation.
  - R8-9: dev-server ports per worktree.
  - R8-10: CI evidence in the first release. The author's own verdict against a reviewer's approval needs one line.
  - R8-12: the record surviving a reclaimed sandbox.

## Next

The stakeholder decides how the pitch answers the other seats: one
pitch for the developer with a line per seat, or seat-specific pages
behind it. The SRS gaps go into the requirements follow-up.
