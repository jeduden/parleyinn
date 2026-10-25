# Round 3: returning owner

Target: proposal.md at 77ea1d9. Three round-2 findings resolved, four
partly, two not; no blocker.

## Round-2 findings

- Resolved: failed capture and lanes started with capture broken
  (VIEW-08, VIEW-09 footers); the D7 veto (CLI views stay P1).
- Partly resolved:
  1. The weekend fixture is in plan 2610022338's RED test, but nobody
     says what it holds. Draft: it MUST contain a failed capture, a
     session in no lane, a missing segment, a late ingest, a
     quarantine, a purge, an unsynced writer and an altered segment,
     and the test MUST fail when Catch up omits any.
  2. Proof nothing was altered: the prefix check and off-node receipts
     are in; see N1.
  3. Quiet over a dead lane: fixed by VIEW-04 and §8.2; see N3.
  4. Unsigned tails: short now, but VIEW-10's "proves the record
     unchanged" overclaims for the unsigned tail, and §10.11 still
     quotes "signed record".
- Not resolved: §8's words are cited by VIEW-04, -05, -10 and -14 but
  never made normative, and Catch up has no line order or first-screen
  cap. Draft: the SRS MUST carry §8's status words, marks and Needs you
  order, and Catch up MUST show integrity and capture gaps, Needs you,
  failures, then finished work, in that order.

## New findings

N1. Important: VIEW-08's boundary head has no stated home; one in
    `PARINN_HOME` can be rewritten with the record. Draft: Catch up MUST
    take boundary heads from the newest receipt outside `PARINN_HOME`
    when one exists, name it, say "checked against ParleyInn's own copy
    only" otherwise, and offer to write a receipt when the owner leaves.
N2. Important: legitimate removals look like nothing happened. Draft:
    Catch up MUST list, each linked to its event, every tombstone,
    quarantine, release, identity rebind, new or re-minted writer,
    refused segment and lane merge or reassignment since the boundary.
N3. Important: `unrecorded` has no source, and sessions in no lane
    appear only in Catch up. Draft: `unrecorded` MUST be derived from a
    transcript longer than its ingested position, or from a `parinn-run`
    launch with no hook event, and each such session MUST appear on
    Fleet and in `parinn lanes` as an unattached row.
N4. Minor: rows outside tables (REC-24, REC-25, VIEW-19), the stray
    heading in §10.12, and §11.2 says D6 where it means D7.
N5. Minor: the time filter MUST use the writer's recorded time, marked
    as asserted by the writer.
N6. Minor: D7's alternative contradicts the guarantee in the same row;
    M7 MUST list the CLI forms of VIEW-08..11 as their own exit.

Verdict: now serves me with no blocker, but N1, N2 and the
non-normative vocabulary stop me proving nothing was altered or removed.
