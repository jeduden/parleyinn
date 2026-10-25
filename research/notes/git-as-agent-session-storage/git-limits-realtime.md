# Git as storage and transport for agent session events: limits, retrieval, real time, mutability

Scope: whether git and git hosts (GitHub, Forgejo/Gitea) can hold millions of small
append-only records (tens of GB over time), take frequent appends from many concurrent
writers, and serve retrieval and near-real-time sharing for ParleyInn. Researched 2026-10-01.

## 1. Repository and object limits (GitHub, Forgejo/Gitea, very large repos)

### Takeaway

GitHub enforces only a few hard limits (100 MiB per object, 2 GB per push); everything
that matters for an event log (repo size, branch count, push rate, read rate) is a
*recommendation*, and GitHub's own advice is that repos stay under 1 GB ideally, 5 GB
strongly, 10 GB on disk. A repo that grows to tens of GB from millions of small objects
falls outside GitHub's guidance. Nixpkgs (83 GiB) and the package-manager history show
GitHub stepping in when repos misuse git as a database.

### Cited Findings

- GitHub repository-limits page, **recommendations** (not enforced): on-disk size "10 GB";
  directory width "3,000" entries; directory depth "50"; number of branches "5,000";
  git read operations "15 operations per second per repository"; push rate
  "6 pushes per minute per repository"; open PRs against one branch "1,000"; merge rate
  "1 merged pull request per minute" — [GitHub Docs: Repository limits](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits)
- GitHub **hard, enforced** limits: push size "enforced at 2GB"; single object "enforced at
  100 MB" with a "recommended maximum limit" of 1 MB — [GitHub Docs: Repository limits](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits)
- Other GitHub hard limits relevant to browsing a log repo: Commits tab shows at most 10,000
  commits; compare/PR commit list 250; PR diff 20,000 lines or 1 MB; 300 files per diff;
  100,000 repositories per account/org — [GitHub Docs: Repository limits](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits)
- "We recommend repositories remain small, ideally less than 1 GB, and less than 5 GB is
  strongly recommended." Git warns above 50 MiB per file; GitHub blocks files above
  100 MiB; browser upload cap 25 MiB; larger files need Git LFS — [GitHub Docs: About large files](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)
- GitHub REST API primary limits: 60 req/h unauthenticated; 5,000 req/h authenticated user
  (15,000 on Enterprise Cloud); `GITHUB_TOKEN` in Actions 1,000 req/h per repository
  (15,000 on GHEC); GitHub App installations min 5,000 req/h, up to 12,500 —
  [GitHub Docs: REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- GitHub **secondary** rate limits: at most 100 concurrent requests; 900 points/min for REST;
  90 s CPU per 60 s real time; "No more than 80 content-generating requests per minute and
  no more than 500 content-generating requests per hour"; 2,000 OAuth token requests/h —
  [GitHub Docs: REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- Forgejo configurable limits (defaults): `[repository.upload] FILE_MAX_SIZE` = 50 (MB),
  `MAX_FILES` = 5; `[server] LFS_MAX_FILE_SIZE` = 0 (no limit); `[git.timeout]` DEFAULT 360 s,
  MIGRATE 600 s, MIRROR 300 s, CLONE 300 s, PULL 300 s, GC 60 s; `[api] MAX_RESPONSE_ITEMS` = 50;
  `[ui] MAX_DISPLAY_FILE_SIZE` = 8 MiB — [Forgejo config cheat sheet](https://forgejo.org/docs/latest/admin/config-cheat-sheet/)
- Microsoft Windows repo (2017): "approximately 3.5M files", "repo of about 300GB",
  "over 250,000 reachable Git commits ... over the past 4 months", ~4,000 engineers,
  "8,421 pushes per day (on average)", 4,352 active topic branches. Without GVFS "many of
  the commands would take 30 minutes up to hours and a few would never complete"; with
  GVFS (P80) clone 127 s, status ~10–15 s — [Brian Harry, Microsoft DevBlogs](https://devblogs.microsoft.com/bharry/the-largest-git-repo-on-the-planet/)
- That scale needed a custom virtual file system (GVFS/VFS for Git), later Scalar, plus
  cache servers between repos and developer PCs — [InfoWorld](https://www.infoworld.com/article/3715261/how-microsoft-scales-git-for-massive-monorepos.html); [Visual Studio Magazine](https://visualstudiomagazine.com/articles/2020/02/14/git-scalar.aspx)
- Nixpkgs (Nov 2025): GitHub reported "periodic maintenance jobs on the Nixpkgs repository
  were regularly failing and causing issues achieving consensus between replicas"; repo
  "around 83 GiB" with "around half a million tree objects and 20k forks" (a local bare
  clone is only 2.5 GiB); "Git backend has various bottlenecks around changes to Git refs,
  the total number of refs and trees across the fork network, and the number of PRs".
  A CI job that polled PR mergeability created a new merge commit and `pull/…/merge` ref
  each time and may have driven object growth — [NixOS Discourse](https://discourse.nixos.org/t/nixpkgs-core-team-update-2025-11-30-github-scaling-issues/72709)
- Package managers that used git as a database (Cargo, Homebrew, CocoaPods, vcpkg, Go)
  hit scaling walls and moved to HTTP/CDN designs. Homebrew: unshallowing homebrew-core is
  331 MB, .git near 1 GB, auto-update cut from every 5 min to every 24 h, because "they are
  expensive to git fetch and clone and GitHub would rather we didn't do that". CocoaPods had
  16,000 entries in one Specs directory. 99% of crates.io requests used the sparse HTTP
  protocol by April 2025 — [Andrew Nesbitt, "Package managers keep using git as a database"](https://nesbitt.io/2025/12/24/package-managers-keep-using-git-as-a-database.html); primary: [Rust RFC 2789](https://rust-lang.github.io/rfcs/2789-sparse-index.html), [CocoaPods sharding](https://blog.cocoapods.org/Sharding/)

### Inferences

- A ParleyInn event stream at "millions of small messages, tens of GB" would cross GitHub's
  10 GB on-disk recommendation. It also leans on the costs that hurt Nixpkgs: object
  count, ref churn and server-side maintenance. Nothing hard-blocks it, but GitHub can and
  does intervene on repos that load its backend.
- Each record as its own blob means millions of loose objects until repacked. Batching
  many events into one file per session or segment (JSONL chunks) is the only way to stay
  near GitHub's "1 MB per object" sweet spot while keeping object counts sane.
- Directory width (3,000 entries recommended) rules out a flat layout such as one file per
  session in one folder; a sharded layout like CocoaPods' would be needed.
- Forgejo/Gitea self-hosting removes the GitHub caps, but its default 60 s gc timeout and
  300 s clone/pull timeouts would bite a tens-of-GB repo unless the operator raises them.

### Gaps

- Did not get a sourced figure for Linux kernel object counts, or measured packfile,
  `git gc`/repack, commit-graph or multi-pack-index costs at tens of millions of objects.
- Forgejo/Gitea have no documented per-repo size cap among the keys found. Gitea has had
  a repo size limit feature in some versions; not verified for Forgejo in this pass.
- GitHub publishes no object-count limit.

## 2. Many small appends from concurrent writers (commit/push cost, ref contention, refs, notes)

### Takeaway

GitHub recommends at most 6 pushes per minute per repository, so a single shared repo
cannot take per-event pushes from many agents. Writers to one branch also conflict
(non-fast-forward). One ref per writer avoids that conflict but runs into the 5,000-branch
recommendation and ref-backend costs. Reftable, the default for new repos in Git 3.0,
makes many refs much cheaper locally, but hosts still treat ref churn as a bottleneck.

### Cited Findings

- Push rate recommendation "6 pushes per minute per repository"; branch count
  recommendation "5,000" — [GitHub Docs: Repository limits](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits)
- Microsoft's Windows repo averaged 8,421 pushes/day (~5.8/min averaged over 24 h) across
  4,000 engineers, using Microsoft's own hosting (Azure DevOps/GVFS), not GitHub.com —
  [Brian Harry, Microsoft DevBlogs](https://devblogs.microsoft.com/bharry/the-largest-git-repo-on-the-planet/)
- Content-creating API calls (e.g., creating commits or files via REST) are capped by
  secondary limits at 80/min and 500/h — [GitHub Docs: REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- Reftable benchmarks from the commit that added the backend (git.git, "refs: introduce
  reftable backend"): creating 1,000,000 refs took 51.9 s with reftable vs 152.8 s with the
  files backend; 100,000 refs in one transaction 2.768 s vs 10.085 s (~3.6x). Reftable
  updates two files per transaction where files writes one file per ref; deletions are
  constant time — [git commit 57db2a094d mirror](https://timplab.syktsu.ru/admin/git/commit/57db2a094d) (mirror of git.git; found via search snippet, not fully read)
- Git 2.51: fetch with 10,000 refs is 22x faster on reftable (1.25x on files); push with
  10,000 refs is 18x faster on reftable (1.21x on files), via batched ref updates. Reftable
  "eliminates expensive packed-refs rewrites" and uses geometric compaction. Git 2.51
  "marks the switch to using the 'reftable' format as default in Git 3.0 for newly created
  repositories" — [GitLab: What's new in Git 2.51.0](https://about.gitlab.com/blog/what-s-new-in-git-2-51-0/)
- Git 3.0 had not shipped as of mid-2026; it targets the second half of 2026 with reftable
  and SHA-256 as defaults for new repos — [git-scm BreakingChanges](https://git-scm.com/docs/BreakingChanges.html); [DEV Community summary](https://dev.to/truongandev/whats-new-in-git-in-2026-the-road-to-git-30-30e) (secondary)
- Hosting side: GitHub named "changes to Git refs, the total number of refs and trees
  across the fork network" as backend bottlenecks for Nixpkgs — [NixOS Discourse](https://discourse.nixos.org/t/nixpkgs-core-team-update-2025-11-30-github-scaling-issues/72709)
- git notes: an unsharded notes tree is slow. A 2/38 fanout took traversal "from tens of
  seconds to sub-second", and git auto-fans-out notes trees and loads only the subtrees it
  needs — [git commit 23123aecf8, "Teach the notes lookup code to parse notes trees with various fanout schemes"](https://github.molgen.mpg.de/git-mirror/git/commit/23123aecf8418a6b0ec23378555ed78c438ae894)

### Inferences

- Many agents pushing to one branch will get non-fast-forward rejections, then
  fetch, rebase or merge, and retry. This is standard git semantics, not measured here.
  Append-only data never conflicts at the content level if each writer writes its own
  files, but retries still cost round trips and burn the 6-pushes/min budget.
- Workable patterns: (a) one ref per writer or session, e.g. `refs/parinn/<machine>/<session>`,
  so pushes never contend; (b) batch events locally and push on session end or on a
  timer, never per event; (c) one repo per project or agent group to spread the per-repo
  push budget. Pattern (a) needs pruning or merging of old refs to stay under ~5,000 refs on
  GitHub.
- Custom ref namespaces outside `refs/heads` are not counted as "branches" in GitHub's UI,
  but they still count toward the host's ref load; no source quantifies GitHub's handling
  of them.
- git notes do not help for high-volume appends. They are one more ref (`refs/notes/*`)
  that every writer contends for, with merge semantics on conflict.

### Gaps

- No published per-commit or per-push latency/cost benchmark for small commits was found.
- No GitHub statement on enforcement behaviour when the 6 pushes/min recommendation is
  exceeded (throttling vs. contact from support).

## 3. Retrieval: searching history, partial/shallow clone, side indexes

### Takeaway

Git has no query index. Content search over history (pickaxe, grep over revisions) walks
and diffs every commit. Partial clones trade clone size against on-demand fetches that make
history walks slow. That is why every package manager that used git as a database moved
lookup to an indexed or HTTP layer. For ParleyInn, git could at most be a durable or
transport log, with a local index (such as SQLite FTS5) for recall.

### Cited Findings

- Blobless clone (`--filter=blob:none`) downloads all commits and trees and fetches blobs
  on demand; recommended for developers. Treeless (`--filter=tree:0`) suits throwaway CI;
  "`git log -- <path>` then a treeless clone will start downloading root trees for almost
  every commit". Shallow clones break `git log`/`merge-base`, and "a `git fetch` operation
  in a shallow clone might end up downloading an almost-full commit history"; GitHub does
  "not recommend shallow clones except for builds that delete the repository immediately
  afterwards" — [GitHub Blog: partial clone and shallow clone](https://github.blog/open-source/git/get-up-to-speed-with-partial-clone-and-shallow-clone/)
- vcpkg's git-as-database design hit a structural limit: "there is no way to deduce the
  commit that added a specific tree hash" (git has no reverse index) — [Nesbitt](https://nesbitt.io/2025/12/24/package-managers-keep-using-git-as-a-database.html)
- Go module fetches dropped from 18 minutes to 12 seconds once served from a proxy instead
  of git (Grab engineering, as cited) — [Nesbitt](https://nesbitt.io/2025/12/24/package-managers-keep-using-git-as-a-database.html); primary: [arslan.io](https://arslan.io/2019/08/02/why-you-should-use-a-go-module-proxy/)
- Cargo users saw "Resolving deltas" hang for long periods on the git index. CI was worst,
  since stateless runners downloaded the full index and threw it away every build —
  [Nesbitt](https://nesbitt.io/2025/12/24/package-managers-keep-using-git-as-a-database.html)

### Inferences

- `git log -S/-G` and `git grep <pattern> $(git rev-list --all)` scale linearly with
  history size and need blobs present, which defeats blobless clones. For
  millions of events this is far from the sub-second recall ParleyInn needs. This follows
  from how pickaxe works; no benchmark was found.
- Natural split: git (if used) holds append-only segment files; each machine builds a
  rebuildable local index (fits ParleyInn's I10: derived state rebuildable from the record).

### Gaps

- No sourced benchmark of `git log -S` or `git grep` over large histories.
- Did not fetch git-scm docs on commit-graph changed-path Bloom filters, which speed
  `git log -- <path>` but not content search.

## 4. Real time: git as a live transport

### Takeaway

Git is a poor real-time transport on hosted forges. Pushes are capped at 6/min per repo by
recommendation. Webhooks get a 10-second response window, failed deliveries are not
retried, and no delivery-latency SLA is published. Live sharing needs a separate channel,
with git used, if at all, as the batched durable log.

### Cited Findings

- "GitHub does not automatically redeliver failed deliveries"; "if your server is down or
  takes longer than 10 seconds to respond, GitHub will record the delivery as a failure" —
  [GitHub Docs: Handling failed webhook deliveries](https://docs.github.com/en/webhooks/using-webhooks/handling-failed-webhook-deliveries)
- "Your server should respond with a 2XX response within 10 seconds"; use the
  `X-GitHub-Delivery` header for dedup (redeliveries reuse it); "If your server goes down,
  you should redeliver missed webhooks once your server is back up" — [GitHub Docs: Webhook best practices](https://docs.github.com/en/webhooks/using-webhooks/best-practices-for-using-webhooks)
- Receiving webhooks also needs a publicly reachable HTTP endpoint (implied by the above).
- Forgejo webhooks: `[webhook] QUEUE_LENGTH` 1000, `DELIVER_TIMEOUT` 5 s,
  `PAYLOAD_COMMIT_LIMIT` 15 commits per push event — [Forgejo config cheat sheet](https://forgejo.org/docs/latest/admin/config-cheat-sheet/)
- Polling cost: 15 git read ops/s per repo recommended on GitHub; API 5,000 req/h per user
  — [Repository limits](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits); [REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- Homebrew cut auto-update polling of its git tap from every 5 minutes to every 24 hours
  because git fetches are expensive for GitHub — [Nesbitt](https://nesbitt.io/2025/12/24/package-managers-keep-using-git-as-a-database.html)

### Inferences

- Webhooks need an inbound HTTP endpoint, and receiving them conflicts with ParleyInn's I4
  (no network). Even git push/fetch is network, so git transport would have to live
  outside the I4 core (e.g., the user's own git, invoked by the user, or a separate tool).
- Rough budget: 6 pushes/min per repo, shared across all writers. With 20 agents on one
  repo, each could push about every 3–4 minutes at best. "Near real time" in the
  sub-second sense is not achievable via the host.

### Gaps

- GitHub publishes no webhook delivery latency or SLA; no sourced p50/p99 found.
- The webhook redelivery window (commonly cited as 3 days) and the payload cap (commonly
  cited as 25 MB) were not confirmed on pages fetched in this pass.
- No sourced evaluation of git-sync tools or CRDT-over-git projects was gathered.

## 5. Mutability and privacy: secrets, rewriting history, erasure

### Takeaway

Removing data from a pushed git history is a multi-party, support-ticket process on
GitHub, and often incomplete (forks, clones, cached SHAs, PR refs). GitHub's first advice
is to rotate the secret, not to rewrite. Push protection is on by default for users pushing
to public repos and will block pushes of transcripts that contain recognised secrets.
Append-only, content-addressed history is structurally at odds with erasure requests.

### Cited Findings

- First step: "revoke and/or rotate that secret"; that "may be sufficient". Data persists
  "in any clones or forks of your repository", "directly via their SHA-1 hashes in cached
  views on GitHub" and "through any pull requests that reference them". Collaborators must
  clean their own clones, or "the sensitive data will return". Rewriting changes commit
  hashes, breaks automation, needs branch protections disabled, makes PR diffs inaccessible
  and invalidates signatures. Full removal requires contacting GitHub Support, which
  dereferences PRs, runs server-side GC and removes cached views, and only "in cases where
  we determine that the risk can't be mitigated by rotating affected credentials". Tool:
  `git-filter-repo` ≥ 2.47 with `--sensitive-data-removal` — [GitHub Docs: Removing sensitive data](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)
- Push protection blocks pushes containing detected secrets from the CLI, the web UI, file
  uploads, the REST API and GitHub MCP. "Push protection for users ... is enabled by default"
  for public repositories. Repository-level push protection is off by default and enabled
  by admins. Writers can bypass with a reason ("used in tests", "false positive", "I'll fix
  it later"); orgs can require delegated bypass; custom patterns are supported —
  [GitHub Docs: About push protection](https://docs.github.com/en/code-security/secret-scanning/introduction/about-push-protection)
- Atlassian (Bitbucket) says personal data in git history cannot be removed by its
  standard deletion processes: "The personal data stored within a Git repository is
  essential for auditing and providing a chain of license contribution and authorship".
  It points to filter-branch for anonymisation but "strongly recommend[s] against purging
  the user from Git history" — [Atlassian: Right to erasure in Bitbucket](https://confluence.atlassian.com/bitbucketserver078/right-to-erasure-inbitbucket-server-and-data-center-1037995074.html)
- Erasing author data changes commit hashes; a deleting commit leaves the data reachable by
  hash in every clone and fork — [hoop.dev (secondary)](https://hoop.dev/blog/why-gdpr-compliance-lives-in-your-git-history); GitLab's long-running GDPR erasure issue — [GitLab work item 20994](https://gitlab.com/gitlab-org/gitlab/-/work_items/20994)

### Inferences

- Agent transcripts routinely contain tool output with tokens, env values, file contents
  and personal data. Committing them to a shared or public host risks (a) blocked pushes
  from push protection, which hurts I9 if the agent depends on them, and (b) permanent,
  host-wide exposure that only Support can partly undo. That matters for ParleyInn's redaction
  (I6) and I2/I8 posture.
- Encrypting segments before commit (as some git-based secret stores do) would sidestep
  push protection and public exposure, but erasure then depends on key destruction
  (crypto-shredding), not history rewriting. No source checked here on whether that
  satisfies GDPR.
- Rewriting history to redact one session breaks every other writer's clone and ref, which
  clashes with an append-only, many-writer design.

### Gaps

- No authoritative legal source (EDPB or regulator) on GDPR erasure and git, or on
  crypto-shredding, was found; available sources are vendor or blog opinion.
- GitHub's scan limits for push protection (max push size scanned, timeouts) are not on
  the fetched page.

## 6. Git in ephemeral CI and cloud sandboxes

### Takeaway

Pushing to a branch is a common, sanctioned way for ephemeral agents to get work out of a
sandbox. Claude Code on the web does exactly that through a credential-holding git proxy
restricted to the session's branch. Constraints include scoped credentials, branch
restrictions (claude/* only), the low `GITHUB_TOKEN` API budget, and branch protection on
default branches.

### Cited Findings

- Claude Code on the web: "Inside the sandbox, the git client authenticates to this service
  with a custom-built scoped credential." "The proxy verifies this credential and the
  contents of the git interaction (e.g. ensuring it is only pushing to the configured
  branch)." Credentials "are never inside the sandbox with Claude Code" — [Anthropic Engineering: Claude Code sandboxing](https://anthropic.com/engineering/claude-code-sandboxing)
- A user-reported limitation: the git proxy restricts pushes to `claude/*` branches even
  when a task names another target branch — [claudeissues.com #24535 (community mirror of GitHub issue)](https://claudeissues.com/issue/24535-allow-pushing-to-the-task-assigned-branch-not-just-claude-branches); a 403 on `git-receive-pack` via the proxy was also reported — [claudeissues.com #57829](https://claudeissues.com/issue/57829-bug-git-proxy-returns-403-forbidden-on-git-receive-pack)
- GitHub Actions `GITHUB_TOKEN`: 1,000 REST requests/h per repository — [GitHub Docs: REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- NVIDIA OpenShell documents a pattern for granting scoped GitHub push access to a
  sandboxed agent — [NVIDIA OpenShell docs](https://docs.nvidia.com/openshell/latest/tutorials/github-sandbox.html) (not read in full)

### Inferences

- ParleyInn running in a cloud sandbox cannot assume push rights to arbitrary refs. A proxy
  like Claude Code's may allow only one branch, so a design with one custom ref per writer
  (`refs/parinn/...`) may be rejected there. ParleyInn data riding on the session's own branch
  would mix session records into the code branch and PR, a privacy and noise problem.
- Persisting ParleyInn state out of a sandbox via git would also put ParleyInn on the network
  path, which I4 forbids for shipped code. It would have to be done by the host
  environment or the user's git, not by ParleyInn.

### Gaps

- No official Anthropic doc page on the exact branch-name rules was fetched; the `claude/*`
  restriction comes from community issue mirrors.
- Did not research GitHub branch-protection or ruleset behaviour for custom ref
  namespaces (`refs/parinn/*`), or whether hosts accept pushes to non-`refs/heads` refs
  without extra permissions.
