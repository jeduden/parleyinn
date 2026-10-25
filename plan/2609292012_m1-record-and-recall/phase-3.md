---
n: 3
title: "Settings layers and principal acts at the terminal"
status: "✅"
result: false
---
# Phase 3: settings layers and principal acts at the terminal

Requirements. The phase builds ADM-04's settings layers and SEC-11's
repository configuration. It builds PRV-04's deployment mode, PRV-05's
ignored trust policy keys and SEC-22's M1 part. It adds the principal
acts at the terminal that OWN-11, OWN-12 and OWN-22 ask of M1. No
scenario comes off `@pending`: each has a row a later phase reaches.

Settings layers (ADM-04, SEC-11), a new `configuration` crate:

- Every layer is TOML through `toml` (its ADR lands here), validated
  against §9.6's keys. An error names the key and the problem, never
  the layer's text.
- Defaults, then `config.toml` in the home, then `.parinn.toml` at the
  top of the repository containing the working directory, then managed
  policy at one documented path per OS. A development build may name
  another through `PARINN_TEST_MANAGED_POLICY`.
- Managed policy the running OS user could write, or that fails, is
  refused, audited and counted; the core keeps recording.
- A repository key that does more than tighten is ignored, audited
  and counted. So is a `trust_policy.<class>` key in any layer.
- A change awaiting acceptance stands as last accepted until `parinn
  configuration accept` records the configuration's digest and
  settings. Settings derive from that act (I10).

Principal acts (OWN-02, OWN-11, OWN-12, OWN-22 in part):

- An `operator` event of kind `principal_act` in the device seat,
  naming its class and principal surface.
- Every verb refuses without a terminal, first, audited and counted.
- A widening act is refused while managed policy fails or requires an
  authenticator (SEC-22). It is refused while a run leaves a residual
  risk open that no standing risk acceptance names, naming the risks
  and the runs. No run records a sandbox state yet, so every run
  leaves all eight open.
- `parinn risk-acceptance show|record|withdraw`. Record names each open
  risk and that an agent can forge it, unless managed policy forbids
  it. Withdraw is a cut act.
- `parinn counter ack` records the neutral act; `doctor` and `status`
  read the acknowledgement from the record.

Commands. `parinn status` exits 2 on a failing configuration or
repository configuration and shows the deployment mode. `parinn ingest`
audits each failing layer and ignored key, and records on (I9).

Where the gate's scenarios go live:

- `@ADM-04`: rows 1 to 4 pass here, and it comes off `@pending`. The
  stakeholder ruled row 5, flags staying on until a widening act, into
  `@PRV-07`'s scenario, which flags need (P1, M5), and asked for rows
  for the rest M1 enforces: a widening setting awaiting acceptance, a
  repository configuration's widening key ignored and counted, and
  managed policy the person's OS user can write refused and counted.
- `@PRV-04`: phase 9, which matches `user` events to prompts.
- `@PRV-05`: phase 8 ingests phase 7's provenance classes; `post`
  and `summary` need later milestones.
- `@SEC-22`: phase 14's purge, for its retention row.
- `@OWN-11`, `@OWN-12`, `@OWN-22`: P1 rows landing in part; the rest
  needs pins, rooms, presence proofs and sandbox state.

Tests. Unit tests beside each function. End-to-end tests in
`crates/parinn/tests` drive the confined core through each part.

Parked to phase 7: the configuration parser's fuzz target, with the
first `fuzz/` crate, its nightly and its ADR.

Gate: the end-to-end tests pass against the built core, confined, and
`@ADM-04` passes off `@pending`.
