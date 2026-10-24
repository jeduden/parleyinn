---
n: 11
title: "Trust, origin, branch links and repository identity"
status: "✅"
result: false
---
# Phase 11: trust, origin, branch links and repository identity

Requirements. PRV-02's trust policy, PRV-06's taint, PRV-13's origin,
LANE-02's repository identity, LANE-01's routing, LANE-31's rule for two
branch links naming one branch, OWN-01, MEM-01 and ADM-13's M1 steps.
LMK-03's sanitizer lands early for LANE-01, with a fuzz target.
`@OWN-01` and `@MEM-01` come off `@pending`; `@PRV-02`, `@PRV-06` and
`@LANE-02` stay pending where M1 cannot show them. PRV-02 and PRV-13 must
pass a recorded I2 security review before they ship: the
persona-security-officer reviews this phase's code.

The trust policy (PRV-02, I2): `trust_policy::trust_level` takes an
event's provenance, kind, origin, recorder, writer, recording node, the
recorded deployment mode, prompt match, attended mark, and whether a run
leaves R3 or R8 open. No trust level is stored.

- Trusted: `operator` events, and structural events that are no hook
  observation, in `witnessed` writers recorded here.
- Untrusted: all else, every `user` event too. Under M1 to M5 each run
  leaves R3 and R8 open (OWN-22, OQ-40). Rows needing a signed sandbox
  state are raised.
- `bundle` reads as `peer`. Stamps, grants and device-key acts wait for
  PRV-10 (M7).
- `event_get` and `event_search` show the derived level, and the `trust`
  filter works. `parinn verify` reports a widening act failing OWN-22.

Taint (PRV-06): a `Taint` lists the addresses it `derives_from`. It is
untrusted if any is, or if it holds the free text of a cut or neutral
principal act. Sanitized structural fields stay usable. Only the recall
result exists in M1; landmark (M3), kernel output (M4) and export rows
are raised.

Origin (PRV-13, I10): `administration::origin` derives it per event, in
PRV-13's order, from the record and the node's key set alone. A seat
counts when a key of this node certified it and its certificate record
is in the genesis device seat. A replaced or minted-anew device key's
writers read `peer`. Quarantine changes none. The envelope takes it from
this one function.

- `parinn rebuild` rebuilds the `origins` table and the search index,
  reading arrival events and appending none. `parinn verify` exits 3,
  naming "derived artifact consistency", when `origins` differs.
- No arrival event exists in M1 (M3, M8, M9), so `@PRV-13` stays pending.

Repository identity (LANE-02): crate `repository` reads `.git` inside the
core, no program start, no socket (I4, ADM-13), with an inflate of its
own so the core adds no dependency (ENG-18). It finds the lowest
parentless commit of the default branch within a deadline (ENG-03).
`RepositoryIdentity` is `Bound(commit_id)` or `Provisional`.

- `SessionStart` binds it once per directory as a structural event of the
  device seat. A clone without commits is provisional and binds later,
  without rewriting the record.
- A change is audited and counted `repository_identity_changed`; no
  branch link is relinked. `parinn repository show|bind`; `bind` is a
  widening principal act.
- Local names and logs hash the identity under a node secret.

Branch links and routing (LANE-01, LANE-31): `BranchIdentity` holds the
repository identity, remote URL and name, sanitized by
`structural_field::sanitize` (80 characters, else `<field:HASH8>`).
`branch::standing_link` lets the first link in causal order stand, the
lower commitment among concurrent ones. The rest are void and shown.
`route::seat_for_run_event` picks the run seat of a room naming the run's
branch, else the personal-room seat, never refused. Unit and property
tests prove joined rooms (ENG-08).

OWN-01: a run records under one principal. MEM-01: no file outside the
store holds a canary after any hook or `parinn rebuild`. ADM-13: no
command changes a repository's worktree, refs, notes or hooks.

Gate: `@PRV-02`, `@PRV-06`, `@LANE-02`, `@OWN-01` and `@MEM-01` pass,
or name what keeps them pending. The M1 steps of PRV-13, LANE-01,
LANE-31 and ADM-13 pass. The sanitizer's fuzz target runs.
