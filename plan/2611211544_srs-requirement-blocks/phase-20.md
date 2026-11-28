---
n: 20
title: "What a review keeps for the next in a requirement of its own"
status: "🔳"
result: false
---
# Phase 20: What a review keeps for the next in a requirement of its own

Requirements. This phase is task 6's third pull request, and ENG-28's
first part. ENG-28 holds 202 lines. Its last five sentences say what a
review keeps for the next review of its pull request: its tracked
findings, which the next carries; what it spent, which the next sums;
and its outcome, which a review a stakeholder's comment brings reuses,
since a comment changes nothing an agent reads. They become ENG-37,
after ENG-32 at the end of §10.5, word for word, in 41 lines, and
ENG-28 keeps the rest in 166. ENG-37 keeps ENG-28's priority and
verification method, P0 and T. Where its first sentence passed the
findings to the tool, it names the tool that runs the review (ENG-28).
No scenario comes off `@pending`: ENG-37's is live, as ENG-28's is.

Why these first. No agent's instructions cite ENG-28 for them, so no
reviewer component changes and the evaluation plans none. The parts
that follow are the reading of reports, the runs, and the cost cap
with the answer store; the reading's part changes the shared sections
that cite ENG-28 for a report's form, so it runs their cases.

The citations. Four scripts, the review crates' comments on the
previous findings, the spend file, the kept verdict and the consent the
bundle leaves out, four engineering checks, the CI page and the
findings' decision record name ENG-37 where they cite these sentences.
Of the drift cases, 31 move to ENG-37: the carried findings' 7, the
reused outcome's 6, 15 of the spend's and the 3 of a comment that
changes what an agent reads. The spend's other 13, on the cap, the
runs in flight, what the gate says of a capped review and the reserve,
stay ENG-28's.

The scenarios. ENG-28's scenario gives ENG-37's its three steps: the
findings carried, the spend summed and the outcome reused. Their
bindings read the checkout alone, so the new scenario starts from it.

The rest. Appendix B counts 33 P0 engineering requirements. Appendix C
gives ENG-37 ENG-28's personas, U8 then U1, in SEC-33's row, which has
them, so the table keeps its width. The parse gate counts 35
engineering requirements. The srs kind lists ENG-28 at 166 lines and
SEC-31, which fits since phase 18, no longer.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates and the scenario gate pass, and the cucumber
  scenarios `@ENG-28` and `@ENG-37`;
- `review-eval check` finds every case applies, and the evaluation
  plans no component;
- the drift suite catches every case that injects a file this changes;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
