# Re-review: reviewer

Target: proposal.md at ff6fa8e. Nine of eleven earlier findings
resolved, two partly, pitch fix not yet applied.

## Earlier findings

- Resolved: B1 evidence set (LANE-05, §8.3), B2 author rule (LANE-07),
  B3 landed link (LANE-06, `parinn why commit`), I5 label and
  side-by-side (LANE-08), I6 confirmed re-runs (OWN-18, SEC-29), I7
  strict landing by default (LANE-07), I8 hunk attribution (LANE-04),
  minors 9–11.
- Partly resolved:
  1. Double review with a forge: forge approvals are untrusted, so a
     protected repo still needs two approvals. Draft: with a forge
     configured as authoritative, a lane policy MUST be able to hand
     approvals to the forge, so a reviewer approves exactly once.
  2. The author's own agent: OQ-22 leaves agent verdicts open. Draft:
     an approval MUST NOT count when it comes from an agent session, or
     from any key whose principal authored, suggested or endorsed a
     change in the approved range.
  3. The pitch still sells the gate without "Next" (pitch.md).

## New findings

1. Important: an unsandboxed agent can sign an approval as its user;
   OWN-11 and OWN-12 omit approve and request-changes. Draft: an
   approval or request-changes MUST require a presence check (OWN-11),
   or the terminal path of OWN-12.
2. Important: "bound human" is undefined. Draft: the author rule MUST
   treat every key that chains to one owner key as one human, and the
   gate policy MUST name approvers by owner key.
3. Important: a run is not tied to its tree mid-turn. Draft: a result
   MUST carry the exact tree it ran on, and MUST be marked unbound when
   edits since the last checkpoint make that tree unknown.
4. Important: request changes then see the follow-up has no
   requirement. Draft: for each request-changes verdict, ParleyInn MUST
   show its delivery state (LANE-12) and the diff and events since that
   verdict's head.
5. Minor: §8.3 and LANE-07 disagree on whether a witness run counts by
   default; "own run or stronger" assumes an undefined ranking.
6. Minor: `contains approved diff` is proven in LANE-06 (P1) while
   approvals exist only with LANE-07 (P2).
7. Minor: the new pitch wording promises reviewer re-runs and CI as
   current; mark them Next.

Verdict: v1 serves journeys 1 and 2 and the landed trace; approving and
requesting changes still need presence-checked signatures, a defined
human and single review with a forge.
