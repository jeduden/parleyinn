# Round 3: fleet developer

Target: proposal.md at 77ea1d9. Seven round-2 findings resolved, four
partly or not; one new blocker.

## Round-2 findings

- Resolved: allow once is not a loosening (OWN-11); overlap items clear
  on acknowledgement (LANE-13, VIEW-05); branch moves (LANE-01); Ready
  for review (OWN-21); Bash edits via checkpoints and `unbound`;
  `parinn open` prints a link; interrupt moved to `I`.
- Partly resolved:
  1. `parinn-run -- <harness>` is still a step at every launch, with no
     wrapper set up once; an unavailable control names no location.
  2. Speed gate: phase 1 now has the budgets, but the `Notification`
     hook added in §6.14 has no NFR-01 row.
  3. S9: OWN-15 lists that case, but not OQ-17 removing hook decisions.
- Not resolved: answering from the shell still needs a code that only
  `parinn-ui` or `parinn-run` shows (finding A widens it).

## New findings

A. Blocking: no owner act works with only the core installed. OWN-11
   allows pin changes without an authenticator only from the CLI under
   OWN-12, and OWN-12 needs a code from `parinn-ui` or `parinn-run`. So
   the P0 `parinn pin add` (PIN-01) needs an optional B1 binary; PIN-01's
   `/pin` path contradicts OWN-11; with B1 and `parinn-run` locked off
   by policy, no owner act is possible. Against NG5 and the pitch.
   Draft: every owner act MUST be completable with only the core
   installed and no B1 binary running; PIN-01's `/pin` MUST stay valid
   or be retired explicitly.
B. Important: REC-24, REC-25, VIEW-19 and OWN-21 sit outside their
   tables after blank lines; SEC-30 has empty columns; §10.12 has a
   stray heading. Draft: every id MUST sit in its family's table with
   Ver, Inv and Personas filled.
C. Important: OWN-15 MUST list answering held requests from another
   surface whenever S9 or the OQ-17 decision removes it.
D. Important: NFR-01 MUST give every §9.1 hook a p95 row,
   `Notification` included.
E. Minor: a minted phone credential counts as a surface that can
   answer, so a request can be held with nobody connected. Draft: hold
   only while an answering surface is connected.
F. Minor: OWN-16 MUST say whether a phone may answer allow for session,
   and under which presence check.
G. Minor: plan 2610022338's phase 1 promises steering background agents
   and says "a local run"; the pitch still has the sentence §10.11 says
   was replaced.

Verdict: most of round 2 fixed, but A and the shell-answer leftover
trip "the UI is a required step".
