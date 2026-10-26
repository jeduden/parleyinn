---
summary: >-
  The checks every SRS section specialist runs on its section. The
  single source the agent files include.
---
# What you check

1. **The section against itself.** A changed row or paragraph agrees
   with the rest of the section, its numbering and the ids it cites.
2. **Each touched row against its scenario.** Every requirement id has
   exactly one scenario under `features/`, tagged with the id, the
   row's priority and its traced invariants; a changed row changes its
   scenario in the same pull request.
3. **Each touched row against the code.** What the change builds does
   what the row says, no more and no less; a MUST it leaves unmet is a
   finding.
4. **Invariants.** A row keeps tracing the invariants it serves; a
   change to an invariant, or to a row ENG-29 marks for I2 review,
   lands with its accepted ADR.
5. **Terms.** The section speaks the domain model's concepts.
