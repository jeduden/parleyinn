# Persona review: reviewer

Target: pitch v7, plan 2610012322 (plan, traces, ux/) and plan
2610022338, at 618fafb.

## Blocking

1. Five vocabularies for "what verified it", and the contract draft is
   the weakest: LANE-04's "external" rests on tool output any injection
   can forge; LANE-06 counts "local_run or stronger", the opposite of
   the review design; phase 1 promises canonical CI in standalone mode,
   which nothing can back. Draft: each result MUST carry exactly one
   evidence class from a single SRS-defined set that separates claim,
   own-node run, non-author witness run and CI attested for the exact
   SHA, and a class resting only on tool output MUST NOT satisfy a
   required check.
2. The author-cannot-approve rule exists only in a UX draft; solo
   self-approval is unsettled; a reviewer's applied suggested patch
   makes the reviewer approve their own code. Draft: an approval MUST
   NOT count toward the gate when its key or bound human authored,
   suggested or endorsed any change in the approved range, including
   patches applied on their behalf.
3. The link from a landed commit to its lane has no requirement id, and
   three proof vocabularies compete. Draft: ParleyInn MUST derive, from the
   record and the local clone alone, the lanes behind any landed
   commit, each with one proof class from a single defined set, and
   MUST show "not proven" with a typed, counted reason otherwise.

## Important

4. The merge gate is P2 and M9, but the pitch sells it now; the pitch
   still says "a pull request in all but its interface" and "only the
   owner instructs agents".
5. With a forge, I review twice. Draft: with a forge configured, ParleyInn
   MUST label every ParleyInn verdict that does not count toward the
   forge's required reviews, and the gate panel MUST show both verdicts
   side by side whenever they differ.
6. "Re-run here" runs untrusted recorded text. Draft: a witness run
   MUST show the exact command, hidden characters visible, for the
   reviewer's confirmation, run it outside any agent context, and
   record its own command, exit status and checkout tree.
7. Landing on a partial view bypasses the gate after the fact; strict
   landing should be the default when a required approver is
   unreachable.
8. "Why" depends on the owner marking decisions. Draft: ParleyInn MUST
   attribute each diff hunk to the event, actor and preceding message
   that produced it, or mark it "from checkpoint".

## Minor

9. The phone's approval scope contradicts itself.
10. Two review surfaces; pick one.
11. "Carried: rebase only" should name the range-diff tool.

Verdict: the review UX serves my seat, but almost none of it reaches
the proposed SRS; as drafted, the contract lets authors approve, counts
tool-output claims as checks, and leaves the landed link unbound.
