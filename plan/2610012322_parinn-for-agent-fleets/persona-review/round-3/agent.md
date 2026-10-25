# Round 3: Claude, the agent

Target: proposal.md at 77ea1d9. Seven round-2 findings resolved, one
partly; one new blocker.

## Round-2 findings

- Resolved: away policy never starts a turn (OWN-07, LANE-14, OWN-09);
  parked approval on the plain-hook path (OWN-07); `lane_status` closed
  fields (VIEW-15); endorsement source (OWN-08); random lane ids
  (LANE-01); restore block names its lane (PIN-10); PIN-03's address.
- Partly resolved: chain-state words. RCL-09 cites §8.5, which has no
  `unverified`; §9.3 still lists an `imported` field, not `origin`.

## New findings

N1. Blocking: a branch switch silently drops lane pins. LANE-01 moves a
    session's later events to the new branch's lane; PIN-10 restores
    only the current lane's pins. One `git checkout -b` in Bash loses
    every lane pin at the next compaction, with no notice. Draft: a
    restore block MUST hold the pins of every lane the session has
    belonged to since it started, plus the project-wide pins; a lane
    change MUST NOT remove a pin from a running session's restore block
    unless the owner removes it; any pin left out MUST be stated by
    count and lane id and audited.
N2. Minor: when lanes merge, pins scoped to the merged-away lane MUST be
    restored as pins of the surviving lane, and the next restore block
    MUST name both ids.
N3. Minor: every held-request id shown to an agent MUST resolve through
    `get` to the request's event, and the OWN-07 template MUST go only
    to the session that made the request.
N4. Minor: petnames are trusted on pushed paths (INJ-10, OWN-08) and
    banned on the pulled path (VIEW-15). Draft: classify a petname once
    and apply that class on every agent-facing path.
N5. Minor: plan 2610022338's phase-1 RED test MUST show that neither
    harness's events reach the other's context except through enveloped
    recall, and that each restore block holds its own lane's pins.
N6. Minor: the stray heading in §10.12.

Verdict: mostly serves; N1 trips "constraints come back missing".
