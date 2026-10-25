# Domain model: twenty-third round of blind reviews, merged

Five domain-model agents reviewed the model after the twenty-second-round
decisions were applied (commit 3c7ef8e): one across the whole model (X)
and four focused on concept groups, as in round 22: W (principals,
places, seats, harness facts), T (acts, roles, trust and flow), G (git
and forge, components and surfaces) and R (record, pins and context).
They read every file from disk. This note keeps the needs-fix findings,
each checked against the files. Each question takes its recommended
option, as the stakeholder asked, except the I4 question in section 4.

## 1. Questions and decisions

| #   | Question                                                                 | Found | Decision                                                                                                                                                 |
| --- | ------------------------------------------------------------------------ | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Should I2 carry the foreign-room limit?                                  | 1/5   | No: the model reads I2's trusted sources as PRV-02 applies them, which RCL-10 narrows in a foreign room. Narrowing stays within I2, so no ADR is needed. |
| Q2  | What is the short writer name in `A2·4812`?                              | 1/5   | A **writer label**: a writer's display alias, unique per node and room, in first-seen order; shown `A1·12`, typed `A1:12`, the writer id also accepted.  |
| Q3  | Is the recorder set closed?                                              | 1/5   | Yes, with the TUI added; a principal act from the browser room view is recorded by the CLI; a peer's or bundle's event records no recorder.              |
| Q4  | Can a collapsed landmark line cross writers?                             | 1/5   | It lists one address range per writer.                                                                                                                   |
| Q5  | Is changing a retention policy widening?                                 | 1/5   | Yes, and it applies only once its configuration is accepted (ADM-04).                                                                                    |
| Q6  | How does a dev server presentation appear?                               | 1/5   | As its address in inert text the person opens in their own browser; SEC-21 and I4 stand.                                                                 |
| Q7  | What does a commit land on without a forge?                              | 1/5   | The repository's default branch, or a branch the forge protects.                                                                                         |
| Q8  | What makes a check expected?                                             | 1/5   | A criterion of the intent naming it; the forge's required checks show as `asserted`.                                                                     |
| Q9  | Rename the CI carrier?                                                   | 1/5   | Yes: **CI bridge**.                                                                                                                                      |
| Q10 | Where does the room view listen?                                         | 1/5   | On loopback only (SEC-20); the launcher keeps the local endpoint.                                                                                        |
| Q11 | Is a result a derived artifact, and how is it addressed?                 | 1/5   | Derived from the events it rests on, addressed by the event recording its exit status or claim text, among the derived artifacts.                        |
| Q12 | Is a node's re-minted personal-room device seat a member?                | 2/5   | Yes, from its first event, as run and paired-phone seats are.                                                                                            |
| Q13 | Does a clone keep the old personal room?                                 | 1/5   | Yes: the personal room is carried over to a clone.                                                                                                       |
| Q14 | Where does the seat `parinn ingest` appends to after the MCP server sit? | 2/5   | In the run seat's room, taking over its add and role.                                                                                                    |
| Q15 | What signs with the principal key?                                       | 1/5   | The principal's own key tooling; no ParleyInn component holds it (PRV-10).                                                                               |
| Q16 | Which seats does an invite's role cover?                                 | 1/5   | Every seat chaining to the invited principal key that joins under it, run seats included.                                                                |
| Q17 | May a run seat hold the moderator role by role assignment?               | 1/5   | No: only by appointment, within SEC-32; a handover's role assignment covers the former owner's device seats.                                             |
| Q18 | Should the model state what makes an act widening?                       | 1/5   | Yes, one line; a join stays neutral, with its reason.                                                                                                    |
| Q19 | How are refusals classed?                                                | 1/5   | Refusing a quarantine, erasure or purge request is neutral; applying one keeps its class.                                                                |
| Q20 | Does OWN-19 keep a path to cross-machine takeover?                       | 1/5   | No: rejected, as §6.3 row X3 says; I4 stands.                                                                                                            |
| Q21 | How is a role removed?                                                   | 1/5   | **Revoke a role assignment**, a cut principal act (`parinn role revoke`).                                                                                |
| Q22 | Does `uninstall --purge` record a purge?                                 | 1/5   | Yes: a recorded purge with tombstones, a widening principal act.                                                                                         |
| Q23 | What role has a paired phone's personal-room seat?                       | 1/5   | None: it reads within its device scope.                                                                                                                  |
| Q24 | Should the settings-key rule allow a qualifier?                          | 1/5   | Yes: `<concept>.<room>.<qualifier>`.                                                                                                                     |

## 2. Fixes that need no decision

- **Model:** trusted text covers only OWN-04's and OWN-07's templates;
  the Pin entry defers to Intent and keeps PIN-10's trusted-creation
  condition; flags give counts only to landmarks; natural-language
  headline, capability, required checks and commit hook defined; a
  witnessed run's run-seat key alone lives in its MCP server; one
  transcript per subagent where the harness writes them apart; a held
  request stays open while the agent stops waiting; stamps survive a list
  removal; no one kicks, bars or mutes the owner; the room view's seats
  hold only events routed to them; "cloud environment" for nodes.
- **SRS and scenarios:** SEC-32's kick is undone only by the seat's own
  principal; OWN-19 drops the cross-machine path; `join-request
  decline`; `uninstall --purge` is widening; `--dry-run` → `--preview`;
  "Claude Code runs" → harness sessions; LANE-25's member rule; scenario
  addresses in the writer-label form; landmarks.feature "landmark list";
  the CI bridge throughout.

While applying Q17, `room_list_removal` turned out never to succeed: a
run seat is a moderator only by appointment, and SEC-32 refuses an
appointed moderator a list removal. As T recommended, the tool is
dropped from §9.2; a list removal goes through the CLI, as unbar and
unmute do.

## 3. Optional, not chased

Retry filed under `correction`; `parinn peer … status`; "Stale" for a
check state and a verdict; the MCP server's tool list; §6.3 "outside"
as a boundary; "Pause" in three senses; VIEW-21's `no evidence` and `no
verdict` in §9.7; PIN-06's "in v1"; the hub's excluded-term pairing.
W's question on who mints a run seat's key before the MCP server starts
is ASM-21's, which spike S4 tests; the model keeps ASM-21's wording.

## 4. Invariants

- **I2.** Unchanged (Q1).
- **I4, commit trailers.** G found that LANE-28's room trailers write
  room and pin ids into commits, which the principal's own `git push`
  carries off the machine through none of I4's channels. The reviewer
  recommends adding to I4 "or as repository content the principal's own
  tools push", by ADR with a security review. That widens an
  invariant's wording, so it waits for the stakeholder.
