# Bernstein: a deterministic orchestrator and governance layer for CLI agents

Date: 29 October 2026. Sources: <https://bernstein.run/>, its `llms-full.txt`,
`/why-bernstein` page and documentation at <https://docs.bernstein.run/>, read
on this date, and a shallow clone of
<https://github.com/sipyourdrink-ltd/bernstein> at commit `022101f` (28 October
2026, v3.23.0). Source claims below cite paths in that clone. This is a reading
of documentation and selected modules, not a full audit or a test of the
running product; Bernstein is about 2,200 Python files and 860,000 lines. A
guarantee not found in these sources is unknown, not absent.

## What it is

Bernstein describes itself as "the open-source governance layer for AI agents"
that "runs on policy as code". It is an Apache-2.0 Python package, beta, with
one maintainer, that runs many CLI coding agents in parallel: Claude Code,
Codex, Gemini CLI, Aider and, by its own count, 40 to 53 adapters in all. A run
has four stages: a manager decomposes a plan into tasks; each task's agent
spawns in its own git worktree; a "janitor" runs lint, type and test gates,
with optional cross-model review; and a merge queue squash-merges only the
diffs that pass. Failing tasks move to a dead-letter queue and do not merge.

Its central claim is that no model sits in the coordination loop: the tick
that decides who runs, who is blocked and what merges is ordinary Python, so
"the same plan replays byte-identically". Models are used only in opt-in
places: routing a task to a model, and an LLM judge for best-of-n. Agents are
short-lived, one to three tasks each, so "nothing carries over between tasks
except the file system". Sources: <https://bernstein.run/>,
`docs/architecture/WHY_DETERMINISTIC.md`,
`docs/architecture/ARCHITECTURE.md`.

State lives in `.sdd/` in the repository: task queues, worktrees, cost
records, a write-ahead log, and the audit log. A FastAPI task server listens on
`127.0.0.1:8052` during a run (`BERNSTEIN_BIND_HOST` can widen it to
`0.0.0.0`), and `bernstein mcp` serves an MCP server over stdio or HTTP.
Sources: `src/bernstein/core/server/server_app.py`,
`src/bernstein/core/server/server_launch.py`.

## Where it overlaps

Bernstein is not a context layer; it is closer to ParleyInn's launcher and
lane than to its record. The overlap is in four places.

**Tamper-evident record.** The lineage spine and replay journal are always on.
With `--audit`, Bernstein also writes an HMAC-SHA256 chained JSONL audit log,
one file per UTC day under `.sdd/audit/`, keyed from
`~/.local/state/bernstein/audit.key` outside the audited directory so that
"whoever can edit the log cannot re-sign it". A run can end with an Ed25519
signed receipt that binds the journal and lineage heads and verifies offline
without the HMAC key. `bernstein replay RUN_ID --verify` recomputes the
journal's Merkle head and names the first divergent step. ParleyInn's
per-writer hash chain (REC-10), audit log with its own chain (OPS-02),
`parinn verify` (ADM-09) and head receipts cover the same need with a
different trust anchor: a writer's own chain and seat keys rather than one
install-wide HMAC key. Sources: <https://bernstein.run/>,
<https://docs.bernstein.run/>.

**Memory that is never edited in place.** `MemoryChain` makes every memory
write an append-only, content-addressed, HMAC-tagged record naming its actor,
time and the lineage step that produced it; `why` walks a claim back to its run
and step, and `forget` appends a signed tombstone instead of deleting. This is
the same shape as ParleyInn's I1 record with tombstones (ADM-07) and its
provenance (PRV), at a smaller scope. Source:
`src/bernstein/core/memory/chain.py`.

**Compaction.** Bernstein compacts context itself in tiers: `micro` every turn
(lossy on tool-call bodies), `auto` at a per-session threshold (summarises tool
runs), `session_memory` at session end, and `time_based` pruning. It hashes the
regions it compacts and the files they reference, so a later check can detect
drift between a summary and the disk, and stamps each result with
`COMPACTION_POLICY_VERSION`. Its blog mentions a gate on credential-shaped
content entering compaction. Nothing in these sources holds constraints out of
the summary; a constraint is as lossy as any other text. ParleyInn's I3 takes
the opposite position: it leaves compaction to the harness and restores
qualifying pins verbatim after it (INJ-01), or names them by id and count.
Sources: `src/bernstein/core/memory/compaction/tiers.py`,
`src/bernstein/core/memory/compaction/verification.py`.

**Worktree isolation and verification before merge.** One worktree per task,
gates before merge, and cross-model review overlap with ParleyInn's lanes,
branches and results with evidence classes
([git and forge](../../../docs/domain-model/git-and-forge.md)). Bernstein's
reviewer prompt may contain only the spec, the diff and the test output; the
gate rejects any attempt to add the implementer's transcript or bulletin
history, so the reviewer is not steered by what it reviews. Source:
`src/bernstein/core/quality/review_pipeline/review_gate.py`.

## Trust and what reaches the model

Bernstein's injection defence is structural and orchestration-time. Every tool
is tagged with which of Simon Willison's "lethal trifecta" capabilities it
carries (private data, untrusted input, external communication); a spawn whose
configured tool chain covers all three is refused, enforced by default, and an
unknown tool counts as carrying all three. New agents start at an `UNTRUSTED`
trust level: read-only, no network, no `.sdd/` writes. Sources:
`src/bernstein/core/security/capability_matrix.py`,
`src/bernstein/core/agents/agent_trust.py`.

Content still flows between agents through the prompt. Persistent memory
replays into every spawned prompt verbatim; a `MemoryTrustPolicy`, on by
default, now replays only rows that are untagged or written under an adapter
the operator allow-listed. Its docstring records why: before it, a row written
under any adapter "was replayed into every other adapter's spawned prompt", a
cross-adapter memory-poisoning path. Recent bulletin-board activity, which
other agents wrote, enters each spawn prompt as a plain "Team awareness"
section, and a parent task's gathered context enters as "Parent context
(inherited)". The sources reviewed show no untrusted-data envelope around
either. Under ParleyInn's I2 both would be untrusted content (the model's
replies and another writer's output) and could reach an agent only through
explicit, enveloped recall or a closed path a principal act names. Sources:
`src/bernstein/core/memory/trust_policy.py`,
`src/bernstein/core/agents/spawn_prompt.py`.

## Network boundary

Telemetry is off by default; `DO_NOT_TRACK=1` wins over every other setting,
and an opted-in install keeps an on-disk queue of every event it sent as the
operator's own audit record. Optional outbound paths include storage sinks,
tunnels, chat bots, a hosted read-only MCP server and Cloudflare sandbox
execution. An `--profile airgap` denies egress. ParleyInn's I4 is stricter in
kind: the core opens no socket at all, and a loopback component is B1, not
part of the core. Bernstein's core is itself a loopback HTTP server, and its
task server is the coordination point every agent calls. Sources:
`src/bernstein/core/telemetry/__init__.py`, <https://bernstein.run/>.

## What ParleyInn can learn

1. **The prompt is the injection channel between agents.** Bernstein already
   met cross-adapter memory poisoning in practice and closed one path with a
   provenance allow-list, while bulletin and parent-context sections still
   enter raw. That is the concrete failure I2 exists to prevent. It argues for
   keeping ParleyInn's posts, handoffs and any future team-awareness feature on
   the enveloped recall path or inside a fixed template, never as free text
   in a spawn prompt.
2. **A structural capability check complements the envelope.** The lethal
   trifecta matrix refuses a configuration, not a string, and fails closed on
   unknown tools. ParleyInn's I2 governs what ParleyInn writes to an agent; it
   says nothing about the tool set the launcher hands a run. A launcher check of
   the same kind, recorded as a structural event and counted when it refuses
   (I6), could be worth a use case.
3. **Isolate the reviewer from what it reviews.** The review gate's closed
   input list (spec, diff, test output) is a cheap, testable rule. It fits
   ParleyInn's verdicts and evidence classes: a verdict whose reviewer saw the
   implementer's transcript is weaker evidence than one that saw only the
   result.
4. **Compaction outputs need a version and a drift check.** Even though
   ParleyInn does not compact, its restore block is a derived artifact. Naming
   the restore format's version in what it records, and hashing what a pin
   refers to so a later recall can show it changed, would make I3's restore
   auditable the same way.
5. **Receipts that verify offline sell the audit story.** `bernstein verify
   receipt` needs no key and no `.sdd/` directory, and a public web page
   verifies one. ParleyInn's head receipts would gain from the same
   single-command, offline check a reviewer can run on a file alone.
6. **Map controls to the failures they prevent.** Bernstein publishes a
   governance coverage page that maps each control to a failure class and to
   DORA, NIS2 and EU AI Act Article 12 evidence needs. ParleyInn's appendix B
   maps requirements to invariants; a security officer or platform engineer
   would also want the outward mapping.
7. **Degraded sessions can be detected from outcomes.** Bernstein counts
   consecutive cross-model rejections per session and, past a threshold,
   checkpoints the session and hands a fresh agent a summary of what was tried.
   ParleyInn has the record that would let a principal see the same signal,
   but a fresh agent's summary must reach it only through I2's paths.

## Where ParleyInn differs

Bernstein optimises for throughput and replay of a deterministic plan: fresh
agents per task, a shared file system, summaries handed forward. Its record
proves what the orchestrator decided; it does not promise that every event an
agent saw stays recoverable by stable address (I1), and its install-wide HMAC
key makes tampering evident only to someone who does not hold that key.
ParleyInn instead serves a long-running agent and its principal: a lossless
record by writer and seq, pins that survive every compaction verbatim, and a
closed set of paths to the model. The two could stack: a Bernstein-style
orchestrator could launch harnesses that ParleyInn records, provided its own
prompt sections stay outside ParleyInn's trusted sources.

## Competitive reading

Bernstein is an adjacent tool, not a direct competitor. It competes with
ParleyInn's fleet and lane story for developers running many agents at once,
and with its audit story for regulated buyers, where its offline receipts,
compliance mapping and air-gap profile are already shipped. It does not offer
lossless recall, verbatim pin restore or a closed injection contract. Its
strongest lesson is negative: a capable, security-minded orchestrator still
moves agent-written text between agents as plain prompt text, and had to patch
one such path after the fact.
