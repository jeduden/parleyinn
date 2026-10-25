# Git interface over custom storage and transport (as of October 2026)

Scope: systems that keep git as the client-facing interface (wire protocol, smart HTTP, SSH, or a git remote helper) while owning storage and/or transport underneath. Written to feed ParleyInn (Go, security-first, append-only agent-session record, invariant I4 = no network in shipped code, crypto-shredding under consideration).

## 1. Server-side engines that speak git but store differently

### Takeaway

2026 produced a wave of "git-compatible, agent-scale" backends (Cursor Origin/Continuity, Entire, Cloudflare Artifacts, Pierre Code.Storage, GitLab next-gen SCM, ERSC) that all keep the git wire protocol and replace the storage layer with an object-store log, a sharded/regional store, or Durable Objects; none of them document encryption at rest with customer-held keys or a crypto-erasure story. The only systems in scope with client-held keys are Keybase git (dormant) and git-remote-gcrypt; the decentralized ones (Radicle, Tangled, NIP-34) explicitly do not encrypt at rest.

### Cited Findings

**Cursor Origin / Continuity (proprietary, beta)**

- Continuity is the storage engine behind Cursor's Origin git hosting; write-up by Vicent Martí, August 18, 2026 — [Cursor blog](https://cursor.com/blog/git-at-any-scale)
- Storage model: a write-ahead log in S3-compatible object storage is the source of truth; each node also keeps "a normal Git repository stored on a very fast NVMe drive" that is a warm cache rebuildable from the log — [Cursor blog](https://cursor.com/blog/git-at-any-scale); [DevClass](https://www.devclass.com/devops/2026/08/24/how-cursor-beat-gits-scalability-shortcomings/5291479)
- Each push = packfile + reference transaction; "We never acknowledge a push until it has been fully persisted"; pushes are linearizable via atomic compare-and-swap on S3 — [Cursor blog](https://cursor.com/blog/git-at-any-scale)
- Replicas: optimistic replication via UDP gossip; replicas check S3 with conditional GET (ETag); 304 = up to date, 200 = catch up from the WAL index. Claim: "Every view of every repository we access is always fully consistent." — [Cursor blog](https://cursor.com/blog/git-at-any-scale)
- Missing repos are materialized on demand from the WAL using rendezvous hashing; only the primary compacts, replicas download pre-compacted packs from S3 ("trading bandwidth for CPU") — [Cursor blog](https://cursor.com/blog/git-at-any-scale)
- Performance claims: up to 120 pushes/s per repo on S3 Standard, >300 pushes/s on S3 Express One Zone; reads scale linearly across up to 100 tested replicas — [Cursor blog](https://cursor.com/blog/git-at-any-scale)
- Contrasted with GitHub Spokes: no quorum/three-phase commit, scales from one replica, repos are "a warm cache" not "pets" — [Cursor blog](https://cursor.com/blog/git-at-any-scale); [DevClass](https://www.devclass.com/devops/2026/08/24/how-cursor-beat-gits-scalability-shortcomings/5291479)
- Beta available on paid Cursor plans (cursor.com/origin); Cursor described as a SpaceX subsidiary — [DevClass](https://www.devclass.com/devops/2026/08/24/how-cursor-beat-gits-scalability-shortcomings/5291479); [Dealroom summary](https://app.dealroom.co/news/feed/cursor-tackles-git-scalability-with-s3-backed-architecture)
- Encryption, deletion/erasure and licensing are not addressed; not open source — [Cursor blog](https://cursor.com/blog/git-at-any-scale)

**GitLab next-generation SCM (private beta)**

- Announced at GitLab Transcend, blog dated June 10, 2026: "a different motor underneath, designed for agentic access from the start", keeping "same Git compatibility and auditability"; targets the "clone tax", concurrency collapse under agent load, and lack of isolation between agents — [GitLab blog](https://about.gitlab.com/blog/gitlab-transcend-announcements/)
- Claims (early internal results): "up to 2x fewer tokens", "up to 50x faster wall clock time", "up to 1,000x less network traffic"; status private beta / early-access request — [GitLab blog](https://about.gitlab.com/blog/gitlab-transcend-announcements/)
- No storage, encryption, or Gitaly-relationship details published in that post — [GitLab blog](https://about.gitlab.com/blog/gitlab-transcend-announcements/)
- Note: the brief mentions "Aug 2026"; the primary announcement found is June 10, 2026. I found no August 2026 GitLab primary source on it.

**Gitaly / Praefect (GitLab's production engine, MIT/Go)**

- Praefect is "a router and transaction manager for Gitaly"; components: load balancer, Praefect nodes, PostgreSQL for metadata, multiple Gitaly nodes — [GitLab docs](https://docs.gitlab.com/administration/gitaly/praefect/)
- Strong consistency "writes changes synchronously to all healthy, up-to-date replicas"; otherwise falls back to asynchronous replication after the primary write; documented limitations include Geo incompatibility and PostReceiveHook races — [GitLab docs](https://docs.gitlab.com/administration/gitaly/praefect/)
- Replacement in progress: epic "Raft-based decentralized architecture for Gitaly Cluster" aims to remove Praefect and Postgres, adds ACID transactions with snapshot isolation and a write-ahead log, with Raft replicating the WAL; epic still open, last recorded progress June 2024 (transactions in staging, Raft PoC started) — [GitLab epic 8903](https://gitlab.com/groups/gitlab-org/-/epics/8903)
- No encryption-at-rest feature documented for Praefect — [GitLab docs](https://docs.gitlab.com/administration/gitaly/praefect/)

**GitHub Spokes (formerly DGit, proprietary)**

- "three copies of every repository, on three different servers", replicated at the application layer "via Git protocols"; writes "are only committed if at least two replicas confirm success"; repos on local SSD rather than SAN/distributed FS to keep DAG-walk latency low; announced April 2016 — [GitHub blog](https://github.blog/engineering/infrastructure/introducing-dgit/)
- Automatic healing re-replicates under-replicated repos when a server leaves — [GitHub blog](https://github.blog/engineering/infrastructure/introducing-dgit/)

**Entire distributed git network (preview, waitlist)**

- Founded by ex-GitHub CEO Thomas Dohmke; preview mirrors an existing GitHub repo onto Entire, agents "clone and pull from a regional Entire mirror"; regions US, EU, Australia — [GeekWire](https://geekwire.com/2026/former-github-ceos-startup-entire-unveils-its-answer-to-the-crush-of-ai-coding-agents); [SiliconANGLE, July 8 2026](https://siliconangle.com/2026/07/08/ex-github-chiefs-entire-opens-distributed-git-network-agent-era/)
- Architecture: "global control plane for identity and placement, and regional data planes for content-addressed Git storage"; Smart-HTTP; "Ref updates remain protected by compare-and-set semantics, while object writes fan out across storage nodes"; writes replicated across multiple AZs with repair/catch-up flows; data can be pinned to one region or several; branches under `entire/unmirrored/` stay local to their push region — [Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network)
- Ships a git remote helper in the Entire CLI for `entire://` URLs, e.g. `entire://aws-us-east-2.entire.io/gh/OWNER/REPO` — [Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network)
- Performance claim "up to 25x" over competitors such as Cursor Origin (vendor claim, early testing); benchmark tool ForgeMark is MIT-licensed — [SD Times](https://sdtimes.com/softwaredev/startup-entire-launches-distributed-git-network-for-the-agent-era/); [Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network)
- Stated future plans: "decentralize and open source our Git network", "support for self-hosted nodes"; encryption and deletion not documented — [Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network)
- Date conflict: press coverage dated July 8, 2026 ([SiliconANGLE](https://siliconangle.com/2026/07/08/ex-github-chiefs-entire-opens-distributed-git-network-agent-era/)) vs. blog post dated August 28, 2026 with an editorial-modification note ([Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network)).

**Pierre Code.Storage (managed, proprietary)**

- "API-first Git infrastructure layer"; repos created programmatically; SDKs for TypeScript, Python and Go — [code.storage docs](https://code.storage/docs); [search summary of code.storage](https://code.storage/)
- Standard git over HTTPS with JWT auth; remote URL `{org}.code.storage/{repo}`; ephemeral branches (isolated refs, promotable to default namespace); `+import` URL with "immediate cold archival"; git LFS on the same remote; webhooks "real-time HTTP POST notifications for push and repo sync events, verified with HMAC signatures"; can "create commits without git" — [code.storage llms-full.txt](https://code.storage/llms-full.txt)
- Launch post Oct 14, 2025: "distributed, quorum-based" git layer; claim "~60x faster than similar S3/R2 based solutions"; raised $23M led by CRV — [code.storage changelog](https://code.storage/changelog/introducing-code-storage); [code.storage](https://code.storage/)
- Encryption, deletion, replication internals not documented in fetched pages — [code.storage llms-full.txt](https://code.storage/llms-full.txt)

**Cloudflare Artifacts (beta)**

- Launched private beta April 16, 2026 ("public beta by early May"); each repo has its own history, a stable HTTPS remote, separate read and write tokens — [Cloudflare blog](https://blog.cloudflare.com/artifacts-git-for-agents-beta/)
- Storage: Durable Objects with SQLite backend; objects in the DO KV store, chunked over rows (2 MB row limit); R2 for snapshots; KV for auth tokens; ~128 MB memory per DO — [Cloudflare blog](https://blog.cloudflare.com/artifacts-git-for-agents-beta/)
- Git implemented in Zig compiled to WASM (~100 KB, no deps); git protocol v1 and v2, ls-refs, shallow (deepen, deepen-since, deepen-relative), have/want negotiation; git-notes for agent metadata; raw deltas persisted with base hashes — [Cloudflare blog](https://blog.cloudflare.com/artifacts-git-for-agents-beta/)
- ArtifactFS: blobless clone + FUSE mount, 2.4 GB repo mounts in 10–15 s vs ~2 min — [Cloudflare blog](https://blog.cloudflare.com/artifacts-git-for-agents-beta/)
- Pricing $0.15/1,000 ops (10k free), $0.50/GB-month (1 GB free); Go and Python SDKs "planned" — [Cloudflare blog](https://blog.cloudflare.com/artifacts-git-for-agents-beta/)
- "nearing 50,000 new repositories per day"; repos are meant to be kept or deleted when work is done — [Cloudflare product page](https://www.cloudflare.com/products/artifacts/)
- Deletion semantics, retention, encryption specifics not in the docs index — [Cloudflare docs](https://developers.cloudflare.com/artifacts/)

**ERSC — East River Source Control (early trial)**

- Building a server layer that "speaks git and 'jj'", custom storage engine for "thousands of commits per second, instant checkouts, and support for arbitrarily sized files", "fine-grained ACLs", first-class conflicts, inspired by Google/Meta code hosting; early trial with select partners, no date or pricing — [ERSC blog](https://ersc.io/blog/ersc-availability)
- Jujutsu creator Martin von Zweigbergk joined as CTO; CEO Benjamin Brittain; backed by Amplify Partners — [ERSC](https://ersc.io/); [byteiota](https://byteiota.com/git-hits-its-ai-ceiling-ersc-bets-on-whats-next/)
- jj is Apache-2.0, Rust, reads/writes git's on-disk format — [byteiota](https://byteiota.com/git-hits-its-ai-ceiling-ersc-bets-on-whats-next/)
- Date conflict: CTO announcement reported as July 8, 2026 ([ecosistemastartup](https://ecosistemastartup.com/?p=103676)) vs September 2, 2026 ([byteiota](https://byteiota.com/git-hits-its-ai-ceiling-ersc-bets-on-whats-next/)); ERSC site says "as of September 2026" ([ERSC](https://ersc.io/)).

**Meta Mononoke**

- Server component of Sapling SCM, Rust; "used in production within Meta it is not yet supported for external usage" — [Sapling README](https://cdn.jsdelivr.net/gh/facebook/sapling@main/README.md)
- Source tree contains `git` and `servers` directories under `eden/mononoke`, plus blobstore/packblob storage docs — [GitHub facebook/sapling](https://github.com/facebook/sapling/tree/main/eden/mononoke)

**Google (Piper and git bridges)**

- JGit's `storage.dfs` layer "has been in production at Google for many months" as a git storage backend over a distributed store; the earlier `storage.dht` (HBase/Cassandra/Bigtable) design was dropped for poor performance — [JGit commit history mirror](https://git.jakstys.lt/motiejus/jgit/commit/fa4cc2475fb127783e98ddb56b6c1fd155cd2bc4); [JGit DHT notes](https://code.fanruan.com/github/jgit/commits/commit/18e020218b210ef5867bb07ff8f2b889e467d7fc?page=16)

**Keybase encrypted git (KBFS) — dormant**

- End-to-end encrypted: "not even Keybase can see what's in there (nor its name, the filenames...)"; device private keys "never leave your device"; personal and team repos; `keybase://team/<team>/<repo>`; team repos auto-lock to prevent concurrent overwrites — [Keybase book](https://book.keybase.io/git)
- Remote helper built on go-git over a Keybase crypto storage layer that "doesn't really understand git"; server knows team membership, who pushes/fetches, devices, repo IDs; cannot see repo names, branch names, contents; 100 GB per user/team; launched Oct 4, 2017 — [Keybase blog](https://keybase.io/blog/encrypted-git-for-everyone)
- Status: Zoom acquired Keybase in May 2020 — [Decrypt](https://decrypt.co/28121/keybase-users-revolt-following-zoom-acquisition); latest client release on GitHub is v6.6.3, June 3, 2024 (no release in the ~28 months since); repo not marked archived — [keybase/client releases](https://github.com/keybase/client/releases). Treat as unmaintained/dormant in 2026.

**Radicle heartwood (open source, MIT/Apache-2.0)**

- Peer data stored in one git repo per project using git namespaces keyed by Node ID; all refs signed into `refs/rad/sigrefs`; gossip of node/inventory/ref announcements, git fetch multiplexed over the same connection; seed nodes — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Collaborative Objects (issues, patches, identities) stored as git commit DAGs inside the repo, converging under concurrent edits; delegates with a configurable signature threshold define the canonical default branch — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Encrypted in transit (Noise) "not at rest"; private repos rely on selective replication to an allow-list; `git-remote-rad` provides `rad://` — [Radicle protocol guide](https://radicle.dev/guides/protocol)

**Tangled knots (AT Protocol)**

- Knots: "lightweight, headless servers" hosting git repos, single- or multi-tenant; appview at tangled.org aggregates — [Tangled docs](https://docs.tangled.org/)
- Git over SSH (sshd `AuthorizedKeysCommand` → `knot keys`, `knot guard` authorizes by owner/member/collaborator) and HTTP via reverse proxy to port 5555; repos on plain filesystem under `KNOT_REPO_SCAN_PATH` organised by user DID; WebSocket `/events` stream for live updates to the appview; secure mode with Landlock + UID isolation; Knot 2 seals per-repository signing keys with a master key — [Tangled knot guide](https://docs.tangled.org/knot-self-hosting-guide.html)
- Repo metadata lives as `sh.tangled.repo` records (name, knot, description) and SSH keys as `sh.tangled.publicKey` records in users' PDS — [tangled-mcp architecture notes](https://glama.ai/mcp/servers/@zzstoatzz/tangled-mcp/blob/d96d9cbdb3eb939dd2965ba5b8110cd04b3cf18b/docs/architecture.md); [Tangled intro](https://blog.tangled.org/intro)

**ngit / NIP-34 relays**

- "Git repositories are hosted in Git-enabled servers", relays carry coordination: kinds 30617 (repo announcement, with `clone` and `relays` tags), 30618 (repo state: branches/tags), 1617 patches, 1618/1619 PRs, 1621 issues, 1630–1633 status, 10317 grasp server prefs; `nostr://<npub|nip05>/<relay-hint>/<identifier>` clone URLs — [NIP-34](https://github.com/nostr-protocol/nips/blob/master/34.md)
- GRASP servers = "git server + nostr relay"; ngit v3.0.3, MIT, added private repos and CI — [ngit.dev](https://ngit.dev/)

### Inferences

- The 2026 agent-scale engines converge on one pattern: immutable log or content-addressed store as truth, CAS on refs for multi-writer safety, local git repo as a disposable cache. This matches ParleyInn's append-only record + rebuildable projections (I10) closely; Continuity is the most explicit published design to borrow from.
- None of the commercial engines publish customer-held keys or crypto-erasure; deletion is per-repo lifecycle (Cloudflare, Code.Storage "cold archival") at best. ParleyInn would have to build per-key encryption itself.
- Content-addressed git objects make per-record erasure hard: deleting an object means rewriting every descendant commit and GC. Crypto-shredding fits better if encryption sits below git (Keybase/gcrypt style, encrypting packs/blocks), not inside git objects.

### Gaps

- GitLab next-gen SCM: no storage/engine design or an "August 2026" GA source found; only the June 10, 2026 private-beta announcement.
- Google git-on-Piper bridges (git5, Git-on-Borg): no primary source fetched; CACM 2016 paper returned 403. Only JGit DFS in production at Google is sourced.
- Mononoke serving git clients: only directory evidence; no published doc on its git server's protocol coverage or production use.
- Cloudflare/Code.Storage/Entire/Cursor encryption-at-rest and deletion semantics: not documented in fetched sources.
- ERSC: no technical architecture or licence published.

## 2. Git remote helpers that map git onto other stores

### Takeaway

Remote helpers are the lightest way to keep `git push/fetch` while owning storage: git spawns `git-remote-<scheme>` and talks a line protocol over stdio, so no listening socket is required. gcrypt and Keybase are the only ones with client-held keys; S3 bundles rely on provider-side SSE/KMS; IPLD is archived; nostr/rad/entire use helpers to reach their networks.

### Cited Findings

- **git-remote-gcrypt** (`gcrypt::` URLs): each packfile encrypted with its own symmetric key; manifest (refs, pack metadata, Remote ID) signed and encrypted with GPG to listed `gcrypt-participants`; backends local, rsync, sftp, rclone (experimental), any git remote; rsync/sftp upload entire history per push; "Every git push effectively has `--force`. Be sure to pull before pushing."; unpredictable repacking; GPL-3.0; maintained by Sean Whitton since 2016 — [GitHub spwhitton/git-remote-gcrypt](https://github.com/spwhitton/git-remote-gcrypt)
- **awslabs/git-remote-s3**: stores git bundles at `<prefix>/<ref>/<sha>.bundle`; per-ref locking with 60 s TTL (`GIT_REMOTE_S3_LOCK_TTL_SECONDS`), losers get "stale remote" and must fetch/retry; "encrypted at rest and in transit by default", optional customer-managed KMS keys; LFS at `<prefix>/lfs/<oid>`; Python 3.9+, Apache-2.0 — [GitHub awslabs/git-remote-s3](https://github.com/awslabs/git-remote-s3)
- **git-remote-ipld**: git objects as IPLD nodes with SHA-1 CIDs, `ipld://<hash>` URLs, IPNS for mutable refs never finished; archived July 31, 2026, "never left the experimental stage"; MIT — [GitHub ipfs-shipyard/git-remote-ipld](https://github.com/ipfs-shipyard/git-remote-ipld)
- **git-remote-nostr** (ngit): `nostr://` clone URLs resolve repo announcements and signed state (kind 30618) then fetch from listed git servers/GRASP — [NIP-34](https://github.com/nostr-protocol/nips/blob/master/34.md); [ngit.dev](https://ngit.dev/)
- **git-remote-rad**: `rad://` scheme onto the Radicle node — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- **git-remote-entire**: helper in Entire CLI for `entire://` regional endpoints — [Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network)
- **git-remote-keybase**: `keybase://` helper, go-git based, E2E encrypted — [Keybase blog](https://keybase.io/blog/encrypted-git-for-everyone); [Keybase book](https://book.keybase.io/git)

### Inferences

- A ParleyInn-owned `git-remote-parinn` helper speaking to a local store over stdio would keep git as the code interface without any `net` import, preserving I4; network sync would be a separate, opt-in component.
- gcrypt's per-pack key + encrypted manifest is the closest existing design to per-key crypto-shredding: destroying a pack key renders that pack unreadable. But repacking merges packs, so key granularity has to be designed so repack never re-encrypts shredded data under a live key.
- gcrypt's implicit force-push and S3 helper's lock-TTL show the multi-writer weak point of helper-over-dumb-store designs; a CAS on a ref log (as Continuity and Entire do) is the safer primitive.

### Gaps

- No current source checked for other helpers (git-remote-codecommit, git-remote-dropbox, git-remote-bigquery, etc.).
- git-remote-gcrypt latest release date not captured.

## 3. Go libraries for building a git interface

### Takeaway

go-git (Apache-2.0) is the main pure-Go option; v6 exposes server-side `UploadPack`/`ReceivePack` decoupled from transports and supports protocol v2, so it can serve git over a pipe/stdio. soft-serve, gitkit and Gitea all shell out to the `git` binary (an `os/exec` dependency, which ParleyInn's I4 bans in shipped code).

### Cited Findings

- go-git v5 `plumbing/transport/server`: `NewServer(loader)`, `DefaultServer`, `Loader`, `MapLoader` serve upload-pack/receive-pack over any storer; no direct `net`/`net/http` import listed; v5.19.2 published July 29, 2026; highest major is v6 — [pkg.go.dev v5 server](https://pkg.go.dev/github.com/go-git/go-git/v5/plumbing/transport/server)
- go-git v6 `plumbing/transport`: abstractions `Conn`, `Connector`, `Transport`, `Session`; protocol v0/v1 and v2; server functions `UploadPack()`, `ReceivePack()`, `UploadArchive()`, `AdvertiseRefs()`/`AdvertiseCapabilities()`; stateless RPC for HTTP; hooks; transports SSH, HTTP, git TCP, file live in their own subpackages — [pkg.go.dev v6 transport](https://pkg.go.dev/github.com/go-git/go-git/v6/plumbing/transport)
- Keybase's git helper is built on go-git with a custom crypto storage layer, a proof of go-git over encrypted custom storage — [Keybase blog](https://keybase.io/blog/encrypted-git-for-everyone)
- soft-serve (Charm): SSH (with TUI), HTTP(S), git daemon; requires `git` installed (shells out); SQLite default, PostgreSQL optional; repos in `data` dir; access levels no-access/read-only/read-write/admin; server hooks pre-receive/update/post-update/post-receive; MIT; ~7.2k stars — [GitHub charmbracelet/soft-serve](https://github.com/charmbracelet/soft-serve)
- gitkit: Go smart-HTTP server, SSH server, hook receiver (`IsForcePush()`, `ReadCommitMessage()`); shells out to git (`git_command.go`); MIT; ~315 stars — [GitHub sosedoff/gitkit](https://github.com/sosedoff/gitkit)
- Gitea: built-in SSH server (`START_SSH_SERVER`), requires git, repos on local filesystem under `ROOT`; only LFS can use MinIO/S3/Azure blob — [Gitea config cheat sheet](https://docs.gitea.com/administration/config-cheat-sheet)
- Git protocol v2: command-oriented (`ls-refs`, `fetch`, `object-info`), capability advertisement; "stateless by default... permits simple round-robin load-balancing"; same protocol over git://, HTTP (`Git-Protocol: version=2` header), SSH/file (`GIT_PROTOCOL` env); `filter` for partial clone; `packfile-uris` lets a server hand off packs to HTTP(S) URLs (CDN); `object-format` negotiates hash; `server-option` passes custom options — [git-scm protocol-v2](https://git-scm.com/docs/protocol-v2)

### Inferences

- For ParleyInn, the I4-compatible route is go-git v6 core + a custom `storer` (encrypted, append-only) + server `UploadPack`/`ReceivePack` over stdio from a remote helper, importing no `net/http` or `ssh` transport subpackages. Whether the v6 root `transport` package's dependency closure is free of `net` must be checked with `go list -deps` against ParleyInn's import-closure test.
- `packfile-uris` and `server-option` are protocol-level hooks a custom server could use for out-of-band encrypted pack delivery or passing a session/key id without breaking stock git clients.
- Adding go-git counts against ParleyInn's ten-direct-dependency budget and needs an ADR (ENG-18, ENG-26).

### Gaps

- Not verified whether go-git v6 is tagged stable or still pre-release, nor the exact import closure of its server path.
- Forgejo internals not fetched (assumed similar to Gitea; unverified).
- Gitea's use of go-git vs git binary internally not confirmed from a primary source beyond "requires Git".

## 4. Cross-cutting: encryption and keys, multi-writer, propagation, licence and maturity, deletion

### Takeaway

Encryption with user-held keys exists only in Keybase (dormant) and git-remote-gcrypt (GPL, single-writer-ish); multi-writer safety in modern engines comes from compare-and-set on refs over a log; real-time propagation is via gossip (Continuity, Radicle), WebSocket streams (Tangled) or webhooks (Code.Storage); no system documents crypto-erasure.

### Cited Findings

- Encryption at rest, client-held keys: Keybase (device/team keys, server blind to names and contents) — [Keybase blog](https://keybase.io/blog/encrypted-git-for-everyone); gcrypt (GPG participants, per-pack keys) — [git-remote-gcrypt](https://github.com/spwhitton/git-remote-gcrypt)
- Encryption at rest, provider keys: git-remote-s3 via S3 SSE / optional KMS — [git-remote-s3](https://github.com/awslabs/git-remote-s3)
- No encryption at rest: Radicle — [Radicle protocol guide](https://radicle.dev/guides/protocol); Tangled knots store plain repos on disk (only signing keys sealed) — [Tangled knot guide](https://docs.tangled.org/knot-self-hosting-guide.html)
- Multi-writer: Continuity linearizes via S3 CAS — [Cursor blog](https://cursor.com/blog/git-at-any-scale); Entire CAS on refs — [Entire blog](https://entire.io/blog/an-entirely-new-git-hosting-network); Spokes 2-of-3 commit — [GitHub blog](https://github.blog/engineering/infrastructure/introducing-dgit/); Praefect sync to healthy replicas, async fallback — [GitLab docs](https://docs.gitlab.com/administration/gitaly/praefect/); Keybase team-repo locking — [Keybase book](https://book.keybase.io/git); Radicle per-peer namespaces + delegate threshold — [Radicle](https://radicle.dev/guides/protocol); gcrypt implicit force push — [gcrypt](https://github.com/spwhitton/git-remote-gcrypt)
- Real-time propagation: UDP gossip + ETag checks (Continuity) — [Cursor blog](https://cursor.com/blog/git-at-any-scale); gossip ref announcements (Radicle) — [Radicle](https://radicle.dev/guides/protocol); WebSocket `/events` (Tangled) — [Tangled knot guide](https://docs.tangled.org/knot-self-hosting-guide.html); HMAC-signed webhooks (Code.Storage) — [code.storage](https://code.storage/llms-full.txt); nostr relays (NIP-34) — [NIP-34](https://github.com/nostr-protocol/nips/blob/master/34.md)
- Maturity snapshot: production — Spokes, Gitaly/Praefect, Mononoke (Meta-internal); beta/preview — Cursor Origin, GitLab next-gen SCM, Cloudflare Artifacts, Entire; early trial — ERSC; launched product — Code.Storage; dormant — Keybase; archived — git-remote-ipld (July 31, 2026). Sources as cited above.
- Licences: proprietary — Origin, Entire network (ForgeMark MIT), Artifacts, Code.Storage, GitLab next-gen SCM (unstated); open — Radicle MIT/Apache-2.0, ngit MIT, gcrypt GPL-3.0, git-remote-s3 Apache-2.0, soft-serve MIT, gitkit MIT, go-git Apache-2.0, jj Apache-2.0. Sources as cited above.

### Inferences

- For crypto-shredding with a git interface, the workable layering is: git objects (plaintext to the client) → custom storer that encrypts per record/session under a per-key DEK → append-only log. Shredding the DEK makes the encrypted blocks unreadable, but any git object (tree/commit) that references the shredded content keeps its hash, so clones show dangling references; tools must tolerate missing objects (promisor/partial-clone semantics in protocol v2 are the nearest standard mechanism).
- GPL-3.0 (gcrypt) is likely off ParleyInn's licence allow-list; its design can be reimplemented, not vendored.
- Every network-facing engine here conflicts with I4 as shipped code; borrowing designs (WAL + cache, CAS refs, per-pack keys) is compatible, embedding their servers is not.

### Gaps

- No source found describing crypto-erasure or GDPR-style erasure for any git-compatible engine in scope.
- No independent benchmarks; all performance figures are vendor claims.
