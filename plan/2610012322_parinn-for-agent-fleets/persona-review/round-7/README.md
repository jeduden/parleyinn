# Persona review, round 7

Round 7 re-ran the security officer on the per-act assertion model of
round 6, in [proposal.md](../../proposal.md) at commit 96d59f1, and
asked it to separate genuine holes from risks only sandboxing the agent
removes. The report is beside this file.

## Merged findings

| #    | Finding                                                         | Action                                                               |
| ---- | --------------------------------------------------------------- | -------------------------------------------------------------------- |
| R7-1 | Writer-signed widening events take effect with no assertion     | A widening event counts only with a verified stored assertion        |
| R7-2 | Tenant configuration widens with no assertion                   | Widening configuration changes need an OWN-11 act, or managed policy |
| R7-3 | Attestation roots too loose                                     | Roots compiled in, limited to device-enforced user verification      |
| R7-4 | Restore, command confirmation, witness runs, rotation unclassed | All widening; restore reapplies later tightenings                    |
| R7-5 | T21 gaps; clearing without an authenticator; phone over SSH     | Extend T21; clear at a terminal; keep the port                       |

## Residual risks

Seven risks remain that no ParleyInn rule can remove while the agent runs
unsandboxed as the same user: typing into the harness's own prompt;
replacing the harness settings or the `parinn` binary; deleting the
home; consent phishing; reading the store directly; cut acts against
other agents; and the terminal-confirmation opt-in. The proposal lists
them after T24, and OQ-29 asks the stakeholder to accept them or scope
T21 to sandboxed agents.

## Next

The fixes are in the proposal (§10.17). The owner-act model now needs
the stakeholder's OQ-29 decision more than another review round.
