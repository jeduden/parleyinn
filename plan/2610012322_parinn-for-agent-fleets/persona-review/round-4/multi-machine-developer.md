# Round 4: multi-machine developer

Target: proposal.md at ca209ea. Eight round-3 findings resolved, one
partly; no blocker.

## Round-3 findings

- Resolved: the token carries the bound identity (PEER-06, LANE-02);
  the sandbox tail (PEER-05, PEER-06, M8); the entrypoint (PEER-01);
  node identity (REC-24); retired writers and unused tokens; merged
  lane ids (LANE-01, PIN-10); the relay bound (PEER-04, M8); the
  phase-1 RED test; document defects.
- Partly resolved: offline sandbox certification. PEER-06 carries the
  delegation, but PRV-10 admits only owner → device → writer, so the
  self-certified writer has no admitted link (B).

## New findings

A. Important: SEC-10 forbids inheriting the token from the environment
   by default, and PEER-01 does not say how it gets in. Draft: the
   tenant MUST be able to choose, as a recorded opt-in, an environment
   secret or a mounted file; `parinn status` MUST name the source; ParleyInn
   MUST remove the token from the environment of child processes.
B. Important: PRV-10 MUST admit a token key as a link between a device
   and a writer, certified by a device key holding the repository
   delegation and limited to the token's project, lanes and expiry;
   revoking the token MUST revoke every writer it certified.
C. Important: a peer MUST accept a retired writer's segments that
   continue its chain without a fork, mark them delivered after
   retirement and close the gap; only a revocation refuses them.
D. Important: a token that names a lane to continue MUST carry that
   lane's active pins as signed `operator` events; until a peer is
   reached, the restore block MUST say later pins may be missing.
E. Minor: a peer connection MUST carry sealed ranges and owner acts in
   both directions, whichever side dialled it.
F. Minor: a connected ephemeral node MUST offer each sealed range as
   soon as it is sealed.

Verdict: serves me; C would lose work after a partition if peers refuse
post-expiry segments, and B leaves offline certification unadmitted.
