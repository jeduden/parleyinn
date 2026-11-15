---
id: ADR-2611152235
title: "ENG-29 guards the rows that spell out I2 and I4, and itself"
status: accepted
summary: >-
  ENG-29's record of a change to ENG-29's own row: the row now also
  names the requirements that spell out I2's closed write paths and
  I4's boundaries (INJ-01, INJ-02, INJ-04, INJ-10, OWN-08, OWN-24,
  OWN-29, SEC-01) and says that a change to ENG-29 itself needs the
  same record. No invariant's words change. The security reviewer is
  the stakeholder, `jeduden`.
---
# ADR-2611152235: ENG-29 guards the rows that spell out I2 and I4, and itself

## Context

ADR-2611151030 took the stakeholder off the specification. A change
under `docs/srs/` now lands on the review gate's approval alone
(ENG-21). ENG-29 still asks for an accepted decision record. That
record holds a named human security reviewer's approval. It covers
only the invariants and the four requirements marked for I2 review,
PRV-10 to PRV-13.

I2 names its closed write paths by requirement id, and I4 its
boundaries. The words that decide what reaches the model, and what a
component may connect to, are in those rows, not in the invariants. A
change to one of them could widen a path while every invariant's words
stay the same, and it would land with no human.

The review bundle already treats a change to ENG-29's own row as
guarded, since the row holds the list. The row did not say so.

On 15 November 2026 the stakeholder ruled that where a human is
required, it is the stakeholder.

## Decision

- ENG-29's row also names the requirements that spell out I2's closed
  paths and I4's boundaries:
  - the restore blocks, INJ-01, INJ-02 and INJ-04;
  - the opt-in notices, INJ-10;
  - the endorsed post's template, OWN-08;
  - the delegated task's template, OWN-24;
  - the trusted principal's post template, OWN-29;
  - the network boundaries, SEC-01.
- ENG-29's row says that a change to ENG-29 itself lands the same way.
- The review bundle reads the guarded requirements from ENG-29's row,
  so its deterministic finding covers the new ids with no change to
  the tool.
- The named human security reviewer is the stakeholder, `jeduden`,
  until a later record names another.

## Alternatives

- Give `docs/srs/` back a code owner. That makes almost every pull
  request wait for the stakeholder again, which ADR-2611151030 ended.
- Guard every requirement that traces to I2 or I4. That is most of the
  specification, and the same wait.
- Leave the rows unguarded and rely on the deep review. The agents
  weigh such a change, but ENG-29 exists because an agent's approval is
  not enough for the contract.

## Consequences

- A change to one of the eight rows, or to ENG-29's row, needs an
  accepted decision record that names ENG-29 and the row, and the
  stakeholder's comment on its pull request. Until both are there the
  review requests changes.
- Every other change to the specification still lands on the review's
  approval alone.
- A requirement that later joins I2's closed paths or I4's boundaries
  must join ENG-29's row in the same change, which this record's rule
  for ENG-29's own row guards.

## Approval

The stakeholder, `jeduden`, is the named security reviewer ENG-29 asks
for. The stakeholder's message of 15 November 2026 in the session that
made this change names the reviewer: "if any human is required it is
me". The approval of this record is the stakeholder's comment on the
pull request that lands it, which the review holds the change to. It
binds to the change in that pull request, not to a later one, and can
be checked apart from this record.
