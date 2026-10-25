# Beads (bd) and Gas Town: storage and sync models

Method note: primary sources are the two repositories, cloned on 2026-10-01
(beads `main` @ 555b01d, committed 2026-10-01; gastown `main` @ 649b832, last
commit 2026-07-23). `github.com/steveyegge/beads` and `.../gastown` now resolve
to the `gastownhall` org (same HEAD). Legacy design was read from the
`v0.47.0` tag (2026-01-11), the last major line before the Dolt switch.
URLs below use `gastownhall/beads/blob/main/...` for current docs and
`steveyegge/beads/blob/v0.47.0/...` for the legacy design.

## 1. Beads — what is stored, where, and what is the source of truth

### Takeaway

Current (v1.x, 2026): **Dolt is the sole store and source of truth**; data
lives in a gitignored Dolt directory under `.beads/`, every write is a Dolt
commit, and `.beads/issues.jsonl` is only a passive export. Superseded
(Oct 2025 – Feb 2026): a gitignored SQLite cache (`.beads/beads.db`) plus a
**git-tracked `.beads/issues.jsonl` that was the source of truth**, kept in
step by a 5-second debounced auto-export and an auto-import after `git pull`.

### Cited Findings

**Current design (Dolt)**

- "Beads uses **Dolt** as its sole storage backend -- a version-controlled SQL database"; "Dolt is the source of truth. Every write auto-commits to Dolt history" — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Record kinds: issues, dependencies (typed edges `blocks`, `parent-child`, `related`, `discovered-from`), labels, comments, events (audit trail) — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Issue fields include title, description, design, acceptance_criteria, notes, status (`open`, `in_progress`, `blocked`, `deferred`, `closed`, `pinned`, `hooked`), priority 0–4, issue_type (incl. `message`, `molecule`, `gate`, `decision`), assignee, timestamps, `metadata` JSON, plus claim-lease fields (`lease_expires_at`, `heartbeat_at`) and wisp fields (`ephemeral`). Internal `content_hash` (SHA-256 of canonical content) is used for change detection and never exported — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Two modes: **embedded (default)**, Dolt in-process, single writer, data in `.beads/embeddeddolt/`; **server** (`bd init --server`), external `dolt sql-server` for concurrent writers, data in `.beads/dolt/` — [README](https://github.com/gastownhall/beads/blob/main/README.md)
- Layout: `embeddeddolt/` and `dolt/` gitignored; `dolt-server.pid/.log/.port` gitignored; `issues.jsonl` = "Passive JSONL export for viewers and interchange"; `metadata.json` and `config.yaml` tracked in git; `bd init` writes `.beads/.gitignore` — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- "`.beads/issues.jsonl` is an export ... not the source of truth or a backup" — [README](https://github.com/gastownhall/beads/blob/main/README.md); "JSONL import is upsert-only; it cannot infer that records absent from an export were deleted" — [sync-concepts.md](https://github.com/gastownhall/beads/blob/main/docs/core-concepts/sync-concepts.md)
- Embedded mode creates one Dolt commit per `bd` write; server mode defaults auto-commit OFF because "Firing `DOLT_COMMIT` after every write under concurrent load causes 'database is read only' errors" — [architecture/dolt.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/dolt.md)
- Embedded store takes an exclusive flock for the store's lifetime; a second concurrent opener gets an error (fixed a nil-pointer panic, v1.0.0) — [CHANGELOG 1.0.0](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- **Agent memory**: `bd remember "insight" [--key k]` stores a memory "that persists across sessions and account rotations"; memories "are injected at prime time (bd prime)" via a SessionStart hook — [cli-reference/remember.md](https://github.com/gastownhall/beads/blob/main/docs/cli-reference/remember.md). Memories are stored as rows in the database `config` table under a `kv.memory.` key prefix — [memoryops/doc.go](https://github.com/gastownhall/beads/blob/main/memoryops/doc.go), [internal/storage/storage.go](https://github.com/gastownhall/beads/blob/main/internal/storage/storage.go)
- Three "event" systems: fire-and-forget script hooks in `.beads/hooks/`; per-issue audit history (`bd history <id> --events`, old/new field values); and an opt-in **events journal** — "one workspace-wide, sequence-ordered stream of committed mutations", written "in the same transaction as the mutation itself", tailable with a resumable cursor (`bd events tail --since N`), off by default, local to one clone, retention floors 7 days / 100,000 rows with auto-prune — [events-journal.md](https://github.com/gastownhall/beads/blob/main/docs/reference/events-journal.md)
- Optional `.beads/interactions.jsonl` append-only audit sidecar (fields include kind, actor, issue_id, model, prompt, response, tool_name); now opt-in (`audit.enabled` default false) — [internal/audit/audit.go](https://github.com/gastownhall/beads/blob/main/internal/audit/audit.go), [cmd/bd/info.go](https://github.com/gastownhall/beads/blob/main/cmd/bd/info.go)
- Wisps (ephemeral molecule steps) are flagged `Ephemeral=true`, "Local by design: excluded from federation push by default" — [workflows/wisps.md](https://github.com/gastownhall/beads/blob/main/docs/workflows/wisps.md)

**Superseded design (SQLite + JSONL, ~v0.1 – v0.49, Oct 2025 – Feb 2026)**

- Three layers: SQLite `.beads/beads.db` ("Local working copy (gitignored) ... Each machine has its own copy") → auto-sync with "5s debounce" → `.beads/issues.jsonl` ("Git-tracked source of truth. One JSON line per entity") → git push/pull — [v0.47.0 ARCHITECTURE.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/ARCHITECTURE.md)
- Write path: write SQLite immediately → mark dirty → 5 s debounce → FlushManager exports changed entities to JSONL → git hooks commit. Read path: first `bd` command after `git pull` checks if JSONL is newer than DB and imports, merging by content hash (same hash skip, different hash update, no match create) — [v0.47.0 ARCHITECTURE.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/ARCHITECTURE.md)
- Per-workspace daemon (LSP-like) with RPC over `.beads/bd.sock`, batching exports and coordinating auto-sync; CLI fell back to direct DB access — [v0.47.0 ARCHITECTURE.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/ARCHITECTURE.md)
- Legacy layout: `beads.db`, `bd.sock`, `daemon.log`, `export_hashes.db` gitignored; `issues.jsonl` tracked — [v0.47.0 ARCHITECTURE.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/ARCHITECTURE.md)
- Wisps were "never exported to JSONL", lived only in the spawning agent's SQLite DB and were hard-deleted on squash, so only a digest entered git — [v0.47.0 ARCHITECTURE.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/ARCHITECTURE.md)
- Yegge introduced Beads on Medium on 2025-10-13 ("Introducing Beads: A coding agent memory system") — [Medium](https://steve-yegge.medium.com/introducing-beads-a-coding-agent-memory-system-637d7d92514a) (page returned 403 to fetch; date from search index)

**Timeline of the switch**

- v0.49.1 (2026-01-25): Dolt backend "fully supported" but not default — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- v0.50.0 (2026-02-14): Dolt default for new `bd init`; daemon/RPC subsystem removed (~19,663 lines); "JSONL sync layer" removed (~7,634 lines) — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- v0.51.0 (2026-02-16): 8-phase cleanup removing daemon, "3-way merge engine", "tombstone/soft-delete system", JSONL sync layer, SQLite backend; "`bd sync` is now a no-op" — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- v0.57.0 (2026-03-01): "Beads Classic SQLite backend ... removed. Dolt is the only backend" — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- v0.63.3 (2026-03-30) embedded Dolt default; v1.0.0 (2026-04-02) "The Dolt migration that began in v0.55.0 is complete" — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md). DoltHub says the server-only interim "added friction for solo Beads users, who were used to the simpler SQLite and Git model", hence embedded mode ("Beads Classic" restored as a single-player experience) — [DoltHub blog 2026-04-02](https://www.dolthub.com/blog/2026-04-02-restoring-beads-classic/)
- Latest release seen: v1.3.0 (2026-09-15) — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)

### Inferences

- Beads moved from "git is the database, SQLite is a cache" to "a versioned database is the database, git is only a transport ref". For ParleyInn, Beads is evidence that a two-store design (local DB + git-tracked text file) is hard to keep consistent; but Beads' data is mutable rows, whereas ParleyInn's record is append-only, which avoids the delete/resurrection and field-merge problems that drove Beads away.
- The opt-in events journal (same-transaction, sequence-numbered, resumable) is the closest analogue in Beads to an append-only session record, and it is explicitly clone-local and not synced.

### Gaps

- Beads stores no agent conversation transcripts; its "memory" is issues plus `bd remember` key/value notes. I found no evidence of embeddings or vector search (one aggregator claimed embeddings; the official docs and code show none — treat that claim as wrong).
- The exact on-disk JSONL line format of the legacy era (one line per issue with embedded labels/deps/comments, versus one line per entity as ARCHITECTURE.md says) is inconsistent in the docs; the v0.47.0 schema table lists labels/dependencies/comments as fields of the issue object.

## 2. Beads — ID generation and merge-conflict handling

### Takeaway

IDs are short base36 truncations of SHA-256 over (title, description,
creator, nanosecond timestamp, nonce), with adaptive length 4→6+ chars by DB
size and nonce/length retries on collision. Legacy merges used a custom git
merge driver doing field-level 3-way merge of JSONL plus inline tombstones
with a 30-day TTL; today Dolt's cell-level merge does it, with deterministic
primary keys added to make clones merge-safe.

### Cited Findings

- `GenerateHashID` hashes `title|description|creator|timestamp.UnixNano()|nonce` with SHA-256 and base36-encodes it to the requested length — [internal/idgen/hash.go](https://github.com/gastownhall/beads/blob/main/internal/idgen/hash.go)
- Adaptive length: 4 chars for 0–500 issues, 5 for 501–1500, 6 for 1501+; default max collision probability 25%, min 4, max 8 chars; on collision tries base, base+1, base+2 "with 10 nonces per length, giving 30 attempts" — [adaptive-ids.md](https://github.com/gastownhall/beads/blob/main/docs/core-concepts/adaptive-ids.md)
- Hierarchical child IDs `bd-a3f8e9.1`, `.1.1` (up to 3 levels) are counters under a unique parent hash — [hash-ids.md](https://github.com/gastownhall/beads/blob/main/docs/core-concepts/hash-ids.md)
- Sequential IDs (`bd-1`, `bd-2`) were used before v0.20.1, which switched to hash IDs because sequential IDs "caused frequent collisions when multiple agents or branches created issues concurrently" — [pkg.go.dev beads v0.21.1 README](https://pkg.go.dev/github.com/steveyegge/beads@v0.21.1)
- Legacy merge driver (auto-configured since v0.21, beads-merge algorithm vendored from @neongreen): "Field-level 3-way merging (not line-by-line)", matches issues by `id + created_at + created_by`, timestamps → max, dependencies → union, status/priority → 3-way, conflict markers only when unresolvable; registered via `.gitattributes` `.beads/issues.jsonl merge=beads` and `git config merge.beads.driver "bd merge %A %O %A %B"` — [v0.47.0 GIT_INTEGRATION.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/GIT_INTEGRATION.md)
- Legacy deletions: "inline tombstones" — status `tombstone` with `deleted_at/by`, `delete_reason`, `original_type`, kept in `issues.jsonl` to stop "resurrection" on import; TTL default 30 days (+1 h grace), pruned by `bd admin compact`, "Git history fallback handles edge cases where pruned tombstones are needed" — [v0.47.0 DELETIONS.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/DELETIONS.md)
- Current: "Cell-level merge: Concurrent changes merge automatically at the field level" via Dolt — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md). Conflicts on `bd dolt pull` are handled by `bd doctor` / `bd doctor --fix` — [recovery/merge-conflicts.md](https://github.com/gastownhall/beads/blob/main/docs/recovery/merge-conflicts.md)
- `dependencies.id` was `DEFAULT (UUID())`, "a per-clone-random value", so two clones creating the same edge made `bd dolt pull` fail "unrecoverably"; now derived deterministically from `(issue_id, target)`, and same-edge conflicts differing only in audit columns auto-resolve — [CHANGELOG ~1.1.0-rc.1](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- Gas Town's Dolt conflict policy: "`newest` (most recent `updated_at` wins). Arrays (labels): `union` merge. Counters: `max`" — [gastown dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)

### Inferences

- Content-hash IDs need no coordination but are not content-addressed in the strict sense (timestamp and nonce are inputs), so identical content created twice gets two IDs. For an append-only record, a random or ULID-style ID per event plus a per-writer namespace would give the same no-coordination property more simply.
- The random-UUID primary key bug shows that any per-clone nondeterminism in keys breaks merges; ParleyInn's I10 (no clock or randomness in projections) points the same way.

### Gaps

- No published collision statistics from real deployments; COLLISION_MATH.md is theoretical.

## 3. Beads — worktrees, clones, machines, branches, real-time channels, backend history

### Takeaway

Now: all worktrees of a repo share one `.beads` (one Dolt DB); cross-clone
and cross-machine sync is explicit `bd dolt push/pull` to a Dolt remote, which
can be the code repo's own git `origin` under the ref `refs/dolt/data`, so
issue data never touches code branches. The old `sync.branch` (hidden worktree
committing JSONL to a `beads-sync` branch), daemon, `--no-db` JSONL-only mode
and Agent Mail integration are all removed.

### Cited Findings

- "All worktrees in the same repository use the same beads workspace unless you override discovery with `BEADS_DIR`"; "Issue changes are stored in Dolt, not committed to the current Git branch"; "No `sync.branch` or beads-managed Git worktree is required" — [worktrees.md](https://github.com/gastownhall/beads/blob/main/docs/reference/worktrees.md)
- Wire format: "Dolt stores issue history under `refs/dolt/data`, separate from source branches such as `refs/heads/main`"; `bd init` auto-configures the git `origin` as Dolt remote; fresh clones run `bd bootstrap` — [sync-concepts.md](https://github.com/gastownhall/beads/blob/main/docs/core-concepts/sync-concepts.md). Remotes can also be DoltHub, S3, GCS or a filesystem path — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Protected branches: "Beads does not need a protected-branch workaround in current releases ... branch protection rules continue to apply only to your code history" — [protected-branches.md](https://github.com/gastownhall/beads/blob/main/docs/reference/protected-branches.md)
- Git hooks now: pre-commit refreshes `issues.jsonl` when `export.auto=true`; post-merge/post-checkout skip JSONL import when `sync.remote` is set, and on old projects import JSONL only "as a compatibility fallback" with a warning "that this is not durable sync" — [sync-concepts.md](https://github.com/gastownhall/beads/blob/main/docs/core-concepts/sync-concepts.md)
- Sync is manual; trade-off table lists "No real-time collaboration" and "Manual sync to remotes"; guidance "Always sync before switching machines", "Pull before creating new issues" — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Multi-clone warning: concurrent sync from multiple clones causes race conditions, "particularly common in: Multi-agent AI workflows ... Worktree-based development workflows" — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Schema migrations across clones: "exactly one designated clone runs `bd migrate` and `bd dolt push`; other clones install the new binary and run `bd bootstrap`"; binaries refuse to open a DB whose schema is ahead of them — [README](https://github.com/gastownhall/beads/blob/main/README.md)
- Shared server mode: one Dolt server at `~/.beads/shared-server/` for all projects (opt-in) — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Federation: cross-repo peer-to-peer bead exchange; wisps excluded from federation push — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- **Legacy (removed)**: `sync.branch` committed to `beads-sync` via worktrees in `.git/beads-worktrees/`, merged to `main` by PR — [v0.47.0 GIT_INTEGRATION.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/GIT_INTEGRATION.md); now "That workflow has been removed" with cleanup steps — [worktrees.md](https://github.com/gastownhall/beads/blob/main/docs/reference/worktrees.md)
- **Legacy worktree bug**: "Daemon mode does NOT work correctly with `git worktree` ... the daemon may commit changes intended for one branch to a different branch"; recommended `--no-daemon` — [v0.47.0 GIT_INTEGRATION.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/GIT_INTEGRATION.md)
- **Legacy `--no-db`**: in-memory backend that "loads from JSONL at startup and writes back after each command", for multi-process/container cases where SQLite locking was insufficient — [v0.47.0 FAQ.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/FAQ.md)
- **Agent Mail**: an MCP Agent Mail integration (Python adapter, multi-workspace guide) was added in the 0.2x era and later removed ("Legacy MCP Agent Mail integration - Removed obsolete `mcp_agents` package") — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md). Messaging is now a `message` issue type with `--thread` — [README](https://github.com/gastownhall/beads/blob/main/README.md)
- v0.50.0 added `BD_BRANCH` "for branch-per-polecat write isolation in Dolt" — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md); Gas Town later abandoned branch-per-worker for all-on-main (see §6) — [gastown dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Fork exists: `beads_rust` (`br`) keeps the classic SQLite + JSONL-over-git design without a daemon; the Gas City site says br "retains the two-store synchronization problem that caused Beads to move away" (partisan source) — [gascity.com beads-vs-br](https://gascity.com/guide/beads-vs-br/)

### Inferences

- Storing a database under a non-branch git ref (`refs/dolt/data`) on the same remote is a notable pattern: it reuses the repo's hosting and auth, avoids branch protection, and keeps code history clean. ParleyInn could store its record under a custom ref (e.g. `refs/parinn/...`) for the same reasons without adopting Dolt. Note that ParleyInn's I4 forbids network access, so any push would have to be the user's own `git push` of that ref, not ParleyInn's.
- Beads' current model is still "sync by explicit push/pull with merge"; it has no real-time channel between machines. In Gas Town real-time sharing comes from a single shared Dolt server on one host, not from git.

### Gaps

- I did not find how `refs/dolt/data` behaves with GitHub repository size limits or how large it gets in practice.

## 4. Beads — scale, performance, compaction / memory decay, search

### Takeaway

Benchmarks target 10K–20K-issue databases; the docs advise against teams of
10+ people. The main scale problem after the Dolt move is history growth: one
Dolt commit per write produces "gigabytes of storage for a few thousand
beads", fixed by squashing history, which forces every other clone to
re-clone. Semantic "memory decay" (`bd admin compact`) replaces closed issues'
text with agent-written summaries and discards the original. Search is SQL
substring/filter search, not full-text ranking or vectors.

### Cited Findings

- Benchmarks: ready-work over 10K and 20K datasets, search over 10K, create/update in a 10K DB, 100 KB+ descriptions — [BENCHMARKS.md](https://github.com/gastownhall/beads/blob/main/BENCHMARKS.md)
- "When NOT to use Beads: Large teams (10+) — Git-based sync doesn't scale well for high-frequency concurrent edits"; "Real-time collaboration — No live updates" — [architecture/index.md](https://github.com/gastownhall/beads/blob/main/docs/architecture/index.md)
- Legacy claim: SQLite "handles millions of rows"; "Commands complete in <100ms"; "For extremely large projects (100k+ issues), you might want to filter exports or use multiple databases" — [v0.47.0 FAQ.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/FAQ.md)
- History bloat: "Every bead write mints a Dolt commit ... A workspace that has accumulated months of high-frequency writes can grow far past its live data (gigabytes of storage for a few thousand beads) while `dolt gc` reclaims nothing"; the fix "rewrites history. Every other clone of the database becomes unmergeable and must re-clone" — [recovery/history-squash.md](https://github.com/gastownhall/beads/blob/main/docs/recovery/history-squash.md)
- `bd compact` squashes Dolt commits older than N days (recent ones kept via cherry-pick); `bd flatten` squashes all history — [CLI_REFERENCE.md](https://github.com/gastownhall/beads/blob/main/docs/CLI_REFERENCE.md)
- `bd admin compact`: "Compact old closed issues using semantic summarization ... This is permanent graceful decay - original content is discarded"; modes analyze/apply (agent writes the summary, no API key) and legacy auto (needs `ANTHROPIC_API_KEY`); Tier 1 = 30 days closed, "70% reduction"; Tier 2 (90 days) "planned, not yet implemented" — [CLI_REFERENCE.md](https://github.com/gastownhall/beads/blob/main/docs/CLI_REFERENCE.md)
- Gas Town keeps compaction snapshots (`issue_snapshots`, `compaction_snapshots` tables with `original_content` / `snapshot_json`) — [gastown dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Search: `bd search` searches titles and IDs (closed excluded by default); `--desc-contains` for description substring; label/status/assignee filters; `bd sql` gives raw SQL; Dolt adds `AS OF` time-travel and `dolt_history_*` tables — [cli-reference/search.md](https://github.com/gastownhall/beads/blob/main/docs/cli-reference/search.md), [CHANGELOG 0.50.0](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md), [gastown dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Retrieval into context is push-based: `bd prime` (SessionStart hook) prints workflow context and persistent memories — [README](https://github.com/gastownhall/beads/blob/main/README.md)

### Inferences

- Beads deliberately trades losslessness for context budget (summaries replace originals). That is the opposite of ParleyInn's lossless record; ParleyInn can still borrow the idea as a derived, rebuildable projection while keeping the original.
- Per-write commits in a versioned store make history, not data, the cost driver. An append-only log on git would hit the same problem if every event became a git commit; batching events into fewer commits (or one file per session) matters.

### Gaps

- No published numbers for JSONL size limits or for issue counts beyond 20K; no latency figures for the Dolt era found in docs.

## 5. Beads — secrets, privacy, open-source use

### Takeaway

Beads has no secret scanning or redaction and no encryption at rest; its
guidance is "don't put secrets in issues". For open source it offers stealth
mode (keep everything out of git via `.git/info/exclude`) and contributor
routing (planning beads go to a separate repo, default `~/.beads-planning`).
Gas Town's JSONL backup exports are described as "scrubbed".

### Cited Findings

- "Do not store sensitive information (passwords, API keys, secrets) in issue descriptions or metadata"; "bd does not encrypt data at rest"; `.beads/` should be 0700 — [SECURITY.md](https://github.com/gastownhall/beads/blob/main/SECURITY.md). The same file still says "Issue data is committed to git and will be visible to anyone with repository access", a leftover from the JSONL era that conflicts with the current gitignored Dolt layout.
- Stealth: `bd init --stealth` — "keep beads files out of git via .git/info/exclude (sets no-git-ops)" — [cmd/bd/init.go](https://github.com/gastownhall/beads/blob/main/cmd/bd/init.go); "use Beads locally without committing files to the main repo. Perfect for personal use on shared projects" — [README](https://github.com/gastownhall/beads/blob/main/README.md)
- Contributor mode: `bd init --contributor` routes planning issues to a separate repo (e.g. `~/.beads-planning`) "never" mixed into PRs; maintainer role auto-detected from SSH/credentialed HTTPS remotes; role stored in git config — [README](https://github.com/gastownhall/beads/blob/main/README.md), [multi-agent/routing.md](https://github.com/gastownhall/beads/blob/main/docs/multi-agent/routing.md)
- Offline-first: "no command needs the network" — [faq.md](https://github.com/gastownhall/beads/blob/main/docs/reference/faq.md); but network calls exist through git, Dolt remotes and tracker integrations (GitHub/GitLab/Jira/Linear/ADO) — [SECURITY.md](https://github.com/gastownhall/beads/blob/main/SECURITY.md), [CHANGELOG 1.0.0](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- Gas Town: "the JSONL Dog exports scrubbed snapshots every 15 minutes to a git-backed archive" — [gastown dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)

### Inferences

- Beads provides no model for "stored history must not become a prompt-injection channel": `bd prime` injects stored memories directly into the session. That is a contrast point for ParleyInn's I2.

### Gaps

- What Gas Town's "scrubbing" removes was not found in the docs read.

## 6. Gas Town — architecture and where state lives

### Takeaway

Gas Town (Go, `gt`) runs many Claude Code (and other) agents in tmux sessions
on one host. Work state is beads in **one Dolt SQL server per town** (port
3307, one database per rig plus `hq` for the town), with all agents writing
straight to Dolt `main` in transactions for immediate visibility. Code lives in
git worktrees per polecat off a canonical clone. Real-time signalling is tmux
(`gt nudge`) plus beads-backed mail. Sessions themselves are not stored. Gas
Town scrapes Claude Code's own `~/.claude/projects/*.jsonl` transcripts for
cost and resume, logs its own events to `.events.jsonl`, and "seance" resumes
a predecessor's session through `claude --fork-session --resume <id>`.

### Cited Findings

- Roles: Mayor (coordinator Claude instance), Town (`~/gt/`), Rigs (project containers wrapping a git repo), Crew (human workspaces, full clones), Polecats ("persistent identity but ephemeral sessions"), Hooks, Convoys (bundles of beads), Witness/Deacon/Dogs (watchdogs), Refinery (Bors-style bisecting merge queue) — [README](https://github.com/gastownhall/gastown/blob/main/README.md)
- Hook = "A special pinned Bead for each agent ... the agent's primary work queue"; slinging = putting work on an agent's hook — [glossary.md](https://github.com/gastownhall/gastown/blob/main/docs/glossary.md). (README also calls hooks "Git worktree-based persistent storage" — [README](https://github.com/gastownhall/gastown/blob/main/README.md); the two descriptions differ.)
- Two-level beads: town `~/gt/.beads/` (`hq-*`: mail, convoys, agent identity, role beads) and rig `<rig>/mayor/rig/.beads/` (project work, MRs); agent beads per agent (`hq-mayor`, `<prefix>-<rig>-polecat-<name>`); `routes.jsonl` maps ID prefixes to rig paths — [design/architecture.md](https://github.com/gastownhall/gastown/blob/main/docs/design/architecture.md)
- Polecats and refinery are git worktrees of `mayor/rig` (`git worktree add -b polecat/<name>-<timestamp>`); crew are full clones — [design/architecture.md](https://github.com/gastownhall/gastown/blob/main/docs/design/architecture.md)
- "All beads data is stored in a single Dolt SQL Server process per town. There is no embedded Dolt fallback"; data in `~/gt/.dolt-data/<db>/`; daemon health-checks every 30 s and restarts on crash — [design/architecture.md](https://github.com/gastownhall/gastown/blob/main/docs/design/architecture.md), [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- "All agents ... write directly to `main`. Concurrency is managed through transaction discipline: every write wraps `BEGIN` / `DOLT_COMMIT` / `COMMIT`"; this "eliminates the former branch-per-worker strategy (BD_BRANCH, per-polecat Dolt branches, merge-at-done)" — [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Remote machines: Dolt can run on another host (e.g. over Tailscale) via `GT_DOLT_HOST`; otherwise every rig/worktree/polecat "silently connects to localhost and fails" — [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Three data planes: Operational (local Dolt server, high mutation, days–weeks), Ledger ("JSONL export → git push to GitHub", permanent, via the JSONL Dog every 15 min), Design (DoltHub commons, planned) — [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md). Ledger export design: "Export Is One-Way and Append-Only. Ledger records are never updated"; reopen creates a correction record — [design/ledger-export-triggers.md](https://github.com/gastownhall/gastown/blob/main/docs/design/ledger-export-triggers.md)
- Lifecycle CREATE → LIVE → CLOSE → DECAY (Reaper Dog deletes closed wisps >7 d) → COMPACT/FLATTEN (Compactor Dog, daily `DOLT_RESET --soft` to the first commit + `DOLT_COMMIT`, safe on a running server); Dolt founder Tim Sehn: "Your Beads databases are small but your commit history is big" — [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Wisps reuse the `issues` table with `wisp_type`, and are in Dolt's `dolt_ignore` so their mutations make no commits; mail is `issue_type='message'` beads threaded by dependencies — [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md)
- Schema includes an `interactions` table (`kind, actor, issue_id, model, prompt, response`) labelled "Agent interaction log" — [design/dolt-storage.md](https://github.com/gastownhall/gastown/blob/main/docs/design/dolt-storage.md), [beads migration 0014](https://github.com/gastownhall/beads/blob/main/internal/storage/schema/migrations/0014_create_interactions.up.sql)
- Real-time: `gt nudge` "Sends a message directly to an agent's tmux session" (lost if the session is dead); `gt mail send` is persistent but "Every `gt mail send` creates" a bead and Dolt commit — "4 agents x 15 patrol cycles x 2 mails per cycle = 120 commits just for routine chatter" that "live in the git history forever" — [design/mail-protocol.md](https://github.com/gastownhall/gastown/blob/main/docs/design/mail-protocol.md)
- Sessions/conversations: GT "reads Claude Code's conversation transcripts" from `~/.claude/projects/{slug}/*.jsonl` for token cost; a Stop hook appends to `~/.gt/costs.jsonl`; seance discovers sessions from `.events.jsonl` with fallback scan of `~/.claude/projects/` and `sessions-index.json`, and spawns `claude --fork-session --resume <id>` — [design/agent-api-inventory.md](https://github.com/gastownhall/gastown/blob/main/docs/design/agent-api-inventory.md)
- `.events.jsonl`: GT appends events (sling, handoff, done, hook, spawn, kill, session_start/end/death, patrol_*, merge_*) with flock; noted fragility: "Single JSONL file for all events — no rotation or size management", "No correlation ID linking events to specific agent runs" — [design/agent-api-inventory.md](https://github.com/gastownhall/gastown/blob/main/docs/design/agent-api-inventory.md)
- Self-assessment: "No single ID connects: OTel event ↔ conversation transcript ↔ cost entry ↔ session event ↔ bead"; "17 of 28 touch points depend on Claude Code internals" (JSONL format, path slug encoding, `sessions-index.json`, ...) — [design/agent-api-inventory.md](https://github.com/gastownhall/gastown/blob/main/docs/design/agent-api-inventory.md)
- Context is injected by `gt prime` via SessionStart hook; only `~/gt/CLAUDE.md` exists on disk — [design/architecture.md](https://github.com/gastownhall/gastown/blob/main/docs/design/architecture.md)
- Claimed scale: "Scale comfortably to 20-30 agents" — [README](https://github.com/gastownhall/gastown/blob/main/README.md)
- Federation: "Wasteland" links Gas Towns through DoltHub — [README](https://github.com/gastownhall/gastown/blob/main/README.md)
- Gas Town and Beads both hit v1.0.0 on the same day (Yegge post dated 2026-04-03) — [Yegge, "Gas Town: from Clown Show to v1.0" (mirror)](https://drss.io/feed/npub1na2ppcvpx7m7dg4d2rqelpq4xdj9z40cxu3avh4qcdq8dtupdcmqahmfge/gas-town-from-clown-show-to-v10) (original: <https://steve-yegge.medium.com/gas-town-from-clown-show-to-v1-0-c239d9a407ec>)
- Gas City is the follow-on "open software factory platform that runs on Beads" — [gascity.com](https://gascity.com/). A search snippet dates it to 2026-04-24 as an SDK successor of Gas Town built from "packs"; not verified from a primary source. The gastown repo's last commit is 2026-07-23 (from the clone).

### Inferences

- Gas Town is the clearest evidence that git is **not** the real-time channel for a multi-agent fleet: live state is a shared SQL server on one host, git is used for code (worktrees) and for a periodic append-only ledger export. That is a "hot store + cold git archive" split.
- Gas Town has exactly the gap ParleyInn targets: it has no durable, unified, append-only session record. It scrapes Claude Code's transcripts at session end, with three independent parsers and no correlation ID.

### Gaps

- Gas Town has no documented multi-machine story beyond pointing all agents at one remote Dolt server; no cloud-sandbox story found.
- The `interactions` table's actual write paths in Gas Town were not traced.

## 7. Problems and criticisms reported

### Takeaway

The author himself calls the legacy design "bidirectional sync, 3-way merge,
two sources of truth, race conditions, and tombstone hell". He reports weeks
of "massive data loss" in early Gas Town. The Dolt era brought its own
problems: history bloat, split-brain between servers, schema skew across
clones, wedged shared servers, and a 2026-08-11 accidental release linked to
a fleet-wide data-loss incident. Outside critics focus on vibe-coded quality,
code volume and heavy metaphor.

### Cited Findings

- Yegge: the old SQLite + JSONL system suffered "bidirectional sync, 3-way merge, two sources of truth, race conditions, and tombstone hell"; "the 22-nose Clown Show, where the Mayor scored a new clown nose every time it had massive data loss" — [Yegge, Clown Show to v1.0 (mirror)](https://drss.io/feed/npub1na2ppcvpx7m7dg4d2rqelpq4xdj9z40cxu3avh4qcdq8dtupdcmqahmfge/gas-town-from-clown-show-to-v10)
- The legacy changelog "records stale-database overwrites, deleted-issue resurrection, JSONL merge handling, corruption checks, and split-brain failures" — [gascity.com beads-vs-br](https://gascity.com/guide/beads-vs-br/) (project-affiliated summary); examples in the changelog: "Data loss race condition - Removed unsafe `ClearDirtyIssues()`" (v0.30.x), "Database reinitialization data loss bug", "Dolt split-brain root cause eliminated" (v0.49.3) — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- Legacy worktree + daemon could commit to the wrong branch — [v0.47.0 GIT_INTEGRATION.md](https://github.com/steveyegge/beads/blob/v0.47.0/docs/GIT_INTEGRATION.md)
- v1.0 notes "critical reliability issues that affected the v0.55–v0.63 series" (the Dolt transition) — [CHANGELOG 1.0.0](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- v1.2.0/1.2.1 "published by accident on 2026-08-11 without release testing"; v1.2.2 re-released the 1.1 line; later fixes cite "the 2026-08-11 fleet-wide data-loss reflex" where `bd init --force` silently recreated a missing database as empty — [CHANGELOG](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- A "three-city shared hub wedging twice in one day, each time ending in fleet-wide write refusal and a manual SQL repair" — [CHANGELOG Unreleased/1.3.x](https://github.com/gastownhall/beads/blob/main/CHANGELOG.md)
- Git/commit noise in the Dolt era: mail chatter generating permanent commits — [gastown mail-protocol.md](https://github.com/gastownhall/gastown/blob/main/docs/design/mail-protocol.md); history bloat to gigabytes — [history-squash.md](https://github.com/gastownhall/beads/blob/main/docs/recovery/history-squash.md)
- HN/community criticism: Gas Town "100% vibe coded" ("I've never seen the code"); ~189k lines of Go with 44k unreviewed lines from ~50 contributors; described as "fever dreams" that "work, somewhat"; heavy metaphor/branding — summarised from [HN thread 46624883 (mirror)](https://hacker-news-clone.nuxt.dev/item/46624883) and [ratfactor.com/yeggedex](https://ratfactor.com/yeggedex) via search snippets; not individually verified.

### Inferences

- Both eras' worst failures came from **mutable shared state reconciled across replicas** (resurrection, split-brain, re-init over missing data). An append-only, single-writer-per-segment record with derived indexes avoids most of these classes by construction.

### Gaps

- I could not fetch Yegge's Medium posts directly (403); quotes come from a mirror.
- HN comments were only seen via search-result summaries.
