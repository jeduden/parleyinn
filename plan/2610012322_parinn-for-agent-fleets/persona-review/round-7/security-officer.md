# Round 7: security officer

Target: proposal.md at 96d59f1. Six round-6 findings resolved, one
partly; two new blockers, and seven residual risks that only
sandboxing the agent removes.

## Round-6 findings

- Resolved: first enrolment (OWN-11; roots still loose, N3); the
  session bearer, GUI injection and shared RP ID (one assertion per act,
  bound to its digest and exact origin; SEC-20; T21); quarantine as a
  cut act; the surface mark and harness-prompt pins; unlisted acts (a
  few verbs remain, N4); SEC-22 and OWN-06.
- Partly resolved: non-terminal CLI reads are enveloped, but reads over
  `parinn-ui` HTTP with the launch credential are not (residual R5); the
  phase-1 test names none of the four cases; §10.3 still says
  `parinn-run` shows the code.

## New findings

N1. Blocking: trust in widening acts comes from the writer, and the
    writer key is readable by the same user, so an agent can sign a
    pin, rule, grant or enrolment event directly. Draft: derived state
    MUST give effect to a widening `operator` event only when its stored
    assertion verifies against a credential enrolled under OWN-11, and
    `parinn verify` MUST report every widening event that lacks one.
N2. Blocking: tenant configuration widens with no assertion (config
    pins, the deployment mode). Draft: a tenant-configuration change
    that adds or ends a pin, changes the deployment mode, loosens
    redaction, recall scope or holds, or turns on a B1–B3 component MUST
    take effect only after a widening act under OWN-11 records its
    digest, or when root-owned managed policy sets it.
N3. Important: attestation roots MUST be compiled into the binary and
    limited to authenticators whose certified metadata says user
    verification is enforced on the device.
N4. Important: `parinn restore`, confirming a recorded command, witness
    runs and token rotation sit outside the class list. Draft: `parinn
    restore` MUST be widening and MUST reapply every revocation,
    quarantine, pin end and rule tightening recorded after the backup,
    or refuse.
N5. Minor: T21 omits direct store and key writes, config edits,
    restoring a backup, replacing the binary or hooks, and deleting the
    home; OWN-12's "cleared with an assertion" is impossible without an
    authenticator; a phone over an SSH forward must keep the exact
    origin and port.

## Residual risks for OQ-29

- R1: the agent types into the harness's own prompt to approve itself.
- R2: the agent edits the harness settings or replaces the `parinn`
  binary.
- R3: the agent deletes or rolls back `PARINN_HOME`; only an off-machine
  receipt shows it.
- R4: consent phishing: a widening digest waits for a touch while the
  screen shows a harmless act.
- R5: the agent reads every lane from the store or its own `parinn-ui`,
  unenveloped.
- R6: the agent uses cut acts against other agents (denial of service,
  audited).
- R7: the terminal-confirmation opt-in is passable by design.

Verdict: every sanctioned surface needs a per-act hardware assertion,
but writer-signed events (N1) and tenant configuration (N2) still
widen without one.
