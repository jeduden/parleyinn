# Persona review, round 4

Round 4 ran the four personas that still had a blocker after
[round 3](../round-3/README.md), plus the security officer, on
[proposal.md](../../proposal.md) at commit ca209ea. Each report is
beside this file.

## Verdicts

| Persona                 | Verdict                   | Blocking                                                    |
| ----------------------- | ------------------------- | ----------------------------------------------------------- |
| fleet developer         | much closer               | the code path makes the shell worse when the view runs      |
| multi-machine developer | serves                    | none                                                        |
| OSS maintainer          | serves all journeys in P2 | none                                                        |
| security officer        | serves except one point   | terminal confirmation reopens T21 (any process opens a pty) |
| Claude, the agent       | mostly serves             | another tenant's pins can reach a restore block             |

The fleet developer and the security officer pull in opposite
directions on one rule. The proposal follows CLAUDE.md: where
usefulness and safety conflict, pick safety and make the convenience
opt-in. OQ-29 puts the choice to the stakeholder.

## Merged findings

| #    | Finding                                                                                                     | Raised by              | Action                                                                                                   |
| ---- | ----------------------------------------------------------------------------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------- |
| R4-1 | Terminal confirmation can be passed by any same-user pty; the code makes the shell worse when the view runs | security, fleet        | Terminal alone only for acts that cut; core-only answers and pins at the harness's prompt; policy opt-in |
| R4-2 | Another tenant's pins reach a restore block                                                                 | agent                  | Restore only the session's own principal's trusted pins; others by count                                 |
| R4-3 | An updated bundle is refused; nobody says how the binding statement is signed                               | maintainer             | Accept a continuing chain; a detached signature from the contributor's own git tooling                   |
| R4-4 | Offline sandbox certification is not admitted; post-expiry segments; token source; lane pins in the token   | multi-machine          | PRV-10 token link; accept continuing segments; recorded token source; pins in the token                  |
| R4-5 | `parinn-run` cost for ten instances; per-launch step                                                        | fleet                  | Total budgets and keystroke latency; an install opt-in                                                   |
| R4-6 | Same-user tampering: a readable key and a writable receipt path                                             | security               | Warn on the path; a short code to carry off the machine; key store per OS; qualify §1.2                  |
| R4-7 | `lane_status` lacks post addresses; merged ids; a stale note                                                | agent                  | Recall addresses of waiting posts; both ids; fix the note                                                |
| R4-8 | Phase tests miss widening acts, branch switches and core-only answers                                       | security, agent, fleet | Three phase-test cases and M7's exit                                                                     |

## Next

The fixes are in the proposal (§10.14). Run round 5 on the fleet
developer, the security officer and the agent, whose blockers these
fixes answer.
