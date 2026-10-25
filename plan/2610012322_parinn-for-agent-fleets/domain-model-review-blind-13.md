# Domain model: thirteenth round of blind reviews, merged

Three domain-model agents (A, B, C) reviewed `docs/domain-model.md`
after the twelfth-round decisions were applied (commit 16785c5), against
itself, all of `docs/srs`, the invariants and `features/`, reading every
file from disk. This note keeps the needs-fix findings, each checked
against the files; one claim (a stale CLAUDE.md include) was checked and
dropped. Each question takes its recommended option, as the stakeholder
asked.

## 1. Questions and decisions

| #   | Question                                                                         | Found | Decision                                                                                                            |
| --- | -------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------- |
| Q1  | I2's "the turns while … `interactive`" reads as trusting more turns than PRV-02. | 3/3   | Correct I2 by ADR-2610062800: witnessed `harness_meta` events, and witnessed `user` turns only while `interactive`. |
| Q2  | Which provenance does a room summary take, so it is never trusted?               | 2/3   | A new class, `summary`, never trusted (PRV-01).                                                                     |
| Q3  | Does a seat with no role assignment include the owner's device seats?            | 3/3   | No: those keep every capability; the owner's agents' run seats have their role and any appointment.                 |
| Q4  | Who seals an ingested run's writer?                                              | 2/3   | The core; a witnessed run's run seat is sealed by its MCP server (REC-19).                                          |
| Q5  | Does a trust grant reach a foreign room?                                         | 1/3   | No: a foreign room stays untrusted whatever trust grant covers its keys.                                            |
| Q6  | What names a node in a short-lived environment, as against a run's sandbox?      | 2/3   | An **ephemeral node**; "sandbox" stays the confinement around a run.                                                |
| Q7  | Is a pull-request author a principal?                                            | 1/3   | Not unless its key is also a principal key.                                                                         |
| Q8  | Should `parinn peer on\|off` start the peer component?                           | 1/3   | It becomes `parinn peer-component on\|off`; "peer" stays a node, and §6.3, CON-02 and §4.2 say "peer component".    |
| Q9  | Which senses does "budget" carry?                                                | 1/3   | Always named: pin, hook, step or delegation budget; I3's budget is the pin budget, so I3 keeps its words.           |
| Q10 | Does "pause ingestion" pause capture or ingest?                                  | 1/3   | Capture.                                                                                                            |

## 2. Fixes that need no decision

- **Model:** routing names room acts and principal acts before "any
  other event with no run"; pins change by room acts but for the
  principal's neutral unpin of its own agent's run-seat pin; the
  service-account certificate and bridges defined; the restore block
  lists the count, room id and key fingerprint of pins that do not
  qualify; seat certificates name the seat kind; managed policy is the
  only source of settings that is not a principal.
- **SRS and scenarios:** "repository file" → repository configuration
  (OWN-10, principal-acts.feature, security.feature); LANE-14 and OQ-18
  "a user turn"; the "sandbox" that hosts a node → ephemeral node
  (SEC-10, PEER-01, PEER-06, OWN-23, T23, §6.3, 02, 12, §9.5,
  peer.feature, provenance.feature); "attests" → the seat kind its seat
  certificate names (LANE-23, LANE-24, lane.feature).

## 3. Optional, not chased

OWN-11's foreign-room exception; ENG-28's `verdict` field; I4's core
list; §4.2's launcher wording; "writer id" and "room id"; ENG-02 crate
names; PEER-12 "another entity"; appendix A's "secure purge"; "Rooms do
not land" beside **Landed**; seats for stamping and endorsing; the
`room_` prefix on reading tools; `retention_policy.<room>.<provenance>`.

## 4. Invariants

I2 changes, recorded in ADR-2610062800, which corrects ADR-2610062600.
