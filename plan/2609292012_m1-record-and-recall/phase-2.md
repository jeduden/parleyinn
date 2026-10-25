---
n: 2
title: "The home, the audit log and counters"
status: "✅"
result: false
---
# Phase 2: the home, the audit log and counters

Requirements. SEC-02, SEC-03, OPS-02, OPS-03 and ENG-04 (P0) close.
The phase lays OPS-01's first counters and a thin `parinn status` and
`parinn doctor`, which phase 15 completes.

The home (SEC-02, SEC-03):

- Every command checks the home first. Each directory and file in it
  must belong to the running OS user, give no group or other bits and
  be no symbolic link. A failure exits 1 naming the path and its mode
  or owner. The core reads its own OS user from the owner of a pipe it
  makes: no `unsafe`, no new dependency. Windows' ACLs come later.
- Directories are made `0700` and files `0600`.
- `home.json` holds a home's home id (§8.1). Unless `PARINN_HOME_ID`
  supplies the same id, the command exits 1 before any store file is
  opened. Phase 3's configuration sets `home.id`.

The audit log (OPS-02) is a new `audit` crate (ENG-02):

- `audit/audit-NNNNNN.jsonl`, `0600`, one RFC 8785 entry per line: its
  number, time, kind, counter, operation, fixed detail text, the
  previous entry's hash and its own (§8.3). No record content, no
  secret.
- Appends take a lock with a deadline (ENG-03). A file of 1 MiB rotates;
  the next file's first entry chains to the last one.
- `parinn verify` checks the audit chain too, exiting 3 naming the
  broken entry.

Counters (OPS-01, OPS-03):

- Counters derive from the audit log, one per name. Each one this
  phase adds is a failure counter (OQ-50 (i)).
- `parinn counter ack` appends a `counter_ack` entry, and refuses
  unless its standard input and output are a terminal (OWN-12), so an
  agent's tool call cannot quiet `parinn doctor`. The entry is interim:
  phase 3 records the neutral principal act (OWN-11), and
  `parinn doctor` then reads the last acknowledgement from it.
- `parinn doctor` exits 1, naming each counter, while one rose since
  the last acknowledgement. It is read-only (ADM-10).
- `parinn status` shows the home, the store, the audit log and the
  non-zero counters.

Panics and typed errors (ENG-04):

- A new `hook` crate is the entry point of `parinn hook <hook>`: it
  reads stdin with a deadline, writes nothing to stderr and exits 0
  on every path. For now it only checks for a JSON object and injects
  nothing (phase 9). A rejected input is counted `hook_rejected`.
- The hook entry and each MCP request catch a panic, fail open and
  audit `panic recovered`, counted `hook_panicked` or `mcp_panicked`.
  The hook also hides the default panic message.
- Each counted failure class has its own error variant; every error
  keeps its cause.
- In a development build, `PARINN_TEST_FAULT=hook-panic` makes the hook
  handler panic for `@ENG-04`'s scenario.

Tests. The testkit's confined start takes environment variables. A
step gives a path another OS user's ownership as root, else through
`sudo -n`, and gives it back afterwards.

BDD coverage: `@SEC-02`, `@SEC-03`, `@OPS-02`, `@OPS-03` and `@ENG-04`
come off `@pending`.

RED, each test failing first: unit tests of the entry and its chain,
rotation, the counters, the home's checks and the hook's guard, then
the five scenarios' steps.

GREEN: the `audit` and `hook` crates and the home's checks. The core
executable gains `parinn hook`, `status`, `doctor` and `counter ack`.

Gate: the five scenarios pass against the built core in an isolated
home, confined.
