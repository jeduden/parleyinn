---
summary: >-
  The checks every SRS section specialist runs on its section. The
  single source the agent files include.
---
# What you check

1. **The section against itself.** A changed requirement or paragraph
   agrees with the rest of the section, its numbering and the ids it cites.
2. **Each touched requirement against its scenario.** Every requirement id
   has exactly one scenario under [`features/`](../../features), tagged with
   the id, the requirement's priority and its traced invariants; a changed
   requirement changes its scenario in the same pull request. A finding on a
   requirement ends in its id.
3. **Each touched requirement against the code.** What the change builds
   does what the requirement says, no more and no less; a MUST it leaves
   unmet is a finding.
4. **Invariants.** A requirement keeps tracing the invariants it serves.
   A change to an invariant, or to a requirement
   [ENG-29](../srs/10-engineering-quality.md) marks for
   [I2](../srs/invariants.md#i2) review, lands with its accepted ADR: your
   context names it, and a change whose context names none lacks it.
5. **Terms.** The section speaks the domain model's concepts: a term the
   model excludes, outside where its hub allows it, breaks ENG-35.

Your turns are few, and your diff takes most of them. Read your diff, then the
requirements it touches or names, a read or a grep each. Before you open
anything else, send one message with a finding line for each problem they
show; what is fine gets no line, never one that ends with `no finding`, `none`,
`fine` or `sound`, and an id you are unsure of is checked in the SRS before you
state the line, never marked provisional after it. Then open code or a scenario
only to check a requirement your diff touches: a file your diff shows, or the
scenario tagged with its id. Never search the tree past them. State what each
later read shows at once, as a finding, `revised:` or `withdrawn:` line.
