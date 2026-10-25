# Persona review of the fleet proposal

The persona-review skill ran all nine persona agents in
`.claude/agents/`, independently and in parallel, on one target: pitch
v7, plan 2610012322 (plan, both traces, the four UX designs) and plan
2610022338 with phase 1, at commit 618fafb. Each persona's full report
is beside this file.

## Merged findings

Numbered in the order they were merged; the numbers are cited
elsewhere, so they stay. Rank by the severity and raised-by columns,
not by position.

| #   | Finding                                                                                                                                                                          | Raised by                                                 | Severity  | Action                                                                                                 |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------ |
| 1   | The two traces conflict: REC-17/18 swapped, SEC-19..22 reused, tenant versus lane hash key, trust by origin versus nothing imported is trusted, recall scope session versus lane | multi-machine, operator, maintainer, agent, security      | blocking  | Reconcile into one proposal before any SRS edit                                                        |
| 2   | The UX designs contradict each other: status words, queue order, keymap, Adopt versus Endorse, notice default, review surface, phone scope                                       | fleet, collaborator, returning, reviewer, agent, security | blocking  | One vocabulary, one queue order, one keymap, one multiplayer model                                     |
| 3   | Journeys have no requirement ids or phase: catch-up, search, integrity, gaps, invites, roles, handover, landed link, gate, fleet rollout                                         | returning, collaborator, reviewer, operator               | blocking  | Give them ids in the SRS change; extend phase 1's gate                                                 |
| 4   | Post notices push peer-derived bytes into the agent's context                                                                                                                    | fleet, collaborator, agent, security                      | blocking  | Pull-only by default; any opt-in notice uses only local strings                                        |
| 5   | Steering and approvals launder untrusted text into the owner's voice; "Re-run here" executes a recorded command                                                                  | agent, security, maintainer, reviewer                     | blocking  | Owner text only through the harness's input, templated ids, confirmed commands outside agent context   |
| 6   | Trust across the owner's own nodes contradicts the import rule                                                                                                                   | agent, security, multi-machine                            | blocking  | Owner key hierarchy with certified device keys, scoped and I2-reviewed                                 |
| 7   | Network surface beyond B0–B3: https import, phone and terminal over B2, bridges, carriers, `parinn run`; no policy locks; no B2 encryption                                       | security, operator, maintainer, multi-machine             | blocking  | Register every process and protocol; managed policy per boundary; encrypted, mutually authenticated B2 |
| 8   | Purged content stays confirmable through chain hashes; no fleet-wide purge; no redaction on import                                                                               | security, operator, maintainer                            | blocking  | Keyed content commitment in the chain; purge by writer across peers; redact on import                  |
| 9   | Project identity: path-keyed splits a repo across nodes; clone-keyed widens pins across lanes; identity changes are silent                                                       | multi-machine, agent, operator                            | blocking  | Repository identity independent of path; pins scoped to lane or project; audit identity changes        |
| 10  | Verification classes are forgeable or inconsistent                                                                                                                               | reviewer, security                                        | blocking  | One evidence set; tool-output claims never satisfy a required check                                    |
| 11  | Held requests deny on a timeout and may block the native prompt                                                                                                                  | fleet, agent                                              | blocking  | No deny without an away policy; own hook budget for holds                                              |
| 12  | Phase 1 gates miss speed, a recorded weekend, and the security review                                                                                                            | fleet, returning, security                                | important | Add NFR-01 budgets, a weekend fixture and the security review to the gate                              |
| 13  | Owner authentication gaps: user presence, token in argv, cookie across loopback ports                                                                                            | security, operator                                        | important | WebAuthn for sensitive acts; token out of argv; origin-scoped storage                                  |
| 14  | Sandbox tail loss, unattended sandbox start, no fallback when the home node is down                                                                                              | multi-machine                                             | blocking  | Seal and offer segments on every stop; git carrier as fallback                                         |
| 15  | Author-can-approve hole and double review with a forge                                                                                                                           | reviewer                                                  | blocking  | Exclude authors and suggesters; show forge and ParleyInn verdicts side by side                         |
| 16  | Lane handover undefined                                                                                                                                                          | collaborator                                              | blocking  | Signed, accepted transfer that says what happens to agents, pins and requests                          |
| 17  | Operator role demoted; no quotas, resource bounds or CLI observability                                                                                                           | operator                                                  | blocking  | Keep the platform team primary; quotas; RSS and CPU bounds; status and doctor coverage                 |

## What the personas agree already works

- The UI is never a required hop, and the core stays network-free.
- One answer clears a request on every surface.
- Install shows a diff first, and the browser never writes agent
  configuration.
- One principal per agent, and petnames over sender-chosen names.
- Per-writer signed logs, no relay and enrolment by key.
- Claims shown beside the evidence that checks them.

## Round 2

The personas re-reviewed the reconciled proposal; the merged result is
in [round-2/README.md](round-2/README.md). Every finding above has an
answer in the proposal; eight new blockers, R1–R8, came up.

## Round 3

The third round, on the revision that answered round 2, is merged in
[round-3/README.md](round-3/README.md): five personas report no
blocker, and six narrow blockers remain, R3-1 to R3-6.

## Round 4

Round 4 re-ran the five personas with open blockers; it is merged in
[round-4/README.md](round-4/README.md). Three blockers came up, one of
them a conflict between the fleet developer and the security officer,
settled for safety and put to the stakeholder as OQ-29.

## Round 5

Round 5 re-ran the three personas with open blockers; see
[round-5/README.md](round-5/README.md). It settled the owner-act
question: widening acts need a WebAuthn assertion.

## Round 6

Round 6 re-ran the security officer and the fleet developer; see
[round-6/README.md](round-6/README.md). Widening acts now carry one
assertion each, and the first authenticator needs hardware attestation
or a root-owned policy entry.

## Round 7

Round 7 re-ran the security officer; see
[round-7/README.md](round-7/README.md). Two holes closed, and seven
residual risks that only sandboxing removes went to OQ-29.

## Round 8

Round 8 was a blind review of pitch v10 by all nine personas, given
only its text; see [round-8/README.md](round-8/README.md). Everyone
kept reading and nobody would install yet: the pitch serves the owner
at the screen, and the other seats appear only under "next" or not at
all.

## Round 9

Round 9 was a blind review of the room model by all nine personas; see
[round-9/README.md](round-9/README.md). Every seat endorsed the trust
split. Twelve fixes came out, chiefly notices that push other players'
text, undefined admission, a deadlock when an owner leaves, and limits
on the etiquette bot.
