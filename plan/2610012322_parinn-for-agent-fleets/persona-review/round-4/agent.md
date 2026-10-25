# Round 4: Claude, the agent

Target: proposal.md at ca209ea. Five round-3 findings resolved, two
partly; one new blocker.

## Round-3 findings

- Resolved: held-request ids (OWN-07); petnames in the rows (INJ-10,
  OWN-08, VIEW-15); the phase-1 isolation test; the stray heading;
  chain-state words (§8.5 `unverified`, §9.3 `origin`).
- Partly resolved:
  1. Branch switch: PIN-10 is fixed, but M2's exit still says "a
     restore block holds only its lane's and the project's pins", and
     plan 2610022338's phase-1 test never switches branch.
  2. Lane merge: pins carry over, but the restore block names only the
     surviving id.

## New findings

F1. Blocking: another tenant's pins can reach a restore block. LANE-11
    keeps the old owner's pins restoring after a handover, and PIN-10
    restores every pin of the session's lanes with no check of author
    or trust, so a co-author's agent gets the owner's pins too. Against
    I2, OWN-01 and the pitch. Draft: a restore block MUST hold only
    pins whose creating event is trusted on this node under PRV-02 and
    was written by the session's own principal; any other pin in the
    session's lanes MUST be stated only by count, lane id and key
    fingerprint, with no text, and audited.
F2. Important: M2's exit and plan 2610022338's phase-1 test MUST show
    that after a branch switch and a compaction the restore block holds
    every pin of every lane the session has belonged to, word for word.
F3. Minor: `lane_status` MUST return the recall address of each waiting
    post addressed to the caller's lane, so the agent can `get` exactly
    that post, enveloped.
F4. Minor: a restore block holding pins of a merged lane MUST name both
    the merged and the surviving lane ids.
F5. Minor: the §9 note for INJ-10 still says "petnames only".

Verdict: mostly serves; F1 trips "untrusted text arrives looking like
the user's instruction".
