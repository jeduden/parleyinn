# Persona review, round 5

Round 5 ran the three personas whose blockers round 4's fixes answered
(the fleet developer, the security officer and the agent) on
[proposal.md](../../proposal.md) at commit a3fa2bc. Each report is
beside this file.

## Verdicts

| Persona           | Verdict                            | Blocking                                                 |
| ----------------- | ---------------------------------- | -------------------------------------------------------- |
| fleet developer   | serves, if OQ-29 keeps the default | none; three core-only gaps                               |
| security officer  | closer                             | an agent can start its own `parinn-ui` and read the code |
| Claude, the agent | serves                             | none; re-signing and pin tests                           |

The security finding settles what the last two rounds circled: against
an unsandboxed agent of the same user, no software-only check proves
who acted. Only a WebAuthn assertion does.

## Merged findings

| #    | Finding                                                                         | Raised by    | Action                                                                                |
| ---- | ------------------------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------------- |
| R5-1 | An agent can start its own `parinn-ui --print` or `parinn-run` and get the code | security     | Widening acts need a WebAuthn assertion, per act or by the passkey login of a session |
| R5-2 | Screen reads and keystroke injection; an injected `/pin` restores lane-wide     | security     | Name them in T21; an unconfirmed `/pin` stays in its session                          |
| R5-3 | Core-only: no way to end a pin, mark ready or find the right prompt             | fleet        | `/unpin`; marking ready is neutral; `parinn needs` names the terminal                 |
| R5-4 | Re-signing undefined; pin rules untested; trust judged by current mode          | agent        | A re-sign act; tests in M2, M8, M9 and phase 1; trust by recorded mode                |
| R5-5 | Owner-act lists drift; receipt code strength; §1 unqualified                    | security     | One list and a drift gate; 80 bits; qualify §1                                        |
| R5-6 | `lane_status` can flood; pitch decision 9                                       | agent, fleet | Range form with a cap; reword                                                         |

## Next

The fixes are in the proposal (§10.15). Round 6 re-runs the security
officer and the fleet developer on the class model of R5-1.
