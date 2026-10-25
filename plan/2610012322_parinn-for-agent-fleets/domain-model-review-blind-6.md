# Domain model: sixth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the fifth-round decisions were applied (commit 67ed565), against
itself, all of `docs/srs`, the invariants and `features/`, reading
every file from disk. The stakeholder asked for rounds to repeat until
none finds anything that needs addressing, taking the recommended
option on every question unless none exists or it is hard to reverse;
it allowed the model's line limit to rise from 800 to 1000.

## 1. Questions and decisions

| #   | Question                                                                                                | Found | Decision                                                                                                                                                                                                                                                 |
| --- | ------------------------------------------------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | A paired phone's device seat in the personal room has no add, so it is never a member.                  | 3/3   | It is a member with no add, like a run's personal-room seat.                                                                                                                                                                                             |
| Q2  | After a moderator's unpin, does a device-seat pin keep restoring to every agent, or its author's only?  | 2/3   | Every agent it restored to (a room act changes no restore block); LANE-26 follows.                                                                                                                                                                       |
| Q3  | Is a device-seat post `post` or `operator`?                                                             | 2/3   | `post`; `operator` is the class of principal acts and device-seat pins.                                                                                                                                                                                  |
| Q4  | I2's trusted sources omit trust-granted posts and pins; its first sentence contradicts its later paths. | 3/3   | Reword I2 by ADR: content outside the trusted sources reaches the model enveloped by recall, or through a closed path a principal act names; trust-granted posts come in OWN-29's template.                                                              |
| Q5  | I4 says data leaves only as recalled content, yet restore blocks and notices reach the model provider.  | 1/3   | Reword I4 by ADR: data leaves only as what ParleyInn writes to an agent through I2's closed paths, which the harness sends to its model, or through B2 or B3.                                                                                            |
| Q6  | I8 says "held as theirs" and "widens … recall"; I10 says "derived state".                               | 3/3   | Reword by ADR: "kept as theirs", "never makes them trust it nor extends their recall", "all derived artifacts".                                                                                                                                          |
| Q7  | Principal acts the list does not classify.                                                              | 3/3   | Widening: set a room setting, publish, answer a purge request, accept configuration (record its digest), enroll a CI key, enable the git carrier, rotate a device key. Neutral: open the forensic view, ask for a join. Cut: withdraw a named successor. |
| Q8  | How does the facilitator write a room summary, when MCP room tools act under a run seat?                | 2/3   | Through its node's CLI (`parinn room-summary write`), signed with its device seat; it never acts through MCP tools.                                                                                                                                      |
| Q9  | "Token key" breaks "token is always an access token or a model token".                                  | 1/3   | Allowed by name: the key of an access token.                                                                                                                                                                                                             |
| Q10 | "Hidden characters" against the excluded "hide".                                                        | 1/3   | "Invisible characters"; "hiding commitment" stays as a cryptography term.                                                                                                                                                                                |
| Q11 | "Threat source", "single source" against the source rule.                                               | 2/3   | The rule covers "source" as a ParleyInn term; "threat source" (§6.1) keeps its security meaning.                                                                                                                                                         |
| Q12 | Per-room configuration keys break `<concept>.<setting>`.                                                | 1/3   | Allowed: `<concept>.<room>.<setting>`.                                                                                                                                                                                                                   |
| Q13 | Store, projection, backup, generation, storage key, report: define or reword?                           | 3/3   | Define store, backup, transcript generation, at-rest key and rendering (replacing "report" and "storage key"); "projection" and "derived state" become derived artifact.                                                                                 |
| Q14 | Service-account certification has no requirement.                                                       | 2/3   | PRV-10 states it.                                                                                                                                                                                                                                        |

## 2. Fixes that need no decision

- **Active pin:** the qualifying pins are the pins and stamped versions
  that restore to one agent, active pins and those an unpin leaves
  restoring.
- **Member** while its add stands and no bar covers it; **marked
  range** is an address range a range link names (no marking act);
  **blind peer** holds no device key of the room's principals.
- **"Actor"** → acting principal (model, OWN-11); **"addressed"** →
  directed (LANE-12/29/30, PRV-07, scenarios); **"link"** qualified in
  scenarios; OWN-07's "other work" → other tasks; "root-owned" →
  belonging to root; ADM-04's "widening configuration" → loosening.
- **Harness adapter** moves to Components; **Delegation** is one agent
  handing another a delegated task; **Record** names what sits beside
  it (audit log, counters, configuration); **structural event** names
  a hook observation's structural fields; **managed policy** powers
  "among them"; **user turn** is the turn a person's message starts;
  **post** is text a seat writes; **room** is the unit ParleyInn shows and
  shares; **principal surface** names the browser room view.
- **Paired phone:** may read, and reaches its node over B2, or in stage
  one through the principal's tunnel to the room view (§6.3 row 11,
  OWN-16, PRV-10, OWN-17).
- **Names:** `room_join_request`; `parinn room join` and `parinn
  join-request accept` split; `kernel_variable_list`; `parinn export
  --rendering`; `parinn peer` enrolls; `room_pick` names no facilitator;
  "sandbox node" → token-key-only node; check states defined; layout
  words (gutter, sheet, stack, tab, panel) are not concepts.
- **Smaller:** VIEW-12 reconstructs the room's conversation; §6.3 row
  20 "read-only"; retention per room and provenance class; Comparison
  shows each branch's results; "use the model's words"; parked,
  namespace, ingest, reclaimed writer, Adopt, public rooms, the scope
  table's layer names reworded or defined; OWN-07 template wording.

## 3. Invariants

I2, I4, I8 and I10 change, recorded in ADR-2610062100.
