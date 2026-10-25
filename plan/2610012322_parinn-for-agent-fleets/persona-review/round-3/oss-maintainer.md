# Round 3: open-source maintainer

Target: proposal.md at 77ea1d9. Six round-2 findings resolved, three
partly, one declined with a reason I accept; three blockers.

## Round-2 findings

- Resolved: the accepted-change link (publisher-asserted marks, LANE-06
  local); lane-id collision (REC-23, but see B); withheld headers
  (SEC-26, REC-23); witness-run sandbox (OWN-18); recall taint
  (RCL-10); fingerprints for unknown keys (VIEW-07, LANE-15).
- Declined, accepted: none of my journeys in v1 (§10.12).
- Partly resolved:
  1. Blocking, real versus fabricated: the cross-signature runs the
     wrong way. Contributors' public git keys are on their forge
     profiles, so a forger can mint an owner key, cross-sign the
     victim's public key and pass the match; and nothing ties a foreign
     bundle's writer keys to its owner key. Draft: a bundle MUST carry
     a statement signed by the contributor's git commit-signing private
     key binding its owner and writer keys, and LANE-15 MUST count a
     match only from that statement.
  2. Blocking, sharing path: SEC-26 still opens with "Publishing a lane
     (B3)", and the B0 `parinn export --bundle|--report` is not bound to
     it. Draft: every `parinn export` of a bundle or report MUST pass
     SEC-26's review step, publish-only rules and fail-closed scan.
  3. Reject with a reason: no verb, not in OWN-11 or OWN-12; PRV-07 is
     a SHOULD and import does not compute flags locally. Draft: import
     MUST compute PRV-07 flags locally and ignore any the bundle
     carries; rejecting a foreign lane MUST be an owner act with a verb.

## New findings

A. Blocking: `parinn ingest --path` on a contributor's transcript stores
   it under my own writer, so PRV-02 trusts the stranger's `user` lines
   in interactive mode and they land in my lane and widened recall.
   Draft: events ingested from a transcript this node's hooks did not
   report MUST be untrusted whatever their class, and MUST be held only
   as a foreign lane.
B. Important: refusing any bundle whose writer I hold blocks an updated
   bundle or a second pull request. Draft: import MUST accept a bundle
   that continues a held foreign writer's chain, MUST refuse and audit
   one that forks it or claims a lane or writer of this tenant, and
   MUST audit either outcome.
C. Minor: the stray heading in §10.12; and §10.12 says the pitch names
   bundles as next, which it does not.
D. Minor: rows outside tables (REC-24, REC-25, VIEW-19, OWN-21).
E. Minor: phase 1's gate stays closed while A, #1 and #2 stand.
F. Minor: purging a foreign lane MUST stay possible while a landed link
   exists, and the link MUST then read `not proven` with reason
   `evidence-removed`.

Verdict: the trust model mostly serves me, but A, #1 and #2 trip my
give-up conditions.
