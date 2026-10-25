# Re-review: security officer

Target: proposal.md at ff6fa8e. Seven of fifteen earlier findings
resolved, eight partly; two new blockers.

## Earlier findings

- Resolved: #2 notices (INJ-10 off by default, local strings only), #5
  cross-node trust (PRV-02, PRV-10), #6 UI credential (SEC-20), #8 no
  `external` evidence (LANE-05), #12 one id set (§9), #13 signed
  presence hints (PEER-09, SEC-24), #15 (VIEW-17).
- Partly resolved:
  1. #1 confirmable content: REC-17 fixes the chain, but PIN-03 still
     stores a pin's text hash, ADM-07 never names the sealed segment
     files CON-05 makes the record, and nothing erases the HMAC payload
     reference of REC-09.
  2. #3 steering: OWN-03, OWN-04, OWN-07 and OWN-18 close the path;
     CLI endorse reopens it (new finding 1).
  3. #4 boundaries: §7 does not say what `parinn-run`'s children may
     reach, puts `parinn-peer` in both B2 and B3 against CON-04, and
     names a "bridge process" ENG-16's allow-list does not cover.
  4. #7 presence: OWN-11 leaves out endorse (OQ-18), pin, purge,
     quarantine release and peer enroll or revoke.
  5. #9 purge: SEC-30 exists, but copies survive (new finding 2).
  6. #10 tamper evidence: SEC-10 keeps the writer key at `0600`, which
     a same-user agent can read to re-seal a rewritten chain; nothing
     anchors receipts off the node.
  7. #11 review gate: §11.3 only recommends; plan 2610022338's phase 1
     has no such gate, and "prototype" is undefined.
  8. #14 binds: SEC-24 names no default interface and does not refuse
     wildcard binds.

## New findings

1. Blocking: CLI owner acts escape OWN-12, which guards only `parinn
   answer`, `steer`, `rules` and `land`. From Bash an injected agent
   can run `parinn endorse` (a co-author's post reaches a sibling agent
   as owner text), `parinn pin add` (TrustedText in every restore block
   of the lane) or `parinn purge` (erase its own tracks). Draft: every
   owner act the glossary lists, on every surface, MUST pass OWN-12,
   and endorse, pin, purge and quarantine release MUST also pass
   OWN-11.
2. Blocking: erasure leaves copies in other writers' logs: OWN-08's
   exact sent text, recall results in the recalling agent's transcript
   (RCL-11), re-ingestable harness transcripts, git-carrier refs on
   the owner's remote. SEC-30 purges only the writer's own events.
   Draft: purging an event MUST erase or tombstone every stored copy of
   its content in any writer's log, MUST block re-ingestion of the
   purged range, and MUST report copies ParleyInn cannot erase.
3. Important: I2 and OWN-09 say ParleyInn writes to an agent only on an
   owner act, yet restore blocks, INJ-10 notices and the OWN-07
   timeout deny write without one. Draft: I2 MUST list each permitted
   automatic write as a closed set.
4. Important: `parinn-run` children sit outside every boundary, and
   witness runs execute commands that came from untrusted events.
   Draft: the register MUST list processes `parinn-run` starts as their
   own row, and witness runs MUST run network-denied by default.
5. Important: a stolen key can mint events that cite only old heads,
   concurrent with its revocation and so accepted under PRV-10. Draft:
   after holding a revocation, a node MUST refuse every event by that
   key not already held or covered by a seal it held before.
6. Important: PEER-02 replicates whole lanes, tool results and REC-20
   diffs included, to co-authors with no review; PEER-08 pushes
   plaintext segments to a forge. Draft: sharing a lane with another
   tenant MUST pass a review step like SEC-26, and git-carrier segments
   MUST be encrypted to enrolled keys.
7. Important: B3 credentials (forge tokens, git-remote credentials,
   ntfy, Matrix or email) have no storage rule; SEC-10 binds only the
   core.
8. Minor: VIEW-15 returns `lane_status` outside an envelope; titles,
   labels, branch names and lane ids are peer-chosen; a petname should
   default to a fingerprint.
9. Minor: row 13 (LAN discovery) does not say what it advertises.

The target holds no text addressed to the reviewer.

Verdict: serves in structure; findings 1 and 2 still trip the give-up
conditions.
