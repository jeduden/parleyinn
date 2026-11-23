---
n: 14
title: "A derived defect plants one problem"
status: "🔳"
result: false
---
# Phase 14: a derived defect plants one problem

Requirements. None changes, and no scenario comes off `@pending`.
ENG-32 derives, for each SRS section's specialist, a case that adds a
MUST to the section's first requirement with no change to its
scenario, and expects a finding that ends in the requirement's id.

What happened. In plan 2611211544's evaluation of phase 16,
srs-security's case failed once: its answer asked the specification
to change SEC-01 and stated no finding. Across the 53 runs of the case
in phases 16 and 17, 21 answers held such a request. Most said that
the added sentence's "It" and "the change" name nothing; a few that
its audit duty serves I6, which the requirement does not trace.

Why it happens. The sentence the case adds, "It MUST also record the
change in the audit log.", plants a second problem beside the MUST its
scenario leaves untested: a duty no reader can check, since it names
neither its subject nor what it records. A specialist that reads it
asks for the requirement to be fixed, which the report rule makes a
request, not the finding the case expects.

The rule. The added MUST names its requirement and what it records:
"Each failure to meet SEC-01 MUST also be recorded in the audit log.",
with each section's own id.

RED. The derived case's unit test expects the sentence with the
requirement's id; it fails on the fixed sentence.

GREEN. The derived cases build the sentence from the requirement's id.

Gate. `review-eval run` passes the cases of each SRS specialist on its
model, with the requests against the added sentence counted beside the
old sentence's. The workspace's tests, `review-eval check`, `mdsmith
check .` and the drift cases that inject the derived cases pass.

BDD. No cross-host behaviour: no `@S` scenario.
