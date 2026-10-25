---
n: 11
title: "Trust, origin, branch links and repository identity"
status: "✅"
result: true
summary: >-
  @MEM-01 and @OWN-01 off @pending and passing. Each event's trust level
  and origin derive from the record and the node's key set alone,
  nothing stores them, and parinn rebuild and parinn verify keep the
  kept origins honest. A repository's identity is read inside the core;
  @LANE-02 stays @pending for a ruling. @PRV-02 passes narrowed; @PRV-06
  and @PRV-13 are proven by named tests.
---
# Phase 11 result

## Handoff

Outcome:

- MEM-01 and OWN-01 hold: off `@pending`, passing against the built
  core, confined. After every hook and `parinn rebuild`, no file outside
  the store holds a canary. A run's seats are all certified by the
  node's one device key. LANE-02's identity is the lowest parentless
  commit, read from `.git` inside the core. `@LANE-02` stays
  `@pending`: its steps pass narrowed, locally.
- PRV-02 holds for what M1 can show. `administration::trust_policy`
  trusts only `operator` events and the core's own structural events in
  a device seat of origin `witnessed`. No `user` event is trusted (R3 and
  R8 stay open until OQ-40), no hook observation, no `ingested` event,
  no `harness_meta` the launcher did not sign. Nothing stores a level.
  `@PRV-02` stays `@pending`: 11 rows pass narrowed locally; the rest
  need a launcher, a signed sandbox state, another node's key, a hook
  ancestry or a purge.
- PRV-06 is proven by `Taint`'s unit tests: an artifact records its
  sources and is untrusted if any is. `@PRV-06` stays pending: its rows
  need a launcher's `harness_meta`, a landmark, kernel output and an
  export.
- PRV-13's M1 part holds: `administration::origin` derives `witnessed`,
  `ingested` or `peer` per writer from the record and this node's device
  key alone, shared by recall and `parinn verify`. A replaced key's
  writers read `peer`, so do another node's. `parinn rebuild` writes the
  `origins` and worktree tables and the search index, appending no
  event; `parinn verify` exits 3 on a kept origin that differs.
  `@PRV-13` stays pending: no arrival event exists yet.
- LANE-01 and LANE-31's M1 part is proven by the `room` crate's unit and
  property tests: `standing_link` lets the first link in causal order
  stand, the lower commitment among concurrent ones, whatever the
  arrival order; `seat_for_run_event` routes a run's event to a joined
  room's run seat or else the personal-room seat. Both stay pending.
- ADM-13's M1 part holds: `e2e_no_git_write.rs` finds a repository
  unchanged after every component ran. It stays pending (M2).
- The I2 security review (PRV-02, PRV-13) found no path from untrusted
  content to a trusted event. Its hostile-checkout findings on the git
  reader are fixed: only regular files open, every object's SHA-1 is
  verified, the ancestor walk stops short of the home, `commondir` must
  be an ancestor of the git directory.
- I2, I4, I6, I8, I9 and I10 hold: no program, no socket, a changed
  repository identity audited and counted.

Inherited:

- Crates `repository` (`git_dir`, `root_commit`, `has_commit`,
  `commit_object_id`) and `room` (`branch`, `route`);
  `provenance::structural_field::sanitize`.
- `administration`: `origin::{Origin, KeySet, derive, is_device_seat}`,
  `trust_policy::{trust_level, Input, TrustLevel, Taint}`,
  `repository::{bind, show, plan, apply, Outcome}`, `Act::RepositoryBind`,
  `verify`'s `VerifyError::Derived`.
- `store`: `origins`, `worktree_identities` (rebuilt from bind events
  and acts, which carry `worktree_digest` and `replaces_provisional`),
  `repository_worktree_marks`; `EventKind::RepositoryBind`; five
  `repository_*` counters, `repository_identity_rebound` for a bind
  after a node identity change; fuzz target `git_reader`.
- `parinn rebuild`, `parinn repository show|bind`; MCP tool
  `mcp_server_get`; `parinn status` and the untaken-binding audit say
  whether any server took a binding;
  `mcp::Server::new` takes the device key; `event_search`'s `trust`
  filter applies before the ranking cuts.

For phases 13 to 16: the envelope's shape is untouched; `origin` and
`trust` come from `administration`; restore reads `trust_level`;
quarantine (phase 14) must change no origin.

For the stakeholder:

- The SRS and model edits (LANE-02, RCL-01, REC-19, §9.2, the device-key
  secret) landed as pull request #76.
- LANE-02's bind follows the landed row: an MCP server that holds a
  session binding looks at the clone, at its start and as it takes one.
- Deviation, PRV-13: after a node identity change every writer reads
  `peer` until a PRV-12 confirm, which no M1 command makes.
- Residual: a `.git` file may still name a worktree or submodule git
  directory of another repository, which then binds its commit id. M1
  holds no room, so two `@LANE-02` clauses (branch links, state) hold
  vacuously.
- Stakeholder paths touched: `tooling/scenario`, `features/`, `fuzz/` and
  `reach.rs`.

Parked: PRV-02's and PRV-06's remaining rows; PRV-13's arrival steps;
the trust of `user` events once OQ-40 closes.
