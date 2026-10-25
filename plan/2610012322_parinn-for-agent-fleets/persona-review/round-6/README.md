# Persona review, round 6

Round 6 re-ran the security officer and the fleet developer on the
three-class owner-act model of round 5, in
[proposal.md](../../proposal.md) at commit b7ae7fd. Each report is
beside this file.

## Verdicts

| Persona          | Verdict                        | Blocking                                                     |
| ---------------- | ------------------------------ | ------------------------------------------------------------ |
| fleet developer  | serves as the default stands   | none; the step costs are a preference for OQ-29              |
| security officer | the three-class shape is right | first enrolment; the session bearer; quarantine as a cut act |

## Merged findings

| #    | Finding                                                           | Raised by | Action                                                                      |
| ---- | ----------------------------------------------------------------- | --------- | --------------------------------------------------------------------------- |
| R6-1 | First authenticator enrolment undefined; revoking is a cut act    | security  | Hardware attestation or a root-owned policy entry; revoking is widening     |
| R6-2 | A session login authorizes widening; GUI injection; shared RP ID  | security  | One assertion per widening act, bound to the act and the exact origin       |
| R6-3 | Quarantine and revocation remove pins on a cut act                | security  | Anything that removes a pin or stops acceptance is widening                 |
| R6-4 | Unasserted acts recorded as trusted; acts missing from the lists  | security  | A surface mark and untrusted free text; classify all; unlisted is widening  |
| R6-5 | Harness-prompt pins; reads that skip the envelope                 | security  | Session-only `/unpin`; harness-input mark; enveloped non-terminal reads     |
| R6-6 | Core-only clarity: pin scope, the waiting verb, harness terminals | fleet     | Say the scope; wait for the tab's assertion; `parinn lanes` names terminals |

## Next

The fixes are in the proposal (§10.16). The per-act assertion raises
the fleet developer's cost in the view to one touch per widening act,
while the harness's own channels stay free; OQ-29 now also asks
whether to scope T21 to sandboxed agents instead.
