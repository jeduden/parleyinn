# Re-review: multi-machine developer

Target: proposal.md at ff6fa8e.

## Earlier findings

- Resolved: #3 liveness (PEER-04, but see C), #5 B2 confidentiality
  (SEC-24), #6 metadata merge (LANE-09), #8 git fallback (PEER-08, M8),
  #9 and #11 sandbox token (PEER-06), #10 and #12 trust marks and
  REC-18 meaning (§6.0, §8.6, §9).
- Partly resolved:
  - #1 project identity (LANE-02, root commit): shallow clones break
    it (A).
  - #2 sandbox tail (REC-19, PEER-05, M8 exit): nothing records receipt
    by a peer; a sandbox that never dialled out vanishes without a
    trace. Draft: the node that issues a sandbox token MUST record the
    expected writer and show "enrolled, never synced" or "last seq
    received" until the token expires.
  - #4 owner key hierarchy (PRV-10, PEER-07): the chain stops at device
    keys (B).
  - #7 unattended start (PEER-01): nothing says what starts parinn-peer
    in the sandbox; document an entrypoint.
- Not resolved: none, but all four journeys are P2 and arrive in M8.

## New findings

A. Blocking: a shallow clone has no root commit, so a sandbox splits
   the project; "first branch ParleyInn observes" can differ between nodes.
   Draft: a node whose clone lacks the root commit MUST take the project
   identity from its enrollment token or a peer's bound identity, and
   MUST NOT mint a different one.
B. Important: sandbox and session writer keys cannot chain to the owner
   key while the owner key is offline. Draft: an enrolled node MUST be
   able to hold a delegated certificate scoped to issuing segment-only
   writer certificates for one repository; every writer key MUST chain
   owner → device → writer.
C. Important: PEER-04's 5 s from write contradicts sealed-only B2 with
   30 s seals. Draft: while a peer is connected, a writer MUST seal at
   least as often as PEER-04's bound, or B2 streams unsealed events
   shown as unsigned.
D. Important: a partition mints two lanes for one branch, and nothing
   merges them. Draft: lanes created concurrently for the same project
   and branch MUST merge after a partition into one lane by a
   deterministic rule, keeping both creation events.
E. Minor: git-carrier segments sit in plaintext on the remote; offer
   encryption to the owner's enrolled keys.

Verdict: most blockers fixed in text; A brings the split back in
sandboxes, B and D undercut partition merging, and nothing ships before
M8.
