---
id: ADR-2611152235
title: "ENG-29 guards the rows that spell out I2 and I4, and itself"
status: accepted
summary: >-
  ENG-29's record of a change to ENG-29's own row: the row now also
  names every requirement I2 names by id and the one I4's boundaries
  rest on (PRV-08, INJ-01, INJ-02, INJ-04, INJ-10, OWN-04, OWN-07,
  OWN-08, OWN-23, OWN-24, OWN-26, OWN-29, SEC-01), the six rows those
  defer to (INJ-03, LANE-30, OWN-03, SEC-19, SEC-26, SEC-28), and says
  that a change to ENG-29 itself needs the same record. No invariant's words change. The security reviewer is
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

- The rule: ENG-29's row names a requirement when I2's own text names
  it by id, since its words then decide what I2 means. I2 names these,
  and ENG-29's row now names each:
  - when a typed prompt and a hook observation are trusted, PRV-08;
  - the restore blocks, INJ-01, INJ-02 and INJ-04;
  - the opt-in notices, INJ-10;
  - the fixed templates of OWN-04 and OWN-07;
  - the endorsed post's template, OWN-08;
  - the delegation and acceptance grants, OWN-23 and OWN-26;
  - the delegated task's template, OWN-24;
  - the trusted principal's post template, OWN-29.
- I4 names no requirement. ENG-29's row names SEC-01, which defines
  the boundaries B0 to B3 that I4 rests on.
- PRV-10, which I2 also names, was already marked for I2 review.
- A named row can defer to another, which could then be widened in its
  place. ENG-29's row names those too, as the stakeholder ruled on 18
  November 2026:
  - who may build `TrustedText`, INJ-03, which OWN-04 cites;
  - what a room may write to an agent, LANE-30, which INJ-10 cites;
  - how a principal's text reaches an agent, OWN-03, which the
    templates of OWN-08, OWN-24 and OWN-29 are exceptions to;
  - the register and the off-by-default rule for B1 to B3, SEC-19;
  - the limits on export and publish, SEC-26, and on bridges, SEC-28.
- I3 names INJ-05, which decides when a pin is named by id and not
  restored. It opens no path to the model and stays out of the row.
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

- A change to one of the nineteen rows above, to PRV-10 to PRV-13, which
  ENG-29 guarded before, or to ENG-29's row, needs an accepted decision
  record that names ENG-29 and the row, and the
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
