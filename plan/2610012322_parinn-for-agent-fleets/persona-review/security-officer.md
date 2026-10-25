# Persona review: security officer

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

## Blocking

1. Erasure leaves hashes that confirm the erased content: purge keeps
   each row's seq, hash and prev_hash, and REC-10 hashes the canonical
   event, which carries inline text; signed segments carry that hash to
   every peer. The traces also disagree on the key (tenant versus
   lane). Draft: the canonical encoding a chain hash or signature
   covers MUST commit to event content only through a keyed hash, so
   that no retained or replicated value confirms purged content.
2. Peer-derived bytes enter the model unasked: the per-turn post notice
   carries peer-chosen names; INJ-10's "sanitized writer names" decides
   trust from text. Draft: no event from another origin MUST cause any
   bytes to enter a session's context without a recall call; any
   opt-in notice MUST contain only locally generated strings.
3. Untrusted text is laundered into operator instructions (late
   approvals quoting agent-chosen commands; "Re-run here" executing a
   co-author's recorded command). Draft: text delivered as operator
   MUST be typed by the owner or be a fixed template that references
   ids, and MUST NOT embed fields from untrusted events.
4. Components open sockets beyond the B0–B3 register: `parinn import
   https://…`, terminal takeover and a phone view over B2, the forge,
   ntfy, Matrix and email bridges, CI and git carriers, `parinn run`
   with os/exec. Draft: every process and protocol MUST appear in the
   boundary register, and B2 MUST carry only signed segments.
5. Trust across nodes contradicts itself: C5 makes other nodes'
   events untrusted, while steering treats the owner's other devices
   as operator. Draft: an imported event MUST NOT be trusted unless an
   I2-reviewed requirement names its key, scope and maximum rule level.

## Important

6. The UI session cookie leaks to other loopback servers (cookies are
   not port-scoped); keep the credential in origin-scoped storage.
7. A same-user agent is the real adversary, but §6.1 excludes it.
   Draft: loosening a rule, answering a hand-off, landing and
   force-pushing MUST require user presence.
8. Verification is decided from agent-written fields (LANE-04
   "external", LANE-06 "local_run or stronger").
9. Erasure end to end is undefined: T18's control is documentation, a
   compliant peer's gap looks like suppression, and a co-author's
   erasure request has no path.
10. Integrity cannot be verified alone after an incident: open segments
    are unsigned, nothing anchors checkpoints off the node, no key
    revocation or rotation requirement.
11. Phase 1 builds a listening socket, and its gate does not wait for
    the security review of the I4 and SEC-01 changes.
12. The traces give the same ids to different controls, and the UX docs
    cite the backward numbering.

## Minor

13. Presence travels as unsigned hints, bypassing SEC-22.
14. The peer's LAN listener states no interface binding.
15. Desktop notifications push untrusted lane text into the OS.

Verdict: the direction, the boundary model and the endorse design are
strong, but findings 1, 2, 4 and 5 break my give-up conditions; the
proposal does not serve me until they are fixed.
