# Agent Notes

<!-- Included from CLAUDE.md: edit it, then run `mdsmith fix .`. -->

Instructions for AI coding agents (Codex, Copilot, Claude).

<?include
file: CLAUDE.md
strip-frontmatter: "true"
?>
# CLAUDE.md

## Project

ParleyInn — a lossless, security-first context layer for long-running Claude
agents. It keeps an append-only record of every agent run, restores pins
verbatim after compaction, and lets Claude recall exact history on demand.
Stored history never becomes a prompt-injection channel.

Status: pre-implementation. The specification, requirement matrix, CI and
release pipeline, and `parinn` core executable are here; plans add the rest.

## The Invariants Come First

The main principle. The ten invariants' single source is
[docs/srs/invariants.md](docs/srs/invariants.md); this file and every other copy
include it, so edit only the source and run `mdsmith fix .`.

<?include
file: docs/srs/invariants.md
heading-level: "absolute"
?>
### 1.3 Invariants

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

What they mean for everyday code:

- **I2** — only `TrustedText` reaches a restore block; recall is pull-only and
  always enveloped.
- **I4** — no network in the core, and no program start but its own kernel
  worker (ENG-16). The `parinn` crate's clippy.toml refuses `std::net` and
  `std::process`, and its reach test refuses any linked crate.
- **I6** — audit and count each dropped, rejected, redacted or failed operation.
- **I9** — hook handlers fail open unless that would break I2, I4 or I8.
- **I10** — code that builds derived artifacts reads no clock or randomness.

Where usefulness and safety conflict, pick safety; make convenience opt-in.

## Docs

<?catalog
glob:
  - "docs/*.md"
  - "docs/domain-model/index.md"
  - "docs/srs/index.md"
  - "SECURITY.md"
  - "DEPENDENCIES.md"
  - "CHANGELOG.md"
sort: path
header: ""
row: "- [{filename}]({filename}) — {summary}"
?>
- [CHANGELOG.md](CHANGELOG.md) — Release notes and upgrade notes per version, newest first (ENG-23).
- [DEPENDENCIES.md](DEPENDENCIES.md) — Every direct dependency, listed through the decision record that justifies it (ENG-18, ENG-26). The ENG-18 scenario fails the build when the manifests and the dependency ADRs disagree or a license is off the allow-list.
- [docs/development.md](docs/development.md) — Build commands, static analysis and supply-chain checks, and how CI, the nightly fuzz job and the signed release pipeline work. Testing has its own page, docs/testing.md.
- [docs/domain-model/index.md](docs/domain-model/index.md) — ParleyInn's domain model: the closed set of concepts with their definitions, how they relate, the terms that are not ParleyInn concepts, and how names in code, docs and UI follow the model. The SRS links here for every term, and the domain-model agent reviews against it.
- [docs/srs/index.md](docs/srs/index.md) — The ParleyInn Software Requirements Specification — the normative source for every requirement id a feature scenario is tagged with.
- [docs/testing.md](docs/testing.md) — How ParleyInn is tested: the test pyramid of unit, integration and end-to-end tests, line coverage and test counts per test layer with their floors, the executable requirement matrix (one Gherkin scenario per SRS id), drift injection, and the test-engineer agents that monitor and shape the pyramid.
- [docs/tooling.md](docs/tooling.md) — Developer tool setup and cache behavior, with tool pins, check commands and Rust configuration imported from their source files by mdsmith.
- [docs/use-cases.md](docs/use-cases.md) — Pending use cases, one file each under docs/use-cases, before the SRS covers them: who wants each, what it is, the milestone it targets and its status. Not normative; an entry moves into the SRS when a plan takes it up.
- [SECURITY.md](SECURITY.md) — How to report a vulnerability in ParleyInn privately, the 90-day coordinated disclosure policy, which versions get fixes, and how to verify a release (ENG-24, ENG-20).
<?/catalog?>

The SRS sections under [docs/srs](docs/srs/index.md) each carry a one-line
summary in the index there; open only the section a task touches.

## Development Workflow

- Any change follows Red/Green TDD: failing test, then pass, then commit
- Keep commits small and focused on one change
- Run `mdsmith check .` before committing; all Markdown must pass
- Never modify `.mdsmith.yml` (linter configuration) without explicit consent
- Every direct dependency needs an accepted ADR under `docs/adr/`, listed in
  [DEPENDENCIES.md](DEPENDENCIES.md) (ENG-18, ENG-26); aim to add none
- Run `mdsmith merge-driver install` once per clone (development.md says why)

## Review

Agents review and approve each other's pull requests (ENG-21). A pull request
touching a path [CODEOWNERS](.github/CODEOWNERS) assigns — the SRS, the gates
that enforce it, CI and its tooling, the supply-chain policy, the agent
instructions, and the security-sensitive packages until a security reviewer is
named — also waits for the stakeholder's approval. Keep such changes out of code
pull requests. On GitHub, write every `@`-word in backticks: a bare one, such as
`@I2`, mentions that GitHub user (jeduden/parleyinn#22).

## Requirements and Scenarios

The SRS is normative; the scenarios under `features/` make it executable. Every
requirement id has exactly one scenario, tagged with the id, its priority and
its traced invariants, and a gate test fails the build when the two drift. The
mechanics are in [docs/development.md](docs/development.md).

- Implementing a requirement means making its scenario pass: drop `@pending`,
  make the steps concrete, bind them in
  `tooling/scenario/tests/bdd/<section>.rs`. Unit tests alone do not close a
  requirement.
- A new or changed requirement lands with its scenario in the same change,
  `@pending` until written.
- Never delete, retag or re-pend a scenario to make CI green. A scenario that
  cannot pass as written is a finding to raise.
- A check that inspects the repository's own records lands with a drift case in
  `tooling/drift/src/cases.rs` that proves it fails (ENG-27).
- Behavior surfaced mid-work — a bug found while fixing something else — is
  checked against the matrix before being judged covered.

## Tests

Unit tests prove each function directly, 99% per crate; integration and
end-to-end tests guard behaviour ([testing.md](docs/testing.md)). The
`test-engineer` agents review tests; the `test-shape` skill reshapes them.

## Domain Model

ParleyInn's concepts and relations live in
[docs/domain-model/](docs/domain-model/index.md), along with excluded terms and
naming rules. The domain-model agent reviews the whole model. A specialist per
file goes deep on it (`.claude/agents/domain-model-*.md`). Consult them:

- on every change to the model itself, and on every proposal to change it;
- on every change to the SRS under `docs/srs` and to the scenarios;
- before naming a function, type, module, crate, CLI verb, MCP tool, settings
  key or event;
- on documentation, UX and UI copy, and developer experience: error and help
  text, logs, setup.

Speak only in the domain model's concepts. A new concept lands there before
anything uses it. Its findings block a change until fixed or the stakeholder
rules on them.

## Agents

The subagents under `.claude/agents`, each one perspective:

<?catalog
glob: ".claude/agents/{domain-model,persona-*,test-engineer}.md"
sort: path
header: ""
row: "- [{name}]({filename}) — {description}"
?>
- [domain-model](.claude/agents/domain-model.md) — Guards ParleyInn's domain model as docs/domain-model/ defines it. Reviews every change to the model and every SRS change, and is consulted on names (functions, types, modules, command and tool names, other identifiers), documentation, UX and UI copy and developer experience. Reports every term used outside the model. Never approves.
- [persona-agent](.claude/agents/persona-agent.md) — Claude itself as a user of ParleyInn: an agent that needs its constraints back after compaction and exact recall of its own history. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-fleet-developer](.claude/agents/persona-fleet-developer.md) — A developer running five or more agents at once on one machine, each in its own worktree, and steering them through the day. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-live-collaborator](.claude/agents/persona-live-collaborator.md) — A teammate joining someone else's room live, to help, pair or take over, alongside agents that are not theirs. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-multi-machine-developer](.claude/agents/persona-multi-machine-developer.md) — A developer whose agents run across a laptop, a home server and ephemeral cloud environments, often offline or on bad networks. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-oss-maintainer](.claude/agents/persona-oss-maintainer.md) — An open-source maintainer receiving an outside contribution together with its room, from someone they do not know. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-platform-engineer](.claude/agents/persona-platform-engineer.md) — A platform engineer running ParleyInn for many developers on self-hosted runners and Agent SDK workers. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-returning-owner](.claude/agents/persona-returning-owner.md) — Someone coming back after hours or days who asks one question first: what did my agents do while I was away? Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-reviewer](.claude/agents/persona-reviewer.md) — A reviewer deciding whether a room's branch may land: reads the story, the diff and the evidence, and signs off or asks for changes. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [persona-security-officer](.claude/agents/persona-security-officer.md) — A security reviewer who must sign off that ParleyInn adds no new exfiltration or injection path, and that its audit trail holds. Reviews a pull request, plan, pitch, design or spec from this perspective and reports where it fails them. Never approves.
- [test-engineer](.claude/agents/test-engineer.md) — Guards ParleyInn's test pyramid as a whole: each behaviour proven at the lowest test layer that can prove it, each requirement closed by its scenario, coverage held per test layer, and the test kinds the SRS asks for. Reviews a pull request, plan, design or spec and reports where its tests fall short. Never approves.
<?/catalog?>

## Plan Maintenance

Plans live in `plan/`, in frit's format ([plan/proto.md](plan/proto.md)),
indexed by [PLAN.md](PLAN.md). The `plan-*` skills under `.claude/skills` drive
them. When implementing work tracked by `plan/`:

- Update the plan file **as part of implementation**, not a follow-up
- Check off tasks and acceptance criteria as they're verified
- Move front-matter `status`: `🔲` → `🔳` on start, `✅` when done
- If implementation deviates, update plan text to match
- Run `mdsmith fix PLAN.md` after editing front matter
- Each phase names the ids it closes and the scenarios it takes off `@pending`

## Reporting

Report in the SRS's terms, not the source's.

- Speak of requirements met, scenarios taken off `@pending`, and invariants
  upheld — not the functions or types touched.
- Reach for a source entity only when the requirement frame cannot carry the
  point, and trace from the requirement down to it.
- Name the mechanism that verified a claim — cucumber scenario, fuzz target,
  golden file, CI job — before folding its result into plan terms.

## Code Style

- rustfmt, and clippy with every warning denied; the workspace lints forbid
  `unsafe` and refuse `unwrap`, `expect` and `panic!` outside tests
- Small functions, each with a unit test in the `tests.rs` beside its module
- No mutable global state; I/O behind a trait or closure a test can inject;
  every blocking call takes a deadline (ENG-03)
- Errors keep their cause (`source()`); a counted failure class gets its own
  variant; panics stop at the hook and MCP entry points (ENG-04)
- Log with `tracing`, through redaction (ENG-05)
- Error messages: lowercase, no trailing punctuation; return errors, never panic
- Add a defensive branch only after a failing test that takes it (red/green)

## Isolation for Agents and Tests

Never run a command that writes to, deletes from, or purges the real
`~/.parinn`, `$PARINN_HOME` or `~/.claude` of the machine you run on. Point
`HOME` and `PARINN_HOME` at a temporary directory first, as the test world does
(ENG-14). No instruction here may direct an agent to run a destructive command
against non-isolated state.
<?/include?>
