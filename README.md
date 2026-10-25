# ParleyInn

Your Claude agents as part of your team: work with them at your pace,
see what they build while they build it, and point at exactly what you
mean.

## Vision

- **At your pace.** Work with your agents and other people live, after
  the fact or asynchronously: watch runs as they happen, see what waits
  on you, or come back to a Catch up that links each line to the events
  behind it and shows every gap.
- **From every angle.** Understand what is being built while it is
  being built: the conversation, each changed file beside the events
  that changed it, and what backs each result, from a bare claim to a
  check.
- **By reference.** Link your posts to the exact events you mean, by
  address, and find any of them again by a word you remember, so people
  and agents know what you are talking about.

In a room with an intent, people and agents discuss the work, review results
and correct course in one conversation. ParleyInn aims to replace the pull
request
as that collaboration experience. A branch can be linked to the room; a pull
request, when used, remains the forge's review object for that branch. A forge
issue can link directly to the room without defining its intent. After the
forge read credential owner authorizes sharing that exact issue or linked
branch with the room, its discussion or pull-request review comments appear
in the conversation. A person can respond
in the room and explicitly send that response back to the forge.

Underneath, every run stays on your machines unless you turn on peers.
Your pinned constraints come back word for word after each compaction,
or named when the budget leaves them out. Agents recall history only
when they ask, each item enveloped with its trust level, and nothing
untrusted reaches them unless they ask or you act.

## Milestones

Specified, not built yet. Each milestone is a plan, listed here in
delivery order from its own front matter (`milestone-order`);
[§12](docs/srs/12-delivery-plan.md) holds its scope. M7's scope is
provisional, and each M9 item lands through its own proposal.

<?catalog
glob:
  - "plan/*.md"
  - "plan/*/plan.md"
where: '"milestone-order": int'
sort: 'numeric:"milestone-order"'
header: |
  | Milestone | Status | What it brings |
  |-----------|--------|----------------|
row: "| [{title}]({filename}) | {status} | {summary} |"
?>
| Milestone                                                                          | Status | What it brings                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [M1: record and recall](plan/2609292012_m1-record-and-recall/plan.md)              | 🔳     | The append-only record of writer logs under seat keys a node's device key certifies, provenance, trust and origin, pull-only recall over MCP, and the security and operations floor under them.   |
| [M2: pins and restore](plan/2609292013_m2-pins-and-restore.md)                     | 🔲     | Verbatim pins and the deterministic, TrustedText-only restore block after compaction, plus plugin install.                                                                                        |
| [M3: landmarks and lifecycle](plan/2609292014_m3-landmarks-and-lifecycle.md)       | 🔲     | Structural landmarks with tiered roll-up, backup and restore, purge with tombstones, and a rebuild identical in content.                                                                          |
| [M7: standalone room view](plan/2610112057_m7-standalone-room-view.md)             | 🔲     | The room view on one machine: Needs you, Catch up, search, integrity status, overlaps and review with what backs each result, plus the launcher, seals and the principal surfaces behind them.    |
| [M4: the hermetic compute kernel](plan/2609292015_m4-compute-kernel.md)            | 🔲     | A per-run SQL kernel in a resource-limited worker with read-only, quarantine-aware access to the record.                                                                                          |
| [M5: hardening and evaluation](plan/2609292016_m5-hardening-and-evaluation.md)     | 🔲     | The remaining P1 requirements, the full §11 evaluation, an external security review and a two-week pilot.                                                                                         |
| [M8: peer network](plan/2610112058_m8-peer-network.md)                             | 🔲     | Peers enrolled by key: machines and other principals sync rooms after any partition with no central service, ephemeral nodes hand over their record, and invites, directed posts and endorsement. |
| [M9: review, publishing, bridges](plan/2610112059_m9-review-publishing-bridges.md) | 🔲     | Forge review and linked issues in the room with replies sent to the forge, handover, bundles, read-only publishing and outbound bridges, each through its own proposal.                           |
<?/catalog?>

Marking part of an event or an image, and sharing that mark, are
pending use cases ([UC-002, UC-003](docs/use-cases.md)) aimed at M7 to
M9 but not yet specified.

## Status

This repository holds:

- the [Software Requirements Specification](docs/srs/index.md), the
  normative source, and the [domain model](docs/domain-model/index.md)
  that defines its terms;
- one Gherkin scenario per requirement under `features/`, almost all
  `@pending`, kept in step with the SRS by a gate test;
- the `parinn` core executable, with `parinn version` and `parinn help`
  so far;
- CI, a nightly fuzz job, an agent review of every pull request and a
  signed release pipeline, every gate written in Rust;
- the delivery plans in [PLAN.md](PLAN.md).

The executable is `parinn`. Its home is `~/.parinn`, or the directory
`PARINN_HOME` names. Rooms are addressed as `parinn:room/…`, and commit
trailers read `ParleyInn-Room:` and `ParleyInn-Link:`.

<?include
file: docs/srs/invariants.md
heading-level: "2"
?>
## 1.3 Invariants

The invariants are ParleyInn's contract. Every requirement serves at least one
of them. A change that would break an invariant is a design change requiring
security review and a new major version, not a bug fix.

- **I1 — Nothing is lost.** Every event an agent saw or produced remains
  recoverable by its stable address, (writer, seq), across any number of
  compactions and agent runs, on every node that holds its writer's log. The
  only exceptions are secrets removed by redaction before storage or on import,
  and data the node's principal explicitly purges, or removes under a retention
  policy that principal or managed policy sets. Every exception is recorded.
- **I2 — No automatic path from untrusted content to the model.** Content that
  originates outside the trusted sources (tool output, web, MCP servers, files,
  the model's replies, and anything another node or principal produced, except
  the agent's principal's acts signed by a device key it certified, posts and
  pins written from a device seat such a key certified, a pin version that
  principal stamped, or what a trust grant of that principal covers) reaches the
  model only inside an untrusted-data envelope when the agent explicitly calls a
  recall tool, or through one of the closed paths below that a principal act
  names. The trusted sources are this node's own `operator` and structural
  events (hook observations only as PRV-08 allows), the `harness_meta` events
  its launcher recorded, and, while the deployment mode is `interactive` and no
  run on the node leaves R3 or R8 open, the `user` events matched to a prompt
  (PRV-08), and, once PRV-10 ships, principal acts signed by a device key the
  agent's principal certified and posts and pins written from a device seat such
  a key certified, within that key's scope, a pin version a principal stamped,
  for that principal's own agents, and the posts and pins a trust grant of the
  agent's principal covers. ParleyInn writes to an agent only through a closed
  set of paths. Without a principal act: restore blocks (INJ-01, INJ-02,
  INJ-04), built only from qualifying pins, trusted structural fields and fixed
  text ParleyInn ships; and opt-in notices (INJ-10) and the fixed templates of
  OWN-04 and OWN-07, each built only from fixed text ParleyInn ships, trusted
  structural fields and ids. On a principal act recorded at that time: through
  the harness's own input, only principal-typed text, a fixed template that
  references ids, or a post a principal endorsed exactly as shown inside the
  template of OWN-08. Under a delegation grant its principal recorded (OWN-23),
  and, for an agent of another principal, an acceptance grant that agent's
  principal recorded (OWN-26): a delegated task inside the fixed template of
  OWN-24. Once its requirements ship, a principal may also trust another
  principal by key for its own agents, in one room or everywhere; the pins that
  principal wrote from device seats one of its device keys certified then
  restore to those agents, and its posts reach them inside the fixed template of
  OWN-29. ParleyInn applies such grants and never grants trust itself. No other
  write to an agent exists.
- **I3 — Constraints are never summarized.** Every pin that restores is stored
  verbatim and restored verbatim after every compaction, or named by id and
  count when the budget or run ambiguity (INJ-05) omits it.
- **I4 — Each component stays inside one declared network boundary, and only the
  core builds what reaches the model.** B0, the core (hook handlers, the MCP
  server, the kernel worker, the CLI, the TUI and everything that builds what
  reaches the model), opens no socket and makes no outbound connection. B1,
  machine: a component may listen only on loopback or on a local endpoint only
  the same OS user can reach, and connect nowhere else; the launcher carries
  into the harness's input only text the core built. B2, peer: encrypted,
  mutually authenticated connections to nodes and paired phones the node's
  principal enrolled by key, off until that principal turns it on. B3, public:
  read-only publishing and outbound exchange with hosts the node's principal
  names, off until turned on. Managed policy can disable B1, B2 and B3. No
  component sends telemetry or depends on a central or third-party service.
  ParleyInn's components send data off the machine only as recalled content an
  agent receives through a tool call, or as what ParleyInn writes to an agent
  through I2's closed paths, both of which the harness sends to its model, or
  through a B2 or B3 component the node's principal turned on; whatever such a
  component brings in is trusted only as I2 allows.
- **I5 — Bad data can be removed from circulation without destroying the
  record.** Any event, span, run, writer or derived artifact can be quarantined
  from recall immediately on the node that records the quarantine, while the
  record stays intact for forensics. A quarantine reaches another node only as a
  quarantine request that node's principal applies.
- **I6 — No silent failures.** Every dropped, rejected, redacted, timed-out, or
  failed operation is counted, logged, and visible to the node's principal.
- **I7 — Harness configuration changes only on explicit instruction.** ParleyInn
  changes harness configuration only through explicit install and uninstall
  operations, shows the change first, and never overrides managed policy or the
  harness's managed settings.
- **I8 — Isolation follows the principal.** All local state is bound to one
  principal's home with strict permissions, and ParleyInn refuses to operate on
  a home that does not belong to the OS user running it. Content another
  principal wrote, or another node wrote other than this principal's acts signed
  by a device key it certified and its posts and pins written from a device seat
  such a key certified, is kept as theirs: attributed to its seat key, and
  untrusted unless this principal's own stamp or trust grant covers it. On its
  own it never makes this principal's agents trust it, nor extends their recall.
- **I9 — ParleyInn never degrades the agent.** A ParleyInn failure never blocks
  or slows the agent beyond defined budgets. ParleyInn fails open, except where
  continuing would violate I2, I4, or I8.
- **I10 — Everything derived is rebuildable.** All derived artifacts (the search
  index, landmarks, active pins, quarantine set, room state, origins, trust
  levels, run and integrity statuses, queues, evidence and proof classes, stats)
  are a deterministic function of the set of writer logs a node holds and the
  node's own key set, independent of the order in which logs arrived. Rebuilding
  reproduces their content exactly, whatever their storage layout.
<?/include?>

## Development

```sh
cargo test --workspace                          # tests and scenarios
cargo test -p scenario --test bdd               # the requirement matrix
cargo clippy --workspace --all-targets          # lint
cargo run -p coverage                           # coverage per test layer
mdsmith check .                                 # Markdown
```

[docs/development.md](docs/development.md) has the full reference.
[CLAUDE.md](CLAUDE.md) has the working rules for agents and people
alike.

## Security

Report vulnerabilities privately; see [SECURITY.md](SECURITY.md).

## License

MIT; see [LICENSE](LICENSE). Open question OQ-09 in the SRS proposes
Apache-2.0 for its patent grant.
