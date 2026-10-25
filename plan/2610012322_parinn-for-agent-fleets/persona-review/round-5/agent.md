# Round 5: Claude, the agent

Target: proposal.md at a3fa2bc. Four round-4 findings resolved, one
partly; no blocker.

## Round-4 findings

- Resolved: the blocker, F1 (PIN-10 restores only the session's own
  principal's trusted pins, others by count); waiting-post addresses
  (VIEW-15); both ids for a merged lane; the §9 note.
- Partly resolved: M2's exit is fixed, but plan 2610022338's phase-1
  test checks "its own lane's pins" after a branch switch. It MUST
  check that the block holds the pins of both the old and the new lane,
  word for word, and names both ids.

## New findings

N1. Important: re-signing a former owner's pin (LANE-11) is undefined.
    Draft: it MUST be a presence-checked owner act (OWN-11) that shows
    the new owner the pin's exact text and records a new pin event
    citing the original's address, audited; until then the pin MUST
    stay count-only in the new owner's restore block.
N2. Important: M2 cannot test "no pin another principal wrote". Draft:
    M8's exit MUST show a co-author's pin reaching the owner's restore
    block only as count, lane id and fingerprint, and M9's exit MUST
    show the old owner's pins reaching the new owner's agents only after
    re-signing.
N3. Minor: PIN-10 MUST state the trust and principal filter before the
    scope rule, and M2's exit MUST check project-wide pins and the count
    line.
N4. Minor: a pin's trust MUST be decided by the deployment mode recorded
    with its creating event, never by the current mode.
N5. Minor: `lane_status` MUST return waiting-post addresses in range
    form, capped, with a count of any left out.

Verdict: serves me; the re-sign step and the pin tests remain.
