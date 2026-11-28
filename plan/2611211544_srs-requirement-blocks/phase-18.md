---
n: 18
title: "SEC-31's purge receipt in a requirement of its own"
status: "✅"
result: false
---
# Phase 18: SEC-31's purge receipt in a requirement of its own

Requirements. This phase is task 6's first pull request. SEC-31 holds
53 lines, three past the 50 the srs kind allows. Its last sentence, the
signed purge receipt, becomes SEC-33 at the end of §6.2, word for word,
and SEC-31 keeps every other sentence in 49 lines. SEC-33 keeps
SEC-31's priority and verification method, P0 and T. It traces I1,
since a purge is an exception I1 allows only when recorded, and I6,
since the receipt names to the node's principal each copy the purge
could not erase. No scenario comes off `@pending`.

Why the receipt. Its move alone brings SEC-31 under the limit and
leaves every other citation of SEC-31 true. The code, the domain model
and the rest of the SRS cite SEC-31 for the copies a purge erases, its
residue and its transcript sources. Only the command-line interface
and the model's Receipt entry cite it for the receipt, and they cite
SEC-33.

The scenarios. SEC-31's scenario loses its receipt step, and its title
the words that named the receipt. SEC-33's scenario, `@pending`, purges
an event that a harness transcript file, an exported bundle, a backup
ParleyInn can no longer reach and a refusing peer also hold, and its
receipt names each copy.

The appendices. Appendix B lists SEC-33 under I1 and I6 and counts 93
P0 requirements in §5–§6. Appendix C gives SEC-33 SEC-31's personas, U8
then U1, in a row of its own, so the table keeps its width.

What waits. The srs kind still lists SEC-31 at 53 lines. Its entry
leaves the list with the next change to `.mdsmith.yml`, ENG-28's split,
so this change needs no consent. LANE-34 and LANE-01 come next, ENG-28
once the pull requests that change it land, and PRV-08 and PRV-12 with
the decision record ENG-29 asks for.

Gate. Each of these is recorded in the handoff:

- `mdsmith check .` passes;
- the srs gates and the scenario gate pass;
- `review-eval check` finds every case applies, and the evaluation
  plans no component;
- the drift suite catches every case that injects a file this changes;
- `scripts/review-local.sh` approves.

BDD. No cross-host behaviour: no `@S` scenario.
