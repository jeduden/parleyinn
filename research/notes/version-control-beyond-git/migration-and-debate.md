# Version control beyond git: migration paths, interop strategies and the 2025–2026 "is git fit for agents" debate

Research date: 2026-10-02. Scope: 2025–2026. Every claim carries a URL. Where a source is an aggregator or a vendor, that is flagged. Some primary pages (jj docs mirror, Fossil docs, lobste.rs search, Block's Buzz post) were unreachable during research (503, bot wall or header overflow); those gaps are listed per section.

## Interop strategies and what breaks in each

### Takeaway

Every serious 2025–2026 entrant keeps git somewhere in the stack: as the storage backend (jj, GitButler, Sapling's `.git` mode), as the host-facing protocol under a new storage engine (ERSC, GitLab next-gen SCM), as a carrier for side metadata (Entire, git-ai), or as an interop target (DeltaDB, Diversion, Nostr/ngit). The only notable git-free design, Oak, has no git import/export yet and drew the strongest HN skepticism for exactly that reason. What breaks is consistent: git-level config semantics (`.gitattributes`, hooks, LFS, submodules), metadata that history rewrites drop (squash merges kill trailers; change-ids are not preserved by all git operations), and hosts/CI that only understand plain git.

### Cited Findings

**Strategy 1: a git backend under a new model**

- Jujutsu (jj) combines git's data model with Mercurial-style UX and Pijul/Darcs-style first-class conflicts; it features working-copy-as-a-commit, an operation log with undo, and automatic rebase — [jj docs, git compatibility](https://docs.jj-vcs.dev/latest/git-compatibility/) (summary also in [search of jj docs mirror](https://git.joshthomas.dev/mirrors/jj/src/branch/main/docs/git-compatibility.md)).
- jj supports branches, auth, merge commits (incl. octopus), signed commits (`jj sign`), bare repos, and safe `git gc` — [jj docs](https://docs.jj-vcs.dev/latest/git-compatibility/).
- jj does NOT support `.gitattributes`, git hooks, submodules (preserved but hidden), partial clones, Git LFS, or `git worktree`; annotated tags, shallow-clone deepening and the staging area are only partly supported — [jj docs](https://docs.jj-vcs.dev/latest/git-compatibility/).
- In jj, conflicted commits are stored in git with non-standard representations (`.jjconflict-base-*`/`.jjconflict-side-*` directories, authoritative data in a `jj:trees` commit header); git tools see that non-human-readable form. Change IDs live in a commit header "not preserved through all Git operations" — [jj docs](https://docs.jj-vcs.dev/latest/git-compatibility/).
- Colocated jj+git workspaces risk divergent change IDs and "branch@git" conflicts when commands interleave, slow down with many branches, and are less resilient on NFS/Dropbox — [jj docs](https://docs.jj-vcs.dev/latest/git-compatibility/).
- jj and GitButler agreed to store change IDs as a git commit header named `change-id` (32-char reverse-hex). Gerrit is designing support for jj as a client so that the `change-id` header replaces Gerrit's commit-msg hook — [Gerrit design doc: Support posting changes from Jujutsu](https://www.gerritcodereview.com/design-docs/support-jujutsu-use-cases.html).
- GitButler ($17M Series A, a16z-led, announced ~April 2026) frames its mission as "what comes after Git" but builds on top of git: stacked branches, real-time conflict detection, agent-aware workflows, "preservation of Git compatibility" — [GitButler blog, Series A (Scott Chacon)](https://blog.gitbutler.com/series-a); funding date per [raising.fi](https://raising.fi/news/gitbutler-inc-series-a-april-2026). Chacon: teams are "teaching swarms of agents to use a tool built for sending patches over mailing lists"; "The old model assumed one person, one branch, one terminal, one linear flow" — [GitButler blog](https://blog.gitbutler.com/series-a).
- Sapling has two modes. `.git` mode (from `git clone`/`git init`) keeps a `.git` dir, but "mixing `sl` and `git` commands might not work in all cases"; it may leave a detached HEAD; LFS only partly works; no local tags (remote tags appear as `origin/tags/`); submodules incomplete. `.sl` mode unlocks EdenFS virtual FS and lazy commit graph ("clone and pull roughly O(merges)"). Both speak git network protocols to standard git servers — [Sapling docs, Git support modes](https://sapling-scm.com/docs/git/git_support_modes/).
- Sapling stacked PRs on GitHub: GitHub does not show the individual commits of a stack (Meta's ReviewStack is the workaround), and pushing large stacks can trip GitHub rate limits — [HackerNoon: Experimenting with Sapling](https://sia.hackernoon.com/experimenting-with-sapling-the-new-git-client-from-meta) (secondary source).
- Amplify Partners (VC) argue jj's backend is "swappable" and currently uses git as storage, so adoption is low-friction; they claim that adding "always use jj, never use Git" to a global CLAUDE.md "is sufficient" for agents — [Amplify Partners, "Will agents like Git any more than we do?" (May 20, 2026)](https://www.amplifypartners.com/blog-posts/will-agents-like-git-any-more-than-we-do). Contradicted by a jj user on HN: "Claude forgets to use it all the time despite many obvious instructions" (dbt00) — [HN: Show HN: Oak](https://news.ycombinator.com/item?id=48631726).

**Strategy 1b: keep the git protocol, replace storage (server side)**

- East River Source Control (ERSC, Steve Klabnik et al.) proposes keeping git's protocol but replacing storage with a horizontally scalable engine, plus a second protocol for jj to allow gradual migration: "designed around the constraints of 2005, not 2025, let alone 2035" — original at ersc.io/blog/what-comes-after-git, summarized by aggregator [zeli.app](https://zeli.app/story/49652985) (aggregator; date reported as 2026-09-11). HN critics called it vague on implementation and on what concrete problem agents cause — [zeli.app HN digest](https://zeli.app/story/49652985).
- GitLab "next-gen SCM" (Aug 26, 2026) keeps "Git compatibility" but rebuilds the backend so agents query server-side via purpose-built read/write APIs (files, blame, history) without full clones; claims up to 50x faster, 2x fewer tokens, 1,000x less network traffic (GitLab internal tests). Names three agent problems: the "clone tax" (5–10 GB, 30+ s per invocation), "concurrency collapse", and "no isolation" — [GitLab blog, "Git was built for humans — agents need an upgrade"](https://about.gitlab.com/blog/gitlab-next-gen-scm/).

**Strategy 2: a side-by-side store beside git**

- Zed DeltaDB: operation-based VCS that records every edit as a delta with a stable identity, using CRDTs; "designed to interoperate with Git, but its operation-based design supports real-time interactions that aren't supported by Git's snapshots"; to be open-sourced with an optional paid service — [Zed, Sequoia Series B post (Aug 20, 2025)](https://zed.dev/blog/sequoia-backs-zed).
- The DeltaDB launch essay "Software Is Made Between Commits" (Nathan Sobo, June 11, 2026): "Software now takes shape in the conversation, not the commit"; git stays useful for "running checks and connecting you to the rest of the world" while DeltaDB is the collaborative workspace ("conflict-free replicated worktrees"); beta "ready in a few weeks", waitlist at deltadb.dev — [Zed blog](https://zed.dev/blog/introducing-deltadb). Also discussed on [Hanselminutes podcast](https://pod.wave.co/podcast/hanselminutes-with-scott-hanselman/the-space-between-the-commits-with-zed-and-deltadbs-nathan-sobo).
- Entire (Thomas Dohmke, ex-GitHub CEO): announced Feb 11, 2026 with a $60M seed; ships "Checkpoints" capturing agent prompts/reasoning, at launch for Claude Code and Gemini CLI — [The Stack](https://www.thestack.technology/ex-github-ceo-new-startup-entire); [Wikipedia: Thomas Dohmke](https://en.wikipedia.org/wiki/Thomas_Dohmke). $300M valuation per [Julien Danjou](https://julien.danjou.info/blog/github-wont-work-for-ai-agents/).
- Entire's storage is git-native but off the code branch: local shadow refs (`entire/<HEAD-hash>-<worktree>`), a `post-commit` hook adding an `Entire-Checkpoint: <id>` trailer, and an orphan branch `entire/checkpoints/v1` holding sharded `metadata.json`, `full.jsonl`, `prompt.txt`, `context.md` — [Julien Danjou, "How Entire works under the hood"](https://julien.danjou.info/blog/how-entire-works-under-the-hood/); [Entire docs: checkpoints](https://docs.entire.io/web/checkpoints); CLI is Go (`github.com/entireio/cli`) — [pkg.go.dev](https://pkg.go.dev/github.com/entireio/cli@v0.7.3).
- What breaks in Entire's model: squash merges drop the trailers and sever commit↔session links; transcripts on a shared branch are readable by anyone with repo access (no tiered permissions); the metadata branch grows unbounded with no retention policy; no PR number/URL is stored — [Danjou](https://julien.danjou.info/blog/how-entire-works-under-the-hood/).
- git-ai (1.0 on Nov 8, 2025): agents report exactly which lines they wrote into local checkpoints that "never enter your Git history"; on commit, an Authorship Log is attached as a git note. It rebuilds attribution across rebase, squash (merged into one log), cherry-pick and reset; supports Claude Code, Cursor, Copilot in VS Code; fully offline — [Git AI 1.0 blog](https://usegitai.com/blog/introducing-git-ai); [How Git AI works](https://usegitai.com/docs/how-git-ai-works).
- Danjou's own interim approach before Entire: a git hook linking commits to Claude Code session IDs via trailers, admitted to be "duct tape" — [Danjou, "Agent-written code needs more than Git" (Feb 11, 2026)](https://julien.danjou.info/blog/github-wont-work-for-ai-agents/).

**Strategy 3: two-way bridges**

- git remote helpers (`git-remote-<transport>`) let git talk to any store via `import`/`export` (fast-import streams), `fetch`/`push`, or `connect`; this is how SVN/hg/other systems are exposed to plain git — [git docs: gitremote-helpers](https://git-scm.com/docs/gitremote-helpers).
- Live example: ngit's `git-remote-nostr` lets `git clone nostr://npub.../repo` and normal push/pull work against Nostr (NIP-34) with code on "GRASP" relay-git servers — [Soapbox: What is ngit (July 21, 2026)](https://soapbox.pub/blog/what-is-ngit).
- Pijul: a `pijul git` command (compiled with `--features git`) imports a git repo by replaying history and keeps a commit↔patch mapping for incremental import; slow on large repos; symlinks treated as regular files — [Pijul discourse](https://discourse.pijul.org/t/how-to-convert-git-to-pijul-repositories/728/2); export goes via a separate `pijul-export` (git fast-export) tool — [nest.pijul.com/laumann/pijul-export](https://nest.pijul.com/laumann/pijul-export). Git import work was still active in Feb 2026 — [nest.pijul.com change](https://nest.pijul.com/tzemanovic/inflorescence/change/XQDYES5MDSTFO7OPUPCRQLLQ6NAVELJOCCLSS6YE7TI6QDDFANDQC). Pijul remained at 1.0.0-beta.x — [docs.rs pijul](https://docs.rs/crate/pijul/1.0.0-beta.21).
- Diversion (centralized, cloud VCS) offers "bi-directional sync between GitHub and Diversion" — [Diversion blog](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world) (vendor).

**Strategy 4: git as an export format only / no git at all**

- Oak (Show HN, June 22, 2026; 216 points, 189 comments): new VCS from scratch, FUSE/FSKit mounts so agents need no full clone, chunked storage, built-in large files, messageless intermediate commits, JSON output; claims "50% fewer VCS-related tokens and 90% faster per operation". It currently lacks git import/export; the author may add a git backend later "similar to Jujutsu's approach"; no CI/issues yet — [HN thread](https://news.ycombinator.com/item?id=48631726).

**Strategy 5: git as one carrier among several**

- Agent Trace is explicitly VCS-agnostic: "a data specification, not a product… Storage mechanisms are implementation-defined" (git notes, files or DBs); supports git, jj, hg and svn — [agent-trace.dev](https://agent-trace.dev/).
- Block's Buzz (July 21, 2026) ships git hosting with repos on object storage "coordinated through Nostr events", every message/approval/commit signed by per-person and per-agent keypairs — [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/); [Block announcement](https://block.xyz/inside/introducing-buzz-where-humans-and-agents-work-together) (not fetched: header overflow).
- Tangled (launched March 2, 2025) hosts plain git on self-hostable "knots" with AT Protocol for identity/social and an App View at tangled.org — [Tangled blog](https://blog.tangled.org/intro).

**What breaks: cross-cutting evidence**

- Hosting/review: GitHub shows stacked PRs poorly (Sapling) — [HackerNoon](https://sia.hackernoon.com/experimenting-with-sapling-the-new-git-client-from-meta); Entire stores no PR linkage and loses links on squash — [Danjou](https://julien.danjou.info/blog/how-entire-works-under-the-hood/).
- CI/hooks: jj runs no git hooks (pre-commit checks silently skipped) — [jj docs](https://docs.jj-vcs.dev/latest/git-compatibility/).
- Repo config semantics: jj ignores `.gitattributes`, which is what blocked Gradle (below).
- Models' knowledge: "Models know git because there's a monstrous amount of git in their training data. Models never heard of a new thing 'for agents', so you have to teach them" (SwellJoe) — [HN: Oak](https://news.ycombinator.com/item?id=48631726).
- Security: git itself is an agent attack surface. "GitSpawn" (Manifold Security, Sept 2026): a repo-supplied `core.fsmonitor` runs when agents probe git status at startup, before workspace-trust prompts; affected Claude Code, Cursor, goose (patched) and others (some unpatched as of Sept 1, 2026) — [The Hacker News](https://thehackernews.com/2026/09/malicious-git-configs-can-make-claude.html).

### Inferences

- The dominant 2025–2026 pattern is "new model on top, git underneath or alongside": nobody with real users has removed git from the host/CI boundary. The deepest replacements (ERSC, GitLab) replace git's storage while keeping its wire protocol, because the protocol is what hosts, CI and editors depend on.
- Side-by-side stores that ride git refs (Entire's orphan branch, git-ai's notes) inherit git's weakest properties for an audit record: refs and notes are mutable and force-pushable, squash merges break trailer links, and access control is all-or-nothing per repo. For a security-first append-only record, git is a poor primary store but a workable *mirror/carrier*.
- Git notes (git-ai) survive rewrites better than trailers (Entire) only because git-ai actively rebuilds them; neither is tamper-evident.
- The GitSpawn finding suggests that a tool which shells out to git, or which reads repos it did not create, takes on git's config-execution surface. That is relevant to any ParleyInn design that calls the `git` binary (and ParleyInn's I4 already forbids `os/exec`).

### Gaps

- Could not fetch Fossil's docs (503) for `fossil git export` (one-way mirror) or hg-git's docs; I have no 2025–2026 cited material on either bridge.
- No first-hand source on how DeltaDB serializes to git (export format, commit granularity) — Zed's posts say only "interoperate".
- GitButler's on-disk model (how much it keeps in git vs its own store) was not documented in the fetched sources.
- No source on Forgejo/Gitea support for jj's `change-id` header, or on GitHub's handling of it.

## Migration case studies (onto jj, Sapling, Pijul, and back)

### Takeaway

I found few published team-level migration reports in 2025–2026; individual-developer adoption of jj dominates. The one detailed organizational report found, Gradle (June 2026), evaluated jj and stayed on git because jj ignores `.gitattributes`. The VC/vendor narrative ("adding one line to CLAUDE.md is enough") is contradicted by practitioners who say agents keep reverting to git.

### Cited Findings

- Gradle evaluated replacing git with jj in daily work and did not adopt it: jj "doesn't read `.gitattributes`, so it can't apply a per-file rule", producing "a persistent phantom modification on `gradlew.bat`" (CRLF required on Windows). "The one fix that works asks every Gradle project to commit CRLF and trust that no one's editor quietly rewrites it, and we're not comfortable betting the ecosystem on that." Gradle stays on git plus `git worktree` — [Gradle blog, "The (Petty) Reason We Didn't End Up Using jj" (Laura Kassovic, June 2, 2026)](https://blog.gradle.org/the-petty-reason-we-didnt-end-up-using-jj-at-gradle).
- Martin von Zweigbergk is paid by Google to work on jj; I found no evidence of Google migrating its Piper monorepo to jj — [search synthesis of jj sources](https://www.everydev.ai/developers/jj-vcs) (weak source; treat as unconfirmed).
- Gerrit's design doc says "Jujutsu is gaining traction and support from developers", motivating native support — [Gerrit design doc](https://www.gerritcodereview.com/design-docs/support-jujutsu-use-cases.html).
- Practitioner friction with agents on jj: "I use a new VCS already (jj, highly recommended) and Claude forgets to use it all the time despite many obvious instructions" (dbt00); a counter-report says agents run custom tools "flawlessly" when told to (StorageGuy) — [HN: Oak](https://news.ycombinator.com/item?id=48631726). jj maintainer steveklabnik mentions a Google intern project on an open-source VFS for jj — [same thread](https://news.ycombinator.com/item?id=48631726).
- LWN discussion (Feb 2026) of jj vs git includes the argument that git won over Mercurial on network effects ("everyone else started using git") and performance — [LWN comment](https://lwn.net/Articles/1058555/); a commenter called jj's technical advantages over git "marginal and subjective, and in some cases worse" — [LWN](https://lwn.net/Articles/1058308/) (via search snippet).
- Sapling `.git`-mode users must use `sl rebase --continue` rather than git's, and may be left in detached HEAD — the documented "mixing tools" hazard for gradual migration — [Sapling docs](https://sapling-scm.com/docs/git/git_support_modes/).
- Pijul: migration in means replaying git history (slow on big repos); the project remains 1.0 beta — [Pijul discourse](https://discourse.pijul.org/t/how-to-convert-git-to-pijul-repositories/728/2); [docs.rs](https://docs.rs/crate/pijul/1.0.0-beta.21). A Pijul forum thread on "Adoption plans and git" exists — [discourse.pijul.org](https://discourse.pijul.org/t/adoption-plans-and-git/777) (not read in full).
- Shakespeare was the first large adopter of Nostr git (ngit), "pushing thousands of projects" — [Soapbox](https://soapbox.pub/blog/what-is-ngit) (vendor-partner source).

### Inferences

- The Gradle case is the archetype of "back to git": the blocker was not the model but a single git-level semantic (`.gitattributes`) that the ecosystem relies on. A git-compatible layer must reproduce git's *working-copy* semantics (attributes, hooks, LFS), not just its object store, or it breaks real projects.
- For agents specifically, adoption friction is behavioral (models drift back to `git` commands), which favors designs where the agent keeps using plain git and the new layer works passively (hooks, refs, notes) — the Entire/git-ai pattern.

### Gaps

- No published 2025–2026 report found of a company migrating fully onto Sapling outside Meta, or onto Pijul, or a team-wide jj migration with metrics.
- No direct "we moved back from jj to git" post beyond Gradle's evaluation; lobste.rs search was blocked by a bot wall, so lobste.rs case studies are uncovered.

## The debate: is git fit for AI agents?

### Takeaway

The "git is unfit" camp (Zed, Entire, GitButler, GitLab, ERSC, Amplify, Oak, Diversion, pepicrft) is heavily venture-backed. It makes four distinct claims: scale (clone tax, merge throughput), context (the conversation, not the commit, is the source), concurrency/isolation, and workflow ergonomics (staging, conflicts block). The "git is fine" camp is mostly practitioners on HN/LWN. Its arguments: models are trained on git, the ecosystem lock-in is huge, git performance is not the bottleneck, and the real problem is workflow, not the VCS. Notably, even the leading critics (Dohmke, Chacon, Sobo, GitLab) keep git as the source of truth or the interop layer.

### Cited Findings

**"Git is not fit" — strongest arguments**

- Context loss: "Git was built for humans writing code. It assumes you know what you changed and why"; agent reasoning and transcripts vanish when sessions end — [Danjou (Feb 2026)](https://julien.danjou.info/blog/github-wont-work-for-ai-agents/).
- "Software is made between commits": "the conversation that generates the code is becoming the true source of our software" — [Sobo, Zed (June 2026)](https://zed.dev/blog/introducing-deltadb).
- Session logs as primary artifact: "Session logs are now the most important artifact in software development and should be stored alongside the code itself"; yet "Git will remain the source of truth for code. It will capture provenance, coordinate agent work"; hosting should return to "a distributed network of many hosts" — [Dohmke, Entire blog, "How version control will evolve for the agent boom"](https://entire.io/blog/how-version-control-will-evolve-for-the-agent-boom).
- Scale and concurrency: clone tax 5–10 GB and 30+ s per agent invocation; "concurrency collapse"; agents "share accounts and one branch space… leave no clean way to discard abandoned work" — [GitLab (Aug 2026)](https://about.gitlab.com/blog/gitlab-next-gen-scm/).
- Merge throughput: git caps at "2-3 merges per second"; modal file states add "reasoning overhead"; unresolved conflicts block progress; provenance is hard when "Who wrote this?" means "An agent, probably" — [Amplify Partners (May 2026)](https://www.amplifypartners.com/blog-posts/will-agents-like-git-any-more-than-we-do).
- Ergonomics: git "built for sending patches over mailing lists" — [Chacon, GitButler](https://blog.gitbutler.com/series-a).
- Rethink the forge: "sessions replacing branches", "prompt requests over pull requests", cryptographically attested local hermetic builds (Nix/Bazel) instead of re-running CI — [Pedro Piñera, "Rethinking Version Control for an Agentic World" (Jan 20, 2026)](https://pepicrft.me/blog/rethinking-version-control-for-agents/).
- Centralization is now preferable, monorepos will dominate, and agents lower migration costs by teaching teams new tools — [Diversion (vendor)](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world).
- "Prompts are the new source": git tracks what/when but not whether a prompt edit shifts behavior — [Tessl (Nov 11, 2025)](https://tessl.io/blog/prompts-are-the-new-source-code/). OpenAI's Sean Grove, "The New Code": the spec is what you version, review and defend, with implementations regenerated — summarized by [HackerNoon](https://hackernoon.com/openais-new-code-could-make-spec-authors-the-new-rockstars-of-dev) (secondary). The "shred the source and version the binary" framing of discarding prompts — [Quesma, "Vibe coding needs git blame"](https://quesma.com/blog/vibe-code-git-blame).
- Capital behind the thesis: Entire $60M seed ([The Stack](https://www.thestack.technology/ex-github-ceo-new-startup-entire)), GitButler $17M A ([GitButler](https://blog.gitbutler.com/series-a)), Zed $32M B partly for DeltaDB ([Zed](https://zed.dev/blog/sequoia-backs-zed)). A DEV post frames it as "$17M betting" agents kill git, without naming the company — [DEV Community](https://dev.to/jtorchia/will-ai-agents-kill-git-theres-17m-betting-they-will-n43) (low-quality source; the $17M matches GitButler).

**"Git is fine" — strongest counter-arguments**

- Training data: "Models know git because there's a monstrous amount of git in their training data" — SwellJoe, [HN: Oak](https://news.ycombinator.com/item?id=48631726). Oak's author concedes the "upfront learning tax" and hopes future models see more Oak — zdgeier, [same thread](https://news.ycombinator.com/item?id=48631726).
- Wrong bottleneck: "the performance of git isn't a bottleneck for agents" (hnlmorg); "agentic development is all about throughput, not latency" (danpalmer) — [HN: Oak](https://news.ycombinator.com/item?id=48631726).
- Ecosystem lock-in: lack of GitHub, CI and tooling integration; compared to "replacing WordPress" (jazz9k) — [HN: Oak](https://news.ycombinator.com/item?id=48631726). Git won by network effects, not technical merit — [LWN comment](https://lwn.net/Articles/1058555/); same point in [DEV](https://dev.to/jtorchia/will-ai-agents-kill-git-theres-17m-betting-they-will-n43).
- Hype skepticism: the "for agents" framing masks unclear value ("And there we go") — [HN: Oak](https://news.ycombinator.com/item?id=48631726); ERSC's announcement criticized as vague and promotional — [zeli.app HN digest](https://zeli.app/story/49652985).
- It is a workflow problem, not a VCS problem: agents ignore branch strategy, commit granularity and review ergonomics unless constrained — [buildmvpfast](https://www.buildmvpfast.com/blog/git-workflow-ai-assisted-development-agent-commits-2026) (low-authority blog); git worktrees give agents isolated workspaces — [Towards Data Science](https://towardsdatascience.com/ai-agents-need-their-own-desk-and-git-worktrees-give-it-one/).
- Even critics keep git: Dohmke ("Git will remain the source of truth") — [Entire](https://entire.io/blog/how-version-control-will-evolve-for-the-agent-boom); GitLab keeps "Git compatibility" — [GitLab](https://about.gitlab.com/blog/gitlab-next-gen-scm/); Zed keeps git for checks and the outside world — [Zed](https://zed.dev/blog/introducing-deltadb).
- Git-native agent projects keep appearing on HN, e.g. GitAgent (agent defined as files in a git repo; 147 points, Mar 2026) — [HN](https://news.ycombinator.com/item?id=47376584); DoltLite (git-style versioned SQLite built with 2k agent PRs; Sept 2026) — [HN](https://news.ycombinator.com/item?id=49516848).

### Inferences

- The debate splits along a seam: almost no one argues git's *object model and wire protocol* must die; the argument is that git's *unit of record* (the commit snapshot) misses the agent session, and its *operational model* (full clones, refs as the only namespace) does not scale. That seam is exactly where ParleyInn sits: the session record is the missing artifact, and it need not live in git.
- The training-data counter-argument is strongest for the agent's *command surface*, and weak for a passive record the agent never has to drive. It argues for keeping git as the agent-facing interface while moving the record elsewhere.

### Gaps

- No HN thread found for "Software Is Made Between Commits" with discussion metrics (the WebFetch of HN search returned only higher-scoring threads), and none for the GitLab or Entire essays.
- lobste.rs could not be searched (bot wall); lobste.rs arguments are uncovered.
- No primary text of Sean Grove's "The New Code" talk was fetched.

## Standards that could replace git as the exchange format for agent work

### Takeaway

No single standard is emerging to replace git. Instead, layers are standardizing separately. Attribution: Agent Trace (RFC v0.1.0, Jan 2026; Thoughtworks "Assess"; Cognition, Cloudflare, Vercel, Jules, Amp, OpenCode, Cline). Trajectories: Harbor ATIF (v1.7; Claude Code, Codex, Gemini CLI, OpenHands). Tamper-evident logs: C2SP tlog-tiles (Sigstore, Go checksum DB, and Anthropic's own Access Transparency log). Signed attestations: in-toto (only hobby/hackathon agent predicates so far). Decentralized hosting: Nostr NIP-34 (got its biggest boost from Block's Buzz) and AT Protocol (Tangled). None carries integrity guarantees for agent sessions except tlog/in-toto, which don't define session content.

### Cited Findings

- **Agent Trace**: "Version 0.1.0", "Status: RFC" (Jan 2026), CC BY 4.0; JSON trace records linking files → conversations → line ranges, contributor type (human/AI/mixed/unknown), model id (models.dev convention), conversation URLs, content hashes, VCS metadata for git/jj/hg/svn; storage implementation-defined — [agent-trace.dev](https://agent-trace.dev/); field details in [Morph explainer](https://www.morphllm.com/agent-trace-spec). Backed by Cognition, Cloudflare, Vercel, Google Jules, Amp, OpenCode — [morphllm](https://www.morphllm.com/agent-trace-spec). Thoughtworks Radar Vol. 34 (April 2026), ring **Assess**: "early signals of adoption, with support from tools such as Cline and OpenCode" — [Thoughtworks](https://www.thoughtworks.com/radar/platforms/agent-trace). Combined with jj in practice — [Classmethod: Agent Trace × Jujutsu](https://dev.classmethod.jp/en/articles/agent-trace-jujutsu-ai-code-tracking/).
- **Harbor ATIF**: JSON trajectory spec, current ATIF-v1.7, RFC 0001 with changelog 1.0–1.7; ordered steps with tool_calls, observations and metrics; no signing or integrity fields — [Harbor docs](https://docs.harborframework.com/core-concepts/agents/atif). Supported producers include Terminus-2, OpenHands, Mini-SWE-Agent, Gemini CLI, Claude Code, Codex; trajectories are portable and can seed another agent — [Harbor docs (search summary)](https://harborframework.com/docs/trajectory-format). Third-party uptake: Arize Phoenix ATIF import (April 2026) — [Arize](https://arize.com/docs/phoenix/tracing/how-to-tracing/importing-and-exporting-traces/importing-atif-trajectories); NVIDIA NeMo Agent Toolkit ATIF converter — [NVIDIA docs](https://docs.nvidia.com/nemo/agent-toolkit/1.8/api/nat/utils/atif_converter/index.html); Letta research post (July 23, 2026) — [Letta](https://www.letta.com/blog/trajectory/).
- **in-toto for agent actions**: no official in-toto agent predicate found. Hackathon project "Pedigree" wraps a predicate (model, prompt hash, AGENTS.md hash, approver) in an in-toto Statement, DSSE-signed (ed25519 or Sigstore keyless), stored in git attestation refs — [lablab.ai](https://lablab.ai/ai-hackathons/ibm-bob-hackathon/ctrlcats/pedigree); vendor guide on agent output attestation — [fast.io](https://fast.io/resources/ai-agent-output-attestation/). Regulatory driver claimed: EU AI Act Art. 50 enforcement from Aug 2, 2026 — [lablab.ai](https://lablab.ai/ai-hackathons/ibm-bob-hackathon/ctrlcats/pedigree) (secondary; verify).
- **C2SP tlog-tiles**: Tessera (tlog-tiles library) reached v1.0 GA — [transparency.dev blog](https://blog.transparency.dev/tessera-v1-0-is-here); Sigstore uses tile-based logs — [Sigstore blog](https://blog.sigstore.dev/); witness spec implemented by litewitness — [torchwood](https://awesome.ecosyste.ms/projects/github.com%2Ffilosottile%2Ftorchwood). Anthropic's Access Transparency log (beta) is "an append-only, cryptographically signed record" that "follows the C2SP tlog-tiles format", with C2SP signed-note checkpoints, inclusion and consistency proofs, and the advice to keep your last checkpoint: "That saved checkpoint is what turns 'the log is consistent today' into 'the log has been consistent since you started watching'" — [Claude Platform docs](https://platform.claude.com/docs/en/manage-claude/access-transparency-log).
- **Nostr NIP-34**: ngit + `git-remote-nostr`; repos/PRs/issues as signed events; code on multiple GRASP servers — [Soapbox (July 2026)](https://soapbox.pub/blog/what-is-ngit). Block's Buzz (July 21, 2026): open-source workspace for humans and agents with built-in git hosting coordinated through Nostr; every agent has its own keypair plus a second signature tying it to its human owner ("a verifiable passport and an audit trail"); works with Claude Code, Codex and goose via the Agent Client Protocol — [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/).
- **AT Protocol**: Tangled (since March 2025) — git on knots, identity/social over atproto — [Tangled blog](https://blog.tangled.org/intro); [Tangled docs](https://docs.tangled.org/).
- **Change-id header** as a de facto interop micro-standard across jj, GitButler and (planned) Gerrit — [Gerrit design doc](https://www.gerritcodereview.com/design-docs/support-jujutsu-use-cases.html).
- **git-ai** calls its git-notes format "an open standard for tracking AI-generated code" — [usegitai.com](https://usegitai.com).

### Inferences

- Traction ranking (by evidence of independent implementers): ATIF (several agent and observability vendors) ≈ Agent Trace (multi-vendor backing, Thoughtworks Assess, still v0.1 RFC) > NIP-34 (Block's Buzz is a big single backer) > Tangled/atproto (niche) > in-toto agent predicates (no standard predicate yet). C2SP tlog-tiles is mature but domain-general; it is the strongest candidate for the *integrity* layer of an agent record, not its content.
- A credible exchange stack composes them: ATIF (session content) + Agent Trace (code attribution) + tlog-tiles checkpoints (tamper evidence) + in-toto/DSSE (signed claims), with git as the code carrier. Nothing in the sources does all four yet.

### Gaps

- Did not verify whether Entire or git-ai emit Agent Trace or ATIF.
- No adoption numbers (downloads, repos) for Agent Trace, ATIF or NIP-34 beyond named supporters.
- No sources found applying C2SP tlog to agent session records specifically (other than Anthropic's access log, which covers a different domain).

## What a credible "after git" stack for multi-agent work looks like, and where sources converge

### Takeaway

The sources converge on five points. (1) Keep git compatibility at the edge: hosts, CI, review and models' muscle memory. (2) Add a first-class record of sessions, conversations and reasoning, linked to code changes. (3) Use stable change identities that survive rewrites. (4) Give agents cheap, isolated, parallel workspaces without full clones. (5) Use signed per-agent identity and provenance, increasingly decentralized. They diverge on centralized vs distributed hosting and on whether the record lives in git refs or a separate store.

### Cited Findings

- Git as edge/source of truth: Entire ("Git will remain the source of truth for code") — [Entire](https://entire.io/blog/how-version-control-will-evolve-for-the-agent-boom); Zed (git for checks and the outside world) — [Zed](https://zed.dev/blog/introducing-deltadb); GitLab and ERSC keep the git protocol — [GitLab](https://about.gitlab.com/blog/gitlab-next-gen-scm/), [zeli.app/ERSC](https://zeli.app/story/49652985); GitButler builds atop git — [GitButler](https://blog.gitbutler.com/series-a).
- Session/intent as first-class: Entire session logs — [Entire](https://entire.io/blog/how-version-control-will-evolve-for-the-agent-boom); DeltaDB links messages and edits — [Zed](https://zed.dev/blog/introducing-deltadb); "sessions replacing branches" — [pepicrft](https://pepicrft.me/blog/rethinking-version-control-for-agents/); "prompt provenance, decisions, reasoning traces" — [Diversion](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world); "intent graphs" and "agent-verifiable history" — [DEV](https://dev.to/jtorchia/will-ai-agents-kill-git-theres-17m-betting-they-will-n43).
- Stable change identity / operation log: jj change IDs and op log — [Amplify](https://www.amplifypartners.com/blog-posts/will-agents-like-git-any-more-than-we-do); `change-id` header — [Gerrit](https://www.gerritcodereview.com/design-docs/support-jujutsu-use-cases.html); DeltaDB stable delta identities — [Zed](https://zed.dev/blog/introducing-deltadb).
- First-class (non-blocking) conflicts / CRDT merging: jj — [Amplify](https://www.amplifypartners.com/blog-posts/will-agents-like-git-any-more-than-we-do); DeltaDB CRDTs — [Zed](https://zed.dev/blog/sequoia-backs-zed).
- Cheap parallel workspaces: Oak mounts — [HN](https://news.ycombinator.com/item?id=48631726); GitLab server-side reads — [GitLab](https://about.gitlab.com/blog/gitlab-next-gen-scm/); Sapling EdenFS — [Sapling docs](https://sapling-scm.com/docs/git/git_support_modes/); Diversion's no-local-replica working copies — [Diversion](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world).
- Signed identity and provenance: Buzz per-agent keypairs bound to human owners — [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/); attested local builds — [pepicrft](https://pepicrft.me/blog/rethinking-version-control-for-agents/); in-toto/DSSE commit attestations — [lablab.ai](https://lablab.ai/ai-hackathons/ibm-bob-hackathon/ctrlcats/pedigree).
- Divergence on topology: decentralize hosting (Dohmke; Nostr; atproto) — [Entire](https://entire.io/blog/how-version-control-will-evolve-for-the-agent-boom), [Soapbox](https://soapbox.pub/blog/what-is-ngit), [Tangled](https://blog.tangled.org/intro); versus centralize (Diversion: "centralized architecture is now preferable") — [Diversion](https://www.diversion.dev/blog/diversion-version-control-for-an-agentic-world).
- Divergence on where the record lives: in git refs (Entire orphan branch; git-ai notes) — [Danjou](https://julien.danjou.info/blog/how-entire-works-under-the-hood/), [git-ai](https://usegitai.com/docs/how-git-ai-works); versus a separate operation store (DeltaDB) — [Zed](https://zed.dev/blog/introducing-deltadb); versus implementation-defined (Agent Trace) — [agent-trace.dev](https://agent-trace.dev/).
- Privacy of the record is an open problem: Entire's transcripts are readable by anyone with repo access — [Danjou](https://julien.danjou.info/blog/how-entire-works-under-the-hood/).

### Inferences

- For ParleyInn, the evidence argues for decoupling rather than a wholesale move. Keep version control of *code* on git, or on a git-backed layer like jj, for host/CI/model compatibility; the Gradle and Oak cases show the cost of leaving. Move the *session record* to a dedicated append-only store with tamper evidence (a tlog-tiles-style Merkle log with signed checkpoints, as Anthropic's own access log does). Bind it to code through stable identifiers (commit hash plus `change-id`) instead of trailers, which squash merges drop.
- Git refs or notes can then serve as an optional, export-only *carrier* (for example, Agent Trace or ATIF records projected into notes) without making git the system of record. This keeps I10-style rebuildability: the git projection is derived from the log.
- A git-ref-based record (Entire style) conflicts with security-first goals in three documented ways: no per-reader access control, unbounded growth with no retention, and mutable refs. A separate store sidesteps all three. It also avoids GitSpawn-class exposure from invoking git on untrusted repos.
- Standards to emit, not to adopt as storage: ATIF for session export (it has the broadest agent coverage) and Agent Trace for code attribution. Watch NIP-34/Buzz for signed agent-identity conventions.

### Gaps

- No source describes an end-to-end multi-agent "after git" stack that is actually deployed; the convergence above is synthesized across vendors' visions.
- No independent benchmarks validate the vendors' performance claims (GitLab's 50x/1,000x, Oak's 50%/90%).
