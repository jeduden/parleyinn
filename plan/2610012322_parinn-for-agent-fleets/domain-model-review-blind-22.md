# Domain model: twenty-second round of blind reviews, merged

Five domain-model agents reviewed the split model after the
twenty-first-round decisions were applied (commit 31e9491). They read
every file from disk, the SRS, the invariants and `features/`. One
cross-cutting reviewer (X) read the hub and every concept file. Four
focused reviewers each read a group of concept files against the SRS
sections and scenarios that use it: W (principals, places, seats, harness
facts), T (acts, roles, trust and flow), G (git and forge, components and
surfaces) and R (record, pins and context). This note keeps the needs-fix
findings, each checked against the files. Each question takes its
recommended option, as the stakeholder asked. The one exception is the
I2 wording in section 4, which waits for the stakeholder.

## 1. Questions and decisions

| #   | Question                                                                  | Found | Decision                                                                                                                                        |
| --- | ------------------------------------------------------------------------- | ----- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is a keystroke relayed in terminal takeover text the launcher carries?    | 2/5   | No: the keystrokes are the harness's own input from the terminal the launcher hosts, never carried, so I4 keeps its words; the model says why.  |
| Q2  | Should I2 carry the foreign-room limit?                                   | 2/5   | Yes, recommended; left for the stakeholder (section 4). The model's narrower Trusted sources stand meanwhile.                                   |
| Q3  | What is one run?                                                          | 1/5   | One agent's execution within one harness session on one node: a transcript holds its main agent's run and one per subagent; a clear starts one. |
| Q4  | Who seals what `parinn ingest` appends after the run's MCP server ended?  | 1/5   | A new seat and writer the core seals, naming the run seat (REC-19).                                                                             |
| Q5  | Does a re-minted personal-room seat need a join?                          | 1/5   | No: it is a member from its first event; outside the personal room a re-minted seat joins as any seat does.                                     |
| Q6  | Before PRV-10 ships, which device seat records a principal act?           | 1/5   | This node's own device seat, in the room the act acts on.                                                                                       |
| Q7  | Which key do an invite, the admission list, handover and successor name?  | 1/5   | A principal key.                                                                                                                                |
| Q8  | Is the facilitator's program an agent?                                    | 1/5   | No: neither a ParleyInn component nor an agent; it acts through its node's CLI.                                                                 |
| Q9  | Can a personal-room seat leave or be kicked?                              | 1/5   | No.                                                                                                                                             |
| Q10 | How is a service account certified by managed policy revoked?             | 1/5   | Managed policy stops listing it.                                                                                                                |
| Q11 | Away policy: a settings key, or an act only?                              | 1/5   | An act only: no settings key sets it, as with the notice opt-in.                                                                                |
| Q12 | Is tightening a rule level widening?                                      | 1/5   | No: loosening a rule level is widening; tightening is cut.                                                                                      |
| Q13 | Are the roles a whole-room mute leaves posting to a setting or an act?    | 1/5   | A room setting.                                                                                                                                 |
| Q14 | Is `unbound` a check state or a result mark?                              | 1/5   | A result mark, as `from checkpoint` and `outside intent` are.                                                                                   |
| Q15 | Is a check state a derived artifact under I10?                            | 1/5   | No: computed where shown, as room status is.                                                                                                    |
| Q16 | Which commits must a witness node's git identity not have authored?       | 1/5   | Any commit on the branch since it left its base.                                                                                                |
| Q17 | Should the git carrier, the CI carrier and the outcome window be renamed? | 1/5   | No: they keep their names; the git carrier stays in the publish component, whose B3 covers outbound exchange.                                   |
| Q18 | Does the exception for excluded words cover a tool's own sense of a word? | 1/5   | Yes: it extends to ENG-28's review tool and engineering.feature's review outcome.                                                               |
| Q19 | After a handover, whose agents does the intent restore to?                | 1/5   | Only its stampers' agents, until the new owner revises or stamps it.                                                                            |
| Q20 | Do derived outputs inherit taint?                                         | 1/5   | Yes: a recall result, kernel output and an export inherit it, as derived artifacts do.                                                          |
| Q21 | Which origin does an erasure or quarantine request take?                  | 1/5   | `operator` when a principal act sends it, `structural` when a retention policy does.                                                            |
| Q22 | Are the latency targets of NFR-01, NFR-03 and NFR-15 I9 budgets?          | 1/5   | No: they are targets; I9's budgets are the named budgets and limits.                                                                            |
| Q23 | How are the opt-ins of PEER-06 and SEC-10 set?                            | 1/5   | As configuration settings under ADM-04.                                                                                                         |

## 2. Fixes that need no decision

- **Model:** a run, principal or agent *has a seat in* a room; role
  "has work"; trusted text, CI attestation, typing hint, quarantine list
  and the core's parts defined; the trust policy reads whether a room is
  foreign; a leave, kick or bar keeps pins within the foreign-room
  limit; a list removal never takes the intent; repository identity is
  provisional until bound; Result is what a room's seats established;
  segments are encrypted to device keys or a token key; the Harness
  facts heading says ParleyInn changes only harness configuration.
- **SRS and scenarios:** worktree, git carrier, key set, envelope and
  address in place of near-synonyms; ADM-14's seat scope; LMK-01's
  spans; REC-22's ingest-marker origin; SEC-30's erasure request; the
  `unbound` result mark in LANE-05, LANE-19, VIEW-21 and §9.7.2; the
  browser room view client; §9.5's widening and cut lists; OWN-06's
  managed policy; rule level; typing hint; NFR latency targets; run
  identity in appendix A, OQ-12 and §8.2; §6.3's row names.

## 3. Optional, not chased

Layout words beyond the list; "presence hint" beside "typing hint";
"reason class"; "surface mark"; §6.3's short component names; "config";
"cost"; ENG-02 crate names; "resume" as a run control and a trigger.

## 4. Invariants

- **I4, terminal takeover.** Unchanged; the model now says why the
  person's keystrokes are not text the launcher carries.
- **I2, the foreign-room limit.** X and T found that the model's Trusted sources
  and PRV-02 trust nothing in a foreign room but its principal's own device-seat
  pins and stamped versions, while I2 lists those sources without the limit. The
  recommended fix adds to I2's trusted sources "none of them in a foreign room
  but that principal's own device-seat pins and the pin versions it stamped". It
  only narrows trust, but the edit to the invariant waits for the stakeholder's
  review in an ADR. The model rule, narrower than I2, binds meanwhile. Round 23
  settled it without an ADR: the model reads I2's trusted sources as PRV-02
  applies them, which RCL-10 narrows in a foreign room.
