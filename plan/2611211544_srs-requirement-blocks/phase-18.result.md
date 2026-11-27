---
n: 18
title: "SEC-31's purge receipt in a requirement of its own"
status: "✅"
result: true
summary: >-
  SEC-31's signed purge receipt is SEC-33, word for word, at the end of
  §6.2, so SEC-31 fits in 49 lines, under the srs kind's 50. Its
  scenario, Appendix B and C, the parse gate's count and the two
  citations of the receipt follow; no other citation of SEC-31 changes.
  No scenario comes off @pending.
---
# Phase 18 handoff

## Done

- SEC-33 holds the sentence that ended SEC-31, word for word: each node
  writes a signed purge receipt the node's principal can collect. It is
  P0, verified by test, and traces I1 and I6. SEC-31 keeps its other
  sentences in 49 lines.
- SEC-31's scenario no longer names the receipt in its title, and a
  step checks what SEC-31 still asks of it: a purge that times out
  names each residue there. SEC-33's `@pending` scenario purges an
  event that a harness transcript file, an exported bundle, an
  unreachable backup, a refusing peer and an unreachable one also hold,
  and its receipt names each copy.
- `parinn purge-receipt check` in the command-line interface and the
  domain model's Receipt entry cite SEC-33, and §6's summary names
  SEC-01..33.
- Appendix B lists SEC-33 under I1 and I6 and counts 93 P0 requirements
  in §5–§6. Appendix C gives SEC-33 a row of its own with SEC-31's
  personas. The parse gate counts 240 requirements in §5–§6.

## Verified

- `mdsmith check .` passes.
- The srs gates and the scenario gate pass.
- `review-eval check` finds every case applies: 242 cases, 75
  components. Against main, the evaluation plans no component.
- The drift suite catches the 4 cases that inject a file this changes.

## Next

- LANE-34 and LANE-01, the next splits. ENG-28's waits for #154 and
  #183, which change it, and takes SEC-31's entry off the srs kind's
  list. PRV-08 and PRV-12 need the decision record ENG-29 asks for.
- An evaluation case plants ENG-36 as a requirement no section
  defines, so ENG-28's split takes ids past it.
- SEC-19 and the boundary register's opening say a purge receipt
  carries only digests, though SEC-31 already had it name its scope,
  ranges and copies. SEC-19 changes only with the record ENG-29 asks
  for, so its wording waits for the PRV-08 and PRV-12 split's.
