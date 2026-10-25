# Scope: the convergence narrows to M1–M5

On 9 October 2026 the stakeholder narrowed the convergence loop from the
full domain model to milestones M1 to M5 of §12.2, after a sizing pass
showed that 54 of the 84 finding themes so far arose only from features
that ship in M6 to M9 (the comparison page is
<https://claude.ai/artifact/FoPg5NJj1KLYkvyVNGEfF5>). Nothing is deleted
and no feature is dropped: entries whose requirements ship later stay in
the model and are dormant for the review.

## How a round reads the model

The review reads the full model, with the scope rule below. A finding that
arises only from M6–M9 features is reported as `later` with its milestone
and is not a needs-fix finding of this loop; the ledger defers it to OQ-47
with that milestone, for that milestone's ENG-29 review gate. A finding
about behaviour that ships in M1–M5 is needs-fix as before, also where its
fix reaches later text.

Scope rule: a requirement is in M1–M5 when §12.2 names it in an M1–M5 row
(in part where the row says so), when it is a P0 row of a family that row
names by priority, or when it is a P1 row that no M7–M9 row names. A
clause that applies only once a later requirement ships is dormant.

## Questions settled before the first narrowed round

The sizing pass raised these questions. The stakeholder decided S-1 to
S-4; S-5 to S-18 follow from the SRS, the invariants and safety first.

| Id   | Question                                                             | Settlement                                                                                                                                                                                                                                             |
| ---- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| S-1  | P1 rows in M5 that need the room view, results or checkpoints        | Stakeholder: LANE-20 (the intent) stays in M5; LANE-17, LANE-19, LANE-21, VIEW-20 and VIEW-21 move to M7, with M5's exit criteria that need the room view (the pilot with the room view in daily use, "answered from Catch up alone").                 |
| S-2  | Seals are M7, yet M1 and M3 rows assume them                         | Stakeholder: seals stay in M7. The M1–M5 record is hash-chained and unsigned; the seal clauses of M1–M3 rows apply once REC-18 ships, and spike S12 speaks of closed segments.                                                                         |
| S-3  | No sandbox state before the launcher                                 | Stakeholder: every M1–M5 run counts as unsandboxed, as OWN-22 says, so widening acts wait for the risk acceptance the principal records at the terminal.                                                                                               |
| S-4  | Joins are M7                                                         | Stakeholder: the own-principal part of LANE-23 comes to M2: a subagent joins its parent's rooms under its parent's join, and a run joins a room when its principal asks at the CLI, a widening act; admission names principal keys and stays in M7.    |
| S-5  | P0 rows no milestone names                                           | SEC-10, SEC-15, SEC-17 and MEM-01 are M1.                                                                                                                                                                                                              |
| S-6  | OWN-01 is M7                                                         | OWN-01 is M1; its clauses on other principals' text apply once their requirements ship.                                                                                                                                                                |
| S-7  | `parinn ingest` is M7                                                | REC-22 is M1, and REC-19's seat that `parinn ingest` starts comes with it (REC-19 in part), so ingested events are untrusted from the first release (I2).                                                                                              |
| S-8  | Roles and capabilities are M7                                        | LANE-16 in part comes to M1: the owner's device seat has every capability, and a run's seat in its personal room, in a room it created or joined at its principal's ask has the contributor role, so the M1–M2 checks of work and pin capability hold. |
| S-9  | The branch-link tie rule is LANE-31                                  | LANE-31 in part comes to M1: the rule for two branch links naming one branch.                                                                                                                                                                          |
| S-10 | A managed policy may require an authenticator before presence proofs | Until presence proofs ship (M7), such a policy refuses every widening act; quotas apply from ADM-15 (M7).                                                                                                                                              |
| S-11 | M1–M5 scenarios exercise M6–M9 behaviour                             | Each step that needs a later requirement moves to that requirement's pending scenario; tag lines stay.                                                                                                                                                 |
| S-12 | Terms defined inside postponed entries                               | Action class, commit author and review step get entries of their own.                                                                                                                                                                                  |
| S-13 | The invariants name later concepts                                   | The model keeps every concept defined; the hub says that §12.2 decides when each one's requirements ship.                                                                                                                                              |
| S-14 | Address forms are RCL-08 (M7)                                        | RCL-08 is M1.                                                                                                                                                                                                                                          |
| S-15 | The trusted-only export has no review step before SEC-26             | SEC-26 in part comes to M5 with ADM-12: a trusted-only export is a widening principal act with a review step; §9.5 and the boundary register give it ADM-12's P1.                                                                                      |
| S-16 | The boundary register makes the TUI P0                               | The register's CLI row stays P0 and the TUI follows VIEW-14 (M7).                                                                                                                                                                                      |
| S-17 | REC-21's refused segments (M3) sit in Integrity status (M7)          | The model records them under Segment; the integrity status shows them once VIEW-10 ships.                                                                                                                                                              |
| S-18 | Seat ids derive under LANE-23 (M7)                                   | LANE-23 in part comes to M1: how a seat id derives, which quarantine and purge by seat need.                                                                                                                                                           |

## Applying the settlements

The settle workflow (`m1-m5-settle.json`) drafted 206 edits to §12.2, the
boundary register, §9.5, the requirement rows, the model and the pending
scenarios; the ledger closes them as DM-CJ to DM-DA. Two points arose
while applying them:

- A requirement §12.2 brings forward in part leaves `@pending` only when
  its whole text ships; until then the milestone that brings the part
  forward verifies it by the steps that need nothing later (CLAUDE.md:
  a requirement is implemented when its scenario passes).
- PRV-09's scenario takes every writer from a peer's segments (M8), so it
  has no step that M1 can pass. It needs an M1 step, such as a writer taken
  from the key that wrote an event on this node; the first narrowed round
  takes it up.
