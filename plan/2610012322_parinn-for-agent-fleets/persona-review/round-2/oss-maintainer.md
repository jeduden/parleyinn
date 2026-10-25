# Re-review: open-source maintainer

Target: proposal.md at ff6fa8e.

## Earlier findings

- Resolved: #1 Re-run here (OWN-18, SEC-29), #4 https import (REC-23,
  R1), #6 trust by key (§6.0, PRV-09, PRV-02, INJ-10), #3 redaction on
  import (REC-23).
- Partly resolved:
  - #2 real versus fabricated (blocking): LANE-15's key match cannot
    succeed, since writer keys are per node and never tied to the git
    signing key. Draft: a bundle MUST carry a statement, signed by the
    contributor's git commit-signing key, that binds the bundle's writer
    keys, so LANE-15's match can be checked offline against the clone.
  - #5 sharing path and #3 private paths: SEC-26 applies only to B3
    publishing; a plain-file bundle through B0 export skips it. Draft:
    every bundle or report export MUST pass SEC-26's review step and its
    publish-only rule set, whatever carries it.
  - #8 reject with a reason: reject is not an owner act and has no verb;
    PRV-07 is a P1 SHOULD; flags could arrive forged. Draft: import MUST
    compute PRV-07 flags locally for every imported event and ignore any
    flags the bundle carries.
  - #7 accepted change link: LANE-15 says marks are publisher-asserted,
    LANE-06 recomputes them locally. Draft: evidence and proof marks for
    a foreign lane MUST be recomputed locally, never imported.
- Not resolved: none.

## New findings

1. Important: an imported lane can claim one of my lane ids. Draft:
   import MUST refuse and audit a bundle whose lane id matches a lane
   this node holds, or file it as a separate foreign lane.
2. Important: a bundle's chain cannot be checked across withheld or
   redacted content. Draft: a bundle MUST keep the chained header of
   every withheld or redacted event, and the view MUST show them as
   withheld with the chain intact.
3. Important: a witness run executes a stranger's code as my user.
   Draft: a witness run of a foreign lane MUST run with network and the
   user's home denied, or the view MUST say before confirmation that it
   does not.
4. Important: none of my journeys works in v1 (RCL-10 in M8; REC-23,
   SEC-26, LANE-15 in M9 behind their own proposal and ADR).
5. Minor: recalling a foreign lane should taint the session like a post
   (OQ-28, RCL-10).
6. Minor: a stranger's key with no petname should show a fingerprint,
   never a sender-chosen name (VIEW-07).

Verdict: the trust model serves me, but I cannot tell a real lane from
a fabricated one until #2 and #5 are fixed.
