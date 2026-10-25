---
n: 2
title: "The home, the audit log and counters"
status: "✅"
result: true
summary: >-
  @SEC-02, @SEC-03, @OPS-02, @OPS-03 and @ENG-04 off @pending and
  passing, confined. Every command checks the home first; the audit log
  chains across rotation; parinn hook fails open and audits.
---
# Phase 2 result

## Handoff

Outcome:

- SEC-02, SEC-03, OPS-02, OPS-03 and ENG-04 close: their five
  scenarios, eight runs, pass against the built core, confined. The
  workspace's 76 scenario runs pass, and `cargo run -p coverage` holds
  every crate at 100%.
- I6, I8 and I9 hold: a degraded operation is audited and counted, a
  refused home is left untouched, and a hook exits 0 on every path.

Inherited:

- `Context::home()` locates the home, then `Home::check` (owner-only,
  the running OS user's, no symbolic link inside) and
  `check_home_id` against `PARINN_HOME_ID`. Every command goes through
  it; the running OS user is a pipe's owner.
- The `audit` crate: `Log::append(&Occurrence, time)`, `entries`,
  `verify`; `counters::counts`. An `Occurrence` holds only
  `&'static str`, so no content reaches the log. New counters join
  `audit::entry::Counter`; new kinds of failure each get a variant.
- The `hook` crate: `Hook`, `Failure`, `handle`, `guarded`. `parinn
  hook` waits 500 ms for its whole hook input, and audits under its
  own panic guard.
- `mcp::serve` takes the handler and a callback per caught panic; it
  answers a request whose handling panicked under the request's id.
- The testkit's `Confinement::run_in` takes environment variables; the
  steps' `run_core` and `last_run` live in `bdd/record.rs`, and
  `World::env` carries the variables.

Deviations, now in phase-2.md: counter names `hook_panicked` and
`mcp_panicked`, and the entry kind `counter_ack`, on the domain-model
agent's review.

For phase 3:

- `parinn counter ack` refuses without a terminal on standard input
  and output (OWN-12), which the `@OPS-03` scenario proves with the
  agent's tool call, and the testkit's `run_at_terminal` gives a step
  one through `script`, outside the trace. It still only appends a
  `counter_ack` entry: record the neutral principal act in the device
  seat and move doctor's last acknowledgement to it.
- `home.json` is written by hand; configuration's `home.id` should
  write it.

Parked:

- A hook input that times out or cannot be read counts as
  `hook_rejected`; phase 9's hook budgets add their own counters
  (NFR-02, OPS-01).
- Audit appends are not synced to disk, and a torn last entry fails
  every later append until repaired (ENG-06, phase 16).
- A hook whose home is refused or whose audit log takes no entry
  leaves no trace; `parinn status` and `parinn doctor` name the cause.
- The home check walks every file, payloads included once phase 6
  writes them; it may need narrowing to fit the hook budget. An append
  reads the whole last audit file, and `parinn status` and `parinn
  doctor` re-check the whole log each run.
- `parinn verify`, `status` and `doctor` read the audit log without its
  lock, so an append mid-write can read as a torn entry.
- `parinn verify` needs a store, so a home with only audit entries
  cannot have its audit chain checked.
- Windows' ACLs are not checked yet.
- The `@SEC-02` steps for another OS user need root or `sudo -n`.
- OQ-50 (i): every counter so far counts as a failure counter.
- Counter names disagree across pending scenarios: `@OPS-01` counts a
  malformed hook input as `hook_rejected`, `@SEC-16` as
  `hook_input_rejected`. Settle them before either goes live.
- The SRS names no variable for SEC-03's home id; `PARINN_HOME_ID`
  should join it beside `PARINN_HOME`, in its own pull request.
