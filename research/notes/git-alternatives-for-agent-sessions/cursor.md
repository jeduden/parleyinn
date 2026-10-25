# Cursor's (Anysphere's) git alternative: Origin, its Continuity storage engine, and the attribution layer around it

Research date: 2026-10-01. Main sources: Cursor's own pages (changelog, docs, the engineering blog). Press and Hacker News are used for dates, reception and criticism. Vendor numbers are marked as vendor numbers.

## Q1. What is it called, when was it announced, is it shipped, open or proprietary, and how does it relate to Graphite, cloud agents or a new VCS?

### Takeaway

The product is **Cursor Origin**, "a git forge for the agentic era". It is a proprietary, git-protocol-compatible code host. It is not a new VCS. Cursor announced it with a waitlist on 16 June 2026 at its Compile event. It shipped as an early beta on all paid plans on 17 August 2026. The novel part is the server-side storage engine, **Continuity**. Continuity keeps an append-only write-ahead log (WAL) of pushes on S3-compatible object storage as the source of truth, and on-disk git repos are rebuildable caches. Two adjacent Cursor efforts handle linking conversations to code: **Cursor Blame** (proprietary, Enterprise plan) and **Agent Trace** (an open RFC spec). The Graphite team, which Cursor acquired in December 2025, fronts Origin.

### Cited Findings

**Candidate 1: Origin (the main answer)**

- Name and tagline: "A git forge for the agentic era". "Code is moving faster than any infrastructure was built to handle. Origin was designed for this moment." The landing page says "Early beta now available on all paid plans." — [cursor.com/origin](https://cursor.com/origin)
- Announced on 16 June 2026 at Cursor's Compile event in San Francisco. It was waitlist-only then, with general availability targeted for fall 2026. — [learncursor.dev (third party)](https://www.learncursor.dev/learn/cursor-origin/commits-per-second); [Dealroom news note](https://app.dealroom.co/news/note/cursor-launches-origin-its-own-code-hosting-and-git-platform)
- On the 16 June HN thread, a commenter who said they work at "Graphite->Cursor->???" wrote that Origin was "announced by Tomas, Graphite co-founder, at Cursor's Compile conference today". Others complained the page had "literally zero information on what this is". — [HN item 48558605](https://news.ycombinator.com/item?id=48558605) (via [Algolia API](https://hn.algolia.com/api/v1/items/48558605))
- Shipped as an early beta on **17 August 2026**: "Cursor can now host your code." It is "rolling out in early beta to all paid plans", and Enterprise admins can opt out. — [Cursor changelog: Origin code hosting](https://cursor.com/changelog/origin-code-hosting)
- Plans: paid only (Start, Pro, Pro+, Ultra, Teams, Enterprise). "Teams on legacy privacy mode cannot enable Origin." — [Cursor docs: Origin](https://cursor.com/docs/origin)
- Proprietary and closed source. InfoQ calls it a "proprietary Cursor platform", available only to paying Cursor customers. — [InfoQ, Aug 2026](https://www.infoq.com/news/2026/08/cursor-origin-alternative-github/)
- Launch day coincided with a major GitHub outage. — [Notebookcheck](https://notebookcheck.net/Cursor-launches-Origin-a-built-in-GitHub-alternative-on-the-same-day-GitHub-suffers-a-major-outage.1371041.0.html); [The New Stack headline](https://thenewstack.io/cursor-origin-github-alternative/)

**Candidate 2: Continuity (the storage engine behind Origin)**

- Vicent Martí, Cursor principal systems engineer, described it in "Git at any Scale" on 18 August 2026. "The core primitive behind it is a write-ahead log, which we store in S3-compatible object storage." — [Cursor blog: Git at any scale](https://cursor.com/blog/git-at-any-scale); author and title confirmed by [The Register](https://www.theregister.com/a/5291421)
- HN noted that the author was a core GitHub developer and also worked on Vitess at PlanetScale. — [HN item 49348141](https://news.ycombinator.com/item?id=49348141)

**Candidate 3: Graphite (acquired; supplies stacked PRs and review, not storage)**

- Cursor signed a definitive agreement to acquire Graphite on 19 December 2025. Graphite is a code review and stacked-PR platform, and the price was "way over" its $290M last valuation. Graphite was to keep operating independently. — [TechCrunch](https://techcrunch.com/2025/12/19/cursor-continues-acquisition-spree-with-graphite-deal); [SiliconANGLE](https://siliconangle.com/2025/12/19/cursor-acquires-ai-code-review-startup-graphite/)
- One third-party source calls this a three-part stack: editor and Composer write, Graphite handles stacked PRs and review, Origin stores. — [learncursor.dev](https://www.learncursor.dev/learn/cursor-origin/commits-per-second) (secondary source; not confirmed by Cursor)

**Candidate 4: Agent Trace and Cursor Blame (the layer linking conversations to code, not a VCS)**

- Agent Trace is "a draft open specification" from Cursor for attributing AI-generated code, released as an RFC. It is v0.1.0, dated January 2026, under CC BY 4.0. — [agent-trace.dev](https://agent-trace.dev/); [InfoQ, Feb 2026](https://www.infoq.com/news/2026/02/agent-trace-cursor)
- Cursor Blame is git blame plus AI attribution (Tab, Agent with model, Human), with summaries of the producing conversations. It is Enterprise only and off by default. — [Cursor docs: Cursor Blame](https://cursor.com/docs/integrations/cursor-blame); [Cursor forum: Cursor 2.4](https://forum.cursor.com/t/cursor-2-4-cursor-blame/149405)

**Corporate context**

- SpaceX signed an agreement on 16 June 2026 to acquire Anysphere for an implied $60B in stock, with closing expected around Q3 2026. That is the same day Origin was announced. — [Tribune India](https://www.tribuneindia.com/news/business/spacex-to-acquire-ai-coding-startup-cursor-in-60-billion-deal-expects-q3-2026-close/); [Gigazine](https://gigazine.net/gsc_news/en/20260617-spacex-acquires-curso-anysphere/)

### Inferences

- "Cursor's git alternative" is a git *forge* alternative (to GitHub and GitLab), not an alternative to git itself. Clients still speak plain git over HTTPS. What is new is how the server stores history.
- Graphite people front Origin (Tomas Reimers announced it and answered HN questions). The intended differentiation is the stacked-PR and merge-queue workflow, but the August beta does not ship it yet (see Q3).

### Gaps

- I could not find a Cursor blog post for the 16 June announcement. The landing page had little content, and the June feature claims come only from third-party coverage.
- I could not confirm whether the SpaceX deal has closed as of October 2026.

## Q2. What does it store, where, and in what data model?

### Takeaway

Origin stores ordinary git repositories: commits, trees, blobs and refs. It does not store agent conversations, checkpoints or keystrokes. Server side, Continuity is event-sourced. Each accepted push is an immutable `.wal` object (a packfile) in S3-compatible storage. A single index object, `gitwal.pb`, linearizes the pushes and is updated with an atomic compare-and-swap (CAS). Local NVMe git repos are warm caches that can be garbage-collected and rebuilt from the WAL. The model is a snapshot DAG (git) under a log of pushes. It is not a CRDT and not a per-keystroke operation log.

### Cited Findings

- What it hosts: "Create Origin repositories, including from Cursor agents", plus clone, push, pull, browse and search at cursor.com/codebase, and pull requests. "Origin is the source of truth" for native repos. — [Cursor docs: Origin](https://cursor.com/docs/origin)
- Storage primitive: "The core primitive behind it is a write-ahead log, which we store in S3-compatible object storage." — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- WAL layout: each push is a separate `.wal` object holding the uploaded packfile. A WAL index file, `gitwal.pb`, holds pointers to the linearized pushes. — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Dual write: "pushes are uploaded into S3 in a write-ahead log (WAL), capturing all changes as immutable objects. Simultaneously, the pushes are written to the local 'reference' copy of the repository (usually an NVMe disk). Once both actions complete, other replicas of the repository can download the changes as needed." — quoted in [The Register](https://www.theregister.com/a/5291421)
- Cache semantics: the on-disk repo is "a warm cache on disk, but the source of truth is always the write-ahead log." — quoted in [DevClass](https://www.devclass.com/devops/2026/08/24/how-cursor-beat-gits-scalability-shortcomings/5291479)
- Durability before acknowledgement: "We never acknowledge a push until it has been fully persisted." The order is: packfile written to local NVMe, WAL entry uploaded to S3, ref transaction prepared locally, then the pointer recorded in the WAL index in one atomic operation. Cursor states the result is that all pushes are linearizable, with no eventual-consistency window. — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Rebuild: when a repo is missing on disk, "we just materialize it from the WAL. We can do this very efficiently." The WAL also gives full push provenance, which Cursor says enables debugging and rewinding. — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Compaction: "Only the primary does compactions, and the result of the compaction applies to both the on-disk repository and the WAL." Replicas download pre-compacted packfiles from S3 instead of repacking locally, trading bandwidth for CPU. — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Routing: stateless rendezvous hashing with no routing database. Replica count can range from 0 to 100+ per repo. — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Contrast with GitHub Spokes: Spokes used three-phase commit across at least 3 replicas per repo, an external routing database and active repair jobs ("pets not cattle"). Continuity instead "synchroniz[es] the reference transaction with a single local repository instead of a quorum of replicas", so it can "ingest pushes as fast as our disk allows". — [Git at any scale](https://cursor.com/blog/git-at-any-scale); [The Register](https://www.theregister.com/a/5291421)
- A commenter described the design as "event sourcing at its finest + cqrs". — [HN item 49348141](https://news.ycombinator.com/item?id=49348141)
- A community reimplementation appeared within days: "Show HN: Waltier – a generalized version of Cursor's S3 WAL". — [GitHub: danthegoodman1/waltier](https://github.com/danthegoodman1/waltier); [HN Algolia search](https://hn.algolia.com/api/v1/search?query=git%20at%20any%20scale&tags=story)

### Inferences

- Continuity follows the same principle as ParleyInn's I10, but on a server. The append-only log is the record, and every git repo on disk is a derived, rebuildable projection. Compaction is a derived artifact written back to the log, not a mutation of history.
- The unit of the log is a whole push (a packfile plus a ref transaction), not a fine-grained event. Agent session events would need their own record, because Origin does not capture them.
- Linearizability comes from one CAS'd index object per repo. That is a single-writer-per-repo design: any node can be primary, and contention is resolved by CAS retry.

### Gaps

- The exact schema of `gitwal.pb` and the `.wal` objects is not published beyond the blog's description.
- Encryption at rest, retention and deletion semantics for the WAL are not documented. It is unclear how a force-push or a secret purge interacts with an immutable push log, and Cursor's mention of "rewinding" suggests old pushes are kept.

## Q3. How does it handle many concurrent agents and humans, branching, merging and conflicts, and how does it relate to git?

### Takeaway

Concurrency is solved at the storage layer: pushes are linearized through S3 CAS, so any server can accept them. At the workflow layer, Origin uses ordinary git branches and pull requests. Merge conflicts are surfaced in the PR and resolved by a human or an agent. There is no automatic CRDT-style merge. Origin is fully git-compatible: HTTPS remote, `origin` CLI for auth. It can mirror GitHub with GitHub as the source of truth, and it has a failover write path while GitHub is down. Stacked PRs and agent-aware merge queues were shown in the June demo coverage but are not in the August beta.

### Cited Findings

- Concurrent writes: "All updates to the write-ahead log are synchronized with an atomic compare-and-swap (CAS) operation on S3." A failed CAS triggers a retry. "Any server can be the primary. It also doesn't matter!" — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Replica catch-up: after each push, a gossip UDP packet is broadcast and may be lost. Replicas also make a conditional GET with ETag on the WAL index, a metadata-only operation of about 10ms. A 304 means the replica is current; a 200 triggers catch-up before reads are served. "It doesn't matter if the replication UDP packet is lost." — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Design maxim: "The system is designed to always be correct when degraded, and always fast when healthy." — [Git at any scale](https://cursor.com/blog/git-at-any-scale)
- Agent-shaped workloads: agents create "vast numbers of small repositories, many of them throwaway". "Millions of tiny repositories created by agents can be served with one replica each; we don't need more than one to ensure availability, because S3 is the source of truth." Idle repos are garbage-collected and rebuilt on demand. — [Git at any scale](https://cursor.com/blog/git-at-any-scale); [XenoSpectrum](https://xenospectrum.com/en/cursor-continuity-wal-git-scale/)
- Branching and PRs: "Push a branch to Origin, then open a pull request against the base branch". "Origin surfaces merge conflicts so you can resolve them before merging". Checks report status per branch. "Cursor cloud agents can also open pull requests as part of a task. On a mirrored GitHub repository, those agents open GitHub pull requests." — [Cursor docs: pull requests](https://cursor.com/docs/origin/pull-requests)
- Git interop: HTTPS remote `https://origin.cursor.com/{owner}/{repo}.git`, with `origin auth login` before first use. SSH is not mentioned. — [Cursor docs: git](https://cursor.com/docs/origin/git)
- GitHub mirroring: "Pushes to this remote go to GitHub. Origin updates after GitHub accepts the push." It syncs history, branches, tags and PRs, but not Issues, Actions workflows or secrets. Admin access on the GitHub repo is required. — [Cursor docs: mirror GitHub](https://cursor.com/docs/origin/mirror-github)
- PR comments sync both ways on synced repos: "comment in Cursor and it posts to GitHub." — [Cursor changelog](https://cursor.com/changelog/origin-code-hosting)
- Failover: "Keep working on the Origin copy with `origin/` branches and `origin push local`. `/local` is the supported write path while GitHub is down." Origin serves reads from its existing copy, but "Git LFS batch does not fail over." — [Cursor docs: mirror GitHub](https://cursor.com/docs/origin/mirror-github)
- On launch day, GitHub's degradation affected Origin's mirrored repos (Cursor status incident). — [HN item 49336919](https://news.ycombinator.com/item?id=49336919) linking [status.cursor.com](https://status.cursor.com/incidents/l9h9vrd726jv)
- Stacked PRs and merge queues, where sources conflict:
  - Coverage of the June demo describes "stacked pull requests and agent aware merge queues", with merge queues keeping CI green and automated fixes for failing builds. — [InfoQ](https://www.infoq.com/news/2026/08/cursor-origin-alternative-github/); [learncursor.dev](https://www.learncursor.dev/learn/cursor-origin/commits-per-second)
  - A hands-on review of the August beta lists "Stacked PRs, merge queues" as not supported. The same review lists no SSH, no public repos, no native CI, and no Issues or Actions. — [Appwrite review, 18 Aug 2026](https://appwrite.io/blog/post/cursor-origin-review-an-engineers-perspective)
  - The Origin PR docs do not mention either feature. — [Cursor docs: pull requests](https://cursor.com/docs/origin/pull-requests)
  - Tomas Reimers on HN, launch day: "Over the next few weeks ... automatically getting your PRs to a mergable state". — [HN item 49334209](https://news.ycombinator.com/item?id=49334209)
- Visibility: "Currently Origin repositories are private to members of a Cursor team. Over time we plan to expand access" (tomasreimers). — [HN item 49334209](https://news.ycombinator.com/item?id=49334209)

### Inferences

- Origin's answer to "many agents at once" is a fast, linearizable push path plus cheap per-repo replicas, with git's branch model left unchanged. It offers nothing new for merging. Agents work on branches or throwaway repos, and conflicts come back through PRs.
- The failover mode (`origin/*` branches, a local write path while the upstream is down) is a pragmatic two-remote design. It is relevant to any tool that treats a hosted remote as its record transport.

### Gaps

- Origin's documentation does not say whether it has branch protection, force-push rules, push size limits or rate limits.
- Cursor does not document how simultaneous changes are reconciled when a write lands on Origin's `/local` path during a GitHub outage and GitHub later changes.

## Q4. Does it link agent conversations to code changes? Real-time behaviour, scale claims, privacy and security?

### Takeaway

Origin itself does not store or link agent conversations. The changelog promises only that "your code, PRs, and agents are now in the same place." The links between conversations and code live in two other places. **Cursor Blame** is server-side and proprietary: attribution is fetched from Cursor's servers. **Agent Trace** is an open JSON spec that deliberately leaves storage to the implementer (files, git notes or a database). Cursor's only first-party throughput figures are from its storage blog: about 120 pushes/s on S3 Standard and 300+ pushes/s on S3 Express One Zone. Privacy documentation was thin at launch.

### Cited Findings

- Agent integration claim: "Your code, PRs, and agents are now in the same place. Ask Cursor questions about code you're browsing." — [Cursor changelog](https://cursor.com/changelog/origin-code-hosting)
- Cloud agents operate on Origin repos natively ("clone, branch, commit, push, and open PRs"), and automations can trigger on schedules or source-control events. The reviewer also notes agent-created PRs are "not immediately visible" because the PR list defaults to the user's own PRs. — [Appwrite review](https://appwrite.io/blog/post/cursor-origin-review-an-engineers-perspective)
- Cursor Blame storage: "Cursor Blame caches attribution data locally for performance. When you view files and commits, data is fetched from Cursor's servers." "Conversation summaries are retrieved on-demand and show brief descriptions rather than full conversation history." It requires "a Git repository with Cursor-tracked changes". — [Cursor docs: Cursor Blame](https://cursor.com/docs/integrations/cursor-blame)
- Agent Trace data model: a trace record has `version`, `id` (UUID), `timestamp` (RFC 3339), `files[]`, and optional `vcs`, `tool` and `metadata`. Each file groups `conversations[]`. A conversation holds a `url`, a `contributor` (type `human`/`ai`/`mixed`/`unknown`, plus a model id in models.dev form such as `anthropic/claude-opus-4-5-20251101`), and `ranges[]` with `start_line`, `end_line` and an optional `content_hash` (for example `murmur3:9f2e8a1b`). The `vcs` field covers git (40-character SHA), jj (change ID), hg and svn. — [agent-trace.dev](https://agent-trace.dev/)
- Agent Trace storage: "This specification intentionally does not define how traces are stored. This could be local files, git notes, a database, or anything else." Line ownership is looked up via VCS blame, then the trace record for that revision. — [agent-trace.dev](https://agent-trace.dev/)
- Agent Trace backers named in coverage include Cognition, Cloudflare, Vercel, Google Jules, Amp and OpenCode. — [InfoQ, Feb 2026](https://www.infoq.com/news/2026/02/agent-trace-cursor); Thoughtworks lists it on its Technology Radar — [Thoughtworks Radar](https://www.thoughtworks.com/radar/platforms/agent-trace)
- First-party scale figures: 120 pushes/s on S3 Standard while compacting and replicating, and 300+ pushes/s on S3 Express One Zone. At that point local git compaction, not the network, was the bottleneck. Read scaling was linear up to 100 replicas in synthetic tests. — [Git at any scale](https://cursor.com/blog/git-at-any-scale); [XenoSpectrum](https://xenospectrum.com/en/cursor-continuity-wal-git-scale/)
- Demo figures from third parties (unverified vendor numbers): about 22.6 commits/s in one repo, "sub-400-millisecond global synchronization", automatic S3-backed failover, and "hundreds of thousands of clones and pushes per hour". The source itself says to treat them "as direction, not a guarantee". — [learncursor.dev](https://www.learncursor.dev/learn/cursor-origin/commits-per-second)
- Privacy:
  - Origin cannot be enabled under legacy Privacy Mode; teams must switch to the standard Privacy Mode. — [Cursor docs: Origin](https://cursor.com/docs/origin)
  - An HN commenter quoted Cursor's docs on that switch: "Legacy Privacy Mode disables code storage...Switch to a different privacy mode...to do training". — [HN item 49334209](https://news.ycombinator.com/item?id=49334209)
  - "Origin shipped without clear data retention or training use policies" (TechTimes, quoted by InfoQ). InfoQ also says code is hosted on "SpaceX/xAI-controlled infrastructure". — [InfoQ](https://www.infoq.com/news/2026/08/cursor-origin-alternative-github/)

### Inferences

- Cursor splits three concerns. Code history goes in git (Origin/Continuity). Attribution and conversation links live in a sidecar: server-side for Cursor Blame, storage-agnostic for Agent Trace. Full conversations stay in Cursor's own backend. Conversation data never travels inside the git record.
- Agent Trace is the closest published schema for linking a session to a commit. ParleyInn could export it, for example as git notes keyed by commit SHA, without making git its primary record.
- InfoQ's "SpaceX/xAI-controlled infrastructure" is a corporate-ownership statement. The S3 Express One Zone benchmarks suggest AWS, but Cursor did not name a cloud.

### Gaps

- Cursor does not document how Cursor Blame captures attribution: commit hooks, trailers or server-side diffing. It is also unknown whether Origin PRs show the agent conversation that produced them.
- I found no published Origin documentation on retention, deletion, encryption at rest, data residency or training use.
- The real-time behaviour of the PR UI is undocumented beyond GitHub PR comments syncing "within seconds" (per [Dealroom](https://app.dealroom.co/news/note/cursor-launches-origin-its-own-code-hosting-and-git-platform)).

## Q5. What did Hacker News commenters and other technical critics say?

### Takeaway

The launch thread had 597 points and 454 comments. It was dominated by trust and ownership objections (Musk and SpaceX/xAI, training on code), lock-in, and disappointment that Origin looked like "a GitHub clone". The "Git at any scale" thread had 371 points and 116 comments. It praised the engineering. Critics argued that the design hands the hard distributed-systems problems to S3, a proprietary product, and asked whether git itself is the right foundation for agents.

### Cited Findings

**Thread metrics** — [HN Algolia story search](https://hn.algolia.com/api/v1/search?query=cursor%20origin&tags=story)

| HN item                                                   | Story                                        | Points | Comments | Date       |
| --------------------------------------------------------- | -------------------------------------------- | ------ | -------- | ---------- |
| [49334209](https://news.ycombinator.com/item?id=49334209) | "Cursor launches Origin, GitHub alternative" | 597    | 454      | 2026-08-17 |
| [49348141](https://news.ycombinator.com/item?id=49348141) | "Git at any scale"                           | 371    | 116      | 2026-08-18 |
| [49336919](https://news.ycombinator.com/item?id=49336919) | "GitHub degradation affects Cursor Origin"   | 67     | 27       | —          |
| [48558605](https://news.ycombinator.com/item?id=48558605) | 16 June teaser                               | 24     | 8        | 2026-06-16 |

**Launch thread ([item 49334209](https://news.ycombinator.com/item?id=49334209))**

- Trust and ownership:
  - "I would *never* host my code with Elon Musk." (slowin)
  - "Given Grok was just caught uploading whole codebases and sensitive .envs without permission, how can anyone possibly trust this?" (dbbk; the Grok claim is the commenter's and was not verified here)
  - Reliability: "Cursor/spacetwitterai have no experience keeping a system like that up" (peterldowns)
- Interop and silos: "Please make it Fediverse/Forgejo/etc compatible. It is 2026 and I'm so tired of silos" (icrbow). A related point: "it must be pretty much the same data model, so they could probably have a compatible API" (justincormack).
- Differentiation: "I expected more than a GitHub clone, this is a bit of a let down" (dutchCourage). Others asked whether stacked PRs exist and how it handles agent swarms and high request rates (throwaway613746, arjie).
- Alternatives promoted: Tangled, a decentralized ATProto-based forge (LelouBil), and plain git with custom CI (gritzko).
- Optimism: "Origin has a chance to be the future of this space, with a clean slate" (rvz, quoted by [InfoQ](https://www.infoq.com/news/2026/08/cursor-origin-alternative-github/)).
- Press summary of developer sentiment: "some developers say they would rather tolerate GitHub's downtime than host code on infrastructure ultimately controlled by Elon Musk". Origin works as "an additional hosting surface that lives alongside GitHub rather than replacing it". — [InfoQ](https://www.infoq.com/news/2026/08/cursor-origin-alternative-github/)

**Storage thread ([item 49348141](https://news.ycombinator.com/item?id=49348141))**

- Pushing hard problems into S3: "There's a trend of doing impressive things by pushing many hard problems into S3 and assuming S3 'just works'...S3 is a proprietary product, not an algorithm" (wrs). A similar point: you need S3-compatible storage "with similar latency...four nines availability and 10+ nines durability...impossible for most engineering teams" (doodlesdev).
- Precision of the Spokes comparison: "Doesn't 3PC require all nodes to agree, not just a majority?" (eatonphil)
- Git's own limits: the external API "depends on git packs which you might have to reconstruct on the fly" (luke5441). "Is Git the right solution for version control given where we are heading?" (warmwaffles)
- Market: "There's nothing Cursor can do that GitHub/Microsoft can't" (nikolay). Another reader: "This changes my perception...But I could never use it because of whose leadership they're under now" (bilalq).
- Praise:
  - "Read after write guarantee on a distributed object store..." (nojvek)
  - "builds on things that work, like leveraging S3" (jillesvangurp)
  - "one of the best technical articles I've read" (iamandoni62986)

**Other critical reviews**

- "I would treat it as an interesting early beta rather than a Git forge I would immediately move an established engineering workflow onto." The reviewer also says much repo management "still happens outside the UI": README creation, branch operations and file edits need an agent, the CLI or git. — [Appwrite review](https://appwrite.io/blog/post/cursor-origin-review-an-engineers-perspective)

### Inferences

- The technical community accepted the storage idea (an append-only log on object storage, rebuildable caches) as sound, and called it well-known event sourcing. The objections were about trust, custody and lock-in, which are exactly the axes ParleyInn's I2, I4 and I6 invariants address.
- The criticism that "S3 is a proprietary product" maps onto ParleyInn. A record transport whose correctness depends on a hosted service's CAS and durability guarantees conflicts with a local-first, no-network design (I4). Origin's pattern carries over to ParleyInn only if the CAS'd log lives locally or in git refs, not in a cloud bucket.

### Gaps

- The Algolia summaries were sampled. I did not read all 454 launch-thread comments, so less-upvoted technical points may be missing.
- No Cursor staff answered the storage-thread criticisms, according to my sampled summary.
- I did not look for Lobsters or Reddit discussion.
