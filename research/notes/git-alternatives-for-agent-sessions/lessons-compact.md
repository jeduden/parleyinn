- claude-code-sync (perfectra1n) [shipped] (<https://github.com/perfectra1n/claude-code-sync>) | where: Working-tree files in a separate git repository the user chooses: a local directory with an optional remote (GitHub, any git URL). Mercurial is an alternative backend, and Git LFS is optional (default pattern *.jsonl). R | lesson: This project shows that git can carry append-only agent JSONL between machines without textual conflicts: `merge=union` plus rebuilding the uuid/parentUuid tree is enough. It also shows what happens when the sync format is a mutable per-session file with no common base. Every append looks like a conflict. Same-uuid edits are settled by last-writer-wins on timestamp, and merged transcripts are re-serialized and rewritten in place, so history is no
- clync [shipped. Early alpha] (<https://github.com/Saturate/clync>) | where: A separate git repo per user, by default ~/.clync/data. `clync init` can create a private GitHub repo for it via the gh CLI, and pushes are plain `git push`. Alternatives are a local or network folder (NAS, Dropbox, USB) | lesson: Do not move history between machines by encrypting and merging the tool's own mutable transcript file. clync encrypts each ~/.claude JSONL file as a whole, which leaves git unable to diff, delta-compress or merge it. Divergence then means a hand-run `git pull --rebase` on binary blobs and a single shared manifest. The UUID-tree merge rewrites the source transcript: last-writer-wins on edits, metadata entries reordered, null keys dropped, and a ti
- syncestra [beta] (<https://pypi.org/project/syncestra/>) | where: Primary store: a private GitHub repo that syncestra auto-creates through POST /user/repos with private=True. The default layout is one repo with one branch per service or service/profile, e.g. `claude-code/personal`. Alt | lesson: If git carries session history, push sealed, immutable, append-only segments. Never mirror the live, mutable transcript files.

Syncestra first raw-tracked ~/.claude/projects (2.6 GB, JSONL files appended every turn). That caused a 6.4 GB out-of-memory kill and a dead mirror holding 6,446 conflict files. Its fix still recompresses each whole transcript into a new zstd blob on every append. Deletions are not handled, merges fall back to file-mtime

- Lore (tt-a1i/lore, npm @tt-a1i/lore) [beta. It exists and ] (<https://github.com/tt-a1i/lore>) | where: Local files in a .lore/ directory in the repo's working tree. These are plain files, not git notes or refs. The README says the directory "travels with the repo", but every `lore scan` creates .lore/.gitignore containing | lesson: Link sessions to commits by matching the content of each edit, and keep that link as a projection that can be rebuilt from the record. Do not write it into git objects. Lore's best result is that the structuredPatch + lines and the gitOperation SHA already inside a Claude Code transcript are enough to attribute commits. Content overlap still works after squash, rebase and worktrees, and it needs no trailers (only 4.4% of commits had them) or git
- Traces (traces.com), Lab 0324, Inc. [shipped] (<https://traces.com>) | where: Three places.
(1) Local SQLite cache. The docs mention 'local SQLite' (<https://www.traces.com/docs/sharing/git-hooks,> <https://www.traces.com/docs/cli/troubleshooting>). The CLI binary embeds the path ~/.traces/traces.db ( | lesson: Git works as a small pointer index, not as the store. Traces keeps all content in its own service and puts only one-line pointers in refs/notes/traces. Each write is a compare-and-swap transaction on a temporary ref. Pushes run inside a time-bounded pre-push hook that never blocks: fetch to an incoming ref, 'git notes merge --strategy=cat_sort_uniq', push, then re-merge and retry once if the push fails. notes.rewriteRef keeps notes on amended and
- SmolForge (forge.smol.ai, by swyx / Smol AI) [beta] (<https://forge.smol.ai>) | where: server/cloud. Rows live in Cloudflare D1 (SQLite), in the shared cloudforge-next database. That database holds 198 tables and is being split into separate databases per domain (<https://forge.smol.ai/docs/architecture/pla> | lesson: Copy SmolForge's per-commit segment model, which is a derived, rebuildable index. Avoid its ingest path.

The useful idea: each transcript→commit link is a message-ordering range ending where the commit first shows up. It is computed from the stored messages, checked against the object store on every read, and rebuilt on demand (/transcripts/reindex). ParleyInn can derive the same segments locally as a projection over its hash-chained record, which k

- Agent Note (wasabeef/AgentNote; npm package "agent-note") [shipped. v1.0.0 (fir] (<https://github.com/wasabeef/AgentNote>) | where: - **Permanent record:** git notes on the dedicated ref refs/notes/agentnote, one note per commit SHA. It is shared by `git push` through a generated pre-push hook and fetched through a refspec that `init` adds to remote. | lesson: Copy the cheap link, not the storage key. A short session trailer on each commit, plus a hook that keeps plain `git commit` working, is a robust, agent-agnostic way to tie history to code; `why` then works by walking git blame → trailer → record.

Do not key the record itself by commit SHA, as Agent Note does. Its own docs list what that costs:

- rebase and amend orphan notes
- squash-merge erases them
- a forced `+refs/notes/agentnote*` fetch ca
- git-memento (mandel-macaque/memento) [shipped] (<https://github.com/mandel-macaque/memento>) | where: Git notes refs inside the code repository itself (<https://github.com/mandel-macaque/memento/blob/main/README.md>):
- `refs/notes/commits` holds the default note or the summary.
- `refs/notes/memento-full-audit` holds the  | lesson: If git carries ParleyInn history, keep the private record out of any ref namespace that a wildcard push or fetch reaches, and do not let writers share one mutable ref.

memento shows what goes wrong otherwise:

- Its `git push <remote> refs/notes/*` published the supposedly private full-audit transcripts, which contain `/Users/...` paths, alongside the summaries.
- The same push leaked its own backup refs and nested `remote/origin/remote/origin` copie
- Git Prompt Story (QuesmaOrg/git-prompt-story) [beta. A working Go C] (<https://github.com/QuesmaOrg/git-prompt-story>) | where: Everything lives in git notes refs and custom refs inside the project's own repository. `refs/notes/prompt-story` is an ordinary notes ref: a commit history whose notes are keyed by commit SHA, holding the JSON manifests | lesson: Never let git carry shared session history on one force-pushed, history-less ref. Git Prompt Story keeps its transcripts as a bare tree ref and pushes both refs with "+" on every push. Two machines or sandboxes therefore silently overwrite each other's record. That is exactly the silent loss ParleyInn's I6 forbids, and it breaks append-only. If ParleyInn uses git as transport, it should give each writer its own fast-forward-only ref (for example refs/cai
- oobo (ooboai/oobo) [shipped] (<https://github.com/ooboai/oobo>) | where: Shared data lives on the git orphan branch refs/heads/oobo/anchors/v2 in the code repo; oobo/anchors/v1 is read-only legacy, and v1 writes were removed in 2.0.0 (<https://github.com/ooboai/oobo/blob/main/CHANGELOG.md>). Th | lesson: oobo's v2 layout shows git can carry multi-writer agent history without merge conflicts, provided the shared tree is designed for it. Paths are derived from ids (sha or UUIDv5 session_uid, with prefix fan-out), turn directories are write-once, and the two merge rules beyond that (per-field merge of session.json, line union of JSONL) are explicit. Its weak spots mark where ParleyInn must go further.

1. Reconcile resolves divergent files as 'remote wi

- ai-session (gammons/ai-session) [beta. The repo is pu] (<https://github.com/gammons/ai-session>) | where: A separate local git repo, never the code repo: ~/.ai-sessions/<project>/sessions/<YYYY-MM-DD_HHMMZ>_<slug40>__<short_id>/ (README; DEFAULT_CACHE_BASE at L13). --cache overrides the location. <project> is the basename of | lesson: The lesson: a trailer naming an immutable record position is the durable link. Do not depend on mutable side files or derived SHA lists.

In ai-session the commit trailer is the one part that survives rebase and forges. Everything mutable broke in testing:

- The path in the trailer depends on a folder name, and when that name diverged the tool dropped the trailer for every later commit in the session.
- At the same time it rewrote meta.json's com
- agentgit (monperrus/agentgit, a fork of btucker/agentgit) [beta. This is an exp] (<https://github.com/monperrus/agentgit>) | where: A separate local git repo with a working tree, one per code repo, at ~/.agentgit/projects/<repo-id>/. repo-id is the first 12 hex characters of the code repo's root commit (`git rev-list --max-parents=0 HEAD`) (<https://g> | lesson: Record the link between a session and the code when the event is captured, and make blame a projection that can be rebuilt. Do not reconstruct the link afterwards by matching content across a shared object store.

agentgit shows how fragile the after-the-fact approach is. Its links rest on a 64-bit hash of 3 neighbouring lines, path prefixes hardcoded to the author's home directory, a blame index that is never refreshed, and an alternates link by

- Gitify Prompt (gauravkrp/git-for-prompts, npm package gitify-prompt) [beta] (<https://github.com/gauravkrp/git-for-prompts>) | where: Working-tree files inside the project, committed into the project's own git history on whatever branch is checked out. There is no separate ref, no git notes and no separate repo.

The file lifecycle (<https://github.com/> | lesson: Link sessions to code by identity, never by timing or by self-reference. Gitify guesses the conversation by picking the JSONL with the most messages after session start minus 60 s. It then attaches every pending session to the next commit, and tries to write a commit's own SHA into that commit's tree. That forces a pending file plus an automatic second commit, and the link goes stale on rebase or squash.

What ParleyInn should do instead:

- Take sess
- Engram (Gentleman-Programming/engram) [shipped. The latest ] (<https://github.com/Gentleman-Programming/engram>) | where: The authoritative store is a local SQLite file at ~/.engram/engram.db, overridable with ENGRAM_DATA_DIR. Engram refuses to open it on NFS or SMB data directories.

Optional git sync writes `.engram/manifest.json` plus `. | lesson: Engram shows both sides of keeping memory in git. Immutable, content-addressed segment files really do merge without conflicts. But one shared, hand-appended index (manifest.json) brings conflicts straight back: I reproduced one with two branches, and Engram ships no merge driver. Its SessionStart hook then auto-imports whatever segments a commit adds and pipes them into Claude's context with no envelope, which makes the repo a prompt-injection c

- Letta Context Repositories / MemFS (Letta Code harness, formerly MemGPT) [shipped] (<https://www.letta.com/blog/context-repositories>) | where: One git repository per agent.
- Cloud agents: hosted by Letta at <base>/v1/git/<agentId>/state.git, default base api.letta.com (getGitRemoteUrl in <https://github.com/letta-ai/letta-code/blob/e1fa9af688e4f8921a72d2b8aee65> | lesson: Letta, the most advanced git-backed agent memory found, uses git only for a small, curated, human-readable layer: at most ~20k characters per file, about 2,000 files, defragmented to 15–25 files. It deliberately keeps raw conversation history out of git, in a cloud database or per-conversation JSONL with its own search.
To keep git merges safe it needs:
- a worktree and branch per writer;
- rebase-on-push;
- reset-to-remote with backup refs;
- an
- claude-nomad [shipped] (<https://github.com/funkadelic/claude-nomad>) | where: A separate private git repo that the user owns. `nomad init` creates it on GitHub through the gh CLI. Every other host clones it to ~/claude-nomad, or to a path set by NOMAD_REPO (<https://funkadelic.github.io/claude-noma> | lesson: Main lesson: put the secret and redaction gate on the record, not on the transport command. claude-nomad scans inside `nomad push`, so a plain `git push` skips it. Its scan policy (.gitleaksignore, .gitleaks.toml) travels in the same writable repo that it protects. Pulling hosts trust whatever comes down and replay it to the model.

For ParleyInn, if git (GitHub or Forgejo) ever carries session history:

- Redact and audit at ingest, before an event i
- ctxmem [shipped] (<https://github.com/DoppiaG93/ctxmem>) | where: Files in the working tree of the project repo, under `.ctxmem/`:
- memory.jsonl and config.json are committed and travel with whichever branch is checked out.
- index.db (SQLite FTS5, plus a sqlite-vec table in semantic  | lesson: A committed JSONL log is a good source of truth with a gitignored, rebuildable SQLite index derived from it, which fits I10. If ParleyInn uses git as transport, it should copy that split but not ctxmem's single shared append file. Shard the record per writer or session, for example one file named by session or agent id, so a merge is always a pure file addition.

The reason, reproduced against ctxmem 2.1.0: two branches that each append one line conf

- Cloudflare Artifacts (plus the open-source ArtifactFS client) [beta. Artifacts went] (<https://developers.cloudflare.com/artifacts/>) | where: Server/cloud only (Cloudflare). Each repo is one Durable Object, and a Worker front end routes to it (<https://blog.cloudflare.com/artifacts-git-for-agents-beta/>). Git objects live in that Durable Object's SQLite database | lesson: Make the session the unit of storage and replication, not one shared repo. Cloudflare's own guidance is one repo per agent or session and "do not use one shared repo as a queue for many autonomous agents". It ships no server-side merge and documents no semantics for concurrent pushes. ParleyInn's hash chain should therefore be many single-writer chains (one per session or agent) plus a rebuildable index (I10), rather than one log that several writers
- Zed Delta / DeltaDB [beta. Timeline: the ] (<https://delta.dev> (design post: <https://zed.dev/blog/introducing-deltadb;> launch post: <https://zed.dev/blog/introducing-delta;> public beta: <https://zed.dev/blog/delta-public-beta>)) | where: Two places: a local database on every client, and Zed's servers on Cloudflare. Commits stay in git.

- **Local:** 'Each connected Delta client holds a copy of the database' (<https://delta.dev/docs/concepts/delta-and-git>) | lesson: Delta shows a split that works: git keeps commits, and a separate replicated operation log keeps everything between them. That log is a per-thread SQLite-backed sync object, with packs and checkpoints in object storage. Every message, edit and comment gets a stable ID and an anchor that survives code motion. Non-users still see a plain git repo.

For ParleyInn this means:

- Key session records to commit SHAs plus ParleyInn's own stable event IDs.
- Keep
- Diversion (cloud VCS) + Diversion Claude Code plugin ("Diversion Ask", sold as "Diversion Trajectory") [beta] (<https://www.diversion.dev/diversion-claude-code-plugin>) | where: Server/cloud: Diversion's proprietary backend, reached over its REST API. The binary shows the endpoints /agent-sessions, /checkpoints/repos/, /transcript/repos/, get_git_commit_transcript and ask_queries, plus a transcr | lesson: Copy Diversion's linking model, which needs no server: an append-only per-session transcript, appends that must state the expected line offset (rejected on mismatch), a checkpoint id minted in PreToolUse just before a commit and covering a [start,end) range, and a commit trailer such as `ParleyInn-Checkpoint:` that carries the id to the code. This works over git or GitHub mirroring without a server.

Fix three things Diversion leaves open:

1. Its tra

- Gemini CLI checkpointing (/restore, shadow git + conversation JSON) [shipped. It is a bui] (<https://geminicli.com/docs/cli/checkpointing/>) | where: Local files only. Nothing is pushed anywhere.

- Shadow git repo: `~/.gemini/history/<id>/.git`. Under macOS seatbelt it is `~/.cache/.gemini/history/<id>`.
- Checkpoint JSON: `~/.gemini/tmp/<id>/checkpoints/*.json`.

So | lesson: A shadow git directory gives cheap local snapshots that pair a code tree with a conversation. Gemini's version shows three ways the pairing breaks:

- The trigger is a UI state, not the side effect. Auto-approved and shell edits get no snapshot.
- The session↔tree link lives in a sidecar file that is wiped on every launch, while the commits pile up unindexed.
- A failed snapshot falls back silently to the old HEAD.

What ParleyInn should do:

- Record
- container-use (Dagger) [beta] (<https://github.com/dagger/container-use>) | where: A separate local bare git repo, plus custom notes refs that are mirrored into the user's own repo.

- The fork is `git init --bare` at ~/.config/container-use/repos/<origin host/path>. If there is no origin, the path com | lesson: Don't use one shared, mutable git ref as the log that many writers append to. container-use sends every environment's log and state into two global notes refs keyed by code-commit SHA. That cost it three things:
- An empty commit per environment to stop notes colliding (#174, #240).
- Host-local file locks to stop ref-lock races and truncated JSON (#251, #257).
- On the user side, divergence is settled by deleting and re-fetching the ref, so the
- noa / libnoa (celestia-island) [beta] (<https://github.com/celestia-island/noa>) | where: Local layout. When the root has no .git directory, data goes in `.noa/`, and noa adds `.noa/` to .gitignore with the comment "keep agent iteration data out of git". When `.git/` exists, a "parasitic mode" puts everything | lesson: Make the integrity guarantee cover the record itself, not only the state derived from it. noa advertises "JSONL append-only logs" and an "immutable hash chain", but the chain covers only snapshot metadata. The op log has no hashes, is rewritten by compact_to after every CLI snapshot, and skips corrupt lines with only a warning. Snapshot content is also re-read from the shared working tree rather than taken from what the log recorded. As a result,
- AVCS (Agentic Version Control System) [beta] (<https://github.com/izagood/avcs>) | where: Everything lives in a per-repo local directory, `.avcs/`, plus optional transport layers.

On disk (src/store/objectStore.ts; docs/01):

- `objects/<aa>/<oid>.json` holds loose immutable objects, sharded by the first 2 he | lesson: If ParleyInn ever lets git carry records, copy AVCS's committed mode: make every record an immutable file named by its content hash, and mark the folder `-diff -merge`. Git then merges two writers' records with no conflicts, because different hashes never collide and identical hashes have identical bytes. Only mutable pointers can conflict. Also re-compute every record's hash on import rather than trusting the sender's. That makes any transport safe
- MCP Agent Mail (Python, plus its Rust rewrite mcp_agent_mail_rust) [shipped. Active as o] (<https://github.com/Dicklesworthstone/mcp_>agent_mail) | where: There is one shared archive git repository at STORAGE_ROOT, with a subtree per project at projects/<slug>/. It is kept apart from the code repo. The discovery note's 'own per-project git repo' is wrong on the code.
- Pyt | lesson: Use git as a one-way, rebuildable projection that a single writer produces from the record. Do not make git the live store or the merge and transport layer.
Agent Mail is a real, heavily used system that tried git as the store. It ended up with SQLite as the source of truth, which also assigns ids. Git became a ledger that needed:
- commit queues and coalescing;
- lock files with owner metadata;
- index.lock retries and gc tuning;
- a move from s
- GitButler Claude Code hooks (2025, removed) and its successor `but agentlog` on git-meta (2026) [archived. The Claude] (<https://blog.gitbutler.com/parallel-claude-code>) | where: (A) Hooks feature: the code went into ordinary git commits on GitButler virtual branches in the user's working repo, with the prompt in the commit message. The session mapping, locks and action log went into GitButler's  | lesson: GitButler tried "prompt in the commit message, one commit per chat round". That was lossy (an LLM reword could overwrite the prompt), polluted code history and leaked prompts. Within a year GitButler deleted it and replaced it with a separate metadata layer. That layer keeps a local DB as the working store and uses git only as the exchange format: a hidden `refs/meta/*` ref, one blob per transcript record named `<ts>-<hash>` so concurrent appends
- GitHub Copilot cloud agent (formerly Copilot coding agent) [shipped. GA on 2025-] (<https://docs.github.com/en/copilot/concepts/agents/coding-agent/about-coding-agent>) | where: Server/cloud (GitHub.com), with the work products in git. The agent runs in an ephemeral GitHub Actions environment that "is destroyed when the session ends, but the session log remains available on GitHub.com" (https:// | lesson: Use a commit trailer as the pointer from code to session, but make the trailer verifiable and keep the record outside git and the forge. GitHub's Agent-Logs-Url trailer, sitting on signed commits, is a cheap link that survives forges. But it points at a log only GitHub holds: it cannot be deleted, has no documented export, and disappears when a squash is built from the PR body. Meanwhile the PR body copies the raw prompt, untrusted issue comments
- Replit Agent checkpoints and rollbacks (App History) [shipped] (<https://docs.replit.com/replitai/checkpoints-and-rollbacks>) | where: On Replit's servers, in three layers. (a) The project filesystem, the development PostgreSQL data and the Agent state files all sit on Replit's "Bottomless" storage. This is a set of virtual block devices served over the | lesson: Replit shows that "conversation state versioned with git" works best when git carries only a pointer. Every checkpoint commit gets an opaque Session-Id and Event-Id trailer. The transcript and agent state stay in a separate store that the agent cannot write to. That store is append-only and holds a snapshot per checkpoint. Rollback becomes new forward commits (Replit-Restored-To), never a rewrite. Replit's own leaks are the two places content wen
- Kiro (AWS): CLI session management, specs, Web cloud sessions and autonomous mode [shipped. The IDE, CL] (<https://kiro.dev/docs/cli/chat/session-management.md>) | where: The answer differs by surface.
- Local CLI sessions: the docs say "SQLite database in ~/.kiro/", keyed by directory path (<https://kiro.dev/docs/cli/chat/session-management.md>). The CLI V3 pages contradict this: they say  | lesson: Kiro shows a useful split between what goes in git and what stays local. Durable, reviewable intent (specs and steering as plain Markdown) goes in the repo and merges through ordinary git. Raw transcripts stay out of git: either a single-writer local store locked per process, or a vendor cloud store that deletes them after 90 days.

Its one git path for transcripts is a counter-example for ParleyInn. The documented save-via-script recipe writes a sin

- Rekal (rekal-dev/rekal-cli) [shipped] (<https://github.com/rekal-dev/rekal-cli>) | where: Two places, both git-native.

Local machine:

- Two DuckDB files, .rekal/data.db and .rekal/index.db. The directory is gitignored and lives in the main worktree.
- Every linked worktree resolves to that one store via `git | lesson: Copy Rekal's transport shape: one append-only ref per writer, holding framed segments, with merges done at read time. If each writer owns exactly one ref and only appends length-prefixed, versioned frames, git never merges history. Readers union the refs into a rebuildable index, and unknown frame types are skipped by length. That fits ParleyInn's append-only record and I10 rebuildability. It also works on GitHub or Forgejo with no server.

Fix three

- claude-session-management (deemkeen) [shipped] (<https://github.com/deemkeen/claude-session-management>) | where: Working-tree files in ./saved-sessions/, relative to Claude's current working directory in the user's project. The directory is created if missing. The model is told to commit each snapshot to git, which puts it on whate | lesson: Git works fine as a dumb carrier for uniquely named, add-only files: snapshots from different machines merge with no conflicts. Everything that goes wrong in this plugin sits around git, and each failure has a matching ParleyInn rule:

- The model writes the record. That makes it lossy and impossible to check. ParleyInn should capture hook events byte-for-byte (I10, losslessness).
- Redaction is a free-text sed pass that is never called and catches no se
- Origin CLI (origin-cli), by Origin / opsworks-co (Artem Dolobanko) [shipped. Actively de] (<https://github.com/opsworks-co/origin-cli>) | where: Mostly inside the code repo's own git object store, under custom refs:
- refs/notes/origin, origin-memory, origin-memory-brief, origin-acceptance and origin-repo-brief (notes refs).
- refs/heads/origin-sessions: an orpha | lesson: Git can carry multi-writer session history, but only if every writer appends to objects that nobody else rewrites. Origin's history shows what happens otherwise. Each shared, mutable blob or ref (the orphan branch, a single memory note on the root commit, a forced notes refspec) caused silent data loss and needed its own fix: CAS with 20 retries, tree unions, `-s ours`, payload-level merges, and staging refspecs. Things stabilised only after it m
- claude-context-sync (Daniel de Oliveira Trindade; VS Code extension "Claude Context Sync" by DaleuStudio) [shipped. It is early] (<https://github.com/Daniel-de-Oliveira-Trindade/claude-context-sync>) | where: The source is Claude Code's local files under ~/.claude. Three transports exist:
(a) A separate private git repo, not the code repo. It is cloned to ~/.claude-sync-git. Each bundle is a file at {sanitized-project-name}/{ | lesson: The automatic path must be the safe path. In claude-context-sync, encryption is a CLI flag the installed hooks and watcher never pass, and `--auto` silently falls back to plaintext. So the default flow commits verbatim transcripts and file contents to a GitHub repo, with commit messages that quote the first prompt. Import then trusts an unkeyed SHA-256 and bundle-supplied paths.

If ParleyInn uses git as a transport, it should:

- seal every object be
- Semantica CLI (semanticash/cli) [shipped] (<https://github.com/semanticash/cli>) | where: Local database files in a .semantica/ directory next to .git, added to .gitignore automatically (<https://github.com/semanticash/cli/blob/main/README.md>). Contents:
- lineage.db: SQLite through modernc.org/sqlite, in WAL  | lesson: Keep the session record out of git, but write one small, durable pointer into the commit. Semantica keeps all session content in a gitignored local store. Its only write into history is a `Semantica-Checkpoint: <id>` commit trailer. That trailer is linked crash-safely:

1. pre-commit writes a handoff file first.
2. post-commit checks the tree and parent, then atomically promotes the handoff to a durable receipt before touching the database.
3. A w

- GrayCodeAI/trace. One GitHub name and Go module path has held two unrelated codebases: (A) the v0.1.0–v0.1.4 "git-native session capture" library, a rename of Entire CLI that GitHub no longer serves; (B) the current main, a self-hosted Git forge with signed agent history. [beta. (B) is pre-1.0] (<https://github.com/GrayCodeAI/trace>) | where: (B) Primary store: a JSON file, data/agent-sessions.json, on the forge server, protected by flock and atomic rename (<https://github.com/GrayCodeAI/trace/blob/main/cmd/trace/agent_>sessions.go). Two opt-in ways to move it: | lesson: Keep each writer on its own ref so no merge is ever needed. Trace B names every writer's ref by the hash of its public key: refs/<ns>/<sha256(pubkey)>. Each ref holds signed snapshots that only that writer appends to, and import works like this:
- verify against a key pinned out of band;
- reject any change to records already imported;
- require contiguous sequence numbers;
- never re-sign another writer's history.

That turns git into a merge-fr

- Kurrent Capacitor (kcap) [shipped] (<https://www.kurrent.io/how-it-works/>) | where: On a server in the cloud, not in git. Each organisation gets its own Kurrent-hosted workspace at https://<slug>.kcap.ai, hosted in the EU (<https://www.kurrent.io/docs/capacitor/getting-started/setup-server/,> <https://www.> | lesson: Give every session its own single-writer stream and give every event a deterministic ID derived from content. Then multi-machine or cloud sharing needs only idempotent append and dedup, never a merge. kcap's watcher advances its cursor only after an acknowledged send and re-sends freely, because the server dedups on xxHash128 event IDs. It quarantines a session whose source transcript gets rewritten instead of reconciling it. When redaction fails
- GitHub Copilot CLI session store, /chronicle and session sync (also used by the GitHub Copilot app) [shipped] (<https://docs.github.com/en/copilot/how-tos/copilot-cli/chronicle>) | where: Local working directory: ~/.copilot/session-state/<session-id>/ holds the authoritative per-session files, and ~/.copilot/session-store.db is the SQLite index. Both move with COPILOT_HOME (<https://docs.github.com/en/copi> | lesson: Copilot confirms ParleyInn's split: an authoritative per-session append-only log plus a disposable SQLite index rebuilt from it (/chronicle reindex). It also shows what to do differently. (1) Segment the log and read it as a stream, never as one blob. A single events.jsonl that grew past 512 MB bricked a session (#4325). (2) Chain events by hash, not random parentId UUIDs, if the log is to be tamper-evident. (3) Make deletion and redaction an appende
- GitHub Copilot cloud agent session logs (formerly "Copilot coding agent"), plus the gh agent-task CLI and the Agent tasks REST API [shipped] (<https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/track-copilot-sessions>) | where: On GitHub's servers, not in the repository. The agent runs in an ephemeral, GitHub Actions-powered environment that "is destroyed when the session ends, but the session log remains available on GitHub.com" (<https://docs.> | lesson: A commit-trailer back-link from code to session is useful but cannot be relied on, so ParleyInn's record has to be the source of truth for the session-to-commit link. Two observations support this. First, GitHub promises a session link in every agent commit, yet only 3 of 24 recent agent commits in two GitHub-owned repos carried `Agent-Logs-Url`. Second, where the trailer was present, it survived rebase and squash only because of how the message was
- gitmemory (doronp/gitmemory) [shipped] (<https://github.com/doronp/gitmemory>) | where: A separate, dedicated local git repository at $GITMEMORY_HOME, default ~/.gitmemory. Segments and manifests are ordinary working-tree files committed to that repo. It is not git notes, not custom refs, and not inside the | lesson: Pair ParleyInn's hash chain with a byte-range tiling proof over the source transcript. A hash chain only shows that recorded events were not altered. It cannot show that no source bytes were skipped. gitmemory gets that second property cheaply: each captured chunk carries (source identity, generation, [start,end), sha256), plus a whole-prefix digest, and verify checks arithmetically that the chunks tile [0,size) and hash to the digest (DESIGN §2.5).
- Hugging Face Agent Traces (Hub trace viewer, Session Traces Format, Storage Buckets) [shipped] (<https://huggingface.co/docs/hub/en/agent-traces>) | where: Server/cloud on the Hugging Face Hub, in one of two places.
(a) A Dataset repo. Dataset repos are git repositories backed by the Xet chunk store, with full history and pull requests (<https://huggingface.co/docs/hub/repos> | lesson: HF has run this exact workload: raw Claude Code JSONL, many machines, kept syncing. Its own docs say git is the wrong live store. "A git repository is not meant to work as a database with a lot of writes", the experience degrades "after a few thousand commits", and live traces should go to a non-versioned bucket or be batched into commits at least 5 minutes apart.

What HF chose instead gives up everything ParleyInn needs:

- Buckets have no history a
- Claude Code commit/PR attribution (Claude-Session trailer, Co-Authored-By, PR byline) [shipped] (<https://code.claude.com/docs/en/cloud-environments>) | where: The pointer sits in the git commit message as a standard git trailer, and as plain text in the PR description on GitHub.

The transcript it points to lives on Anthropic servers (claude.ai). This holds for hosted cloud se | lesson: Copy the pattern of a git trailer as the only thing in git that points from code to a session: small, parses natively, and survives clone, push and cherry-pick. Fix its two weak spots, since Claude-Session is a model-written, unauthenticated URL to mutable server-side storage.

(1) Have a deterministic commit-msg or prepare-commit-msg hook write the trailer, the way the self-hosted runner writes Co-authored-by. Never ask the model to write it: pr

- GitHub Copilot cloud agent (formerly "coding agent"): Agent-Logs-Url commit trailer and hosted session logs [shipped. The trailer] (<https://github.blog/changelog/2026-03-20-trace-any-copilot-coding-agent-commit-to-its-session-logs>) | where: The pointer is a plain-text trailer in the commit message, inside normal git objects. The log is on GitHub's servers, outside git and outside the repository. gh fetches it from the Copilot API at `{capiBaseURL}/agents/se | lesson: The good idea: put a small pointer in the commit and keep the heavy, sensitive session record out of git. Trailers survive squash merges and rebases as plain text, which GitHub's approach confirms at scale. The GitHub version has three gaps for ParleyInn's goals:

- The pointer is a forge-hosted URL with access control, not a content address. It dangles when history is mirrored, transferred or forked (see the XBOX-Godot-Sample commit linking to anoth
- agentdiff (codeprakhar25/agentdiff) [shipped. Pre-1.0 and] (<https://github.com/codeprakhar25/agentdiff>) | where: There are three tiers, all inside the git repo and none in the working tree. Tier 1 is a local buffer at .git/agentdiff/traces/{branch}.jsonl, with "/" in branch names written as %2F. Tier 2 is a per-branch custom ref, r | lesson: A CAS on a git ref is not a merge. agentdiff keeps each tier as one mutable JSONL blob and retries a non-force ref update without recomputing the content, so concurrent writers silently lose each other's records. Its per-record signatures with no chain cannot detect the loss, and its custom-ref store also escapes branch protection. If ParleyInn moves its record over git, two rules follow. First, make every append content-addressed and collision-free,
- AgentBlame (mesa-dot-dev/agentblame, npm @mesadev/agentblame) [shipped. It is on np] (<https://github.com/mesa-dot-dev/agentblame>) | where: Custom git notes refs inside the project's own repo:
- refs/notes/agentblame holds per-commit attribution. The notes tree is keyed by commit SHA.
- refs/notes/agentblame-analytics holds the analytics note, attached to th | lesson: AgentBlame tests the 'put agent provenance in git notes' idea in practice. The upside is real: the data travels with the repo, the forge API can read it with no backend, and it works offline. Its failure modes are the ones ParleyInn must design out.

What goes wrong:

1. The notes ref must be pushed by hand and has no merge strategy, so diverged writers silently lose data.
2. Notes are keyed by commit SHA, so every squash or rebase needs a privileged

- git-prompt-log (parasti/git-prompt-log) [beta] (<https://github.com/parasti/git-prompt-log>) | where: There are two homes for the data.

(1) Git notes on the local ref refs/notes/commits (DEFAULT_NOTES_REF, configurable with --ref). Hooks and config sit in the common git dir, so all linked worktrees share them (<https://g> | lesson: Pick the replication unit to match the privacy boundary, and make what humans review the same bytes that machines ingest.

git-prompt-log rightly refuses to push a repo-wide notes ref. That ref is all-or-nothing and would leak private-branch prompts, so it ships an export scoped to a branch inside the PR. It also handles forge squash merges by re-anchoring on subject. But import trusts a hidden JSON comment rather than the visible text, and nothi

- ai-trailers (EslaMx7/ai-trailers) [shipped. v0.2.1 is o] (<https://github.com/EslaMx7/ai-trailers>) | where: In two places. (1) Staging: a plain file, `.ai-trailers`, in the hook process's current directory. It is added to `.gitignore` and appended to on every prompt (<https://github.com/EslaMx7/ai-trailers/blob/main/src/capture> | lesson: Putting data in the commit object is the most portable way to move it through GitHub and Forgejo: it needs no extra refs or notes and survives every clone and mirror. But it is the wrong place for session content. It cannot be redacted, it is public, its size is unbounded, and it feeds whatever agent next reads `git log` (the I2 risk). It is also fragile: in tests, a hand-appended block lost a prompt with a blank line, displaced Co-Authored-By tr
- OpenFab (openfab/generation in-toto predicate) [beta. The Phase-0 MV] (<https://github.com/Open-fab-ai/openfab>) | where: Split between the working tree in git and local files.
- Committed: the attestation and SBOM go in the repo's provenance/ directory, on a forge branch: `openfab/<spec>-v<ver>` for releases, `openfab/draft/<run>` for unsi | lesson: If ParleyInn ever writes anything about a session into git, copy OpenFab's 'manifest, not transcript' split and fix its two weak spots.
- What to copy: commit only a small signed record next to the code, bound to file digests and pointed to by a greppable commit trailer. Keep the transcript in ParleyInn's local store.
- Weak spot 1: OpenFab publishes a bare SHA-256 of the prompt, which confirms guesses. ParleyInn should use a keyed or salted commitment, such
- whogitit (dotsetlabs/whogitit), Rust CLI by Greg King [archived] (<https://github.com/dotsetlabs/whogitit> (404 on 2026-10-02). The surviving primary source is <https://crates.io/crates/whogitit,> with the full source browsable at <https://docs.rs/crate/whogitit/1.0.0/source/>) | where: Git notes under the single ref refs/notes/whogitit: one blob per annotated commit OID in the standard fanout notes tree. Notes are pushed by a pre-push hook and fetched through a refspec that `whogitit init` adds to remo | lesson: A single shared git-notes ref is a last-writer-wins store. whogitit uses one force-fetched ref and swallows push errors, so attribution from one agent silently disappears when another pushes first. Its "retention delete" also leaves every deleted prompt in the notes-ref history on every remote.

If ParleyInn uses git as a transport, it should:

- give each writer its own append-only ref (for example refs/parinn/<writer-id>/<chain>), so every push is a
- DiffMem (Growth Kinetics) [shipped] (<https://github.com/Growth-Kinetics/DiffMem>) | where: A local git repo on a mounted volume is the source of truth, `/data/storage` by default. It is created with `git.Repo.init`, which makes a non-bare repo; the code's docstring calls it "Bare-ish". The discovery note's "on | lesson: A branch is a namespace, not a security boundary. DiffMem shows this exactly where git would be tempting for ParleyInn. It isolates tenants as orphan branches in one repo with shared worktrees. Any read-only git command reachable from recall (`git show user/<other>:file`, `git log --all`) crosses tenants, and one PAT with `repo` scope covers every tenant on the mirror. Its "read-only" retrieval also runs LLM-composed strings through `shell=True` behi
- agent-memory (xChuCx) [shipped] (<https://github.com/xChuCx/agent-memory>) | where: Working-tree files under .agent-memory/ in the project repo. Only the durable files are git-tracked by default: index.md, conventions.md, decisions.md, pitfalls.md, modules/, archive/, meta/manifest.yaml, meta/schema.yam | lesson: Git works well here because what it carries is small, curated, human-reviewed Markdown whose sections have stable IDs. With that data shape, a custom merge driver keyed by those IDs can union concurrent appends across clones without conflicts. Even so, the project leaves its high-volume, per-session data (sessions/, local/, staging/, the index) out of git, and it has no real-time or cross-host coordination. For ParleyInn this suggests a split. If git
- Yunaki Memory MCP (yunaki-memory-mcp) [beta. It is an exper] (<https://github.com/Nanda-Kiran/yunaki-memory-mcp>) | where: A separate local git repo per project, outside the working tree: `~/.yunaki/memory/<repo-id>/`. The root can be changed with YUNAKI_MEMORY_ROOT, and YUNAKI_REPO pins the target repo. `<repo-id>` is the first 12 hex chara | lesson: Do not let concurrent writers share one git index and working tree, and do not commit a rebuilt index next to the records. Yunaki's design is mutable files, a shared `.git/index.lock`, `git add -A` and a rebuilt MEMORY.md plus a mutable repo.json in every commit. That fails as soon as two sessions write at once: errors are returned for writes that did persist, writes are attributed to the wrong commit, and every future cross-machine merge is guar
- MCP GitHub Memory Server ("gitmem", panosAthDBX) [beta] (<https://github.com/panosAthDBX/mcp-github-memory-server>) | where: The default is working-tree files in a local git clone, pushed with libgit2 (the optional "remote-git" feature, git2 0.18) to a GitHub or any git remote, on a per-device branch devices/<host>. Other options are a plain l | lesson: If git carries ParleyInn history, name each writer's ref by a unique writer id, never by hostname. Make the ref append-only and push without force. Converge on the reader side by taking the union of all writer refs. gitmem shows what goes wrong otherwise:

- **Shared branch names clobber history.** It force-pushes devices/$HOSTNAME and "pulls" by hard-resetting to that same branch. Writers that share a name silently overwrite each other (an unset HOS
- Serena memories (oraios/serena) [shipped] (<https://github.com/oraios/serena>) | where: Plain files in the working tree. Project memories live in `<project>/.serena/memories/`. Global memories live in `~/.serena/memories/global/`, under `$SERENA_HOME/memories/global` when SERENA_HOME is set (<https://oraios.> | lesson: Committing agent memory to the repo works well for a small, curated, human-reviewed layer: the PR review is the trust gate, and the memory reverts together with the code. But Serena can only call this safe because it declares the repo and the LLM trusted (<https://oraios.github.io/serena/02-usage/070_>security.html). Once memories travel through git, any PR author or poisoned session can plant text that a later agent reads as guidance. Issue #1251
- Memorix (AVIDS2/memorix) [shipped] (<https://github.com/AVIDS2/memorix>) | where: Local DB. The canonical store is SQLite at ~/.memorix/data/memorix.db (MEMORIX_DATA_DIR overrides it), in WAL mode. Derived indexes can be rebuilt: Orama in memory for small stores, a persistent SQLite FTS5 index, and an | lesson: Treat git or GitHub as a dumb transport for immutable, device-owned event files, never as the database or the merge engine. Memorix's relay path, events/<ns>/<device>/<seq>.jsonl, gives every device its own write-only lane. Merge then becomes a deterministic replay keyed by (deviceId, sequence) with Lamport (revision, writer) ordering, durable tombstones and conflict evidence, and the live SQLite/WAL never crosses the wire. Git never sees a text
- ByteRover CLI v3 (`brv`, formerly Cipher). Its successor is ByteRover V4 (Desktop app plus the agent skill campfirein/skills). [archived] (<https://github.com/campfirein/byterover-cli>) | where: v3 has three places. (1) Working-tree files in `<project>/.brv/context-tree/`, with config in `.brv/config.json` (<https://github.com/campfirein/byterover-cli/blob/main/src/server/constants.ts> ; README). (2) A nested, sep | lesson: ByteRover is a real-world case of "git as transport for agent memory", and the git part was dropped within months. v3.0.0 added real git through isomorphic-git, in a nested repo outside the code repo, pushed over Smart HTTP to a vendor remote. Line-based 3-way merges left conflict markers inside knowledge files. Branch, checkout and remote errors kept needing fixes (CHANGELOG 3.x). The V4 successor replaced all of it with a polling daemon, last-w
- bettermemory (0Mattias / Mattias Rask) [shipped. PyPI 8.0.0 ] (<https://github.com/0Mattias/bettermemory>) | where: Everything sits in local working files under one store directory, optionally mirrored to a git repo the user controls.
- The store resolves to `$BETTERMEMORY_DIR`, else `./.claude-memory/` if that exists, else `~/.claude | lesson: If git ever carries ParleyInn history between machines, a pull must be treated as untrusted ingest, not a copy.

What bettermemory does on pull: every arriving file passes an admission chain (size cap, parser, id-alias check, credential detector). A failure is quarantined where git put it, so no deletion propagates. Provenance (local/synced/unaccounted) and "verified here" state live in local derived state built from a host-local event log, never in

- Claude Code auto memory + CLAUDE.md (with subagent memory and the cloud Projects memory variant) [shipped. CLAUDE.md h] (<https://code.claude.com/docs/en/memory>) | where: Auto memory lives in local files under `~/.claude/projects/<project>/memory/`. The <project> name "is derived from the git repository, so all worktrees and subdirectories within the same repo share one auto memory direct | lesson: Claude Code keeps three things in three places:
- what humans write: CLAUDE.md, shared through git, bounded in size, re-injected from disk after compaction
- what the agent writes: auto memory, mutable Markdown that stays on one machine and is never synced or merged
- the raw history: session JSONL, deleted after 30 days

Two parts of this were later fixed in the field. MEMORY.md, which the model writes itself, is injected into every session auto

- cass (coding-agent-search), by Jeffrey Emanuel (GitHub: Dicklesworthstone) [shipped. The project] (<https://github.com/Dicklesworthstone/coding_>agent_session_search) | where: Local DB plus local files. Optional export of an encrypted static site to a GitHub Pages or Cloudflare Pages git repo.

Local data dir (Linux default ~/.local/share/coding-agent-search, override with CASS_DATA_DIR or --d | lesson: Move the raw evidence between machines, never the database, and rebuild every merged view locally under source-scoped identity.

cass's whole multi-machine story rests on this:

- rsync pull of raw provider logs, additive only, never propagating deletes
- re-parse on the controller, with conversations keyed by (source_id, agent, external_id)
- SQLite as the single canonical store, with every index derived and rebuildable
- the author explicitly de
- episodic-memory (obra / Jesse Vincent) [shipped] (<https://github.com/obra/episodic-memory>) | where: Local disk only, as plain files plus a local SQLite database. There is no git, server or peer-to-peer component.
- Archive: ~/.config/superpowers/conversation-archive/<project>/...jsonl
- Index: ~/.config/superpowers/con | lesson: Main lesson: anything that re-processes stored history must run with zero authority and outside the user's live configuration.

episodic-memory's summarizer resumed archived sessions with the user's tools, MCP servers and settings loaded. In one reported case it acted on that old session and committed to a working repo. Version 1.5.0 fixed this with tools: [] and settingSources: [], and 1.1.2 fixed a separate recursive hook cascade.

For ParleyInn, d

- lcm (lossless-claude), npm package @lossless-claude/lcm [shipped. v0.13.0 is ] (<https://github.com/lossless-claude/lcm>) | where: Everything lives in a local database under LCM_HOME, which defaults to ~/.lossless-claude/ (<https://github.com/lossless-claude/lcm/blob/main/docs/privacy.md>):
- ~/.lossless-claude/projects/<sha256(realpath of cwd)>/db.sq | lesson: Take lcm's way of grouping worktrees and clones without git as a store. Key each store by realpath cwd. Also record a normalized git remote set: host/path, with ssh, https and scp forms collapsed into one. The set only grows, so a moved or renamed repo keeps grouping with its old sessions. Record the repo-relative path too, and union the per-checkout stores at read time. Never merge them physically, and resolve every cross-store id through the gr
- lossless-code (GodsBoy) [shipped. The repo is] (<https://github.com/GodsBoy/lossless-code>) | where: A local SQLite database at ~/.lossless-code/vault.db (WAL mode; the directory can be overridden with LOSSLESS_VAULT_DIR). Side files live under ~/.lossless-code/. There is one vault per user per machine, shared across al | lesson: A "lossless" claim fails at the ingest boundary, not in storage. lossless-code calls itself lossless, but it loses data in three silent ways:

1. It keeps only text blocks, so tool_use, tool_result and thinking content are gone.
2. It dedups by row count against a table that other hooks also write, so one message is skipped per marker or tool row.
3. It reads a hook field name ("query") that Claude Code never sends.

ParleyInn should do the opposite:

- Letta Context Repositories (MemFS), in the Letta Harness, formerly Letta Code [shipped. Announced F] (<https://www.letta.com/blog/context-repositories>) | where: A separate git repo per agent (not the user's project repo), checked out locally.
- Cloud agents: the repo is "hosted by Letta for cloud agents, alongside its conversations and configuration". MemFS "projects" it onto wh | lesson: Letta puts in git only what git merges well: a small, capped set of curated Markdown files (15–25 files, 20k characters each). Raw history stays in a separate searchable message store. Even at that size, line-based merges still conflict, and the code needs three layers of recovery: pull --rebase, merge --abort with a worktree retry, and reset --hard with backup refs. The last layer is an LLM "repair worker" that resolves conflicts by rewriting co
- Plandex [shipped, but dormant] (<https://github.com/plandex-ai/plandex>) | where: Server side, never in the user's project repo.

1. A private git repository per plan, on the server's persistent filesystem at $PLANDEX_BASE_DIR/orgs/<orgId>/plans/<planId>/. The default base is /plandex-server in produc | lesson: Plandex is the clearest working example of git used as a private, server-side snapshot engine for agent history: one repo per plan, one commit per event, a branch per alternative. It also shows what that costs.

- Git, the filesystem and the database only behaved as "a single transactional database" after Postgres row locks, heartbeats and an in-process queue were added on top.
- Because each repo has one checked-out working tree, every plan has a
- OpenCode snapshots and sessions (anomalyco/opencode) [shipped. The current] (<https://opencode.ai/v2/docs/snapshots/>) | where: Local data directory: XDG data, default ~/.local/share/opencode. (a) Snapshots: `snapshot/<projectID>/<fastHash(worktree)>/` is a separate git dir driven with --git-dir/--work-tree. On init its `objects/info/alternates`  | lesson: Recording code state as git trees in a private object DB is a cheap, non-invasive way to bind each agent step to exact file state. The DB borrows the repo's objects through alternates, never touches refs or the index, and keeps one store per worktree. ParleyInn can copy this, but any tree hash it writes into its append-only record must be pinned, for example by a ParleyInn-owned ref or pack. Unreferenced trees die at the next `gc --prune` and break I10-s
- Kilo Code checkpoints (snapshots) — new VS Code/JetBrains extension and Kilo CLI (OpenCode-derived), plus the legacy shadow-repo checkpoints [shipped. Current sna] (<https://kilo.ai/docs/features/checkpoints>) | where: Local files and a local DB, all on the developer machine. Nothing is pushed to a git remote.
- Snapshot objects: a separate bare-style git dir, ~/.local/share/kilo/snapshot/<project-id>/<sha1(worktree path)>/ (XDG data d | lesson: Treat the content-addressed git tree as a cheap code anchor you can rebuild, but do not make the session record depend on a garbage-collected object store.
Kilo puts a tree hash in a mutable SQLite row and lets a separate 7-day pin/GC policy delete the objects. The session-to-code link then quietly dangles, and its own docs admit this. Revert falls back to conversation-only behind a banner.
For ParleyInn:
- Record the tree or commit hash inside the h
- AgentFS (Turso) [beta] (<https://github.com/tursodatabase/agentfs>) | where: Local SQLite-format files written by the Turso engine. The MANUAL says `.agentfs/<ID>.db` (<https://github.com/tursodatabase/agentfs/blob/main/MANUAL.md>). On Linux, `agentfs run --session X` actually writes ~/.agentfs/run | lesson: An "append-only audit log" that is only a convention drifts in every way it can. AgentFS's spec says the tool_calls table MUST be insert-only, yet:
- three of its four SDKs INSERT a pending row and then UPDATE it;
- the fourth (Go) uses a schema that conflicts with the others;
- the docs query a table name that does not exist;
- nothing in the product (FUSE, sandbox, MCP) writes to the log, so it is empty unless integrators call the SDK;
- syncin
- sudocode (sudocode-ai/sudocode) [shipped, but dormant] (<https://github.com/sudocode-ai/sudocode>) | where: Working-tree files committed to the project's own git repo, plus a gitignored local SQLite cache.

The actual files are `.sudocode/issues.jsonl` and `.sudocode/specs.jsonl`, directly under `.sudocode/`. The README's `.su | lesson: Even this git-first tool keeps transcripts out of git. Small, curated, mergeable records (specs and issues) travel through git. Raw agent transcripts stay in a gitignored local SQLite and are pruned after 30 days.

Its git path also shows two traps ParleyInn must avoid:

1. **Latest-wins merging.** It rewrites one shared snapshot JSONL and settles concurrent edits with a latest-`updated_at`-wins merge driver. That silently drops the losing write, whic

- swarm-tools (Swarm: Hive, Hivemind, Swarm Mail). npm packages opencode-swarm-plugin, claude-code-swarm-plugin, swarm-mail [shipped. The repo is] (<https://github.com/joelhooks/swarm-tools>) | where: A local database plus files in the working tree, synced through git.

The current code always uses one machine-wide libSQL/SQLite file, ~/.config/swarm-tools/swarm.db (overridable with SWARM_DB_PATH). Projects share it a | lesson: Treat any git transport as publishing, and keep recall pull-only.

swarm-tools keeps a good local event log (one machine-wide SQLite file, events with typed schemas, replayable projections). Its git bridge, though, takes snapshots from a store shared by every project and commits them to the working branch with no scoping or redaction: hive_sync exports every memory in the global database into the project's .hive/memories.jsonl. In its own public

- GitHub Agent HQ / mission control (third-party coding agents on GitHub: Anthropic Claude, OpenAI Codex, plus partner "agent apps") [beta. Agent HQ and "] (<https://github.blog/news-insights/company-news/welcome-home-agents/>) | where: Server/cloud: GitHub's proprietary backend on GitHub.com, not git notes, custom refs or a side repo. Cloud-agent sessions run in an ephemeral environment powered by GitHub Actions that "is destroyed when the session ends | lesson: GitHub itself keeps the code in git but keeps the session record outside git. Git carries only a pointer: the session-log link in each commit message and an agent_session_id on audit events. The transcript stays in a proprietary store that exports metadata only. The tasks REST API returns no transcript, admin session lists cover 24 hours, the full-content stream is EMU-only, pulls cover 48 hours, and bodies are truncated at 1 MB. For third-party
- GitLab Duo Agent Platform: Sessions (internally "Duo Workflows"), plus the experimental external-agent session tracking for Claude Code and OpenCode [shipped. The Duo Age] (<https://docs.gitlab.com/user/duo_>agent_platform/sessions/) | where: Server/cloud, inside the GitLab instance's PostgreSQL. The design doc says "All state management are inside GitLab". The Duo Workflow Service "will not have any persisted state": it keeps running state in memory and chec | lesson: The lesson is to keep the content local and ship only a hash and IDs. GitLab's own Claude Code integration (`glab govern`, behind a default-off flag) does exactly this. A Stop hook reads ~/.claude/projects/.../<session>.jsonl from a byte-offset cursor, sends only tool names and timestamps, and moves the cursor forward only after the server confirms the write. Each event gets a deterministic UUIDv5 built from the tool_use ID, so replaying it is ha
- Sourcegraph Agentic Batch Changes (ABC) [shipped. Beta announ] (<https://sourcegraph.com/docs/agentic-batch-changes>) | where: On a server: the customer's Sourcegraph instance, either Sourcegraph Cloud or self-hosted. The architecture doc puts Batch Changes state (batch specs, changeset specs, changesets) in the instance's Postgres 'frontend-db' | lesson: Let git carry a pointer to the session, and keep the session itself out of git. ABC never puts the transcript in git. Instead, every commit it makes carries a `Sourcegraph-Batch-Change` trailer that links back to the conversation, and that trailer stays even when the PR-body link is turned off (<https://sourcegraph.com/docs/admin/config/batch-changes>). ParleyInn can do the same without using the network. While a session is active, it can stamp commits
- JetBrains Junie (CLI, IDE plugin / Air, GitHub Action, GitLab CI/CD) [shipped. JetBrains a] (<https://junie.jetbrains.com/>) | where: Storage is hybrid:
- **Working-tree files:** `.junie/` in the repo. It travels via git only if a human commits it; no custom refs or git notes are involved.
- **Local per-user directory:** `~/.junie/sessions` holds JSONL | lesson: Junie splits its data into two layers:
- **Curated design artifacts** (plans, AGENTS.md, skills) live in-tree under `.junie/` and travel by git.
- **The raw record** (`events.jsonl` plus `transcript.md`) stays outside git in `~/.junie/sessions`.

On ephemeral CI runners, the raw record's only way off the box is a post-step that uploads it as an unredacted CI artifact. The step runs always, uses include-hidden-files, and keeps the artifact for 7 d

- Devin (Cognition): Devin Cloud sessions, Devin CLI /handoff and /pickup, Session Insights [shipped] (<https://docs.devin.ai/cli/cloud.md>) | where: Server/cloud. Cloud sessions run on Devin-managed VMs and the CLI only streams them (<https://docs.devin.ai/cli/cloud.md>). The 'brain' is stateless and always runs in Cognition's cloud, whatever the deployment (<https://do> | lesson: Devin uses git only to carry code (cloud to local is 'check out the session's PR branch'), and it keeps the session record on its own server, tied to the code only by a session URL in the PR body and a pull_requests field on the session. Its local-to-cloud handoff is lossy and unverifiable: an LLM summary plus `git diff HEAD` (the plugin truncates it to 100KB) pasted into a new session's prompt. The new session therefore starts from a paraphrase,
- OpenHands (Software Agent SDK + Agent Server, Agent Canvas, OpenHands Cloud/Enterprise) [shipped. The SDK V1 ] (<https://docs.openhands.dev/sdk/guides/convo-persistence.md>) | where: Plain working-tree-style files on the backend host's local filesystem, not in git.
- SDK: under persistence_dir/<conversation-id>/, with workspace/conversations/ as the default (<https://docs.openhands.dev/sdk/guides/conv> | lesson: Take OpenHands' record shape and leave out its gaps.

The shape worth copying: one append-only directory per session, holding one immutable event per file plus a rebuildable snapshot. Branches are parent_id pointers with a movable HEAD inside the record. Forks deep-copy and record forked_from_event_id. Compaction is a Condensation event, so the log stays lossless and only the view loses detail. Each session has exactly one writer, enforced by a l

- Google Jules (Jules REST API v1alpha, Jules Tools CLI) [shipped] (<https://jules.google/docs/api/reference/>) | where: Google's servers and cloud, inside the Jules service. Transcripts, plans, bash output and memory never go into the repository.

Code results go to GitHub only on request. "Publish branch" or "Publish PR" in the web app p | lesson: Jules shows a split ParleyInn should copy. History lives in its own store as a typed, append-only event union. Every record has exactly one event kind and an originator (user, agent or system), and code-bearing artifacts are anchored as (baseCommitId, unidiffPatch, suggestedCommitMessage). Git carries only outcomes: branches, PRs and commit attribution.

For ParleyInn:

1. Give each record a code anchor: the base commit plus a patch digest. Any session ev

- Claude Code cloud sessions (formerly "Claude Code on the web") [shipped. It launched] (<https://code.claude.com/docs/en/claude-code-on-the-web>) | where: Server/cloud. Transcripts and session state are held by Anthropic and reached through claude.ai/code, not git (<https://code.claude.com/docs/en/data-usage>).

The VM is disposable: "Cloud sessions stop after a period of in | lesson: Copy the split, but make the link verifiable offline. Anthropic keeps code in git branches and the transcript in a separate single-writer event store, joined only by a `Claude-Session: <url>` commit trailer. Code stays reviewable through ordinary PRs, and the history never pollutes the repo. Teleport is a one-way fork rather than a merge, which shows they avoid multi-writer merging inside one transcript.

The trailer, though, is a mutable URL int

- Roo Code checkpoints (shadow-git snapshots in the Roo Code VS Code extension) [archived. The discov] (<https://roocodeinc.github.io/Roo-Code/features/checkpoints>) | where: A separate shadow git repo on the local disk, one per task, inside VS Code's extension globalStorage. The path is <globalStorageUri>/tasks/<taskId>/checkpoints/.git (RepoPerTaskCheckpointService.ts), for example ~/.confi | lesson: A shadow git repo pointed at the workspace (core.worktree) is a cheap, network-free way to snapshot code. Linking a session to code by writing the snapshot hash into the session log ("checkpoint_saved" with from/to hashes) is the right shape. Roo's version failed in three ways ParleyInn must design out:
- Unbounded storage: one full repo per task, with no shared object store, retention or size budget, led to 40 GB-to-TB blowups.
- Unredacted copying:
- Cursor Cloud Agents (formerly Background Agents) [shipped. Announced a] (<https://cursor.com/docs/cloud-agent>) | where: server/cloud (Cursor-managed). Locations by data type:
- Transcripts and conversation state: the "Cursor backend, encrypted with per-agent keys". Snapshots: an encrypted "snapshot and cache layer outside the active VM".  | lesson: Cursor keeps transcripts out of git, and ParleyInn probably should too. At more than a million public agent PRs, Cursor puts only a pointer in git and keeps the transcript where it can be deleted:
- Git gets signed commits authored "Cursor Agent", a Co-authored-by trailer and a PR-footer link to `bc-<uuid>`.
- The transcript sits in a backend encrypted under per-agent AES-256 keys, with a Delete Agent API and a 90-day retention option.
What ParleyInn can
- Code.Storage (The Pierre Computer Company) [shipped] (<https://code.storage>) | where: On a server, in the cloud. Pierre hosts the git repositories, multi-tenant by default. Single-tenant and VPC/private networking are offered on the Scale and Enterprise plans (<https://code.storage/pricing>). The listed sub | lesson: Code.Storage confirms that git works as the storage and transport backend for agent state at very large scale. What it sells beyond plain git is a set of server-side rules that a plain git host cannot express for custom refs:
- Compare-and-swap on every ref update (expectedHeadSha / expectedRefSha).
- Exactly one writer per ref: one session branch per session, written only by a sandbox token whose ref policy allows fast-forwarding its own branch
- Warp cloud agents on the Warp Automation Platform (formerly "Oz"; the CLI, API and web app still use the Oz name) [shipped. Cloud agent] (<https://docs.warp.dev/platform/>) | where: On a server / in the cloud. Run data (transcripts, artifacts, attachments) "lives in Warp-managed storage, independent of where runs execute" (<https://docs.warp.dev/platform/architecture/>). Warp hosts on GCP (<https://doc> | lesson: Warp keeps session history out of git and splits it in two:
- A small, ordered control-plane index: runs, lifecycle events, and mailbox messages that share one global sequence number.
- Bulk run-data blobs (transcripts, artifacts, attachments), each category routable to a customer-owned bucket through a narrowly scoped credential.

Git carries only code state, as branches, PRs and snapshot diffs.

ParleyInn could adopt the split and the customer-owne

- S2 (s2.dev) durable streams API, plus s2-lite (open-source self-hosted server) [shipped] (<https://s2.dev/>) | where: On the server or in the cloud, never in a git repository or the working tree.

Hosted s2.dev runs as per-region "cells" on AWS. Current locations are aws:us-east-1, aws:us-west-2 and aws:eu-north-1 (<https://s2.dev/docs/c> | lesson: Use S2's conditional-append contract as ParleyInn's sync primitive. Keep integrity and confidentiality on ParleyInn's side.

The model: one stream per session (or per agent run). A writer appends only with `match_seq_num` set to its local next sequence number, and puts the previous record's hash and a writer UUID in headers. Two worktrees or sandboxes racing on the same session then get a 412 and must re-read and re-chain. Nothing interleaves silently, a

- Durable Streams (Electric / durable-streams org) [beta] (<https://github.com/durable-streams/durable-streams>) | where: On a server, either self-hosted or Electric Cloud. Clients hold only offsets. There are several server implementations. (1) The Caddy plugin in Go is called "production" (<https://github.com/durable-streams/durable-stream> | lesson: Borrow the write contract, but not the trust model. Durable Streams' idempotent producer gives exactly-once appends from ephemeral writers without any merge step. Each append carries Producer-Id, Producer-Epoch and Producer-Seq; the server serialises validate-then-append per producer, treats duplicates as success, reports gaps, and fences stale epochs with 403. A fork is just (source, offset) metadata. Combined with one stream per agent session,
- Turso AgentFS [beta] (<https://github.com/tursodatabase/agentfs>) | where: A local DB file in Turso's SQLite-compatible format, one file per agent or session, with optional sync to Turso Cloud.

Paths:

- `agentfs init <id>` / SDK `open({id})` create `.agentfs/<id>.db` (README; MANUAL "Files" se | lesson: Do not use a synced, mutable database as the record. Ship sealed, hash-addressed segments instead.

AgentFS shows two ways the record gets lost:

(1) Its "audit trail" is a normal table. The spec says insert-only, but three of four SDKs UPDATE rows, and nothing checks the rule.

(2) Moving data between machines uses Turso's row-level "last push wins" with rollback-and-replay. On top of that, encryption and sync cannot be used together.

Either pa

- GitHub Copilot cloud agent session logs (formerly "Copilot coding agent") [shipped. The session] (<https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/track-copilot-sessions> (now 301-redirects to <https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/manage-and-track-agents>)) | where: Server/cloud: GitHub.com's Copilot backend. The data residency deployment on ghe.com is also covered (<https://github.blog/changelog/2026-07-02-copilot-agent-session-streaming-is-now-in-public-preview>). The log is not in  | lesson: Copy the commit trailer, but make it content-addressed and keep the log out of git. GitHub shows that one signed git trailer beside a Co-authored-by line gives code-to-session provenance. That provenance survives squash merges and needs no git storage for the history itself. Its weakness is the pointer: a host URL to a server-side blob that is shared by default and cannot be deleted. It has no digest, so it cannot be checked offline, it rots if t
- Claude Code cloud sessions (claude.ai/code), --teleport handoff, and the Claude-Session commit trailer [shipped. Cloud sessi] (<https://code.claude.com/docs/en/claude-code-on-the-web>) | where: The authoritative copy lives on vendor servers behind claude.ai (the Anthropic control plane). Code lives on GitHub branches; GitHub Enterprise Server is supported on Team and Enterprise. Non-GitHub repos can only be upl | lesson: Copy the split, which keeps a small pointer to the session in git and the history itself outside it. Then fix its three weaknesses.

1. Write the trailer deterministically. Anthropic's Claude-Session trailer is written by the model from a prompt reminder, so it can be skipped, overridden by CLAUDE.md, or forged. ParleyInn should add its trailer from a commit-msg or prepare-commit-msg hook.
2. Make the pointer verifiable offline. Anthropic's trailer i

- claude-session-tracker (ej31) [shipped. Stable npm ] (<https://github.com/ej31/claude-session-tracker>) | where: server/cloud, in GitHub's forge database rather than in git objects.

- **Records:** Issues and issue comments in a private repo (NOTES_REPO, default <user>/claude-session-storage), plus a private GitHub Projects v2 boar | lesson: Integrity metadata embedded in forge content is mostly for show unless three things hold.

1. Appends are serialized and atomic. Here two async hooks read-modify-write a chain head held in a local JSON file with no lock.
2. A verifier actually ships and runs. Here verify_chain() exists but nothing calls it.
3. The anchor lives outside the medium that writers can rewrite. Here the hashes are unkeyed and sit next to the content they protect, on a f

- ctx (ctxrs/ctx) [shipped] (<https://github.com/ctxrs/ctx>) | where: - **Local data root, not git.** The root is `~/.ctx`, overridable with `CTX_DATA_ROOT`. It holds:
  - `search/lexical/` (Tantivy Core generations);
  - `search/semantic/`;
  - `search/attribution/`;
  - `usage.sqlite`, ` | lesson: **Attribute code from evidence already in the record, not from git metadata.**

ctx blame proves that a session produced a commit by:

1. reading the commit output of `git commit`, `gh pr create` and similar tool results;
2. resolving that output against local Git objects;
3. answering proven, possible, conflicting or missing, or abstaining, and never accepting timestamps or similar diffs as proof.

It never writes into the repository. ParleyInn alre

- CommitLore [shipped. v1.7.2 was ] (<https://github.com/MongLong0214/commitlore>) | where: There are two authoritative stores, both git objects (SPEC §1; ADR-0003, <https://github.com/MongLong0214/commitlore/blob/v1.7.2/docs/adr/ADR-0003-git-ssot-derived-index.md>):

1. The trailer block of the commit message.
2. | lesson: If ParleyInn ever carries records over git refs, it should copy CommitLore's hard-won notes-sync discipline:

- Fetch the remote ref into a separate scratch ref and never force-update the working ref. CommitLore's forced `+refs/notes/*` refspec silently destroyed unpushed records (#417).
- Merge as a set union (cat_sort_uniq), and when git cannot merge, refuse and report rather than picking a winner. That fits I6, no silent failures.
- Keep all networ
- GNAP (Git-Native Agent Protocol), farol-team/gnap [announced. It is a d] (<https://github.com/farol-team/gnap>) | where: Working-tree files in a .gnap/ directory, committed to an ordinary shared git repo. It can be any repo, including the code repo, and can be hosted on GitHub, GitLab, Gitea or a bare repo over SSH (<https://github.com/faro> | lesson: GNAP is a useful counter-example for ParleyInn's sharing layer. Pushing to a shared mainline gives cheap cross-machine compare-and-swap with no infrastructure. But GNAP builds it from mutable shared JSON files with sequential IDs, and that forced per-file semantic merge rules (latest-updated wins, sum budgets, keep both messages, human wins), which GNAP then dropped. Its pulled "directive" messages also flow straight into the agent's next action.
If
- FAVA Trails (Federated Agents Versioned Audit Trail), by Machine Wisdom AI [shipped. v0.7.0 went] (<https://github.com/MachineWisdomAI/fava-trails>) | where: Working-tree files in a separate data git repo (the "Fuel" repo), which is distinct from the code repo and from the engine package. It runs Jujutsu (jj >= 0.28.0) in colocated mode on top of git, and the default path is  | lesson: If ParleyInn uses git as transport, every record should be an immutable file with a unique name, written once. All mutable state should be a projection rebuilt from the record: status, promotion, supersession, and "current" versus "historical". This is consistent with ParleyInn's invariant that derived state is rebuildable from the record.

FAVA shows what happens otherwise:

- New ULID-named files merge cleanly under fetch+rebase.
- Its in-place mutation
- VibeStats (stephenleo/vibestats) [shipped. v2.4.5 was ] (<https://github.com/stephenleo/vibestats>) | where: Three places, all in plain GitHub repos:

1. Working-tree files in a private GitHub repo named <user>/vibestats-data, which the installer creates with `gh repo create --private` (<https://github.com/stephenleo/vibestats/bl> | lesson: Give every writer its own path and do all cross-writer merging in a deterministic, read-only recompute. VibeStats keys each file by (day, harness, machine_id), so many machines can sync through one GitHub repo without git merge conflicts. Its aggregator is a stateless pure function over all partitions, close to what ParleyInn's I10 asks for. VibeStats gets this cheaply only because it is lossy and mutable: it overwrites whole-day files and can purge

- claudereview (CLI `ccshare`, npm packages `claudereview` and `claudereview-mcp`) [Shipped, but dormant] (<https://claudereview.com/>) | where: On a server, in a cloud database. One PostgreSQL `sessions` table holds the ciphertext as a base64 TEXT column, up to 100 MB per upload. The server is a Bun/Hono app, deployed to Railway with a Dockerfile (<https://github> | lesson: Share history as inert, structured, signed data. Never share it as rendered or runnable content. Never let a convenience feature quietly move key material to the server.

claudereview uploads a pre-rendered HTML page and runs its scripts in the viewer's origin. That makes a shared session an active-content injection channel, the browser version of what ParleyInn's I2 forbids. Its dashboard features also undid its "E2E" claim: the stored `ownerKey`, t

- hex/claude-sessions ("cs", a session manager for Claude Code) [shipped. The project] (<https://github.com/hex/claude-sessions>) | where: Working-tree files inside a local git repo, one per session.
- Sessions live at `~/.claude-sessions/<name>/`, or under `CS_SESSIONS_ROOT`.
- `cs -adopt` instead puts `.cs/` into an existing project in place, symlinks it  | lesson: Split the record by writer, make segments immutable, and keep it off the user's code branch. cs learned all three the hard way.
- **Union merge loses order and can bring text back.** Its shared append-only files merge with `merge=union`, which interleaves lines in no fixed order. The project measured that renaming or truncating such a file brings the old body back after a merge, so it now forbids those rewrites by protocol. A hash-chained log mer
- GitLab Duo Agent Platform (DAP) sessions, with AI audit events [shipped. DAP has bee] (<https://docs.gitlab.com/user/duo_>agent_platform/sessions/) | where: On the forge server, in database tables. Nothing is written to git. The pieces are:
- Session metadata: the PostgreSQL table duo_workflows_workflows.
- Checkpoints: daily-partitioned PostgreSQL tables. The older p_duo_wo | lesson: Treat compaction as a record type of its own, and never let the forge be where the record lives.

GitLab's checkpoint log is close to what ParleyInn wants:

- append-only rows ordered by UUID v7, with parent pointers for branches
- 'conversation' deltas and 'compaction' full snapshots, grouped by a counter so any reader can rebuild state from the record alone (ParleyInn's I10)
- a separate audit stream with producer UUIDs for retry-safe dedup, plus ingest
- Claude Code Remote Control [shipped. GA on Pro, ] (<https://code.claude.com/docs/en/remote-control>) | where: Server/cloud: Anthropic servers hold the synced transcript, retained under the Data usage policy (<https://code.claude.com/docs/en/data-usage>):
- consumer: 30 days, or 5 years if the user opts in to model training
- comme | lesson: Keep the local record authoritative and make any sharing layer a dumb, idempotent relay of immutable events, run outside the network-free core. Remote Control's history shows what goes wrong when the server mirror is a mutable transcript and messages have weak delivery guarantees. Its changelog fixes duplicate assistant messages after WebSocket reconnects, work items redelivered after token refresh, messages marked read on arrival and then lost,
- OpenCode /share (session share links) [shipped] (<https://opencode.ai/docs/share/>) | where: Local first: sessions live in a SQLite database, opencode.db (WAL), under the XDG data dir (~/.local/share/opencode) <https://github.com/anomalyco/opencode/blob/1ddb0873aee50d209d1a8d7f91b89c5daf692d49/packages/core/src/d> | lesson: Treat a share as a derived, redacted projection pushed one way, never as the record, and never as a source to resume from. OpenCode overwrites a mutable last-write-wins JSON snapshot on a CDN and cannot recall it once cached. It uploads unredacted tool output, and its import/pr path turns someone else's share into model context. For ParleyInn, sharing should be opt-in per session (manual by default, with a `disabled` policy committed in the repo, as
- Goose session export / import, with the encrypted Nostr share link (aaif-goose/goose, formerly block/goose) [archived] (<https://www.mankier.com/1/goose-session-export>) | where: Local: one embedded SQLite DB at $XDG_DATA_HOME/goose/sessions/sessions.db, default ~/.local/share/goose/sessions/sessions.db, or %APPDATA%\Block\goose\data\sessions\sessions.db on Windows. It has used SQLite since v1.10 | lesson: Treat any transport for shared history as a carrier of untrusted, unauthenticated data, and make the payload pure data. Goose's maintainers retired Nostr sharing within about 4.5 months. They gave four reasons: the throwaway per-share keys meant an imported session could not be authenticated, a bearer key in a URL cannot be revoked (and was logged), there was no useful trust point in the UI at import, and the dependency weight was not worth it. A
- Crush (Charm) [shipped] (<https://github.com/charmbracelet/crush>) | where: Local DB. Each project has its own SQLite file at <project>/.crush/crush.db.

Where .crush ends up:

- The default name ".crush" is set in <https://github.com/charmbracelet/crush/blob/76cc5c574e15072b15aaed0f4f843a5711fae0> | lesson: Copy Crush's real-time pattern: one process owns the store under a flock and fans changes out over a local socket and SSE, and clients resync when they reconnect. Do not copy what sits underneath it, a mutable database per worktree.

What Crush gets wrong for ParleyInn:

- It UPDATEs a message's parts while the reply streams and cascade-deletes sessions.
- It has no hash chain.
- It keys each database to `git rev-parse --show-toplevel`, so worktrees e
- Pi coding agent sessions (earendil-works/pi, formerly badlogic/pi-mono), JSONL session tree with /tree, /fork, /clone, /export, /share [shipped] (<https://pi.dev/docs/latest/tree>) | where: Local filesystem, outside any repository:
- Default path: ~/.pi/agent/sessions/--<cwd with separators replaced by '-'>--/<ISO timestamp>_<session-id>.jsonl. So sessions are grouped by absolute working directory, and each | lesson: Copy Pi's core idea: keep each event's parent pointer inside an append-only log. A fork, a branch switch or a compaction then adds entries and never rewrites history, and a context_edit is a new event, not a mutation. But make the IDs content-addressed instead of random 8-hex: hash(parent hash + canonical payload), which also gives ParleyInn its hash chain. Then copies of the same session from worktrees, cloud sandboxes or machines combine by plain s
- pi-share-hf [shipped. npm 0.5.0 w] (<https://github.com/badlogic/pi-share-hf>) | where: Input is read from `~/.pi/agent/sessions/--<cwd with / replaced by ->--/*.jsonl`. Only the session folder whose name is derived from the configured `cwd` is read (<https://github.com/badlogic/pi-share-hf/blob/main/src/col> | lesson: Treat publishing or sharing history as a separate gate that sits in front of any shared, git-backed store and fails closed. Cache each verdict under a key built from the content hash and the policy hash: secret-set version, deny rules, model and prompt version. That way re-runs are cheap and auditable, as pi-share-hf's `redaction_key` and `review_key` are.

Four of pi-share-hf's mistakes are worth not copying:

1. **Leaking secret-derived data.**

- Kilo Sessions (Kilo Code session history, cloud sync, remote mode and share/fork) [shipped] (<https://kilo.ai/docs/collaborate/sessions-sharing>) | where: Local copy:
- A SQLite file, kilo.db, on the machine where Kilo runs: ~/.local/share/kilo/kilo.db, or on the remote host under VS Code Remote SSH. It uses WAL mode, and `kilo db path` prints the location (<https://kilo.ai> | lesson: Kilo uses git only for metadata and moves history through a cloud object per session that keeps only the latest state of each message and part. Concretely:
- Git supplies the remote URL, the branch, and PR links recorded only on tool-observed evidence (a PR-create output or a push of the head branch, with headRef and headSha).
- Writes to the cloud object are last-writer-wins upserts.
- Share links expose that live, changing state.

The consequen

- Roo Code Cloud Task Sync and Task Sharing (Roo Code VS Code extension + app.roocode.com) [archived] (<https://docs.roocode.com/roo-code-cloud/task-sharing>) | where: Primary copy: local files in VS Code globalStorage, under tasks/<taskId>/. These are ui_messages.json, api_conversation_history.json, task_metadata.json and history_item.json (<https://github.com/RooCodeInc/Roo-Code/blob/> | lesson: Make any shared or synced copy a verifiable replica of ParleyInn's local append-only log, on a transport the user controls. Never make it a best-effort side stream into a vendor service.

Roo's sync was fire-and-forget telemetry:

- one POST per message
- duplicates suppressed only by the message's millisecond timestamp
- a 100-entry retry queue that silently dropped its oldest entry when full and stored bearer tokens at rest
- a whole-file backfill o
- Harbor ATIF (Agent Trajectory Interchange Format) [shipped] (<https://docs.harborframework.com/core-concepts/agents/atif>) | where: Plain files in a working tree. In a Harbor trial directory the trajectory is `agent/trajectory.json`, with media in sibling `images/` and `audio/` directories (<https://docs.harborframework.com/core-concepts/agents/atif;>  | lesson: Use ATIF as an export projection that ParleyInn writes, and never as an import path into the model. Concretely: (1) Write an ATIF exporter as a rebuildable projection of the hash-chained record (I10). It would give ParleyInn interop with Phoenix/OpenInference, RL and SFT pipelines, and Harbor's viewer. Its `is_copied_context` flag and `context_management` boundary markers already match ParleyInn's compaction and restore events. The exporter must not fill mis
- OpenHands Software Agent SDK: ConversationState / EventLog persistence (plus agent-server and OpenHands Cloud export) [shipped. The EventLo] (<https://docs.openhands.dev/sdk/arch/conversation.md>) | where: On the local filesystem by default, through a pluggable FileStore. The interface has write, read, list, delete, exists and lock. LocalFileStore and InMemoryFileStore ship (<https://github.com/OpenHands/software-agent-sdk/> | lesson: The design to copy is a per-session single-writer log of immutable, sequence-numbered files (index plus id in the file name), with owner fencing by lease plus a rising generation number. The OpenHands files are trivially copyable by rsync, zip or git.

Its own limits show what ParleyInn has to add before such a log can be shared across machines and sandboxes:

1. **Coordination:** flock-based coordination is officially unsafe on NFS and shared storage

- agentsview (kenn-io/agentsview; formerly wesm/agentsview) [shipped. The latest ] (<https://github.com/kenn-io/agentsview>) | where: **Primary store:** a local SQLite database, ~/.agentsview/sessions.db, in WAL mode with FTS5. A per-data-dir db.write.lock makes one daemon the sole writer (<https://github.com/kenn-io/agentsview/blob/main/docs/configurat> | lesson: Make every replica a single-writer origin. Share only immutable, content-addressed objects plus a per-origin append-only journal. Never sync a database or its WAL. agentsview reached this after trying every other option.

- Its docs warn that copying sessions.db or its WAL 'can corrupt or fork the archive' and would duplicate 'publication authority'.
- Its newest transport is artifact folder sync: SHA-256-named canonical-JSON segments and manifes
- cass (coding-agent-search, crate "coding-agent-search", repo coding_agent_session_search) [shipped. Latest rele] (<https://github.com/Dicklesworthstone/coding_>agent_session_search) | where: Local disk only: a local database plus sidecar files. Git does not hold or carry the archive.
- Canonical DB: ~/.local/share/coding-agent-search/agent_search.db, a frankensqlite (pure-Rust SQLite reimplementation) WAL da | lesson: Make ingest idempotent on stable natural keys, so the transport can be dumb and additive. Then add what cass leaves out: authenticity, a real append-only log, and no raw secrets at rest.

What cass gets right:

- It does multi-machine without git and without merge logic. A hub pulls each machine's raw files with additive-only rsync (no --delete) into per-source read-only mirrors.
- Re-ingest is harmless because conversations are UNIQUE(source_id,
- claude-code-transcripts (Simon Willison) [shipped. The latest ] (<https://github.com/simonw/claude-code-transcripts>) | where: Input:
- Claude Code's local JSONL logs under ~/.claude/projects/<encoded-cwd>/<session>.jsonl.
- Or web-session JSON fetched from undocumented api.anthropic.com endpoints. This path is broken.
- Or any JSON/JSONL URL.

 | lesson: Treat sharing as its own explicit, opt-in step that turns the record into a separate copy. Don't let sharing become a storage or transport layer. In practice that means four things:

1. Make the copy from ParleyInn's own local record, not from vendor endpoints. This tool broke when Anthropic changed private APIs (issue #77).
2. Run every shared copy through a required redaction pass that is audited and counted (I6).
3. Escape all model-written text an

- C3Sync (part of the C3 "Control / Continuity" suite by Code Chat Connect / ccc-app) [beta. The latest pub] (<https://getc3.app/sync>) | where: Several places:
- The canonical shared copy is ciphertext in a single self-hosted NATS + JetStream hub. The hub "provides routing, durable streams, and object storage" but never receives content keys (<https://getc3.app/s> | lesson: C3Sync shows that git does not need to carry session history: put a dumb, ciphertext-only relay behind a local canonical store. Each peer keeps its own record (C3Sync uses SQLite). Ships go out as AEAD-sealed, scheme-versioned envelopes with authenticated algorithm IDs, through a durable-consumer log (NATS JetStream), so offline sandboxes catch up. Git is used only locally, for working-tree snapshots tied to turn IDs ("restore anchors"). Pushes s
- Happy (Happy Coder), by slopus [shipped] (<https://github.com/slopus/happy>) | where: On a server you can host yourself; git is not involved. The hosted relay runs Node (Fastify and Socket.IO) on Postgres via Prisma, with Redis and S3/MinIO, on Kubernetes. The CLI default is api.cluster-fluster.com, and t | lesson: Happy shows that a blind relay with per-session DEKs wrapped to an account public key works in practice for real-time sync across devices. It also gives a useful write-only split: a machine or cloud sandbox can append encrypted records but cannot decrypt anyone else's history. ParleyInn should copy that for ephemeral sandboxes.

Happy also shows that confidentiality is not integrity. Its ciphertexts carry no associated data and no chain, so the relay

- agent-sync (lidongpeng36) [shipped] (<https://github.com/lidongpeng36/agent-sync>) | where: Files in place on each machine. History stays in the agents' own home directories (~/.claude, ~/.codex, ~/.local/share/opencode) on the local machine and on one SSH peer. Sync state lives in the local cache and data dire | lesson: Two things carry over directly. Treat 'strict prefix' as the only automatic merge. When histories truly diverge, name the fork deterministically, so every machine that sees the split arrives at the same fork ID without coordinating: agent-sync uses UUIDv5 of the parent session with the first differing record. For ParleyInn that becomes fork ID = H(parent chain ID ‖ hash of the first differing event), and fork detection is simply the first unequal cha
- claude-sync (tawanorg) [shipped. The latest ] (<https://github.com/tawanorg/claude-sync>) | where: On server/cloud storage the user provisions: a Cloudflare R2, AWS S3, Google Cloud Storage or S3-compatible bucket (Backblaze B2, MinIO, Wasabi, Ceph and others), or a WebDAV directory (Nextcloud, ownCloud) (<https://gith> | lesson: Do not sync append-only session history as mutable whole-file objects. Doing so is what produces almost all of claude-sync's field failures: 98.9% false conflicts on transcripts that only grew (#69), truncated copies that overwrite complete ones without warning and then spread (#70), last-writer-wins loss on history.jsonl (#72), and deleted files that come back because no tombstone exists (#79, #80). ParleyInn should move history as immutable, conten
- Stift (stift-sh/stift) [shipped] (<https://stift.sh/>) | where: Client/server over HTTP. It does not use git for storage or transport. Server, two generations:
(1) Release v0.4.0 (tag dated 2026-08-25), still what the live stift.sh page documents: a single pure-Go stdlib binary, 'sti | lesson: Do not sync agent-native session files as whole-file snapshots keyed by host. Stift does exactly that, and three gaps follow, all confirmed in daemon.go, state.go and store.ts:
- Updates replace the server copy in place, so no history is kept.
- Each record is restored at most once and never overwrites a local file. A session continued on a second machine forks into two diverging records, and later appends never propagate. Conflicts are only logg
- Opaline CLI (formerly Rudel; canonical repo now opalinehq/cli, old obsessiondb/rudel URL redirects) [shipped. The hosted ] (<https://github.com/opalinehq/cli>) | where: Server/cloud (hosted, proprietary). The last public server design (Aug 2026) kept transcripts in ClickHouse tables `rudel.claude_sessions` and `codex_sessions` on an S3 storage policy, plus analytics materialized views.  | lesson: Copy Opaline's transcript-revision watermark when ParleyInn imports Claude Code JSONL into its hash-chained record. Treat the source file as untrusted and possibly rewritten. On each hook:
- Take only whole lines, up to the last newline.
- Record {byteOffset, prefixSha256, recordCount}, per-chunk sha256s and parentRevisionId. revisionId is the SHA-256 of the canonical manifest.
- Before appending, re-hash the old prefix. If the source shrank, the pre
- Review Assist (uditk2/review-assist): MCP server "review-assist-mcp" plus the GitHub App "Review Assist Guided Review" [shipped. The npm pac] (<https://github.com/uditk2/review-assist>) | where: - Document: working-tree file `.intent/<branch>.json`, committed with the code on the PR branch. Branch "/" becomes "-". Files are never cleaned up after merge: 10 such files sit on main today.
- Local files: ~/.review-a | lesson: Use git to carry a projection, never the record. Review Assist keeps the raw transcript local and commits only a derived artifact. That artifact is schema-checked, secret-linted, keyed to commit SHAs and anchored to hunks, and only a deterministic gate in front of the write lets it through.

For ParleyInn this matches I2, I6 and I10:

- Keep the hash-chained log local or private.
- Rebuild from it, with no clock or randomness, a per-branch summary tha
- Exceeds Ink (Exceeds AI, Inc.), the `exceeds-ink` CLI [beta. The product pa] (<https://www.exceeds.ai/ink>) | where: Git notes under the custom ref refs/notes/exceeds-ink (<https://blog.exceeds.ai/code-level-ai-provenance/>), plus a local SQLite DB on the developer machine. Optional remote delivery goes to a collector over OTLP HTTP (def | lesson: Put a reference in git, never the content, and make that reference safe. Ink's one design choice worth copying: the shared, permanent, fork-replicated git channel carries only a small per-commit note (session ID, turn, digest, plus a stated "unknown" bucket), while transcripts stay in a local SQLite store that is pulled on demand. If ParleyInn uses git at all, it should push only a notes ref that maps each commit to a ParleyInn session ID and record-chai
- gitwhy (ghw), surajsrivastav/gitwhy [beta. The README car] (<https://github.com/surajsrivastav/gitwhy>) | where: Default: git notes under the custom ref refs/notes/gitwhy, attached to the commit SHA. It is written with `git notes --ref refs/notes/gitwhy add -f -m <json> <sha>` (<https://github.com/surajsrivastav/gitwhy/blob/master/p> | lesson: Do not use git notes, or any single notes ref shared and force-fetched by every writer, as ParleyInn's multi-writer transport. gitwhy's setup loses records silently.
- A forced fetch overwrote a clone's unpushed notes.
- 30 concurrent `git notes add` all exited 0, yet only 2–5 notes survived (git 2.43.0).
- Notes do not follow rebased or squashed commits unless notes.rewriteRef is set.

If ParleyInn ever carries records over git:

- Give each writer its o
- ai-blame (ai4curation/ai-blame) [shipped. The latest ] (<https://github.com/ai4curation/ai-blame>) | where: Provenance lives in working-tree files, and the user commits them with ordinary git; there is no git notes, custom ref, branch or server. There are three placements:
- a sidecar file, by default '{stem}.history.yaml' nex | lesson: If ParleyInn commits provenance into git, each entry needs a content address that points back to the original immutable record (session id plus record hash in the hash chain), and merging should match entries by that hash. ai-blame shows what goes wrong otherwise:
- Its committed sidecar drops session_id, so shared history can't be traced back to a session or checked against one.
- Its sidecar merge treats entries with the same RFC3339 timestamp as d
- Cursor Blame (with the AI Code Tracking API and the related Agent Trace spec) [shipped. Cursor Blam] (<https://cursor.com/docs/integrations/cursor-blame>) | where: Server and cloud (Cursor's backend), with a local cache and an on-device signature log. Not in git. The docs say "Cursor Blame caches attribution data locally for performance. When you view files and commits, data is fet | lesson: Cursor keeps line attribution out of git. Git gets only a coarse "Made with Cursor" trailer. The real link from session to code is joined on Cursor's server, from signatures that live on one machine and are matched at commit time. That design breaks in exactly the places ParleyInn must work:
- Ephemeral sandboxes, because CLI commits "must be scored on the same machine".
- Formatters, which invalidate signatures.
- Amend, rebase and squash, which lea
- in-toto AI Agent Action predicate, v0.1 (in-toto/attestation PR #588) [announced. The PR is] (<https://github.com/in-toto/attestation/pull/588>) | where: Local append-only JSONL files written by the intermediary. The spec's worked example calls the file audit.jsonl. The reference gateway writes ~/.mcp-audit/audit.jsonl, a ~/.mcp-audit/key.hex HMAC key and a .state.json ho | lesson: Pin exact bytes and test them adversarially, rather than leaning on hash links.

1. Define the record's canonical bytes once. JCS-style is a good fit, and Go's encoding/json fails it by default because it HTML-escapes <, > and &. Require the writer to store exactly those bytes, and have the verifier recompute them and reject on mismatch, so a reparse-and-re-emit step cannot silently break the chain. Keep the chain-hash, signature and payload-diges

- agentattest (AuroraAeon/agentattest) [shipped] (<https://github.com/AuroraAeon/agentattest>) | where: Local file, plus attestation stores the project delegates to but does not itself write.

- `predicate create --out` writes the statement as a local JSON file (<https://github.com/AuroraAeon/agentattest/blob/v0.1.0/README.> | lesson: Publish a signed claim to git or GitHub, not the session.

1. Shape of the claim. For each session segment or commit, ParleyInn could emit an in-toto Statement v1:

  - subject = git commit/tree object IDs;
  - predicate = the ParleyInn hash-chain head and segment range, by digest plus an opaque URI;
  - transcript content excluded by design. Borrow agentattest's rules: visibility classes, `data:`/userinfo URI rejection, closed digest-only extensions,

- SLSA source-tool (sourcetool, SLSA Source Track proof of concept; formerly slsa-framework/slsa-source-poc) [beta] (<https://github.com/slsa-framework/source-tool>) | where: git notes on the attested commit, under the default ref refs/notes/commits in the same repository. The commit's history is never rewritten (DESIGN.md 'Attestation Storage'; storer constant notesRef = "refs/notes/commits" | lesson: Git notes can carry signed, hash-chained records next to commits without rewriting history. But one shared notes ref behaves like a mutable, last-writer-wins mailbox. source-tool fetches, appends and pushes without force, with no retry or merge, and treats any missing record as possible tampering. A lost race or a crashed CI run therefore looks the same as an attack, and the authors list this as unsolved.

If ParleyInn uses git as a transport, it sho

- gittuf [beta] (<https://github.com/gittuf/gittuf>) | where: Custom git refs inside the same repository, pushed to any unmodified forge (GitHub, GitLab, Bitbucket, self-hosted): refs/gittuf/reference-state-log, refs/gittuf/policy, refs/gittuf/policy-staging (staged, not yet applie | lesson: gittuf proves that a signed hash chain can travel in custom refs on unmodified forges. Each entry is one empty-tree commit, linked by parent and signed with standard git signing, at about one object per entry. It also shows the price of one global linear log: every writer must fetch, rebuild and retry an atomic push. Reconcile only works when writers touched different refs. A forge can fork the log undetected until someone compares notes out of b
- Linux kernel Assisted-by trailer (Documentation/process/coding-assistants.rst) [shipped. The policy ] (<https://docs.kernel.org/process/coding-assistants.html>) | where: In the trailer block at the end of the git commit message, so it is inside the commit object and covered by its hash. It travels inline in patches emailed to the mailing lists (lore.kernel.org) and is then frozen when a  | lesson: Never make a git commit trailer the link ParleyInn depends on between session history and code. The kernel's experience shows four problems:
- Maintainers strip or rewrite trailers at apply time (netdev hook, Brauner's rewrite to "LLM").
- The schema changed within two releases (AGENT:MODEL to LLM), leaving mixed, immutable history.
- Checking it is weak and has changed with each format (checkpatch first pattern-matched '\S+:\S+', now accepts any non
- simonw/claude-code-transcripts (PyPI: claude-code-transcripts; first released as claude-code-publish) [shipped] (<https://github.com/simonw/claude-code-transcripts>) | where: Inputs:

1. Local Claude Code JSONL files under ~/.claude/projects/**. The `local` command (the default) offers a picker of the 10 most recently modified sessions.
2. Claude Code for web sessions, fetched from undocumente | lesson: The common way to attach a transcript to a PR today is: render a snapshot, upload it to an unlisted URL, and paste the link into the commit. This tool shows the three ways that pattern fails, and ParleyInn's export path should be designed against each:

1. Recalled text was treated as trusted HTML. Markdown with raw-HTML passthrough plus |safe gives stored injection: issue #105 lost 2 hours of a 1 MB transcript, and <script> would run on a shared ori

- lossless-claw (Martian-Engineering), an OpenClaw context-engine plugin implementing Voltropy's LCM [shipped] (<https://github.com/Martian-Engineering/lossless-claw>) | where: A local SQLite database file on the OpenClaw gateway host, default ${OPENCLAW_STATE_DIR:-~/.openclaw}/lcm.db. It is opened through Node's built-in node:sqlite DatabaseSync with journal_mode=WAL (<https://github.com/Martia> | lesson: Replicate and hash-chain only the raw event log. Treat the summary DAG and context_items as a per-machine projection that can be rebuilt, and never inject it automatically.

lossless-claw keeps the paper's split between "immutable store" and "materialized view". In practice, though, it keeps both in one mutable SQLite file:

- delete paths exist (doctor clean, heartbeat pruning, TUI rewrite, dissolve and transplant);
- summary ids depend on wall-c
- claude-mem (thedotmack / Alex Newman) [shipped. v13.28.0 wa] (<https://github.com/thedotmack/claude-mem>) | where: A local DB per user and machine: SQLite at ~/.claude-mem/claude-mem.db, opened through bun:sqlite in WAL mode with busy_timeout 5000 and synchronous NORMAL. The Chroma vector store sits under the same data dir; every pat | lesson: ParleyInn should take claude-mem's cross-device sync shape but not its trust model. The sync design (services/sync-api plus SyncApply) is a strong template. Each user gets one ordered log with a server-assigned seq. Ops are canonical JSON, SHA-256-hashed and idempotent on origin identity, with monotonic per-entity revs and explicit stale or hash-conflict refusals. Each device keeps a cursor, and applying a batch moves the cursor in the same transacti
- MCP reference Memory server ("Knowledge Graph Memory Server", npm @modelcontextprotocol/server-memory, part of modelcontextprotocol/servers) [shipped. It is an ac] (<https://github.com/modelcontextprotocol/servers/tree/main/src/memory>) | where: It is one local file. The default is memory.jsonl in the package's own dist directory, next to index.js. MEMORY_FILE_PATH overrides it: absolute paths are used as-is and relative paths resolve against the package directo | lesson: A per-process mutex is not multi-writer safety, and a mutable file with whole-file rewrites loses history and updates without telling anyone. MCP stdio starts one server process per client, so every agent pointed at a shared store is a separate OS-level writer. The reference server needed about 15 months (#1819, May 2025, to #4555, Sept 2026) to serialise even writes inside one process. It still loses updates across processes, with no error or co
- Basic Memory (Basic Machines) [shipped (active). La] (<https://github.com/basicmachines-co/basic-memory>) | where: Local: working-tree Markdown files in a user-configured project directory, which may or may not be a git repo. Next to them sit a local SQLite index (sqlite-vec for vectors) and config and state under ~/.basic-memory, in | lesson: Syncing mutable documents breaks down once there are many writers, and Basic Memory shows where. File-level transport could not carry multiple writers safely: rclone bisync uses newer-wins, team push/pull aborts on any divergence, deletes cannot propagate without a baseline, and the team-safe reconciler is still open as issue #862. To get real concurrency, Basic Memory moved to a DB-first, strictly ordered per-project journal of accepted changes
- mcp-knowledge-graph (shaneholloman), npm package "mcp-knowledge-graph", MCP server named "Aim-Memory-Bank" in the README examples [shipped. v1.4.0 was ] (<https://github.com/shaneholloman/mcp-knowledge-graph>) | where: Plain JSONL files on the local filesystem.

How the location is chosen:

1. Auto mode walks up at most 5 directories from the process cwd. It stops at the first directory holding any of .aim, .git, package.json, pyproject | lesson: Two lessons, both checked against the code:

1. **Mark your own files.** The first-line _aim marker is a cheap guard worth copying in hardened form. ParleyInn can open each segment with a versioned, signed or hash-anchored header naming the store, and refuse to read or extend any file without it.
2. **Never let a store pick its own location or rewrite in place.** Whole-file rewrites lost 74 of 80 successful concurrent writes without a single error, w

- Claude API memory tool (memory_20250818) [shipped. It launched] (<https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool>) | where: Wherever the application's handler puts it. "The memory tool operates client-side: Claude requests file operations, and your application executes them." "The /memories path is a prefix that your handler maps onto real st | lesson: memory_20250818 shows how far the cheap design goes and where it breaks. It is a mutable, model-written, last-writer-wins file tree. It has no history, no concurrency control and no provenance. An auto-injected "ALWAYS VIEW YOUR MEMORY DIRECTORY" prompt makes its contents de facto automatically re-read into context, the exact poisoning channel Anthropic's own cookbook calls critical.

Anthropic's own follow-up (Managed Agents memory stores) had t

- GitHub Copilot Memory (agentic memory, "cross-agent memory") [beta. It is still pu] (<https://docs.github.com/en/copilot/concepts/agents/copilot-memory>) | where: GitHub-hosted server-side storage. The backend is undocumented. The engineering blog only says agents "populate the memory database organically, using the 'store_memory' tool" (<https://github.blog/ai-and-ml/github-copilo> | lesson: Anchor every derived memory to code with a citation, and re-check it against the current tree at recall time. Do the check mechanically, not with the model. Copilot proves the pattern works: path:line citations, just-in-time checks, and a self-healing pool showing +7pp merge rate. It also shows two weaknesses ParleyInn should avoid. First, its citations carry no commit or blob hash. Second, the check is "the agent is encouraged" to verify, so an inje
- Roo Code Memory Bank (GreatScottyMac/roo-code-memory-bank) [archived (in practic] (<https://github.com/GreatScottyMac/roo-code-memory-bank>) | where: Working-tree files in a memory-bank/ directory at the project root, next to projectBrief.md (README "File Organization": <https://github.com/GreatScottyMac/roo-code-memory-bank#file-organization>). The behaviour lives in p | lesson: Do not confuse a curated summary with a record, and do not make a mutable file both the source of truth and the thing that gets injected. In the Memory Bank, "append-only", "real-time sync" and "conflict resolution" are only promises in the prompt. The model can rewrite activeContext.md with apply_diff, nothing chains or verifies the history, and every file is fed back verbatim as trusted context, so whoever can commit to memory-bank/ can steer t
- Context Portal / ConPort (GreatScottyMac/context-portal) [shipped. It is beta:] (<https://github.com/GreatScottyMac/context-portal>) | where: Local files inside the project working tree by default. The database is <workspace>/context_portal/context.db, the Chroma persistence directory is <workspace>/context_portal/conport_vector_data/, and the Alembic files si | lesson: ConPort shows two failure modes ParleyInn's invariants exist to prevent.
First, derived indexes drift when they are a second write path instead of a projection. ConPort writes the Chroma vectors beside SQLite in the same request and swallows embedding errors. Updates skip re-embedding and some deletes never reach the index. The vector store is not process-safe, and semantic search ends up returning deleted records (issue #85). ParleyInn should keep every
- MemPalace [shipped. v3.10.0 is ] (<https://github.com/MemPalace/mempalace>) | where: On the local machine. The directory is ~/.mempalace for existing installs, or ~/.config/mempalace / $XDG_CONFIG_HOME/mempalace for new installs since 3.10.0 (CHANGELOG 3.10.0). It holds the palace, config.json and the ca | lesson: MemPalace's replication design gives ParleyInn a template for real-time multi-machine sharing without git merges. Each replica keeps its own append-only, strictly ordered log keyed (origin, seq). Peers exchange version vectors and pull the missing per-origin ranges: snapshot first, then tail, with the cursor based on local arrival order rather than timestamps. Ops are applied idempotently, merge is a union, and vector/FTS indexes are never synced but
- Kilo Code Memory Bank (deprecated) [archived] (<https://kilo.ai/docs/advanced-usage/memory-bank>) | where: Ordinary files in the repository's working tree.
- Original location: .kilocode/rules/memory-bank/, with the protocol file at .kilocode/rules/memory-bank-instructions.md. Source: <https://github.com/Kilo-Org/kilocode/blob> | lesson: Memory Bank is a worked example of what ParleyInn's invariant I2 (no automatic path from untrusted content to the model) exists to prevent. Model-written, lossy summaries sat in the repo as editable files and were loaded every session as system-prompt rules. That had two effects:
- Anything an agent, a contributor or a poisoned file once wrote became trusted instruction for every later agent and every cloner.
- context.md, rewritten in place, guarant
- OpenClaw memory (memory-core plugin, builtin SQLite engine) [shipped] (<https://docs.openclaw.ai/concepts/memory>) | where: Working-tree files plus a local DB, all on the user's host.
- Markdown lives in the agent workspace, default ~/.openclaw/workspace, per agent in multi-agent setups (<https://docs.openclaw.ai/concepts/agent-workspace>).
- T | lesson: Keep trust labels out of band and enforce them structurally:
- Store each record's origin class in metadata that classification code writes from the capture path, never parsed from the content. Default anything unknown to untrusted.
- Make automatic injection impossible for anything outside the trusted tier, rather than relying on content scanning.
- Mark everything ParleyInn re-injects or returns from recall so it can never be captured again as new
- Hermes Agent memory (Nous Research): bounded MEMORY.md/USER.md, SQLite/FTS5 session store, session_search, pluggable memory providers [shipped] (<https://hermes-agent.nousresearch.com/docs/user-guide/features/memory>) | where: Everything lives locally on one machine. Memory files are in ~/.hermes/memories/ (per profile: ~/.hermes/profiles/<profile>/memories/). Session history is in the SQLite file ~/.hermes/state.db. Code checkpoints go to a s | lesson: Store the trust label with each record and apply it again whenever the record is recalled. Do not rely only on wrapping output once, when the tool first runs. Hermes shows the gap. Live web, browser and MCP output is wrapped as `<untrusted_tool_result>`, and the memory files are threat-scanned. But `session_search` returns the same stored content later with no envelope, because it is not on the untrusted-tool list. External provider recall is pus
- MCP Memory Service (doobidoo/mcp-memory-service) [shipped] (<https://github.com/doobidoo/mcp-memory-service>) | where: Local DB plus an optional cloud copy. The sqlite_vec backend is a local SQLite file using the vec0 virtual table plus an FTS5 trigram index. The cloudflare backend keeps rows in Cloudflare D1, vectors in Vectorize, and c | lesson: The local-first, async-replicated shape is right (about 5 ms local reads, a durable outbound queue, offline tolerance, sync owned by one process). But the merge semantics show what to avoid for an append-only record. Mutable rows plus last-writer-wins on updated_at, and tombstones that expire after 30 days, lose concurrent edits, depend on clocks, and can bring deleted data back on a device that was offline too long. Content-addressed dedup on no
- Sculptor (Imbue) [beta] (<https://imbue.com/sculptor/>) | where: All local files, nothing on a server:
- Database: ~/.sculptor/database.db (SQLite).
- Workspaces: ~/.sculptor/workspaces/<internal-id>/, with the working copy at code/.
- Raw transcript: <workspace_root>/artifacts/tasks/ | lesson: Sculptor is a real case of a product that dropped per-agent containers with two-way git sync. Imbue's stated reasons: per-agent isolation stopped agents from inspecting each other's work, users found it confusing, and Docker Desktop hurt performance. It moved isolation to the whole app.

What it settled on is a clean split. Git carries code: workspace branches live in the user's own repo, so there is nothing to sync. A local, insert-only SQLite e

- Claude Code checkpointing (/rewind, file history) [shipped] (<https://code.claude.com/docs/en/checkpointing>) | where: Local disk only, in the user's config directory (default ~/.claude, movable with CLAUDE_CONFIG_DIR):
- Backup blobs go in ~/.claude/file-history/<session-id>/.
- Per-checkpoint metadata goes in the session transcript at  | lesson: Do not treat ~/.claude/file-history as a durable source. Capture at hook time into ParleyInn's own record, and make the capture boundary explicit and audited.

Why the store fails as a source:

- It is tool-scoped (Write/Edit/NotebookEdit only; Bash, background subagents and other sessions are invisible).
- It is capped (100 checkpoints) and swept (about 30 days).
- Its format is internal and version-dependent.
- It copies secrets in plaintext.
- Unti
- Jujutsu (jj) operation log [shipped] (<https://docs.jj-vcs.dev/latest/operation-log/>) | where: A local directory store under .jj/repo/, separate from git:
- op_store/operations/<hex> and op_store/views/<hex>, one file per object (<https://github.com/jj-vcs/jj/blob/0cb02a837f28459cd698734264c9fcd3712ec0d1/lib/src/si> | lesson: Copy jj's write protocol for many writers. Make each record an immutable, content-addressed object whose hash covers its parent ids. Mark heads with empty files named by the head id. Let any writer commit without a lock. Treat several heads as data: the next reader merges them deterministically and records conflicts explicitly ("A to B or C"). This survives worktrees, sandboxes and file-syncing transports, and fits ParleyInn's I10 rebuild-from-record
- Lix (Opral), an embeddable version control system for files and SQL tables [beta] (<https://lix.dev>) | where: Lix runs as a library inside the application, so storage is either a local database or a server.

Local options (<https://github.com/opral/lix/blob/main/docs/persistence.md>):

- Memory (the default; lost on exit).
- `Files | lesson: Copy Lix's architecture, but not its defaults about history.

**Worth copying.** Lix keeps its engine separate from where data lives and how it moves:

- Storage sits behind a small ordered, transactional key-value contract with a public conformance suite (space isolation, coherent reads, ordered scans, atomic commits, a stated durability boundary).
- Sync is a separately versioned protocol. Uploads are idempotent by immutable identity, branch upd
- Oak (Oakspace Inc., "Oak VCS") [beta] (<https://oak.space>) | where: Client side: a single SQLite database per repo at <repo>/.oak/oak.db, holding manifests, blobs, commits and a stat cache (cli/src/resolve.rs and core/src/sqlite.rs in <https://oak.space/oak/oak;> "the manifests, blobs, and | lesson: Fix the hash-chain preimage before the first record ships. Oak's commit hash is BLAKE3 over newline-joined fields, and that one choice caused three integrity bugs in under six months. First, a filename containing \n produced an ambiguous preimage. Second, branch_name was a hashed but mutable field, so branch rename produced commits that "fail verification on clone"; rename is still disabled. Third, Postgres timestamp truncation "quietly chang[ed]
- Mesa (MesaFS), from Mesa Systems, Inc. [beta] (<https://mesa.dev>) | where: Server or cloud. The default is the Mesa-hosted service (app.mesa.dev, api.mesa.dev/v1) on what Mesa calls a "custom storage backend" or "Mesa Distributed Storage" (<https://mesa.dev/blog/what-is-mesa,> <https://mesa.dev>).  | lesson: Even a purpose-built realtime versioned filesystem limits live sharing to one shared mutable head, the "room" of a single change. It then tells users to isolate each session on its own timeline (session/<id> bookmark, one change per prompt) and to merge explicitly. Concurrent edits to one file still end in conflict markers.

ParleyInn should design for the case Mesa avoids:

- Give every agent or session its own immutable, hash-chained append-only seg
- Relace Repos [shipped. Relace anno] (<https://docs.relace.ai/docs/repos/overview>) | where: Server/cloud. Relace hosts the repositories, which are reachable as git remotes at <https://api.relace.run/v1/repo/{REPO_>ID}.git and through REST on the same host (<https://docs.relace.ai/docs/repos/git-commands,> <https://d> | lesson: A vendor building 'git for agents' keeps git only as the interoperability wire for clone and push from ephemeral sandboxes. Everything agents actually need sits outside git, in a central server:
- per-sandbox tokens scoped to specific repos with a TTL
- a per-repo write lock (423 on contention)
- push webhooks
- an asynchronously rebuilt search index that is eventually consistent
- no session-to-code provenance at all (session traces are processe
- Grace (ScottArbeit/Grace), "Version Control for the AI Era" [beta. The README cal] (<https://github.com/ScottArbeit/Grace>) | where: On a server in the cloud. There is no git storage and no p2p.

- Azure Cosmos DB holds Orleans actor state through AddCosmosGrainStorage (<https://github.com/ScottArbeit/Grace/blob/f746b688e0d20fdfdc46eedb42bd8d301be61a34> | lesson: Copy Grace's join model, not its storage model.

What to copy: Grace links agent work to code through a work item. The work item owns typed Prompt and Summary artifacts and references exact repository versions. Those versions are keyed by GUID and carry BLAKE3 + SHA-256 as data fields, which is a clean, hash-agile way to point a session at code.

What to avoid: in Grace the agent session is the weakest-held object.

- Its lifecycle lives only in s
- GitAgent (open-gitagent/gitagent; formerly "gitclaw". Lyzr's agent runtime, distinct from the OpenGAP spec repo) [shipped. The repo is] (<https://github.com/open-gitagent/gitagent>) | where: Working-tree files committed to git in the agent repo itself.

- **Local-repo mode:** the agent repo is the target code repo. Work happens on a branch named `gitagent/session-<8 hex>`, which is pushed to the GitHub remot | lesson: Do not make history a set of mutable files in the code branch's working tree.

GitAgent shows what you get from that design: "git log your memory" comes for free. It also shows the costs:

- **Rewritable history:** the model rewrites MEMORY.md whole, so content can be silently dropped or rewritten, and it is archived only by line overflow. That breaks append-only and lossless guarantees.
- **No merging:** session branches plus a whole-file Markdow
- Backlog.md (MrLesk/Backlog.md) [shipped] (<https://github.com/MrLesk/Backlog.md>) | where: Working-tree files, committed with the code. The folder is project-local: backlog/, .backlog/, or a custom project-relative path set by backlog_directory in backlog.config.yml (README, ADVANCED-CONFIG.md). Git is optiona | lesson: Copy the read side, not the identity or merge model. Backlog.md shows that git can serve as a zero-infrastructure transport. It reads every recent branch and remote-tracking ref without a checkout, pins each tip to a commit SHA, indexes it with ls-tree and log, hydrates only what it needs, and caches by immutable commit. That gives agents in many worktrees and clones a shared view with no server.

Its weak points are the parts ParleyInn must design d

- Task Master (claude-task-master, npm task-master-ai), by Eyal Toledano and Ralph Khreish, now branded under Hamster [shipped. The repo is] (<https://github.com/eyaltoledano/claude-task-master>) | where: Storage spans the working tree, the home directory and an optional cloud.
- Working tree: all tags live in one file, .taskmaster/tasks/tasks.json. The code says "All tags are stored in this one file" (<https://github.com/> | lesson: Task Master shows both sides of putting agent state into git.

What breaks: one shared, mutable JSON file with sequential IDs breaks at the first concurrent branch. Its documented fix is to have the AI renumber IDs after a merge conflict. And when the high-churn per-session data (autopilot state and activity.jsonl) caused git conflicts and worktree clashes, they gave up on the working tree. That data moved to ~/.taskmaster/<sanitized-absolute-pat

- Claude Code agent teams [beta. Shipped as an ] (<https://code.claude.com/docs/en/agent-teams>) | where: Local files under the user's home directory, outside any repository. None of it is in git.
- Mailbox: ~/.claude/teams/{team-name}/inboxes/{agent-name}.json
- Team config: ~/.claude/teams/{team-name}/config.json
- Task li | lesson: Do not copy the mailbox's mutable-state design; do adopt its trust label.
- The mailbox is a mutable JSON array per recipient, with a 'read' flag serving as the delivery state.
- Most of the reported failures trace back to that design: lost messages under concurrent writes, re-delivery after compaction, leaks across /clear, and silent stalls at about 1,000 entries.
- ParleyInn should model coordination as append-only, immutable, hash-chained events w
- Terragon (Terragon Labs), open-source snapshot terragon-oss [archived. The hosted] (<https://github.com/terragon-labs/terragon-oss>) | where: Three hosted places plus a GitHub remote.
- PostgreSQL through the Drizzle ORM holds threads, chats and the message JSONB (<https://github.com/terragon-labs/terragon-oss/blob/83142a17f3970df3e14d1879234a603db6e4f615/AGENT> | lesson: Git worked well for moving code between sandboxes, but the session record lived only in a hosted service, and when that service died the record died with it.
Terragon kept three copies with different owners: native Claude JSONL in R2 (whole-file snapshots with an overwritten pointer), a lossy normalized JSONB log in Postgres, and code on per-task GitHub branches. The only link back from code to session was a PR-body URL. Since the 9 Feb 2026 shut
- Claude Code cross-session messaging (SendMessage / ListAgents, /list-agents) [shipped. Released in] (<https://code.claude.com/docs/en/cross-session-messaging>) | where: Same machine (macOS, Linux):
- Each session binds its own Unix domain socket inbox; native Windows uses a named pipe instead. Local messages never pass through Anthropic (<https://code.claude.com/docs/en/cross-session-mes> | lesson: Treat traffic from other agents as its own provenance class with an explicit, auditable policy, and keep it apart from the record.

Claude Code tags every peer message as "not the user". It runs messages through an accept/hold/refuse policy whose precedence lets repository-scoped config only tighten. It keeps same-machine transport on a uid-restricted Unix socket that rejects symlinks, and sends cross-machine traffic through an opt-in relay. It a

- Claude Code Projects (the redesigned Projects in Claude Code: one coordinator conversation plus parallel "threads") [beta] (<https://code.claude.com/docs/en/claude-projects>) | where: Server/cloud: Anthropic-hosted.

- Transcripts, memory, instructions and the Library live on Anthropic's servers. Each cloud thread runs in an isolated, Anthropic-managed VM, an Ubuntu 24.04 x86_64 sandbox. The VM pauses | lesson: Projects proves a clean split. Code concurrency goes through git: one branch and PR per agent, overlaps resolved "as a merge conflict just like any other PR", and a `Claude-Session: <url>` commit trailer pointing back to the run. Conversational state (transcripts, shared memory, instructions) stays in a separate store, and only a pointer crosses into git.

ParleyInn should copy the split, and the trailer idea in particular, but invert the trust model

- Claude Code background sessions and agent view (`claude agents`, `claude --bg`, `/bg`, `/fork`) [beta] (<https://code.claude.com/docs/en/agent-view>) | where: Storage is a mix of local files and a git working tree. There is no server or cloud copy for local sessions.

- Supervisor and job state: under the Claude Code config dir (`~/.claude`, or `CLAUDE_CONFIG_DIR`, which runs  | lesson: Anthropic's own parallel-agent design keeps the two kinds of history on separate stores, and ParleyInn should copy that split:
- Git (worktree, branch, commits, draft PR) carries code.
- Session history stays in a local, single-writer store.
- The two are joined only by pointers: a PR link or a commit trailer.

Concretely, ParleyInn should:

1. Key each record to repo identity plus the session UUID, not to the cwd or worktree path. The `.claude/worktrees/

- Conductor (Melty Labs) [shipped. The Mac app] (<https://www.conductor.build/>) | where: It is split by plane.
Local workspaces:
- Worktrees live under ~/conductor/workspaces/<repo>/<workspace>. This location became the default in 0.25.0, replacing an in-repo .conductor directory (<https://www.conductor.build> | lesson: Conductor is a well-used product that deliberately keeps agent history out of git:
- Code moves on a branch per worktree.
- Per-turn state lives in a private local git ref, which is the code-to-turn link.
- Handoffs go in an uncommitted .context directory.
- Anything shared across machines or teammates, live, needs a hosted Postgres-backed service. Its read path for agents is an auto-injected workspace token plus a SQL view over every org transcr
- Vibe Kanban (BloopAI/vibe-kanban) [shipped] (<https://github.com/BloopAI/vibe-kanban>) | where: Local DB plus files in the working tree. The SQLite file is 'db.v2.sqlite' in asset_dir(), which closes the discovery note's open question (B/crates/db/src/lib.rs#L77-L83). asset_dir() is ProjectDirs::from("ai","bloop"," | lesson: Treat the session-to-code link as part of the portable record, not as local mutable state, and never let it pass through model output. Vibe Kanban records before and after HEAD for every agent turn and repo, which is the right, cheap link. But it lives only in one machine's mutable SQLite (db.v2.sqlite, journal DELETE, rows updated in place and soft-dropped on restore). The git side gets only the model's own final message as the auto-commit text.
- Agor (preset-io/agor) [shipped. The project] (<https://github.com/preset-io/agor>) | where: The default is a local LibSQL/SQLite file at ~/.agor/agor.db with WAL mode, a 5 s busy_timeout and foreign keys on (<https://github.com/preset-io/agor/blob/main/packages/core/src/db/client.ts>). PostgreSQL is a server data | lesson: Keep the record as the only authority and treat the live channel as a hint. Then do the two things Agor does not do: keep the record immutable and keep it complete.
Agor shows that a team-wide, real-time view of many agents works with one authoritative store plus a best-effort fanout. When Redis drops, missed events are not replayed; clients simply refetch from the database. ParleyInn can copy this: its hash-chained log stays authoritative, and any t
- Xum (formerly Coder Mux, earlier "cmux") [shipped. Actively de] (<https://github.com/coder/xum> (<https://github.com/coder/mux> still resolves; docs at <https://xum.coder.com>)) | where: Everything lives in a local DB and files on the machine that runs the Xum backend, under ~/.xum. ~/.mux and ~/.cmux are symlinked aliases (<https://xum.coder.com/reference/mux-compatibility.md>).

Remote runtimes run tools | lesson: Treat Xum's ADR 0005 as an outside argument for ParleyInn's hash chain. The ADR concedes that its bounded "append receipt" (file identity, size and timestamps) cannot prove append-only history: "File growth alone is not proof of append-only history", same-size rewrites can go undetected, and "stronger detection requires ... verification of the entire prior prefix". A hash chain is exactly that verification.

Ideas worth copying from Xum:

1. Keep the

- CCManager (kbwo/ccmanager) [shipped. The project] (<https://github.com/kbwo/ccmanager>) | where: Local filesystem only.
- CCManager state: ~/.config/ccmanager/ (%APPDATA%\ccmanager on Windows), holding config.json, sessions.json and recent-projects.json.
- Logs: $XDG_STATE_HOME/ccmanager/ccmanager.log, else ~/.local | lesson: Don't carry context across worktrees by copying the record. CCManager's 'Session Data Copying' is a cp -r of ~/.claude/projects/<path>. It produces two directories holding the same sessionIds, with cwd and gitBranch fields that are wrong in the copy. It records no lineage, and it duplicates every secret in the transcript. Its 'restore' only re-runs the launch command.

ParleyInn should model a fork as a new append-only chain whose genesis event names

- NTM (Named Tmux Manager) [shipped] (<https://github.com/Dicklesworthstone/ntm>) | where: Local files plus a local SQLite DB. Nothing is held in git objects, git notes or custom refs. Most data is machine-wide, not project-local, which contradicts the discovery note.
Machine-wide paths:
- Audit logs: $XDG_DAT | lesson: Design the hash chain for many writers from the start. NTM shipped a SHA-256 hash-chained audit JSONL with one shared file per session. Concurrent ntm processes each cached the chain tip and appended the same sequence number with the same prev hash, which forked the chain. Verification then failed after any concurrent invocation, and real tampering looked the same as normal use. The fix was one chain per OS process (<session>-<pid>-<date>.jsonl).
- Nimbalyst (successor to Crystal) [shipped] (<https://github.com/Nimbalyst/nimbalyst>) | where: Mainly a local database. The docs give two different locations:
- Repo docs: ~/Library/Application Support/@nimbalyst/electron/pglite-db/ on macOS (packages/electron/DATABASE.md).
- User docs: ~/Library/Application Suppo | lesson: Copy the shape of Nimbalyst's transport but not its losses.

What to copy is the PersonalSessionRoom shape for ParleyInn's real-time sharing:

- one append-only stream per session, end-to-end encrypted
- the relay sees only (ULID, seq, ts, source, direction, ciphertext)
- duplicates removed by a content-derived ID
- the relay explicitly treated as a cache, never the record

What not to copy is that the record is lossy wherever it lives:

- Tool output
- GitHub Copilot cloud agent (formerly "Copilot coding agent") + Agent HQ / mission control (the Agents tab and panel) [shipped. The cloud a] (<https://docs.github.com/en/copilot/concepts/agents/coding-agent/about-coding-agent>) | where: Mostly server/cloud on GitHub.com. Git carries only the code outcome and a URL. The agent runs in an ephemeral GitHub Actions environment: GitHub-hosted, larger, or self-hosted runners (ephemeral ARC runners are recommen | lesson: GitHub, which owns both git and the agent, still keeps transcripts out of git. Git gets only the code, a signed commit and a one-line pointer back to the session. The transcript lives in an access-controlled store where entries are archived, never deleted. On the local side, an append-only per-session events.jsonl is the source of truth, and the SQLite index can be rebuilt from it with `/chronicle reindex`, which matches ParleyInn's I10. For ParleyInn: p
- GitHub Copilot CLI (local sessions, session sync, and `--cloud` cloud sandboxes) [The Copilot CLI itse] (<https://docs.github.com/en/copilot/concepts/agents/copilot-cli/about-copilot-cli>) | where: Three places:
- **Local files** under `~/.copilot` (overridable with `COPILOT_HOME`): `session-state/<session-id>/` and `session-store.db` (<https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-di> | lesson: Copilot keeps each session's append-only `events.jsonl` as the record and treats the SQLite session store as a derived index that `/chronicle reindex` rebuilds from the files. That supports ParleyInn's I10 split.

The useful part is how it avoids merge conflicts entirely:

- A session (one UUID directory) has exactly one writer process.
- Remote clients only inject commands into that process, and the first response wins.
- Cross-device continuation of
- OpenAI Codex cloud ("Codex Cloud", plus the older "Codex Cloud (Legacy)") [shipped. There are t] (<https://learn.chatgpt.com/docs/cloud>) | where: On OpenAI-managed servers, in the ChatGPT backend. The CLI calls endpoints such as `/backend-api/wham/tasks`, `/wham/tasks/{id}`, `/wham/tasks/{id}/turns/{turn}/sibling_turns` and `/wham/environments/by-repo/github/{owne | lesson: Codex Cloud sends only the code result (a unified diff) to git, through a PR or through `git apply` with a preflight conflict check. The session itself stays in a closed server store: prompts, the worklog and sibling attempts. The sandbox state that produced it disappears 7 days after last use, and the docs say "Saved state doesn't replace source control". Once the VM expires, nothing in the repository lets you recover or verify how the code was
- Factory Droids (Droid CLI, Factory App, Factory web at app.factory.ai) [shipped] (<https://docs.factory.com/droid-cli/settings>) | where: Local disk under ~/.factory, plus Factory's server/cloud database.

LOCAL. The source of truth is per-session JSONL files under `~/.factory/sessions/` (SDK source above). Other local paths:

- `~/.factory/settings.json` a | lesson: Factory never merges a session. Each session is one append-only JSONL file written by one machine. The cloud copy is a read-only mirror, and forking, compacting or rewinding creates a new session id while the old one stays loadable. The search index is a cache that `--reindex` rebuilds.

That pattern sidesteps multi-writer merge and fits ParleyInn's I10 (everything derived is rebuildable from the record). ParleyInn should copy it: one writer per log, a p

- Augment Cosmos (Sessions) [beta. Augment's May ] (<https://docs.augmentcode.com/cosmos/sessions-overview.md>) | where: Server/cloud. Augment's hosted control plane holds sessions, artifacts, events, the VFS and configuration. The web UI and CLI are clients of the same API (<https://docs.augmentcode.com/cli/cloud.md>). Compute runs either o | lesson: Copy the act-time link index, not the trust model. Cosmos records a session-to-code link at the moment it happens: the PR from the tool call that opened it, the branch from the VM's git context. It keeps that link outside git and makes it reverse-searchable ("which session opened PR 4821"). Its shared files also follow good append-only habits: immutable per-write versions, writer attribution, tombstones. ParleyInn can get the same through typed artif
- Coder Agents (successor to Coder Tasks), the Chats API and chatd inside coderd [shipped. The docs la] (<https://coder.com/docs/ai-coder/agents>) | where: Server/cloud, self-hosted: the PostgreSQL database of the Coder control plane (coderd), not the workspace. The docs say 'All chat state is stored in the Coder database, not in the workspace', so history survives when a w | lesson: Keep the durable record and the real-time channel separate. Treat notifications as lossy hints, never as the source of truth.

**How Coder does it:** chatd advances a monotonic per-chat snapshot_version inside each committed transition and publishes only that version. Notifications are post-commit and best-effort, and receivers 'must tolerate duplicates, drops, and reordering'. Each receiver keeps a watermark and re-reads the durable rows when it

- Kilo Code (Sessions & Sharing; Kilo CLI / VS Code / JetBrains / Cloud Agent) [shipped. Sessions an] (<https://kilo.ai/docs/collaborate/sessions-sharing>) | where: Local:
- SQLite database kilo.db, at ~/.local/share/kilo/kilo.db on Linux and macOS and %USERPROFILE%\.local\share\kilo\kilo.db on Windows; `kilo db path` prints the actual location (<https://kilo.ai/docs/code-with-ai/age> | lesson: Kilo's sync shows what to avoid as well as what to copy.

To avoid: the cloud copy is a mutable upsert-by-item_id projection with no history. It is turned on silently by signing in, the opt-out is an undocumented env var, and raw tool outputs and diffs go up with no redaction. That is the opposite of ParleyInn's I2, I4 and I6 contract.

Worth copying (both fit inside ParleyInn's core):

1. Link sessions to code only from evidence the session itself produc

- OpenCode session share (/share, /unshare, auto-share; opncd.ai share service) [shipped. Sharing wor] (<https://opencode.ai/docs/share/>) | where: Remote (current path): opncd.ai runs a SolidStart app named "Teams" (packages/enterprise) on Cloudflare, backed by an R2 bucket called EnterpriseStorage (<https://github.com/anomalyco/opencode/blob/1ddb0873aee50d209d1a8d7> | lesson: Treat OpenCode share as a counter-example on four points. (1) The share is a mutable, last-write-wins keyed snapshot, not a log. A receiver cannot check that what it sees is complete or unaltered, and a concurrent read-modify-write can silently drop data. (2) Transport runs inside the agent process on every bus event, a failed flush is only logged, and no redaction happens at the boundary. Secrets reach a public URL, as issue #48305 shows. (3) De
- goose (Agentic AI Foundation / Linux Foundation; formerly block/goose) [shipped] (<https://goose-docs.ai/docs/guides/sessions/session-management>) | where: One local SQLite database per user, in WAL mode. Paths are ~/.local/share/goose/sessions/sessions.db on Unix and %APPDATA%\Block\goose\data\sessions\sessions.db on Windows (<https://goose-docs.ai/docs/guides/logs>).

Befor | lesson: Do not mint record identity from a local counter. goose's YYYYMMDD_N session IDs are allocated with MAX+1 in the local database. As a result, the only way to move history between machines, JSON export then import, has to create a new session with a new ID. Provenance, dedupe and idempotent re-import are all lost, so two machines can never converge on one history.

For ParleyInn to move history over git or any other transport across worktrees, sandbox

- Vibe Kanban (BloopAI) [shipped, now sunsett] (<https://github.com/BloopAI/vibe-kanban>) | where: A local DB plus local flat files, with git worktrees for code. The SQLite file is db.v2.sqlite in the per-OS data directory, resolved through ProjectDirs('ai','bloop','vibe-kanban'): ~/.local/share/vibe-kanban on Linux,  | lesson: Make the per-turn code anchor a portable record, not a vendor DB or cloud. Vibe Kanban's best idea is cheap: before and after each agent turn it records the HEAD OID per repo. That single fact gives a robust session-to-code link plus exact restore-to-turn.

But Vibe Kanban keeps the link in a machine-local, mutable SQLite file. Its only shared path was a hosted Postgres/ElectricSQL tier, and that tier died with the company after about five months

- Tangled (AT Protocol forge: appview + knots + spindles + Bobbin; Knot 2 Rust implementation) [shipped. It is a pub] (<https://tangled.org>) | where: Data is split across four kinds of place:

1. Self-hostable knot servers hold the git repos. Knot 2 has no SQLite: 'git is the only database, along with a secret-key-file'. Its ACL data lives in custom refs (refs/cobs/*). | lesson: Knot 2 is a working example of 'git is the only database' for append-only, multi-party records. ParleyInn could copy its pattern if git ever carries session history.

What to copy:

- Model each session or log as a COB-like chain of commits under a reserved ref namespace (refs/cobs/<type>/<root-oid>). The server refuses client pushes and deletion there.
- Put the payload in a single canonical DAG-CBOR blob. Put type, author and signature in commit hea
- NIP-34 ("git stuff": git collaboration over Nostr), with its GRASP hosting protocol and the ngit / ngit-grasp / GitWorkshop reference stack [shipped. The spec is] (<https://github.com/nostr-protocol/nips/blob/master/34.md>) | where: Split across two kinds of infrastructure.

- **Events on Nostr relays.** The repo announcement lists its relays, and patches, PRs and issues SHOULD go to those relays (34.md).
- **Git objects on ordinary Git Smart HTTP s | lesson: Copy the split between state and data; do not copy the merge rule.

NIP-34/GRASP keeps bulky, content-addressed data on dumb, interchangeable git servers. Only a tiny signed state pointer (kind 30618, refs → commit ids) is authoritative. The storage host checks each push against that signed pointer in a pre-receive hook and holds it in "purgatory" until both halves arrive. So a host is a cache, not an authority, and hosts can be swapped without t

- ngit / GRASP / gitworkshop.dev (Git collaboration over Nostr, NIP-34) [shipped] (<https://github.com/DanConwayDev/ngit-cli>) | where: Three places plus a local cache. (1) Nostr relays hold the events. The relays named in the repository's announcement are authoritative for collaboration data. v3 fetches state and collaboration events only from those rep | lesson: Split a small signed head from the bulk data, and refuse to publish or serve the head until the data it names is present. ngit puts the authoritative ref state in small signed events and treats git servers as interchangeable caches of content-addressed objects. It broadcasts state only after a git server has accepted the objects, and a GRASP server holds incoming state in "purgatory" until the git data arrives (<https://ngit.dev/how-it-works;> http
- LangGraph checkpointers (langgraph-checkpoint / -sqlite / -postgres), plus the LangGraph Store and the LangSmith Agent Server that host them [shipped] (<https://docs.langchain.com/oss/python/langgraph/persistence>) | where: One of: process memory (InMemorySaver, lost on restart), a local SQLite file (SqliteSaver, aimed at experiments and local workflows), or a server database. The server options are PostgresSaver ('ideal for production', us | lesson: LangGraph's run of advisories (CVE-2025-64439, CVE-2026-28277, CVE-2026-27794, CVE-2026-48775) shows what happens when a session store rebuilds typed objects from stored bytes and trusts what it reads back. Anyone who can write to the store, or slip in a synced record, gets code execution on resume, even when the content never reaches a prompt. ParleyInn moves history between machines, sandboxes and git remotes, so I2's 'untrusted' boundary has to co
- Electric: Postgres Sync, Durable Streams and Electric Agents (electric.ax, formerly electric-sql.com) [beta. This covers th] (<https://electric.ax/>) | where: Server-side, self-hosted or in the hosted cloud that is winding down. (a) Entity and session streams live on a Durable Streams server. The agents server uses an external one if DURABLE_STREAMS_URL is set. Otherwise it st | lesson: Adopt Durable Streams' write contract as ParleyInn's model for moving session history, but not its trust model. Concretely: each ParleyInn writer (agent x worktree/sandbox) is a producer with a stable id, an epoch that bumps on restart, and a per-batch sequence number. A stale epoch is fenced off (403), a duplicate is accepted as idempotent success (204), and a gap is rejected (409). Offsets are opaque and lexicographically sortable, and catch-up ranges
- Cloudflare Agents SDK (Durable Objects SQLite state, State and Sessions capabilities) [shipped. The npm pac] (<https://developers.cloudflare.com/agents/api-reference/agents-api/>) | where: server/cloud. Data sits in the private embedded SQLite database of one Cloudflare Durable Object per agent instance. Queries run synchronously in the same thread as the object. Durability comes from the Storage Relay Ser | lesson: Copy the design split and the topology, not the platform. Cloudflare draws a line: "Sessions stays lossless; context shapes". Compaction is a read-time overlay over rows that are never deleted. Lossy caps live only on the read path, "where a cap can change without having destroyed the stored bytes". Media moves to a content-addressed `sha256` store before persist (design/context.md, design/sessions.md). That matches ParleyInn's I10, where projections
- Tessera (formerly "Trillian Tessera"), transparency-dev/tessera [shipped. It is gener] (<https://github.com/transparency-dev/tessera>) | where: You pick one storage driver per log:
- GCP: Spanner for sequencing, GCS for tiles.
- AWS: a MySQL-compatible DB for sequencing, S3 or any S3-compatible store for tiles.
- POSIX: a plain local or distributed filesystem.
A | lesson: Use Tessera's on-disk contract, not its library.
- Format and code: write ParleyInn's record as a C2SP tlog-tiles log. That means RFC 6962 leaves, 256-entry bundles, files that never change once written, temp+fsync+rename plus a flock for writers, and a signed-note checkpoint. Build it from the network-free subset: transparency-dev/merkle, formats/log and x/mod/sumdb/note, which are about 3 modules.
- Why not the library: importing `tessera` or `stor
- golang.org/x/mod/sumdb/tlog (tamper-evident log library behind the Go checksum database, sum.golang.org) [shipped. It is maint] (<https://pkg.go.dev/golang.org/x/mod/sumdb/tlog>) | where: Wherever the caller puts it. The library talks to storage only through interfaces:
- `HashReader.ReadHashes([]int64)`, for hashes addressed by a dense "stored hash index" (`StoredHashIndex(level, n)`, Crosby and Wallach  | lesson: Adopt tlog's Merkle layer next to the existing hash chain, not in place of it.

The problem it solves: a linear chain needs O(N) work to show that an old event is still in the current head, or that a new head extends an old one.

How it would fit:

- Compute `RecordHash` over each canonical event's bytes.
- Append the output of `StoredHashes` to a sidecar hash file indexed by `StoredHashIndex`. The file is fully rebuildable from the record, so I10
- C2SP transparency-log spec family: tlog-tiles, tlog-checkpoint, signed-note, tlog-witness (plus tlog-cosignature, tlog-proof, tlog-policy, tlog-mirror and https-bastion, which go with them) [shipped. Most of the] (<https://c2sp.org/tlog-tiles>) | where: Static files under one or more URL prefixes, served from any HTTP file host, object store, CDN or POSIX directory (<https://c2sp.org/tlog-tiles>):
- `<prefix>/checkpoint`: mutable, served as text/plain.
- `<prefix>/tile/<L | lesson: Separate commitment from content, and never merge logs.

Make every ParleyInn writer (each agent, worktree, machine or sandbox) a single-writer log with its own origin and key. It writes C2SP-format checkpoints and tiles locally, inside the core, with no network access, so I4 holds. The Merkle leaves are commitments to events (Sigsum-style H(H(event)), or hashes of redacted or encrypted blobs), never raw transcripts. Raw transcripts would also break

- torchwood (filippo.io/torchwood), including litewitness, witnessctl, litebastion and spicy [shipped] (<https://pkg.go.dev/filippo.io/torchwood>) | where: It depends on the component.
- Library: remote HTTP via TileFetcher, a local fs.FS via TileFS (optionally with gzip-compressed data tiles), or zip archives via TileArchiveFS (000.zip, 001.zip, ...). PermanentCache writes | lesson: Use the C2SP tlog formats for ParleyInn's record, but do not import torchwood's root package. I made a minimal local copy of the approach and checked it with go list.
- **Format.** Store each session or machine log as signed-note checkpoints plus immutable tlog-tiles. Full tiles and entry bundles never change; only one small checkpoint file does. Any dumb carrier can move them without merging: a git branch, an object store, rsync or a zip.
- **What i
- immudb (Codenotary) [shipped. Latest stab] (<https://github.com/codenotary/immudb>) | where: A local data directory of append-only files. Per database, the store creates:
- a transaction log ("tx")
- a commit log ("commit")
- N value logs ("val_0".."val_N")
- the appendable hash tree in "aht/"
- the B-tree index | lesson: Commit to the hash of each payload, not the payload, and keep payloads in a separate value log. Wrap the hash chain in an append-only Merkle tree.

How immudb does it: an entry digest is H(metadata, key, SHA-256(value)), and transactions chain through Alh = H(TxID, prevAlh, innerHash). Retention can therefore physically delete value-log bytes while every proof still verifies. The Merkle tree over the Alh values gives logarithmic inclusion and con

- Automerge (core CRDT library) and automerge-repo (storage/network/sync layer) [shipped] (<https://automerge.org/>) | where: Local-first: every peer keeps a full replica in a pluggable StorageAdapter. The shipped adapters are IndexedDB in the browser and a NodeFS directory on Node (<https://github.com/automerge/automerge-repo>). The design targe | lesson: Borrow automerge-repo's storage keying, not the library.

What to borrow:

- Store each increment under its own content hash and each compaction under the heads it covers. A compactor deletes only the increments it loaded itself (<https://automerge.org/docs/reference/under-the-hood/storage/>).
- This lets several worktrees, machines or sandboxes write to one simple shared store with no locks and no lost data: a directory, a bucket, or a synced folde
- Autobase (holepunchto/autobase) [shipped] (<https://github.com/holepunchto/autobase>) | where: A local DB on each peer, replicated peer to peer. Every writer core, the system core and every view is a Hypercore held in a Corestore. Corestore 7 and Hypercore 11 persist through hypercore-storage, which is built on Ro | lesson: Take the shape, not the stack. Keep one single-writer, signed, hash-chained log per agent or sandbox as the only source of truth. Each entry carries a clock of (writerKey, length) for the other logs it has seen. The shared record is then a deterministic, rebuildable projection: a linearizer that reads no clock or randomness, which is exactly I10. It is not a jointly written log, so git-style merge conflicts never arise.

Two cautions from Autobas

- Hypercore (Holepunch / Pear stack) [shipped] (<https://github.com/holepunchto/hypercore>) | where: On local disk, then copied peer-to-peer.
- **Local storage:** Hypercore 11 binds to hypercore-storage, "The storage engine for Hypercore. Built on RocksDB". Cores are keyed by discovery key, and "random-access-storage is | lesson: Keep verification and transport apart, and hash the ciphertext. Hypercore's core library opens no sockets. It produces and checks Merkle proofs over signed (manifest hash, tree hash, length, fork) checkpoints and hands peers a plain duplex stream. Discovery and networking live in a separate module, Hyperswarm. Blocks are encrypted before they are hashed, so a mirror without the key can still verify and serve them.

ParleyInn can copy this design and

- iroh-docs (n0-computer) [shipped. The crate i] (<https://github.com/n0-computer/iroh-docs>) | where: A local embedded database. The persistent store is redb, which keeps all replicas in a single file; an in-memory store is the alternative (<https://github.com/n0-computer/iroh-docs/blob/main/README.md>). Content blobs sit  | lesson: Copy the shape of iroh-docs but not its merge rule.

What to copy:

- Small, doubly signed metadata entries that point to content by BLAKE3 hash.
- Reconcile those entries by fingerprinted range set union.
- Fetch content lazily under a receiver-side download policy. That fits ParleyInn's pull-only recall (I2) and keeps metadata sync cheap across machines and sandboxes.

What not to copy: iroh-docs is mutable last-writer-wins. Writer-declared wall-clo

- iroh (n0-computer / number0), with the iroh-blobs, iroh-gossip and iroh-docs protocols [shipped. Core iroh 1] (<https://www.iroh.computer/>) | where: Local DB plus files on each peer, replicated peer to peer. No server holds the data.
- iroh-docs: persists "the whole store with all replicas to a single file" in redb, an embedded key-value store. In-memory mode is also | lesson: iroh-docs models multi-writer replication well, but ParleyInn should borrow the model and not adopt the transport in-process.

What to borrow: the reconciliation model. Give each writer its own signing key, sign every entry, store only BLAKE3 hashes in the index and keep content-addressed blobs separate, and replicate with range-based set reconciliation. That fits ParleyInn's per-agent, hash-chained, append-only logs especially well. If keys are (agent k

- Nostr NIP-34 ("git stuff") with ngit / git-remote-nostr and GRASP servers [shipped. Spec: NIP-3] (<https://github.com/nostr-protocol/nips/blob/master/34.md>) | where: Next to git, not inside it.
- Signed events live on Nostr relays: independently run WebSocket servers. Each repository names its relays in the `relays` tag (<https://github.com/nostr-protocol/nips/blob/master/34.md>).
- Gi | lesson: The main lesson is to split small, signed, content-addressed records from bulk data, and key every record to the repository's earliest-unique-commit id and to specific commit ids, the way NIP-34's `r` and `euc` tags do. That way, records for the same project group across forks, worktrees and machines without living inside git, and git servers stay interchangeable caches. There are three caveats:

1. Do not copy NIP-34's replaceable, last-writer-wi

- AT Protocol repositories (Merkle Search Tree repo format v3, with Sync v1.1 firehose) [shipped. Repo format] (<https://atproto.com/specs/repository>) | where: On servers. The authoritative copy sits on the account's Personal Data Server (PDS), which the DID document names under the service entry with id ending "#atproto_pds" (<https://atproto.com/specs/did,> <https://atproto.com/> | lesson: Borrow atproto's sync protocol for live sharing, but not its storage model.

What to copy:

1. **Single-writer streams.** Give each agent or machine its own single-writer, signed, monotonically revisioned stream. Do no merging. Treat the combined multi-agent view as a derived projection that can be rebuilt (I10), the way AppViews assemble many single-writer repos.
2. **Self-checking events.** Every streamed event should carry (rev, prev head hash,

- go-ds-crdt (Merkle-CRDT replicated go-datastore) [shipped. The last ta] (<https://github.com/ipfs/go-ds-crdt>) | where: Local database plus a content-addressed block store, replicated peer to peer.
- **Merged state** (set, heads, markers) lives in a caller-supplied thread-safe go-datastore. The README recommends go-ds-pebble, which replac | lesson: The design worth borrowing is the three-way split, not the library itself:

1. **Merge core:** an append-only, content-addressed DAG of deltas whose hash links act as a causal clock (a Merkle-Clock). It needs no wall clock or randomness, which fits I10.
2. **Broadcaster:** announces only head hashes.
3. **DAG syncer:** fetches missing blocks by hash and checks each against it.

ParleyInn can keep the merge core offline (I4) and put announce and fetch

- NATS JetStream (persistence layer of nats-server) [shipped. The latest ] (<https://docs.nats.io/nats-concepts/jetstream>) | where: Server side, on the nats-server host. A stream uses either file storage (the default, under the configured `store_dir`) or memory storage (<https://docs.nats.io/nats-concepts/jetstream/streams,> <https://docs.nats.io/runnin> | lesson: JetStream's per-subject optimistic append fits ParleyInn as an optional relay protocol run in a separate process. The transport should only move ParleyInn's own self-verifying chain, never define it.

**How the protocol would work:**

- Publish each session's events on their own subject.
- Set Nats-Msg-Id to the event hash, so a retry is idempotent.
- Set Nats-Expected-Last-Subject-Sequence to the last sequence the relay acknowledged. This gives one write
- Litestream [shipped. It is activ] (<https://litestream.io/>) | where: Server or cloud object storage, or a local or remote filesystem. LTX files are first staged in a hidden directory next to the database (for example /var/lib/.db-litestream for /var/lib/db). They are then uploaded to a si | lesson: The lesson is Litestream's shape. Litestream keeps the network outside the database: SQLite writes locally and a separate sidecar process ships immutable segments. ParleyInn can do the same. The core writes immutable, range-named segment files locally (`<stream>/<firstSeq>-<lastSeq>.seg`) and never touches the network, so I4 holds. A separate, optional shipper uploads them to any dumb store: S3 or Forgejo/GitHub release assets, a git ref, SFTP or NAT
- Turso Database (Rust SQLite rewrite) and libSQL (SQLite fork, predecessor), plus Turso Sync and AgentFS built on them [shipped. Turso Datab] (<https://github.com/tursodatabase/turso>) | where: A local database file in the in-process engine: SQLite format, plus a -wal file, or a .db-log file in MVCC mode. With Turso Sync it can also be replicated to Turso Cloud, the managed service; the docs give no open-source | lesson: Turso Sync shows what goes wrong when a mutable-row sync layer carries an append-only log. Push replays CDC rows as INSERT ... ON CONFLICT(pk) DO UPDATE under "last push wins". If two worktrees or sandboxes write the same primary key, for example an auto-increment rowid or a per-session sequence number, the later push silently overwrites the earlier event. That breaks I6 (no silent failures) and the hash chain, with no error raised.

If ParleyInn eve

- LiveStore (livestorejs/livestore) [beta] (<https://livestore.dev/>) | where: Every client keeps a local copy, and a central sync backend keeps the authoritative one.

On the client:

- Each client session (for example a browser tab) holds an in-memory SQLite DB.
- One elected leader thread per cli | lesson: LiveStore supports ParleyInn's I10 approach: materializers are pure, deterministic and transactional, IDs are carried in the event payload (no randomUUID() inside a materializer), and read models are rebuilt instead of migrated. It also shows that ParleyInn's shared transport can be a very small contract: pull(cursor, live), push(batch) and ping, with the backend accepting a push only if the batch chains onto its head. That contract lets LiveStore swap C
