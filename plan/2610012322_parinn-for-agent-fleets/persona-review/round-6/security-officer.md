# Round 6: security officer

Target: proposal.md at b7ae7fd. Two round-5 findings resolved, two
partly; three new blockers.

## Round-5 findings

- Partly resolved: agents starting their own surfaces. No launch
  credential, code or phone credential stands in for a widening act,
  OWN-02 drops `parinn-run`, and T21 and the phase-1 test name the
  vectors; but widening now rests on an undefined first enrolment (B1)
  and a session bearer credential (B2).
- Partly resolved: screen reads and injection. `/pin` stays in its
  session, but an injected `/unpin` can drop a lane pin from that
  session, and an injected `/pin` restores as `TrustedText` (I3).
- Resolved: one owner-act list with a drift gate, in form (but see I2);
  the 80-bit receipt code.

## New findings

B1. Blocking: first enrolment is undefined. A same-user agent can talk
    to loopback, write clientDataJSON with any origin and sign with a
    software key that claims user verification; and revoking is a cut
    act, so it can remove the owner's key and enrol its own. Draft:
    enrolling an authenticator while none is enrolled MUST require a
    root-owned managed-policy entry naming that credential, and
    revoking an authenticator MUST be a widening act.
B2. Blocking: a session login authorizes widening acts, its credential
    sits in the browser profile on disk, and clicks can be injected
    (XTEST, System Events, remote debugging, extensions); the RP ID is
    `localhost`, shared by every port. Draft: every widening act MUST
    carry its own user-verifying assertion whose challenge commits to
    the act's canonical digest and whose origin equals `parinn-ui`'s
    exact origin and port; a session login MUST NOT authorize a
    widening act; T21 MUST name browser-profile reads and GUI input
    injection.
B3. Blocking: quarantine is not a cut act. Quarantining a pin's
    creating event, or `--provenance operator`, removes a passkey-set
    constraint; revoking a device key drops its pins. Draft: any act
    that removes a pin from a restore block or stops events being
    accepted MUST be a widening act, whatever verb carries it.
I1. Important: an owner act not backed by an assertion MUST be recorded
    with a mark naming its surface, and its free text MUST be recalled
    as untrusted.
I2. Important: owner acts appear outside the lists (reply, withdrawing
    a request for changes, acknowledgement, choosing a fork,
    reassignment, invite and role change, turning notices on, enabling a
    bridge, retiring a writer, erasure answers). Draft: an unlisted
    owner act MUST be treated as widening, and the gate MUST fail when a
    requirement names an owner act the glossary does not classify.
I3. Important: a `/unpin` typed at the harness prompt MUST end only pins
    that session's own `/pin` created, and such a `/pin` MUST restore
    marked as harness input.
I4. Important: a read verb whose output is not a terminal MUST envelope
    record content, log the call and taint the session under SEC-13.
Minor: SEC-22's "require an authenticator" contradicts D5; OWN-06 still
lists `parinn-run` as an answering surface; the phase-1 test lacks a
stolen session credential, a forged assertion from an unenrolled key,
quarantining a pin and `/unpin` of a lane pin.

Verdict: the three-class shape is right, but first enrolment, the
session bearer and quarantine-as-cut each still let a same-user agent
widen with no hardware assertion.
