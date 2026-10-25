# Domain model: twenty-seventh round of blind reviews, merged

Three blind domain-model agents reviewed the model after the
twenty-sixth-round decisions were applied (commit 2015c0c): one across
the whole model (X), one on principals, harness facts, places, seats,
the record and pins (A), and one on acts, trust, git and components (B).
They read every file from disk. X found no contradiction for the first
time. This note keeps the needs-fix findings, each checked against the
files. Each question takes its recommended option, but the I10 wording
in section 4.

## 1. Questions and decisions

| #   | Question                                                                  | Found | Decision                                                                                                                                       |
| --- | ------------------------------------------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Is a node clone a new node?                                               | 2/3   | Yes: a **node clone** has its own node identity, device key and device seat, mints new seat keys, and shares the carried-over personal room.   |
| Q2  | Is the personal room's visibility fixed?                                  | 1/3   | Yes: it stays private; no invite, admission or visibility change applies to it (LANE-17).                                                      |
| Q3  | Should `stat_list` return its counts enveloped and record a recall event? | 1/3   | Yes, in an envelope marked structural, so every RCL-01 tool is a recall tool.                                                                  |
| Q4  | Does PEER-01's environment setting turn on a publish component too?       | 1/3   | Yes, for the git-carrier remotes its access token names; minting that access token is the widening act.                                        |
| Q5  | Which process records and which starts a component turned on?             | 1/3   | The CLI records the widening act; the component's own entry point runs only while that act stands, started by the person or a service manager. |
| Q6  | Which provenance does a witness check's record take?                      | 1/3   | `structural`: the command by commitment, the exit status and the tree hash.                                                                    |
| Q7  | Are confirming an untrusted command and starting a witness check one act? | 1/3   | Yes: starting a witness check confirms the command (OWN-18).                                                                                   |
| Q8  | Is revoking a device or a peer widening?                                  | 1/3   | Yes, with the reason stated: it can drop pins and stop acts arriving.                                                                          |
| Q9  | Should `room_create` stay an exception to `room_<act>`?                   | 1/3   | Yes, recorded in the naming rule.                                                                                                              |
| Q10 | Should I10 say "run and integrity statuses"?                              | 1/3   | Not now: the model's reading stands; the wording waits for the stakeholder's next invariant ADR.                                               |

## 2. Fixes that need no decision

- **Model:** **closed path** defined (the term I2 and I4 use);
  quarantine leaves landmark counts; the `ingested` mark names an
  ingested run; a backup keeps no seat, device, token or at-rest key;
  peers exchange sealed ranges; the ingest seat shares the add;
  **turn trigger** and **pin author** defined; the service-account
  certificate's relaying mark sits under Principal key; Unpin, List
  removal and Held request reworded; who unbars; "principal surface"
  in full; "harness sessions"; the domain-model agent says "command and
  tool names".
- **SRS and scenarios:** §9.7.6's launcher-carried line; `parinn
  head-receipt` and `purge-receipt`; quarantine selectors gain span and
  derived artifact; "still a member" for seat ids; branch links "void and
  shown"; LANE-21's intent version for a result no turn produced; §9.5
  rows state their class; "result" and "capability" outside their
  meaning; "principal surface"; undefined words ("ceiling",
  "standalone", "invitee", "query compiler").

## 3. Optional, not chased

`--range` beside the address range; `parinn correction send
--worktree-checkpoint` filing a retry; `pin.max_restore_model_tokens`.

## 4. Invariants and reviews

No invariant changes. Waiting for the stakeholder:

- **I4, commit trailers** (round 23).
- **Signing browser principal acts** (round 26).
- **I10, "statuses"** (Q10): reword to "run and integrity statuses" in
  the next invariant ADR, if the stakeholder agrees.
