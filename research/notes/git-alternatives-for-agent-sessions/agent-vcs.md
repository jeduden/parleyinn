# Version control and code storage built for, or adopted by, AI coding agents (excluding Cursor's own system)

Research date: 2026-10-01. Scope: VCS and code-storage platforms positioned for agents as alternatives to or layers over git. Cursor Origin / Cursor's own system is excluded (covered elsewhere); Graphite is included but noted as Cursor-owned since Dec 2025.

Maturity legend used below: SHIPPED (GA or open-source usable), BETA (public or private beta), ANNOUNCED (waitlist / early access only, not generally usable).

## Key question 1: Which systems exist, and what is each one's data and sync model? (per-system survey)

### Takeaway

The field splits into four camps: (a) new operation-level or CRDT stores that sit *between* git commits and tie conversations to edits (Zed DeltaDB/Delta); (b) git-compatible client redesigns that make agent work safer but store no conversations (Jujutsu, Sapling, GitButler, Graphite, Pijul as non-git); (c) server-side git infrastructure re-tuned for agent write volume (Pierre Code Storage, Freestyle Git, Entire's git network, ERSC, Tangled, Mesa, Oak, Diversion); and (d) thin git-native provenance layers that put agent sessions into git refs or notes (Entire Checkpoints, git-ai, GitButler hooks, SpecStory, Diversion Trajectory). Only DeltaDB is a true real-time multi-writer CRDT; almost everything else keeps git (or a git-like snapshot DAG) as the durable record.

### Cited Findings

#### Zed DeltaDB and Delta (Zed Industries)

- DeltaDB was first previewed in Aug 2025 alongside Zed's Sequoia funding as "a new kind of version control that tracks every operation, not just commits"; it "uses CRDTs to incrementally record and synchronize changes as they happen" and is "designed to interoperate with Git" — [Zed blog: Sequoia backs Zed](https://zed.dev/blog/sequoia-backs-zed); [Gus Mueller, Aug 2025](https://shapeof.com/archives/2025/8/deltadb_from_zed.html)
- Zed's own blog index lists the DeltaDB post "Software Is Made Between Commits" (June 11, 2026), "Introducing Delta" (Aug 12, 2026), "Replace PRs with Delta – Now in Public Beta" (Sept 16, 2026) and "Xanadu Was Waiting for Agents" (Sept 1, 2026), all by Nathan Sobo — [zed.dev/blog](https://zed.dev/blog)
- Data model: "DeltaDB breaks your work into a stream of fine-grained deltas. Where Git captures a snapshot at each commit, DeltaDB captures every operation in between." Every delta is individually addressable; "every reference is anchored to a delta instead of a line number, it survives as the code moves underneath it" — [Zed: Software Is Made Between Commits](https://zed.dev/blog/introducing-deltadb)
- Multi-writer: "DeltaDB embeds conflict-free replicated worktrees, many people and agents can edit the same files at once across different machines" — [Zed: introducing-deltadb](https://zed.dev/blog/introducing-deltadb)
- Files are real: "agents work in them through a terminal, and you can mount the whole worktree to disk whenever you want your own tools on it" — [Zed: introducing-deltadb](https://zed.dev/blog/introducing-deltadb)
- Git relationship: "Git and CI stay for what they're good at: running checks and connecting you to the rest of the world" — [Zed: introducing-deltadb](https://zed.dev/blog/introducing-deltadb). "DeltaDB works with the git repository you already have ... You can commit and push like you always did, and teammates who never open Delta see a normal git repo" — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- Public-beta post describes DeltaDB as extending "Git's content-based versioning with incremental versions based on deltas" and recording "edits between commits alongside messages from humans and agents" — [Zed: Delta public beta](https://zed.dev/blog/delta-public-beta)
- Sync/replication: "Every participant gets their own copy of the code on their local machine, kept in sync in real time as the work happens"; DeltaDB "replicates conversations and worktree together in real-time"; threads are "private until you share them" and can be opened in a browser via a WebAssembly build — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- Third-party agents: "Keep working in the terminal you already use, and your session syncs live into a Delta thread" (Claude Code named) — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- Delta is a standalone Rust application, separate from the Zed editor, compiled to WebAssembly for browser use — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta) (via search summary); [AlphaSignal](https://alphasignal.ai/news/zed-launches-delta-to-replace-git-where-ai-agents-write-code)
- Maturity: private beta from Aug 12, 2026 — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta); public beta Sept 16, 2026, free during beta, paid individual/team plans "soon", "There will always be a free version of Delta"; Zed turned off pull requests on its own repo and 33 team members "landed 570 changes to main since we turned off pull requests"; "Replacing pull requests is our first step toward replacing GitHub" — [Zed: Delta public beta](https://zed.dev/blog/delta-public-beta)
- Openness: AlphaSignal reports the plan mirrors Zed's model, "build it, open-source it, and offer an optional paid service" — [AlphaSignal](https://alphasignal.ai/news/zed-launches-delta-to-replace-git-where-ai-agents-write-code). Zed's own Delta posts that I fetched state no licence, no open-source date, no hosting/self-host model, and no retention/encryption policy — [Zed: Delta public beta](https://zed.dev/blog/delta-public-beta); [RuntimeWire](https://www.runtimewire.com/article/zed-deltadb-version-control-agent-conversations) (notes "Local vs. cloud vs. customer-controlled hosting: Unaddressed")
- Lineage: Sobo and co-founders built Atom and co-led Teletype, an early production CRDT for collaborative editing — [RuntimeWire search summary](https://www.runtimewire.com/article/zed-deltadb-version-control-agent-conversations); Zed's CRDT design is described in [Zed: How CRDTs make multiplayer text editing part of Zed's DNA (2022)](https://zed.dev/blog/crdts)

#### Jujutsu (jj) and East River Source Control (ERSC)

- Working copy is a commit; jj snapshots the working copy at the start of most commands, so there is no staging area — [jj tutorial](https://docs.jj-vcs.dev/latest/tutorial/); [jj operation log docs](https://docs.jj-vcs.dev/latest/operation-log/)
- Operation log: "Jujutsu records each operation that modifies the repo"; each operation stores a "view" (bookmarks, tags, git refs, heads, working-copy commit) plus "pointers to the operation(s) immediately before it" and metadata "such as timestamps, username, hostname, description"; `jj undo`, `jj op revert`, `jj op restore` roll back — [jj operation log docs](https://docs.jj-vcs.dev/latest/operation-log/)
- Multi-writer: the op log gives "lock-free concurrency", including "on different machines that access the repo via a distributed file system (as long as the file system guarantees that a write is only visible once previous writes are visible)"; concurrent operations become divergent op heads that later commands merge and surface — [jj operation log docs](https://docs.jj-vcs.dev/latest/operation-log/)
- Storage: jj currently uses git as a "dumb storage layer" with a "swappable backend"; conflicts are first-class; it is being standardized across Google — [Amplify Partners, May 20, 2026](https://www.amplifypartners.com/blog-posts/will-agents-like-git-any-more-than-we-do)
- Agent adoption: practitioners use jj so unfinished agent work always has a revision identity; caveat: "jj doesn't have a background daemon watching for file changes", so an agent crash before any jj command loses unsnapshotted edits; workaround is Claude Code hooks running jj at `SessionStart` and `PreCompact` — [Panozzo, Nov 2025 / updated Jan 2026](https://www.panozzaj.com/blog/2025/11/22/avoid-losing-work-with-jujutsu-jj-for-ai-coding-agents/). Community agent skills exist — [jujutsu-skill](https://github.com/krismolendyke/jujutsu-skill)
- jj does not record agent prompts or conversations; the op log records repo operations only — [jj operation log docs](https://docs.jj-vcs.dev/latest/operation-log/)
- Licence: Apache 2.0; creator Martin von Zweigbergk joined ERSC as CTO while remaining a core jj maintainer — [ERSC: Martin joins ERSC](https://ersc.io/blog/martin-joins-ersc)
- ERSC (founded 2025, New York, Amplify-backed) is building "a code hosting platform inspired by Google and Meta's internal systems" based on jj, with "first class conflicts, fine-grained ACLs, and backwards compatibility with git", targeting "thousands of commits per second, instant checkouts, and support for arbitrarily sized files"; status May 4, 2026: early trials with select partners; "jj does not have an official native protocol yet" — [ERSC availability update](https://ersc.io/blog/ersc-availability). Funding ~$4.86M (aggregator) — [Tracxn](https://tracxn.com/d/companies/east-river-source-control/__D9WLdnnjFtuC23n9TlhJpYYyL8SdfYxFxMS9oY3hHxM). ANNOUNCED/early access.

#### Meta Sapling

- Git-compatible client organised around commit stacks with auto-restack, smartlog, and undo/journal (`unamend`, `unhide`; amended commits marked obsolete with successor pointers); "the Sapling CLI also supports cloning Git repositories ... GitHub", with "Git support may have some remaining kinks"; the "Sapling server and virtual filesystem (not yet available publicly)"; scales "to repositories with 10's of millions of files, commits, and branches" with server infra — [Sapling intro docs](https://sapling-scm.com/docs/introduction/)
- Mononoke server "not yet supported publicly"; EdenFS virtual FS used in production at Meta "not yet supported for external usage"; written in Rust and Python — [LinuxLinks summary of Sapling README](https://linuxlinks.com/sapling-scalable-user-friendly-source-control-system)
- I found no agent-specific features or session storage in Sapling's docs — [Sapling intro docs](https://sapling-scm.com/docs/introduction/)

#### Pijul

- Patch-based, not snapshot-based: repository is "a directed graph G=(V,E) of lines of text"; vertices identified by hash of the introducing change plus position; independent changes commute; conflicts are first-class (three types including "zombie" vertices); versions are computed so that applying changes in any order yields the same version id — [Pijul manual: theory](https://pijul.org/manual/theory.html)
- Still in 1.0.0 beta: an AUR-style aggregator lists 1.0.0-beta.18 through beta.21 releases in July 2026 (secondary source only) — [AUR mirror](https://linux-meta.duckdns.org/en/p/pijul)
- Agent use is fringe: a third-party agent "skill" exists — [Plurigrid skill listing](https://playbooks.com/skills/plurigrid/asi/pijul). No session or conversation storage.

#### GitButler

- Git-based client (GUI + `but` CLI) with "virtual" (independent, parallel in one working directory) and "stacked" (dependent, auto-restacked) branches — [GitButler docs: AI agents overview](https://docs.gitbutler.com/ai-agents/overview); [GitButler blog: stacked branches](https://blog.gitbutler.com/stacked-branches-with-gitbutler)
- Licence: Fair Source (non-compete converting to MIT after two years); ~21.8k GitHub stars; Rust backend, Svelte/TypeScript frontend; ships `.mcp.json` and agent skills — [GitHub: gitbutlerapp/gitbutler](https://github.com/gitbutlerapp/gitbutler). $17M Series A (aggregator) — [TAMradar](https://www.tamradar.com/funding-rounds/gitbutler-series-a-17m)
- Agent integration today is the `but` CLI plus a GitButler skill installed via `but agent setup`; multiple agent sessions can branch, commit, squash, `but pr` in parallel — [GitButler docs: AI agents overview](https://docs.gitbutler.com/ai-agents/overview)
- July 2025 Claude Code hooks integration: `PreToolUse`, `PostToolUse`, `Stop` hooks feed `session_id` and `transcript_path`; per-session branches `refs/heads/claude/{session_id}` built with a shadow index (`GIT_INDEX_FILE`, `git write-tree`, `git commit-tree`); the last user prompt from the transcript becomes the commit message — [GitButler blog: Claude Code hooks](https://blog.gitbutler.com/automate-your-ai-workflows-with-claude-code-hooks); result is "one commit per chat round and one branch per Claude Code session" — [GitButler blog: parallel Claude Code](https://blog.gitbutler.com/parallel-claude-code)
- GitButler now recommends its skill for the `but` CLI over the Claude Code hooks (per search summary of the plugin repo) — [gitbutlerapp/claude](https://github.com/gitbutlerapp/claude)
- "Agents Tab": run Claude Code inside GitButler, sessions "tied to branches", auto-commit finished tasks, show token usage and estimated cost per branch, commit messages written from "your prompts for proper context" — [GitButler blog: Agents tab](https://blog.gitbutler.com/agents-tab)
- Adoption example: Trigger.dev replaced worktrees with GitButler for parallel Claude Code — [Trigger.dev blog](https://trigger.dev/blog/parallel-agents-gitbutler)

#### Graphite (now part of Cursor)

- Cursor signed a definitive agreement to acquire Graphite on Dec 19, 2025; Graphite says its "product and brand aren't going anywhere" and it will "double down on our best-in-class stacked PRs platform" with Cursor/background-agent integrations — [Graphite: Graphite joins Cursor](https://graphite.com/blog/graphite-joins-cursor); [SiliconANGLE](https://siliconangle.com/2025/12/19/cursor-acquires-ai-code-review-startup-graphite/)
- Stacked diffs layer over git/GitHub PRs; no evidence it records agent sessions or prompts — [Graphite: Graphite joins Cursor](https://graphite.com/blog/graphite-joins-cursor)

#### Pierre Computer Company: Code Storage (code.storage)

- "Distributed Git infrastructure" and "programmable storage layer" for code and machine-generated artifacts; native git clone/push/fetch, SDKs in TypeScript, Python and Go, webhooks, grep and file streaming, GitHub/GitLab/Bitbucket sync, Git LFS — [code.storage](https://code.storage/); [GitHub: pierrecomputer/sdk](https://github.com/pierrecomputer/sdk)
- Agent-specific APIs per the HN launch: grep, glob-based archive, ephemeral branches implemented as git namespaces (search summary) — [HN: Code Storage by Pierre](https://news.ycombinator.com/item?id=46957629)
- Scale/pricing claims: unlimited repos; "500+/s/repo" requests seen; "15k+/s" API; 30+ concurrent writes; 32TB max repo; hot tier $0.005/GB/hour, cold (untouched >7 days) $0.0002/GB/hour; 99.99% SLA; SOC 2 Type II, HIPAA — [code.storage](https://code.storage/)
- Customers named: Lovable, Poke, Anything, Bolt, Amp, Parahelp; "Poke stores not just code, but agent memories"; "memory files" listed among stored artifacts — [code.storage](https://code.storage/). Proprietary managed service; SHIPPED.
- A `just-bash` adapter lets a sandboxed bash support `git` backed by code.storage — [pierrecomputer/just-code-storage](https://github.com/pierrecomputer/just-code-storage)

#### Similar agent-era git hosts

- Freestyle Git (May 8, 2026): API-first multi-tenant git for agent platforms ("programmatic repo creation, scoped permissions, branch comparison, file inspection, webhooks, GitHub sync, and LFS support"), argues "Git owns the durable version graph underneath the filesystem" — [Freestyle blog](https://www.freestyle.sh/blog/engineering/version-control-for-ai-agents)
- Entire distributed git network (preview, July 8, 2026): GitHub mirrors on regional nodes (US, EU, Australia) on a "rebuilt Git backend optimized for concurrent agent operations"; claims ~570,000 clones/hour and 586 pushes/second to a single repo/branch, ~470 mixed ops/s at 50–60 ms p50; backend "planned for open sourcing"; waitlist — [Entire news](https://entire.io/news/entire-launches-distributed-git-network-for-the-agent-era)
- Mesa (Apr 28, 2026): POSIX-compatible durable filesystem with built-in version control ("Git-like semantics rather than being Git itself"), branches for parallel agents "without locking", fine-grained replayable history, FUSE or SDK mounts, sparse materialization; private beta, public beta/v1.0 "in the next few months"; not open source; no session storage — [Mesa blog](https://www.mesa.dev/blog/introducing-mesa-filesystem-for-agents)
- Oak (Show HN ~June 2026): Rust VCS "for agent-driven development"; commits point to flat Mercurial-style manifests of `(path, blob_hash, mode)`, BLAKE3 content addressing, FastCDC chunking (~256 KB–4 MB), local SQLite single-file store, `oak serve` self-host on SQLite; lazy mounts via FSKit/FUSE/ProjFS; "One branch per session"; branch descriptions instead of per-commit messages; git import by replay and `oak export` as "the documented escape hatch"; "Oak does not store agent prompts, transcripts, or session state" — [Oak docs](https://oak.space/docs); [HN: Show HN Oak](https://news.ycombinator.com/item?id=48631726). Reported as v0.99.0 public beta, Apache-2.0 — [Developers Digest](https://www.developersdigest.tech/blog/oak-version-control-agents-git-alternative) / [SaaSCity](https://saascity.io/blog/oak-git-alternative-for-ai-agents-2026) (search summaries); "up to 95% faster than Git" snapshot claim is project-published, unreproduced — [Developers Digest](https://www.developersdigest.tech/blog/oak-version-control-agents-git-alternative)

#### Tangled (AT Protocol git forge)

- Architecture: Appview (aggregated view at tangled.org), Knots ("lightweight, headless servers" hosting git repos, single- or multi-tenant, self-hostable; free managed knots), Spindles (CI) — [Tangled docs](https://docs.tangled.org/)
- AT Protocol supplies decentralized identity; social/collaboration records use `sh.tangled.*` lexicons — [Tangled docs](https://docs.tangled.org/). Issues are atproto records ingested from the firehose; pull requests hold "rounds" whose blobs are "gzipped text-based git-format-patches", but the appview database (not the atproto record) is the source of truth for pulls because patches can be megabytes — [tangled-mcp design note](https://glama.ai/mcp/servers/@zzstoatzz/tangled-mcp/blob/d96d9cbdb3eb939dd2965ba5b8110cd04b3cf18b/sandbox/design-pulls.md)
- A repository has its own DID; lexicons were at v1.16-alpha in July 2026 — [Radial ADR on Tangled](https://next.tangled.org/disnetdev.com/radial/blob/main/docs/adr-tangled-forge.md)
- Core monorepo: MIT licence, Go and Rust, ~1.5k stars; contributors use jj change-ids for patch stacking — [tangled.org/core](https://tangled.org/tangled.org/core). No built-in agent session storage found; third parties build agent orchestrators on it (Radial) — [Radial ADR](https://next.tangled.org/disnetdev.com/radial/blob/main/docs/adr-tangled-forge.md)

#### Diversion

- Centralized, cloud-native VCS (not git-based) with bidirectional GitHub mirroring; "the repository lives centrally" so working copies need not clone full history; designed for "gigantic monorepos" and large binary assets (games) — [Diversion: VC for an agentic world](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world)
- Claude Code plugin "connects every commit to the conversation that produced it"; `/diversion:ask` combines code, `dv blame`/history and captured conversations, run by "an isolated analyst sub-agent" — [Diversion Claude Code plugin](https://www.diversion.dev/diversion-claude-code-plugin); [Diversion blog: the conversation behind the commit](https://www.diversion.dev/blog/the-conversation-behind-the-commit)
- "Trajectory": stores conversations, "every reasoning turn", prompts and rejected alternatives "alongside your repo, indexed in real time", queryable by commit, file, feature, line range or time; early access — [Diversion Trajectory](https://www.diversion.dev/diversion-trajectory). Licence not disclosed.

#### Oxen.ai

- Data (not code) VCS for ML datasets: content-addressed Merkle trees, "millions of files and terabytes of data", positioned against Git LFS; Rust core shared by CLI and `oxen-server`; hosted OxenHub; Apache-2.0; ~1.2k stars — [GitHub: Oxen-AI/Oxen](https://github.com/oxen-ai/Oxen)
- No agent session storage found; repo carries an AGENTS.md and `.claude` dir only — [GitHub: Oxen-AI/Oxen](https://github.com/oxen-ai/Oxen)

#### Git-native agent provenance layers (other 2025–2026 projects)

- Entire Checkpoints (Thomas Dohmke, ex-GitHub CEO): announced Feb 10, 2026 with a $60M seed; first product is the open-source Checkpoints CLI — [Entire news](https://entire.io/news/former-github-ceo-thomas-dohmke-raises-60-million-seed-round); [Dohmke on X](https://x.com/ashtom/status/2021255786966708280). MIT, written in Go, ~5.1k stars — [GitHub: entireio/cli](https://github.com/entireio/cli)
- git-ai: open-source git extension; checkpoints in `.git/ai`, consolidated per commit into an authorship log attached as a git note under `refs/notes/ai`; notes rewritten across rebase, squash, cherry-pick, amend, reset, stash/pop; full sessions referenced by `messages_url` and stored locally, in Git AI Cloud, or self-hosted — [git-ai: how it works](https://usegitai.com/docs/get-started/how-git-ai-works). On the Thoughtworks Technology Radar — [Thoughtworks Radar](https://www.thoughtworks.com/radar/tools/git-ai)
- SpecStory: auto-saves AI chats as Markdown in `.specstory/history/`, intended to be committed for PR review, for Cursor, VS Code and Claude Code — [SpecStory docs](https://docs.specstory.com/features)
- Turso AgentFS: SQLite database as an agent's runtime (POSIX-like FS, KV store, insert-only tool-call audit trail; `toolcalls` table with name/status/timestamps/parameters/result; copy-on-write overlay; sync to Turso Cloud) — [AgentFS auditing guide](https://docs.turso.tech/agentfs/guides/auditing)
- Radicle (P2P git, pre-agent but relevant pattern): issues and patches are "collaborative objects" — CRDT graphs stored in git under special refs, replicated by gossip plus git v2 smart transfer, repos self-signing with Ed25519 delegates — [LWN, Mar 2024](https://lwn.net/Articles/966869); [radicle-cob crate](https://docs.rs/radicle-cob)

### Inferences

- The only system whose *primary* data model is operation-level with real-time CRDT merge is Zed DeltaDB; every other agent-era host keeps a commit/snapshot DAG (git or git-like) and solves agent concurrency with cheap branches, namespaces or mounts.
- "Agent-native" hosting (Pierre, Freestyle, Entire network, ERSC, Mesa, Oak, Diversion) mostly targets *throughput and isolation* (many concurrent branches, fast clone, lazy mounts), not provenance.
- Radicle COBs are the closest prior art for ParleyInn's need: an append-only, multi-writer, mergeable non-code record stored in git refs and replicated without a central server.

### Gaps

- DeltaDB's storage/sync topology (central server vs peer), licence, retention and encryption are not stated in Zed's own posts as of 2026-10-01; the open-source intent comes only from a secondary source.
- ERSC's product details, pricing and any agent features are unpublished; the year of the "Martin joins ERSC" Sept 1 announcement is inferred from coverage.
- Oak's licence and version come from secondary coverage, not the repo itself.
- Lix (lix.dev), which pitches versioned agent filesystems over SQLite/Postgres, appeared only in a search snippet — [awesome.ecosyste.ms entry](https://awesome.ecosyste.ms/projects/github.com%2Fopral%2Flix); I didn't verify it.
- An "Atlas" Tauri app (git storage plus agent control plane) appeared only in a search summary; not verified.

## Key question 2: How do they compare on what is stored, data model, location, multi-writer/conflict model, real-time, scale, openness and maturity?

### Takeaway

Along every dimension ParleyInn cares about, the field converges on "git (or a content-addressed snapshot DAG) as the durable, portable record, plus a side channel for agent metadata". DeltaDB is the outlier: real-time, operation-level, CRDT, conversation-linked, but closed and server-backed so far. Multi-writer safety in the git-based tools comes from per-writer refs or branches, never from shared-branch appends.

### Cited Findings

| System              | What is stored                                                          | Data model                                                     | Where                                                       | Multi-writer / conflicts                                      | Real-time | Openness                               | Maturity (2026-10)                   |
| ------------------- | ----------------------------------------------------------------------- | -------------------------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------- | --------- | -------------------------------------- | ------------------------------------ |
| Zed DeltaDB/Delta   | Every edit between commits + conversations, comments anchored to deltas | Operation-based CRDT, stable delta ids; git for commits        | Local replica per participant + Zed service; mounts to disk | CRDT, "always converge"                                       | Yes       | Not yet stated; OSS intent (secondary) | Public beta (Sept 16, 2026)          |
| Jujutsu             | Commits + operation log of repo views                                   | Snapshot DAG on git backend; op log DAG                        | Local; git remotes                                          | Lock-free op log; divergent ops merged; first-class conflicts | No        | Apache-2.0                             | Shipped                              |
| ERSC                | jj-based hosting                                                        | jj + custom backend, git-compatible                            | Server                                                      | First-class conflicts, fine-grained ACLs                      | n/a       | Unstated                               | Early trials                         |
| Sapling             | Commit stacks, mutation/obsolescence history                            | Git-compatible client; Mononoke/EdenFS internal                | Local; GitHub                                               | Stacks, auto-restack                                          | No        | Open source (client)                   | Shipped client; server not public    |
| Pijul               | Patches (changes)                                                       | Commuting patches over line graph                              | Local / Nest                                                | First-class conflicts, order-independent                      | No        | Open source                            | 1.0 beta                             |
| GitButler           | Git commits; prompt as commit message                                   | Virtual/stacked branches over git                              | Local git                                                   | One branch per session in one worktree                        | No        | Fair Source → MIT                      | Shipped                              |
| Graphite            | Stacked PR metadata over git                                            | Git + PR stacks                                                | GitHub + Graphite SaaS                                      | Stacks                                                        | No        | Proprietary                            | Shipped (Cursor-owned)               |
| Pierre Code Storage | Code, memory files, artifacts                                           | Git repos; namespaces for ephemeral branches                   | Managed cloud                                               | 30+ concurrent writes                                         | No        | Proprietary SaaS; SDKs public          | Shipped                              |
| Entire              | Code + checkpoints (transcripts, prompts, tool calls)                   | Git; checkpoint branch or per-checkpoint refs; commit trailers | Same repo or separate private repo; Entire mirror network   | Per-checkpoint refs avoid contention                          | No        | CLI MIT; network backend "planned" OSS | CLI shipped; network preview         |
| git-ai              | Line attribution + session metadata; transcripts by pointer             | Git notes `refs/notes/ai`                                      | Repo notes; transcripts local/cloud/self-host               | Notes rewritten through history edits                         | No        | Open source                            | Shipped                              |
| Oak                 | Code snapshots                                                          | Flat manifests, BLAKE3, FastCDC, SQLite                        | Server source of truth + lazy mounts                        | Conflicts on `oak pull`; JSON conflict output                 | No        | Apache-2.0 (secondary)                 | Public beta v0.99                    |
| Mesa                | Documents/code with history                                             | Git-like, not git                                              | Managed service, FUSE/SDK mounts                            | Branches, no locking                                          | No        | Proprietary                            | Private beta                         |
| Tangled             | Git repos on knots; social records on PDS                               | Git + atproto records                                          | Federated knots + user PDS                                  | PR rounds as patch blobs                                      | No        | MIT                                    | Alpha (lexicons v1.16-alpha)         |
| Diversion           | Code/assets + Claude Code conversations                                 | Proprietary centralized VCS                                    | Cloud; GitHub mirror                                        | Central server                                                | Auto-sync | Proprietary                            | Shipped VCS; Trajectory early access |
| Oxen                | Datasets                                                                | Merkle tree, content-addressed                                 | Local + oxen-server/OxenHub                                 | Commit/branch                                                 | No        | Apache-2.0                             | Shipped                              |

Sources for each row: Zed — [introducing-deltadb](https://zed.dev/blog/introducing-deltadb), [introducing-delta](https://zed.dev/blog/introducing-delta), [delta-public-beta](https://zed.dev/blog/delta-public-beta); jj — [op log docs](https://docs.jj-vcs.dev/latest/operation-log/), [ERSC](https://ersc.io/blog/martin-joins-ersc); ERSC — [availability](https://ersc.io/blog/ersc-availability); Sapling — [intro](https://sapling-scm.com/docs/introduction/); Pijul — [theory](https://pijul.org/manual/theory.html); GitButler — [hooks blog](https://blog.gitbutler.com/automate-your-ai-workflows-with-claude-code-hooks), [repo](https://github.com/gitbutlerapp/gitbutler); Graphite — [joins Cursor](https://graphite.com/blog/graphite-joins-cursor); Pierre — [code.storage](https://code.storage/), [HN](https://news.ycombinator.com/item?id=46957629); Entire — [cli repo](https://github.com/entireio/cli), [checkpoint storage docs](https://docs.entire.io/guides/checkpoints/checkpoint-storage), [network](https://entire.io/news/entire-launches-distributed-git-network-for-the-agent-era); git-ai — [how it works](https://usegitai.com/docs/get-started/how-git-ai-works); Oak — [docs](https://oak.space/docs); Mesa — [blog](https://www.mesa.dev/blog/introducing-mesa-filesystem-for-agents); Tangled — [docs](https://docs.tangled.org/), [core](https://tangled.org/tangled.org/core), [Radial ADR](https://next.tangled.org/disnetdev.com/radial/blob/main/docs/adr-tangled-forge.md); Diversion — [agentic world](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world), [Trajectory](https://www.diversion.dev/diversion-trajectory); Oxen — [repo](https://github.com/oxen-ai/Oxen).

- Shared-branch contention is a documented failure mode: Entire introduced ref-based checkpoint storage (CLI 0.8.42, July 15, 2026) so "multiple agents can also save their work at the same time without competing to update a shared branch" and so pushes and reads stay fast "as your agent history grows" — [Entire blog: ref-based checkpoint storage](https://entire.io/blog/introducing-ref-based-checkpoint-storage)
- jj's concurrency model is explicitly multi-machine-safe without locks, provided the shared filesystem orders writes — [jj op log docs](https://docs.jj-vcs.dev/latest/operation-log/)
- Agent write volume is the stated driver for new servers: ERSC targets "thousands of commits per second" — [ERSC](https://ersc.io/blog/ersc-availability); a secondary source attributes to GitHub's COO a figure of 275 million commits per week, 14× year over year (unverified; secondary) — [byteiota](https://byteiota.com/?p=21923)
- Amplify Partners argues git's merge throughput ceiling (it cites 2–3 merges/second) and hard-blocking conflicts are bottlenecks for agents — [Amplify Partners](https://www.amplifypartners.com/blog-posts/will-agents-like-git-any-more-than-we-do)
- Skeptic view from the Oak HN thread: "agents don't have trouble with git", and models carry vast git training data, which raises adoption costs for new VCSs — [HN: Show HN Oak](https://news.ycombinator.com/item?id=48631726)

### Inferences

- For ParleyInn's "many agents, worktrees, machines, cloud sandboxes" case, the proven git-transport pattern is per-writer, append-only refs (Entire refs backend, GitButler `refs/heads/claude/{session_id}`, Radicle per-node namespaces). A single shared append branch is the pattern Entire moved away from.
- Real-time sync (DeltaDB) needs a CRDT plus a live service. ParleyInn's I4 (no network) rules out a ParleyInn-run live service, so git transport, driven by the user's own git, fits ParleyInn better than a DeltaDB-style live replica.
- New non-git VCSs (Oak, Mesa, Diversion, DeltaDB's delta layer) all keep a git export or interop path, which suggests git stays the lingua franca for the durable record.

### Gaps

- No vendor publishes independent benchmarks; Entire, Oak and Pierre numbers are self-reported.
- Sapling's internal agent use at Meta (if any) is not documented publicly.

## Key question 3: Which systems store agent sessions or chat history at all, and how?

### Takeaway

Six products store agent conversations as first-class data: Zed Delta/DeltaDB (live, operation-linked), Entire Checkpoints (full transcripts in git refs, commit trailer linkage, redaction), git-ai (git notes with a pointer to transcripts stored elsewhere), Diversion Trajectory (proprietary, "alongside your repo"), SpecStory (Markdown files in the repo), and Pierre Code Storage only as opaque "memory files". GitButler keeps just the prompt as a commit message. jj, Sapling, Pijul, Graphite, Oak, Mesa, Tangled and Oxen store no sessions. Entire is the closest precedent for ParleyInn's design: Go, MIT, git-native, multi-agent, with secret redaction before data hits git.

### Cited Findings

#### Zed Delta / DeltaDB — live, operation-linked conversations

- "A message and the edit it produced are recorded side by side, so neither drifts away from the other"; "From any line in a past conversation, you can jump to that code as it stands now or as it stood the moment the agent wrote it" — [Zed: introducing-deltadb](https://zed.dev/blog/introducing-deltadb)
- Delta stores full conversation history, including "thinking blocks", with untruncated transcripts and comments anchored to code — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- Claude Code terminal sessions "sync live into a Delta thread" — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)
- Conversations live in DeltaDB (Zed's service plus local replicas), not in git; teammates outside Delta "see a normal git repo" — [Zed: Introducing Delta](https://zed.dev/blog/introducing-delta)

#### Entire Checkpoints — transcripts in git, linked by trailer

- On commit, Checkpoints captures "the transcript, prompts, files touched, token usage, tool calls, and more" alongside the commit — [OSTechNix](https://ostechnix.com/entire-cli-git-observability-ai-agents/) (search summary)
- Default storage is the branch `entire/checkpoints/v1` in the same repo; can redirect to a separate private repo with `entire configure --checkpoint-remote github:myorg/checkpoints-private`; `--skip-push-sessions` keeps sessions local; settings in `.entire/settings.json` — [Entire on DEV](https://dev.to/entire/how-to-keep-entire-checkpoints-separate-from-your-code-50a1)
- Ref-based backend: "each new checkpoint gets its own Git ref"; ids change from 12-hex (`a3b2c4d5e6f7`) to 26-char time-sortable (`01KVBJCWYA4YW6J5M9GP655HZN`, ULID-like); old branch checkpoints keep working — [Entire docs: checkpoint storage](https://docs.entire.io/guides/checkpoints/checkpoint-storage); [Entire blog, July 15, 2026](https://entire.io/blog/introducing-ref-based-checkpoint-storage)
- Repo README (as summarized) describes refs `refs/entire/checkpoints/<shard>/<id>` with shard = last two id characters, each checkpoint holding `metadata.json` and a `full.jsonl` transcript; code commits carry an `Entire-Checkpoint: <id>` trailer; "Entire never creates commits on your active branch" (shadow branches hold work in progress); each worktree tracks sessions independently; checkpoint data syncs on `git push` to a single elected sync remote — [GitHub: entireio/cli](https://github.com/entireio/cli). (The ref path and shard rule come from one source only; Entire's docs and blog don't spell them out.)
- Redaction: "Entire automatically redacts detected secrets (API keys, tokens, credentials) from transcripts and metadata before writing a checkpoint" with layered scanners and optional OpenAI Privacy Filter — [GitHub: entireio/cli](https://github.com/entireio/cli); the scrubbing uses Betterleaks "before it hits git" — [Entire on DEV](https://dev.to/entire/how-to-keep-entire-checkpoints-separate-from-your-code-50a1)
- Supported agents: Claude Code, Codex, Cursor, Antigravity, Pi, OpenCode, Copilot CLI, Factory Droid, via hooks in each agent's config directory — [GitHub: entireio/cli](https://github.com/entireio/cli)
- Server-side use: "Entire Blame" (session and decision context per line), "Entire Review" and semantic search over sessions stored "directly in the repository alongside code" — [Entire network news](https://entire.io/news/entire-launches-distributed-git-network-for-the-agent-era)

#### git-ai — attribution in git notes, transcripts by reference

- Agent hooks run `git ai checkpoint` around Edit/Write/Bash; on commit an authorship log maps files and line ranges to sessions, with JSON metadata (agent, model, human author, `messages_url`) attached as a note in `refs/notes/ai`; transcripts are deliberately *not* in the note but stored "locally, in Git AI Cloud, or self-hosted"; notes travel with `git push`/`git fetch`; "100% offline, no login required" — [git-ai docs](https://usegitai.com/docs/get-started/how-git-ai-works)
- Supports Cursor (>1.7), Claude Code and GitHub Copilot in VS Code — [git-ai site](https://usegitai.com) (search summary)

#### Diversion Trajectory — proprietary, linked to dv commits

- The Claude Code plugin "checkpoints your conversation and links it to the commit it produces"; Trajectory keeps "every reasoning turn", prompts and rejected alternatives "alongside your repo, indexed in real time", queryable by commit/file/line range/time; early access — [Diversion Trajectory](https://www.diversion.dev/diversion-trajectory); [Diversion blog](https://www.diversion.dev/blog/the-conversation-behind-the-commit)
- Storage location, schema and redaction are not documented — [Diversion blog](https://www.diversion.dev/blog/the-conversation-behind-the-commit)

#### GitButler — prompt only

- Per-session branches from Claude Code `session_id`; the last user prompt is pulled from `transcript_path` into the commit message; the transcript itself stays in Claude Code's `~/.claude/projects/.../<session>.jsonl` — [GitButler blog: hooks](https://blog.gitbutler.com/automate-your-ai-workflows-with-claude-code-hooks)
- The Agents Tab writes commit messages from prompts; transcript persistence isn't documented — [GitButler blog: Agents tab](https://blog.gitbutler.com/agents-tab)

#### SpecStory, Pierre, AgentFS, small tools

- SpecStory writes each conversation as a Markdown file in `.specstory/history/` for committing — [SpecStory docs](https://docs.specstory.com/features)
- Pierre Code Storage stores "agent memories" / "memory files" as ordinary git content for customers such as Poke; no session schema — [code.storage](https://code.storage/)
- AgentFS records tool calls (not prompts) in an insert-only SQLite table — [AgentFS auditing](https://docs.turso.tech/agentfs/guides/auditing)
- simple-ai-provenance logs every Claude Code prompt via a hook and annotates git commits with what was asked — [PyPI: simple-ai-provenance](https://pypi.org/project/simple-ai-provenance/)

#### Explicitly no session storage

- Oak: "Oak does not store agent prompts, transcripts, or session state ... bring your own [agent]" — [Oak docs](https://oak.space/docs)
- Mesa: no mention of sessions or prompts — [Mesa blog](https://www.mesa.dev/blog/introducing-mesa-filesystem-for-agents)
- jj op log records repo operations (with user/host metadata), not conversations — [jj op log docs](https://docs.jj-vcs.dev/latest/operation-log/)
- No session features found for Sapling, Pijul, Graphite, Tangled or Oxen — [Sapling](https://sapling-scm.com/docs/introduction/); [Pijul](https://pijul.org/manual/theory.html); [Graphite](https://graphite.com/blog/graphite-joins-cursor); [Tangled docs](https://docs.tangled.org/); [Oxen](https://github.com/oxen-ai/Oxen)

### Inferences

- Two linkage patterns dominate: commit-level (Entire trailer, git-ai notes, GitButler message, Diversion commit link) and operation-level (DeltaDB deltas). ParleyInn's append-only session record is closer to Entire's: raw JSONL transcript per checkpoint, written outside the code branch.
- The tools split on whether transcripts go *into* git (Entire, SpecStory) or are only *pointed to* from git (git-ai `messages_url`). Pointers keep git small and keep sensitive text out of shared remotes. Entire offers a separate private checkpoint remote for the same reason.
- Redaction before writing to git (Entire) is the only documented secret-handling control in this group. No product documents treating stored transcripts as untrusted when re-injected, which is ParleyInn's I2 concern. Diversion's isolated "analyst sub-agent" is the nearest analogue.
- Entire's move from one shared branch to one ref per checkpoint, with time-sortable ids, is direct evidence for how ParleyInn should shape git transport across many writers: per-record or per-writer refs, never a contended branch.

### Gaps

- Entire's exact on-branch file layout, ref namespace and refspecs are confirmed by only one summarized source (the repo README). Read the entireio/cli source before relying on them.
- No source documents whether Delta lets users export or self-host conversation data, or how long it keeps it.
- Diversion's and GitButler's transcript storage locations are undocumented.
- I found no system that encrypts session records at rest in git, or that does git transport of sessions across cloud sandboxes without the user's git credentials.

## Addendum, 4 October 2026: Cloudflare Artifacts

Cloudflare's Artifacts, in open beta since 1 October 2026, is a
versioned filesystem behind a git interface, built for many agents: a
fork per task, read and write tokens per repo, events on push, fork
and clone, and agent session context persisted with the code. It is
managed and Cloudflare-hosted. See the
[Artifacts note](../cloudflare-artifacts/cloudflare-artifacts.md).
