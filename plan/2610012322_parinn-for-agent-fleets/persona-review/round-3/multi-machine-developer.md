# Round 3: multi-machine developer

Target: proposal.md at 77ea1d9. Five round-2 findings resolved, two
partly; one new blocker.

## Round-2 findings

- Resolved: shallow-clone identity (LANE-02); writer certificates
  (PRV-10); seals at every hook (REC-19, PEER-04); concurrent lanes
  merge (LANE-01); encrypted carrier (PEER-08). One gap: PEER-06 never
  says the token carries the bound identity LANE-02 takes from it.
- Partly resolved:
  1. Sandbox tail: a sandbox that never synced is invisible, and
     PEER-05's "last sealed `seq`" is unknowable to a peer, which only
     knows the last seq it received.
  2. Unattended start: nothing names what starts `parinn-peer` in the
     sandbox.

## New findings

N1. Blocking: a sandbox cannot get its writer certified or deliver its
    segments unless the one issuing peer is online. Draft: a sandbox
    token MUST carry a delegation, scoped to one project and the
    token's expiry, with which the sandbox certifies its own writer key
    without reaching any peer, and MUST list every enrolled peer
    address and the git-carrier remote it may deliver to.
N2. Important: REC-24's node identity MUST come from a value a cloned
    image or restored snapshot cannot carry over, checked before the
    first append after start.
N3. Important: ParleyInn MUST mark a writer retired, when its token expires
    or by an owner act, and show its lost tail as a gap from the last
    received seq; the issuing node MUST show "enrolled, never synced"
    for a token no writer used.
N4. Important: every reference to a merged lane's id (pins, metadata,
    approvals, recall parameters, receipts) MUST resolve to the
    surviving lane, and both lanes' pins MUST stay active; LANE-01 must
    say whether the carried or the derived lane id wins.
N5. Minor: M8's "at most its unsealed tail" overclaims; PEER-04's bound
    MUST hold across one relay hop through an enrolled peer.
N6. Minor: this plan's phase-1 RED test MUST create one branch's lane
    on both writers and derive one lane in both orders; it still says
    "origin seq".
N7. Minor: rows outside tables; the stray heading in §10.12.

Verdict: much closer, but N1 trips "a sandbox's history vanishes" and
N2 trips "a partition needs manual repair".
