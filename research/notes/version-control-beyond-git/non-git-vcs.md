# Version control beyond git: systems that replace git's storage and model

Research date: 2026-10-02. Primary sources fetched where possible; aggregator claims are flagged. "Unverified" means stated from background knowledge but not confirmed by a page fetched in this session.

## Q1. Per-system profile: data model, storage/sync, merging, real-time, attribution, git interop, status, licence, language, scale

### Takeaway

Of the systems asked about, only Pijul, Darcs, Fossil, Grace, Oak, Lore (Epic), Diversion, Unison, Lix, Perforce P4 and Unity VCS actually own their storage. Zed's Delta/DeltaDB is the headline "agent-era" system, but today it **extends git** rather than replacing it; native git storage inside DeltaDB is only a roadmap item. Jujutsu's native backend is officially a proof of concept. The real-time CRDT systems (Delta, Patchwork/Upwelling, Lix) are the only ones that record per-edit history, and Delta and Patchwork are the only ones that link edits to agent prompts or conversations as part of the model itself.

### Cited Findings

#### Zed Delta / DeltaDB (Zed Industries) — public beta, git-extending (not yet a git replacement)

- Timeline: "Software Is Made Between Commits" (DeltaDB, 2026-06-11); "Introducing Delta" (2026-08-12); "Xanadu Was Waiting for Agents" (2026-09-01); "Replace PRs with Delta – Now in Public Beta" (2026-09-16) — [Zed blog index](https://zed.dev/blog)
- Data model: "Where Git captures a snapshot at each commit, DeltaDB captures every operation in between and gives each one a stable identity." Uses CRDT "conflict-free replicated worktrees" so several humans and agents can edit at once across machines — [Zed: Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- Stable per-operation identity lets you "point to the code at any moment in its evolution, even as it keeps changing" — [Zed: Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- Agent/prompt linkage: "A message and the edit it produced are recorded side by side, so neither drifts away from the other." Agents can "convene the prior agents that worked on it and ask why it's written the way it is" — [Zed: Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- Reason for the change: "The conversation that generates the code is becoming the true source of our software... Git, organized around discrete commits, was never designed to support this." — [Zed: Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- Relationship to git: "Git and CI stay for what they're good at: running checks and connecting you to the rest of the world" — [Zed: Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb). "DeltaDB works with the git repository you already have. Every edit and conversation is captured *between* your commits." — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- The beta post describes DeltaDB as something that "extends Git's content-based versioning with incremental versions based on deltas" and "records edits between commits alongside messages from humans and agents"; the roadmap lists "Git storage in DeltaDB" and "content-based builds" — [Zed: Delta public beta](https://zed.dev/blog/delta-public-beta)
- Sync: "DeltaDB replicates the conversation and the worktree together, in real time, for everyone in a thread". Each participant has a local copy kept in sync in real time; work can move to a cloud runner. It connects to third-party agent harnesses, "starting with Claude Code" — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- Delta replaces PRs with threads and review subthreads. It runs on macOS, Linux, Windows and the web, and is free during the beta. Zed's own repository stays on GitHub. "Replacing pull requests is our first step toward replacing GitHub." — [Zed: Delta public beta](https://zed.dev/blog/delta-public-beta); [Zed blog index](https://zed.dev/blog)
- Licence and open-source status are not stated in any of the four posts — [Zed: Delta public beta](https://zed.dev/blog/delta-public-beta)

#### Pijul — patch theory, Rust, long-running 1.0 beta

- Model: "Pijul stores changes (or patches), whereas Git deals only with snapshots." "For any two changes A and B, either A and B can be applied in any order, or A depends on B, or B depends on A." It claims to be "the first distributed version control system to be based on a sound mathematical theory of changes" — [Pijul manual: Why Pijul](https://pijul.org/manual/why_pijul.html)
- Merge/conflicts: merges are associative, so there is no line shuffling like git's 3-way merge. Conflicts are first-class: Pijul applies "edits from both sides of a conflict without resolving the conflict. This guarantees no information ever gets lost". "Conflicting changes always commute in Pijul and never commute in Darcs" — [Pijul manual: Why Pijul](https://pijul.org/manual/why_pijul.html)
- Conflict resolutions apply consistently wherever the same two changes meet; it supports partial clones; it is written in Rust (`cargo install`); hosting is on the Nest (nest.pijul.com); copyright 2016–2026 Meunier & Becker — [pijul.org](https://pijul.org/)
- Status: roughly eight 1.0.0-beta releases since late May 2026. A fuzzer was added; the maintainer says "the recurring bugs around unrecord are gone, all the other commands have been stable for years." The roadmap exists but "can't be shared publicly" (2026-08-02) — [Pijul discourse: roadmap thread](https://discourse.pijul.org/t/is-there-a-roadmap-for-pijul-after-the-recent-beta-releases/1411)
- An aggregator summary names 1.0.0-beta.22 on 2026-08-23 and says the network protocol must change for large binaries (not confirmed on a primary page) — [Pijul discourse: is this project still active](https://discourse.pijul.org/t/is-this-project-still-active-yes-it-is/451)
- Licence: GPL-2.0 (unverified; the front page shows only copyright).

#### Darcs — original patch-theory DVCS, Haskell, maintenance mode

- Latest release 2.18.5 (January 2025). It is written in Haskell, focuses on "changes rather than snapshots", and "does not require a central server, and works perfectly in offline mode" — [darcs.net](https://darcs.net/)
- Darcs and Pijul differ on conflicts: conflicting changes "never commute in Darcs" — [Pijul manual](https://pijul.org/manual/why_pijul.html)
- Licence GPL-2 and git import/export via `darcs convert` are unverified.

#### Fossil — SQLite-backed DVCS with forge built in

- "Fossil stores its objects in a SQLite database file which provides ACID transactions and a high-level query language". It bundles a wiki, tickets, technotes, a forum and chat ("GitHub-in-a-box"), and ships as a single ~5 MB static binary — [Fossil vs Git](https://fossil-scm.org/home/doc/trunk/www/fossil-v-git.wiki)
- Append-only ethos: "There is no rebase in Fossil, on purpose"; "Fossil records history as it actually happened". Amendments add records rather than removing data. Autosync pushes each commit to the parent repository at once. It uses SHA-3 hashes — [Fossil vs Git](https://fossil-scm.org/home/doc/trunk/www/fossil-v-git.wiki)
- The git interop pages (import/export, GitHub mirror) returned HTTP 503 during this session — [fossil inout.wiki](https://fossil-scm.org/home/doc/trunk/www/inout.wiki), [mirrortogithub.md](https://fossil-scm.org/home/doc/trunk/www/mirrortogithub.md). Background knowledge, unverified: `fossil import --git`, `fossil export --git` and `fossil git export` mirroring; written in C; BSD-2-Clause; it hosts SQLite itself.

#### Jujutsu (jj) native backend — proof of concept only

- "Jujutsu has two backends for storing commits. One of them uses a regular Git repo" — [jj docs: Git compatibility](http://docs.jj-vcs.dev/latest/git-compatibility/)
- "The `SimpleBackend` is just a proof of concept. It stores objects addressed by their hash, with one file per object." The `GitBackend` uses gitoxide. The design principle is "it should be easy to change where data is stored". There are separate pluggable backends for commits, operations, op-heads, indexes and working copies; op-heads are designed for lock-free concurrent access — [jj docs: architecture](https://docs.jj-vcs.dev/latest/technical/architecture/)
- An aggregator says a real native backend (ACLs, chunking, cloud storage, FUSE checkouts) is "a significant amount of work and not going to realistically happen soon" — search summary citing [LWN](https://lwn.net/Articles/958900/); the fetched LWN comment did not contain this sentence, so it is unconfirmed.
- East River Source Control is commercialising jj with stacked branches and conflict versioning — [Pablo Santos: Version control second coming](https://psantosl.github.io/posts/version-control-second-coming/)
- Google's internal Piper/CitC backend is not documented in the public jj docs — [jj docs: architecture](https://docs.jj-vcs.dev/latest/technical/architecture/)

#### Grace (Scott Arbeit) — cloud-native, event-sourced, alpha

- MIT licence; F# on ASP.NET Core and Orleans virtual actors; Azure Blob Storage for files and Cosmos DB for actor state; multitenant. It is "event-sourced", storing "every modification to every entity as an event" — [Grace GitHub](https://github.com/ScottArbeit/Grace)
- `grace watch` auto-saves "generally within a second of the file being saved". It "doesn't do merges, it does promotions", through a promotion queue with "automatic promotion conflict resolution according to rules and confidence levels you set" — [Grace GitHub](https://github.com/ScottArbeit/Grace)
- Rebranded "version control for the AI Era": "kindness to humans, and velocity for agents". It adds agent review primitives and a `grace agent bootstrap` command that captures agentic work context — [Grace GitHub](https://github.com/ScottArbeit/Grace)
- Status: alpha with breaking changes, "not ready for or intended for production usage", about 1,925 commits — [Grace GitHub](https://github.com/ScottArbeit/Grace)
- Git interop: git and Grace can run side by side in one directory — [Grace GitHub](https://github.com/ScottArbeit/Grace). The design doc plans a one-time import from git and snapshot export to a git bundle; "two-way synchronization between Grace and Git is an explicit non-goal" — [Grace: Design and Motivations](https://github.com/ScottArbeit/Grace/blob/main/docs/Design%20and%20Motivations.md)
- Note: the design doc mentions Dapr, while the current README says Orleans. The architecture appears to have moved from Dapr to Orleans — compare [Design doc](https://github.com/ScottArbeit/Grace/blob/main/docs/Design%20and%20Motivations.md) with [README](https://github.com/ScottArbeit/Grace)

#### Oak (oak.space) — agent-native VCS, Rust, public beta

- Content-addressed storage with FastCDC content-defined chunking, so deduplication works across versions and across the repository. It uses manifests and lazy hydration ("lazy mounts" via FSKit, FUSE and ProjFS). Each task gets its own mount and branch, avoiding shared `.git` corruption — [oak.space](https://oak.space/)
- Workflow: `main` exists only on the server, and direct pushes to it are refused. Intermediate commits need no message; the branch description becomes the squash-commit message. Claims up to "95% lower p50 latency" than git on snapshots, status and large-binary diffs; cold init is slower — [oak.space](https://oak.space/)
- Git interop: `oak export <dest>` replays full branch history into git, keeping author, email and timestamp. A read-only Smart-HTTP endpoint lets stock `git clone` fetch `main` — [oak.space](https://oak.space/)
- "Oak trains no models on user code and makes no AI calls on your behalf". Built by Zach Geier and Adam Morse — [oak.space](https://oak.space/)
- Apache-2.0; Rust (`oakvcs-core`, `oakvcs-cli`); v0.105.0 public beta; macOS, Linux and Windows; 81 stars; code written "almost entirely using AI with human oversight" — [GitHub oakdotspace/oak](https://github.com/oakdotspace/oak)
- Conflict: an aggregator says v0.99.0 had no Windows build and BLAKE3 hashing — [AgentConn](https://agentconn.com/blog/source-control-multi-agent-repos/). The primary repository lists Windows x86_64 — [GitHub](https://github.com/oakdotspace/oak). The Show HN post reached the HN front page — [HN](https://news.ycombinator.com/item?id=48631726)

#### Lix (Opral) — embeddable, SQL-queryable change control, alpha

- Versions files and SQL tables in one repository. "Tracked writes commit automatically". The `lix_change` table records author, timestamp, schema key and row snapshots, and can be queried with SQL. Plugins give structured diffs and merges (Markdown, CSV) by mapping paragraphs, cells and properties to rows. Pluggable storage: memory, filesystem, OPFS, S3. Changes carry an `account_id`, which can tell agents from humans — [lix.dev](https://lix.dev/)
- "Why not Git?": git versions file bytes, while Lix versions queryable rows. No git interop is mentioned — [lix.dev](https://lix.dev/)
- MIT; the JS SDK is primary and a Rust SDK exists; "Python and Go SDKs are planned". "Lix is in alpha", with more than 350k weekly npm downloads — [lix.dev](https://lix.dev/)
- Positioned as "version control system for AI agents" and as an embeddable library rather than a git replacement — [ecosyste.ms](https://awesome.ecosyste.ms/projects/github.com%2Fopral%2Flix); [feedbagel summary](https://feedbagel.com/post/lix-an-embeddable-version-control-system-for-semantic-change-tracking). Opral put all resources into Lix until v1 — [Opral substack](https://opral.substack.com/p/focus-shift-from-inlang-to-lix)

#### Diversion — cloud-native, proprietary, production

- Centralized: "no need to clone the entire repository history into every local working copy". It owns "the storage model, the protocol, the user experience, the collaboration layer". GitHub mirroring gives "bi-directional sync". A Claude Code plugin captures "the 'why' behind every AI-generated code change" — [Diversion blog: agentic world](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world)
- Scale claims: "50M+ files", "100s of TB", "100+ commits/min", and "thousands of agents and developers branching, committing, and merging simultaneously". It offers real-time sync and partial or shallow checkouts. Proprietary with usage-based pricing; claims "up to 70% lower TCO vs. Perforce" — [diversion.dev](https://www.diversion.dev/)
- An aggregator says prompt provenance is "baked into commit model" and that Epic recommends it for Unreal (unconfirmed by primary source) — [AgentConn](https://agentconn.com/blog/source-control-multi-agent-repos/)

#### Ink & Switch Patchwork / Upwelling — Automerge research prototypes

- Patchwork (2024–2026) is part of "universal version control" across media. Sibling projects: Upwelling (2023), Jacquard (2024), Backstitch (2024–2026, Godot students) — [Ink & Switch: universal version control](https://www.inkandswitch.com/universal-version-control/)
- Built on Automerge, a CRDT JSON store and sync library — [search summary of Patchwork notebook](https://www.inkandswitch.com/patchwork/notebook/2024-version-control/)
- AI bots edit on branches you can "partially or completely merge". "The history timeline also shows which edits came from the bot". Bots are "simply text prompts" that are versioned and branchable like documents (2024-03-19) — [Patchwork notebook 07: AI bots](https://www.inkandswitch.com/patchwork/notebook/2024-version-control/07/)
- The 2026 notebook entries cover a local-first task framework, account history, etc.; the latest is 2026-05-12 — [Patchwork notebook](https://www.inkandswitch.com/patchwork/notebook/)
- Upwelling: Automerge, with layers and drafts for "creative privacy". Floating drafts rebase automatically when another merges. It keeps full keystroke history with author attribution. Research prototype (March 2023), TypeScript, ProseMirror, Node — [Upwelling](https://www.inkandswitch.com/upwelling/)

#### Unison — content-addressed code definitions

- Each definition is identified by "a hash of its syntax tree" (512-bit SHA3). Names are "separately stored metadata", so renames do not break anything. "The Unison codebase is a proper database which knows the type of everything it stores". "A Unison codebase is never in a broken state, even midway through a refactoring" — [Unison: the big idea](https://www.unison-lang.org/docs/the-big-idea/)
- It has projects, branches (including contributor branches), `merge` and `merge.preview`, conflicts resolved in the editor until the file typechecks, and `push` to Unison Share. The docs do not mention git sync — [Unison: project workflows](https://www.unison-lang.org/docs/tooling/project-workflows/)
- It is code-only: it versions Unison definitions, not arbitrary files (inference from the docs above).

#### Perforce P4 (Helix Core) — centralized, proprietary, enterprise scale

- "Git was built for a single maintainer. Perforce P4 was built for the shape of work agentic development takes". The server shows "every file currently open for edit, by each user or agent... with the intent they declared". It offers file locking and file-level permissions, and can "Attach durable, queryable metadata, including prompt context and agent identity, to every version of every file". P4 MCP server. Claims to handle "terabyte-sized repos and thousands of contributors" — [Perforce: Why P4 for AI-powered development](https://www.perforce.com/node/4402)
- MCP capabilities across the portfolio launched in January 2026 — [search summary](https://www.perforce.com/ja/node/4402); a CEDEC 2026 talk covered P4 and AI — [classmethod](https://dev.classmethod.jp/en/articles/cedec-2026-perforce-p4-ai/)

#### Unity Version Control (formerly Plastic SCM) — proprietary, game-focused

- Successor to Plastic SCM; "distributed"; Gluon workflow for artists; code reviews; cloud or on-prem; proprietary — [Unity docs](https://docs.unity.com/en-us/unity-version-control)
- Plastic was acquired by Unity in 2020. Its founder, Pablo Santos, now works on Cursor's Origin forge, which is git-based — [Pablo Santos blog, 2026-09-01](https://psantosl.github.io/posts/version-control-second-coming/)
- No agent-specific features were found. Background knowledge, unverified: C# implementation, semantic merge, GitSync bridge.

#### Other serious non-git systems active in 2025–2026

- **Epic Games Lore**: MIT, © 2026 Epic Games, Rust. Content-addressed chunks, Merkle trees and an "immutable revision chain with cryptographic integrity". Centralized service with caching, on-demand hydration and sparse workspaces. Pre-1.0. APIs for C/C++, C#, Rust, **Go**, Python and JS. Built-in VCS of UEFN — [GitHub EpicGames/lore](https://github.com/EpicGames/lore); [Phoronix](https://www.phoronix.com/news/Epic-Games-Lore-VCS)
- **Oxen.ai** (large AI dataset versioning, non-git) — [Pablo Santos](https://psantosl.github.io/posts/version-control-second-coming/)
- **Layered on git, out of scope but often confused**: Atlas (Tauri/Rust; stores session prompts, tool calls and reasoning in `.atlas/sessions.db` linked to commits by patch-id; Apache-2.0) — [AgentConn](https://agentconn.com/blog/source-control-multi-agent-repos/); Entire (Thomas Dohmke; provenance of agent and prompt on git), Pierre/code.storage, Cursor Origin, GitButler — [Pablo Santos](https://psantosl.github.io/posts/version-control-second-coming/)

### Inferences

- Status buckets as of Oct 2026. **Production**: Fossil, Darcs (maintenance), Perforce P4, Unity VCS, Diversion. **Beta**: Pijul (1.0 beta), Oak (0.105 beta), Delta (public beta, git-backed). **Alpha or pre-1.0**: Grace, Lix, Lore. **Research prototype**: Patchwork, Upwelling. **Proof of concept**: jj native backend. None of the requested systems was found abandoned or archived.
- Only Delta (CRDT ops) and Patchwork/Upwelling (Automerge) offer real-time co-editing. Grace's `watch` auto-save and Diversion's real-time sync come close but are not CRDT merges.
- Go: none is written in Go. Lore ships a Go API, and Lix plans a Go SDK.

### Gaps

- Delta/DeltaDB licence, open-source status, implementation language and on-disk format are not published. Whether DeltaDB will drop git as storage ("Git storage in DeltaDB") is not explained.
- Fossil's git interop pages were down (503). Licences for Fossil, Darcs and Pijul are from memory.
- Lore's merge and conflict model and its git import are not documented on the README.
- The "Oak" in oakvcs.com and the one in oak.space appear to be the same product, but this was not confirmed.
- Diversion's internal data model is not public.

## Q2. Stated reasons for leaving git; claims for many concurrent writers / AI agents

### Takeaway

The reasons fall into four groups: (1) **granularity and provenance** — git sees only commits, while agents need per-edit history tied to the prompt or conversation (Zed, Patchwork, Perforce, Diversion); (2) **concurrency and scale** — clone cost, worktree and index-lock contention, large binaries, thousands of agent branches (Oak, Diversion, Lore, P4); (3) **merge soundness** — 3-way merge heuristics versus patch theory or CRDTs (Pijul, Darcs, Delta); (4) **UX and constraints from 2005** (Grace, Ink & Switch).

### Cited Findings

- Zed: git, "organized around discrete commits, was never designed to support" conversation-driven software — [Zed](https://zed.dev/blog/introducing-deltadb)
- Oak: clone overhead, `git worktree` fragility from a shared `.git`, agents burning tokens on throwaway commit messages, and LFS re-uploading whole files — [oak.space](https://oak.space/). An aggregator adds the shared `.git/index.lock` contention — [AgentConn](https://agentconn.com/blog/source-control-multi-agent-repos/)
- Diversion: monorepos and large files, many working copies, intent metadata, workarounds such as submodules and LFS — [Diversion blog](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world)
- Lore: git treats binaries as "second-class citizens" and has no native multi-tenant isolation — [search summary of Lore](https://github.com/EpicGames/lore)
- Perforce: git was built "for a single maintainer"; P4 gives a server-side view of declared intent to prevent duplicate agent work — [Perforce](https://www.perforce.com/node/4402)
- Pijul: git "can sometimes shuffle lines around"; commutation makes cherry-picks identity-preserving — [Pijul manual](https://pijul.org/manual/why_pijul.html)
- Grace: "Learning Git is far too hard"; git's constraints from 2005 "don't hold anymore"; centralization simplifies the command surface — [Grace design doc](https://github.com/ScottArbeit/Grace/blob/main/docs/Design%20and%20Motivations.md)
- Fossil: records "history as it actually happened" and refuses rebase — [Fossil vs Git](https://fossil-scm.org/home/doc/trunk/www/fossil-v-git.wiki)
- Ink & Switch: git's learning curve and its inability to merge non-text media — [Ink & Switch UVC](https://www.inkandswitch.com/universal-version-control/)
- Multi-agent conflict data: the cross-agent conflict rate is 41.7% versus 19.8% within one agent (the "AgenticFlict dataset", reported second-hand) — [AgentConn](https://agentconn.com/blog/source-control-multi-agent-repos/)
- Industry framing: "the freeze is over"; agents need far higher commit rates, and there is discontent with GitHub — [Pablo Santos, 2026-09-01](https://psantosl.github.io/posts/version-control-second-coming/)

### Inferences

- Most "agent-era" commercial efforts (Entire, Origin, Pierre, Atlas, and even Delta today) chose to *extend* git rather than replace it. Full replacements cluster where git is weakest: binaries and scale (Oak, Diversion, Lore, P4) and merge theory (Pijul).

### Gaps

- No independent benchmarks verify Oak's 95% claim or Diversion's agent-scale claims.

## Q3. Which can serve as both code history and session record?

### Takeaway

**Zed DeltaDB** is the only shipping system designed to store the agent conversation and the code edits as one linked, replicated record. It is proprietary or undisclosed, it is git-backed today, and it treats conversations as first-class content to replay to agents, which conflicts with ParleyInn's I2 (no automatic path from untrusted content to the model). **Fossil** (append-only SQLite, no rebase, built-in tickets, wiki and chat) and **Lix** (SQL rows plus files, auto-commit, `account_id` attribution) are the strongest *structural* candidates for holding a session log next to code. **Perforce P4** offers per-file prompt and agent metadata. Patchwork shows the idea (versioned prompts, bot-attributed edits) but is research only.

### Cited Findings

- DeltaDB records "a message and the edit it produced... side by side" and replicates "the conversation and the worktree together" — [Zed DeltaDB](https://zed.dev/blog/introducing-deltadb); [Zed Delta](https://zed.dev/blog/introducing-delta)
- Fossil keeps ACID SQLite storage, supports SQL queries over history, bundles wiki, tickets, forum and chat, keeps amendments additive, and has no rebase — [Fossil vs Git](https://fossil-scm.org/home/doc/trunk/www/fossil-v-git.wiki)
- Lix keeps custom SQL tables versioned alongside files, a `lix_change` table holding author, timestamp and row snapshots, and auto-commits tracked writes; it is MIT and alpha, with a Go SDK only planned — [lix.dev](https://lix.dev/)
- Grace is event-sourced (every modification stored as an event), has auto-save, and has `grace agent bootstrap` to capture agentic context, but it depends on Azure, Cosmos DB and Orleans — [Grace](https://github.com/ScottArbeit/Grace)
- P4 has "durable, queryable metadata, including prompt context and agent identity, to every version of every file" — [Perforce](https://www.perforce.com/node/4402)
- Diversion's Claude Code plugin captures the "why" behind AI changes — [Diversion](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world)
- Patchwork keeps bot prompts as versioned documents and attributes bot edits in history — [Patchwork 07](https://www.inkandswitch.com/patchwork/notebook/2024-version-control/07/)
- Lore's immutable revision chain with cryptographic integrity, and its Go API — [Lore](https://github.com/EpicGames/lore)

### Inferences

- For ParleyInn's constraints (Go, no network per I4, append-only, rebuildable projections per I10, untrusted content kept away from the model per I2):
  - Server or cloud systems are incompatible with I4 as an embedded store: Diversion, Grace, P4, Unity VCS, Delta, and Lore's service model.
  - Fossil and Pijul are local-first and append-oriented, but neither has a Go library. Integration would mean shelling out, which `os/exec` forbids under I4, or reimplementing the format.
  - Lix's model (SQL rows plus files, attributed changes) maps closely onto a session record, but its Go SDK is only planned and it is alpha.
  - Fossil's "record exactly what happened, no rebase" philosophy matches ParleyInn's lossless append-only record most closely, conceptually.
  - Delta's "replay the conversation to agents" design is the opposite of ParleyInn's pull-only, enveloped recall. It is a useful contrast for the security argument.
- No requested system offers a Go-native, embeddable, local-only store for both code and session records. The realistic options are: (a) keep git or Fossil for code and a ParleyInn-owned append-only log for sessions, linked by content hashes (as Atlas does by patch-id on git); or (b) borrow a design such as Lore's Merkle revision chain, Fossil's SQLite artifacts or Lix's change table, rather than taking on the dependency.

### Gaps

- None of the systems publishes a threat model for stored conversations being replayed into prompts (prompt injection through history). This is a notable gap relative to ParleyInn's I2.
- No source found on tamper-evidence or signing of per-edit or session records in Delta, Lix or Grace.
