# Round 3: reviewer

Target: proposal.md at 77ea1d9. Six of ten round-2 findings resolved,
four partly; no blocker.

## Round-2 findings

- Resolved: presence on approve and request changes (OWN-11, OWN-12,
  LANE-07); bound human (glossary, LANE-07); results bound to their
  tree (LANE-05 `unbound`); `contains approved diff` before LANE-07
  (LANE-06); the evidence rank (§8.3); the pitch's re-runs and CI.
- Partly resolved:
  1. Forge hand-off (LANE-08): nothing says what ParleyInn derives after
     it, so every handed-off landing is flagged (N1).
  2. The author's own agent: the author rule now covers it, but OQ-22
     stays open (N2).
  3. Pitch overclaims: pitch.md still says "a pull request in all but
     its interface", which §10.11 claims was replaced; its change
     table still cites "local run" and LANE-04.
  4. Request-changes follow-up: VIEW-19 shows the delta, but see N3
     and N5. §8.3 says witness runs count "unless the policy excludes
     it", and LANE-07 has no exclusion clause.

## New findings

1. Important: the forge hand-off has no derived state. Draft: when a
   lane policy hands approvals to the forge, ParleyInn MUST show each forge
   approval as `asserted` with its forge and time, and MUST NOT flag a
   landing it covers as `landed-not-approved`.
2. Important: OQ-22 invites an exception to OWN-11. Draft: a verdict
   MUST count toward the gate only when a human principal signs it
   under OWN-11; an agent's verdict MUST be shown as a comment.
3. Important: "Changes requested" has no blocking rule. Draft: a
   current request for changes MUST block landing until the same bound
   human approves a later head or withdraws it.
4. Important: plan 2610022338's phase-1 RED test never checks evidence
   classes, and says "a local run or the canonical CI". Draft: the RED
   test MUST fail until every result shows its LANE-05 class, and a
   result known only from text or tool output shows `claim`.
5. Minor: a request for changes MUST create a LANE-12 request
   addressed to the lane owner's agents.
6. Minor: a `from checkpoint` hunk MUST count as authored by the bound
   human of the checkpoint's writer key.
7. Minor: a blank line splits VIEW-19 from the VIEW table.
8. Minor: wrapping turned "U7 #4" into a stray heading in §10.12.
9. Minor: the approval's signing key is named three ways (OWN-02
   writer key, LANE-07 approver's key, OWN-08 device key).

Verdict: serves journeys 1 to 3 and the landed trace; nearest give-up
conditions are forge disagreement (N1) and OQ-22 (N2).
