# Departures from git at scale: who left, what they built, what they learned

Research date: 2026-10-02. Source-quality notes: primary sources are marked as such. Aggregator or vendor sources are flagged inline. Some primary PDFs (the ACM CACM Google paper) returned HTTP 403 to the fetcher, so a few Google numbers come through secondary summaries. Those are flagged.

## Google: Piper, CitC, Fig, and how automation writes to the monorepo

### Takeaway

Google never moved its monorepo to git. It ran one Perforce instance for more than 10 years, then built Piper on Bigtable and later Spanner. Its own paper says git would have forced a split into thousands of repositories. Automated systems commit more than humans: 24,000 bot commits a day against 16,000 human changes (2016). Rosie turns large-scale changes into many small, independently tested and reviewed shards, rather than one giant atomic commit.

### Cited Findings

- Google "relied on a single Perforce instance, using proprietary caching for scalability" for over 10 years. The need to scale further led to Piper. — [Wikipedia: Piper](https://en.wikipedia.org/wiki/Piper_(source_control_system))
- Scale as of 2016:
  - 86 TB of data, two billion lines of code and nine million files, about two orders of magnitude more than the Linux kernel repository.
  - 25,000 developers made 16,000 changes a day, and bots made another 24,000 commit operations a day.
  - Daily read requests are measured in billions.
  - Source: [Wikipedia: Piper, citing Potvin & Levenberg, CACM 59(7), 2016](https://en.wikipedia.org/wiki/Piper_(source_control_system))
- Further 2016 figures: 35 million commits in history, and 15 million lines of code changed weekly. — [AcaWiki summary of Potvin & Levenberg 2016](https://acawiki.org/Why_Google_Stores_Billions_of_Lines_of_Code_in_a_Single_Repository)
- On the git alternative, the paper's summary reads: "Using git would require splitting into thousands of repos and adopting different tools and workflow." Google also invested in "work on mercurial to allow it to support huge monorepos." — [AcaWiki summary](https://acawiki.org/Why_Google_Stores_Billions_of_Lines_of_Code_in_a_Single_Repository)
- Storage: Piper sits on Google's storage, "originally Bigtable and later Spanner." It is distributed across 10 data centers worldwide and replicated through Paxos. — [Wikipedia: Piper](https://en.wikipedia.org/wiki/Piper_(source_control_system))
- CitC (Clients in the Cloud):
  - It uses a cloud backend and a local FUSE filesystem. Together they "create an illusion of changes overlaid on top of a full repository."
  - The average local copy holds "less than ten files."
  - "All file writes are mapped to snapshots."
  - Commits require code review in Critique.
  - Source: [Wikipedia: Piper](https://en.wikipedia.org/wiki/Piper_(source_control_system))
- Large-scale changes (LSCs) and Rosie, from *Software Engineering at Google*, ch. 22 (Hyrum Wright). Source for all points below: [abseil.io SWE book ch. 22](https://abseil.io/resources/swe-book/html/ch22.html)
  - Google has "long ago abandoned the idea of making sweeping changes across our codebase in these types of large atomic changes."
  - "As a codebase and the number of engineers working in it grows, the largest atomic change possible counterintuitively decreases."
  - "Most VCSs have operations that scale linearly with the size of a change … In centralized VCSs, commits can block other writers."
  - "Somebody is always making changes to the repository."
  - Rosie "takes a large change and shards it based upon project boundaries and ownership rules into changes that can be submitted atomically." It then runs each shard through "an independent test-mail-submit pipeline."
  - Rosie "caps the number of outstanding shards for any given LSC, runs at lower priority" because it can be a heavy user of shared infrastructure.
  - It finds reviewers through OWNERS files.
  - Testing uses the "TAP train": batched LSC shards run every affected test, which can take "more than six hours."
  - At the peak of the scoped_ptr→std::unique_ptr migration, Google was "consistently generating, testing and committing more than 700 independent changes, touching more than 15,000 files per day." "Today, we sometimes manage 10 times that throughput."

### Inferences

- Google's answer to a high write rate from bots was not a bigger atomic commit. It was a central, linearised log of many small commits, plus admission control (Rosie's shard caps and lower priority), plus per-shard review and testing. That matches an append-only record with throttled machine writers.
- Commits by automation already outnumbered human commits 1.5:1 in 2016, ten years before the current wave of AI agents. So "machines write more than humans" is a solved workload shape for a centralised, database-backed VCS. Git has not solved it.
- CitC's "write = snapshot" model and the fact that a workspace is an overlay are relevant to agent sandboxes. Every agent edit becomes a durable snapshot without a commit step.

### Gaps

- I could not fetch the primary CACM paper (ACM returned HTTP 403 and the Google PDF mirror returned 404). Two figures are commonly quoted from it but unverified here: about 800,000 queries per second at peak, about 500,000 on average, and "1 billion files" counting history. An aggregator search snippet gave "40,000 commits per workday" and "500,000 QPS".
- Fig (Google's Mercurial-based client for Piper): I found no primary public source in this session. The paper's summary confirms only that Google invested in Mercurial for huge monorepos. Fig's adoption share, its architecture, and why Google chose Mercurial over git for it remain unsourced.
- No public 2025–2026 numbers on the share of Piper commits written by AI agents.

## Meta: Sapling, Mononoke, EdenFS. Why Meta left git and then Mercurial

### Takeaway

In 2012–2014, Meta (then Facebook) rejected git because git status and other operations scale with the total number of files. Git's maintainers advised splitting the repository, and Meta's engineers found git's internals hard to extend. Meta moved to Mercurial and extended it heavily (Watchman, remotefilelog). Those extensions grew into Sapling: a new client, a Rust server (Mononoke) and a virtual filesystem (EdenFS). Meta says public source control systems "were not, and still are not, capable of handling repositories of this size."

### Cited Findings

- In 2013 Facebook's codebase was "many times larger than even the Linux kernel, which checked in at 17 million lines of code and 44,000 files in 2013." — [Meta Engineering, "Scaling Mercurial at Facebook", 2014-01-07](https://engineering.fb.com/2014/01/07/core-infra/scaling-mercurial-at-facebook/)
- Why not git, from the same post: "Git examines every file and naturally becomes slower and slower as the number of files increases." Also, "Git's internals would be difficult to work with for an ambitious scaling project." — [Meta Engineering 2014](https://engineering.fb.com/2014/01/07/core-infra/scaling-mercurial-at-facebook/)
- Why Mercurial, from the same post:
  - It is "written mostly in clean, modular Python … deeply extensible."
  - The community was "actively helping us address our scaling problems."
- Results reported in the same post:
  - With Watchman, Mercurial status became ">5x faster than Git's status."
  - remotefilelog made clones and pulls "10x faster … from minutes to seconds" and cut server network load by more than 10x.
  - The repository saw "thousands of commits a week."
- Sapling (announced 2022-11-15) "started as an extension to the Mercurial open source project, it rapidly grew into a system of its own." Source for this and the points below: [Meta Engineering: Sapling](https://engineering.fb.com/2022/11/15/open-source/sapling-source-control-scalable/)
  - The monorepo has "tens of millions" of files, commits and branches, and supports tens of thousands of developers.
  - "Public source control systems were not, and still are not, capable of handling repositories of this size."
  - Segmented Changelog answers commit-graph ancestry queries in O(number of merges) time.
- Sapling has three components: the `sl` client, Mononoke ("a highly scalable distributed source control server") and EdenFS ("a virtual filesystem for efficiently checking out large repositories"). Sapling is git-compatible on the client side. — [GitHub: facebook/sapling](https://github.com/facebook/sapling)
- Mononoke is "meant to scale up to accepting thousands of commits every hour across millions of files" and is written mainly in Rust. Mononoke "is not yet supported publicly." — [Mononoke README](https://github.com/facebook/sapling/blob/main/eden/mononoke/README.md)
- EdenFS populates working-directory files "only … on demand as they are accessed." — [GitHub: facebook/sapling](https://github.com/facebook/sapling)

### Inferences

- Meta's path ran git (rejected) → Mercurial plus extensions → its own client, server and virtual filesystem. A general-purpose distributed VCS eventually becomes a client protocol in front of a purpose-built server and storage tier.
- Mononoke's target of "thousands of commits every hour" is a useful benchmark for the write rate a server built for this purpose handles. Cursor's Continuity claims 120–300+ pushes per second (see below).
- Meta kept git-compatible clients while replacing the server. This "git as wire format and user interface, not as storage" pattern recurs with Microsoft's GVFS protocol, Cursor's Origin and Hugging Face's Xet.

### Gaps

- I found no public figure for Meta's bot or automation commit rate, such as the share of commits landed by codemods or bots. Search returned nothing primary.
- I found no explicit Meta statement listing why it left upstream Mercurial (for example, Python performance or a divergent fork). The Sapling post only says it "grew into a system of its own."

## Microsoft: GVFS / VFS for Git → Scalar. Why Microsoft stayed on git and what it changed

### Takeaway

Microsoft moved Windows from Source Depot (40+ depots) to a single git repository of about 300 GB and 3.5M files. It could only do so with a virtual filesystem (GVFS), a custom server protocol (the GVFS protocol in Azure Repos) and a fork of git. Later it replaced the virtual filesystem with sparse-checkout, partial clone and background maintenance, and upstreamed this as Scalar in Git 2.38 (October 2022). Microsoft stayed on git by changing git, not by leaving it.

### Cited Findings

- Windows on git (Brian Harry, 2017). Source for all points below: [Microsoft DevBlogs: "The largest Git repo on the planet"](https://devblogs.microsoft.com/bharry/the-largest-git-repo-on-the-planet/)
  - Repository: 3.5M files and about 300 GB, used by about 4,000 engineers.
  - The code was previously spread across 40+ Source Depot depots.
  - Activity: an average of 8,421 pushes a day; 2,500 PRs a day with 6,600 reviewers; 4,352 active branches; 1,760 builds a day across 440 branches.
  - Over 250,000 reachable commits accrued in 4 months.
- Performance before and after GVFS (same source):
  - Before GVFS, commands took "30 minutes up to hours" or never completed.
  - After GVFS, most operations finished in under 20 seconds at the 80th percentile.
  - A clone in North Carolina dropped from 25 minutes to 70 seconds with geographic proxies.
- VFS for Git (previously "GVFS") "was built specifically to transition the Microsoft Windows OS monorepo to Git." It "lazily load[s] files only when a filesystem read occurs" but required the microsoft/git fork and a kernel-level filesystem driver. — [GitHub Blog, "The Story of Scalar", Stolee & Dye, 2022-10-13](https://github.blog/open-source/git/the-story-of-scalar/)
- Pivot to Scalar. Source for all points below: [GitHub Blog: Story of Scalar](https://github.blog/open-source/git/the-story-of-scalar/)
  - Apple deprecated the kernel features VFS for Git needed on macOS.
  - The Office monorepo's dependency system declares the files a build needs, so sparse-checkout can replace the virtual filesystem.
  - The old sparse-checkout pattern matching was quadratic: "Git would take 40 minutes to evaluate the sparse-checkout patterns" for a large definition. Microsoft added directory-only "cone mode."
  - With fsmonitor, `git status` ran "within three or four seconds."
  - Azure Repos still speaks the older GVFS protocol rather than standard partial clone.
- Scalar ships in Git v2.38 as "a built-in repository manager for large repos." It configures partial clone, sparse-checkout, background maintenance and advanced config options. — [GitHub Blog: Story of Scalar](https://github.blog/open-source/git/the-story-of-scalar/)

### Inferences

- Microsoft's case is the counterexample: git can host a 300 GB, 3.5M-file monorepo with about 8,400 pushes a day. It needed years of core-git work (commit-graph, multi-pack-index, partial clone, sparse index, fsmonitor), a proprietary server protocol and a fork. The work went into making reads lazy and partial. The write path (refs, pushes) was not fundamentally redesigned.
- For a single-tenant tool like ParleyInn, Microsoft's lesson is that reaching git's scale features takes a lot of engineering and heavy client configuration. Its numbers (pushes a day) are also small next to agent-era write rates (pushes a second).

### Gaps

- Microsoft's reasons for choosing git over other systems (for example, Mercurial or a Perforce upgrade) are not stated in the fetched posts.
- No current (2025–2026) figures on Windows repository size or push rate.

## Game studios and binary-heavy workloads: Perforce Helix Core and Unity Version Control (Plastic SCM)

### Takeaway

Large game studios mostly use Perforce, not git. Their assets are large binaries that cannot be merged and need exclusive locks. Projects reach terabytes, and teams want partial sync of a central depot rather than full local history. Git LFS mitigates the size problem, but it does not give enforced locking or a centralised workflow by default.

### Cited Findings

- Perforce offers "exclusive file locking that is critical for binary assets that can't be three-way-merged." Textures and audio "can't be merged." — [Diversion: Perforce vs Git for Game Development (vendor)](https://www.diversion.dev/knowledge-center-articles/perforce-vs-git-for-game-development); also [Sourcegraph: Perforce vs Git](https://sourcegraph.com/blog/perforce-vs-git)
- "Exclusive locking is a requirement for working with editable but not mergeable binary files in a team environment, and it needs to be integrated and enforced by the VCS." — [80.lv: game studios moving beyond legacy version control](https://80.lv/articles/game-studios-are-moving-beyond-legacy-version-control-and-here-s-why/) (industry press, vendor-adjacent)
- Perforce "stores massive binary depots natively and checks out only what you sync — terabyte projects work without ceremony." "Even with Git LFS, repositories can become slower and more complex as projects grow." — [bugnet.io: Perforce vs Git for game development](https://bugnet.io/blog/perforce-vs-git-for-game-development) (vendor blog)
- One heuristic: for teams larger than 10 to 15 people with asset libraries past 50 GB, Perforce is worth evaluating. — [bugnet.io](https://bugnet.io/blog/perforce-vs-git-for-game-development) (vendor opinion)

### Inferences

- The binary-asset failure mode is different from ParleyInn's workload. Agent session records are small, append-only text or JSON. They are not large unmergeable binaries. What carries over is the centralised model (a server-authoritative history plus partial sync), not locking.

### Gaps

- Sources here are vendor or industry-press opinion pieces, not studio engineering posts. I found no primary studio post (for example, Epic, Ubisoft or EA) with numbers in this session.
- Unity Version Control (Plastic SCM): I found no fetched source on its design rationale or scale.

## Package managers that used git as a database and moved off it

### Takeaway

Cargo, Homebrew, CocoaPods and Hugging Face's client all started by making users clone a git repository as the metadata database. All four moved to plain HTTP/CDN fetches of individual files or JSON. The full-history clone grew without bound, every user paid for every package's history, and the host (GitHub) bore the fetch load. Go went straight to a module proxy plus a checksum database built as a verifiable, append-only Merkle log. vcpkg is the holdout: it still uses git.

### Cited Findings

- **Cargo / crates.io**
  - Rust 1.68 (2023-03-09) stabilised the "sparse" protocol at index.crates.io. The git protocol "clones a repository that indexes all crates available in the registry, but this has started to hit scaling limitations." Sparse "will only download information about the subset of crates that you actually use." It was planned as the default in 1.70. — [Rust Blog: Rust 1.68.0](https://blog.rust-lang.org/2023/03/09/Rust-1.68.0/)
  - Index size: the content is 176 MiB as an uncompressed tarball and 16 MiB gzipped, while a git clone downloads 215 MiB, "over 20 times more than a compressed tarball." Typical resolution time dropped from 2–4 minutes to under 10 seconds. — [Cargo team HackMD: help test the new index protocol](https://hackmd.io/@rust-cargo-team/rkm1ttkns)
  - Each index file is newline-delimited JSON with one line per published version. — [Cargo Book: Registry Index](https://doc.rust-lang.org/cargo/reference/registry-index.html)
- **Homebrew**
  - Homebrew 4.0.0 (2023-02-16) replaced "Git-cloned taps" with "JSON files downloaded from formulae.brew.sh." Maintainer Mike McQuaid called it "the largest change we have made to our update process since we split Homebrew/brew and Homebrew/homebrew-core." — [Homebrew 4.0.0 release](https://brew.sh/2023/02/16/homebrew-4.0.0/)
  - Auto-update now runs "every 24 hours rather than every 5 minutes" and no longer needs "the slow git fetch of the huge homebrew/core and homebrew/cask" repositories. Users can `brew untap homebrew/core`. — [Homebrew 4.0.0](https://brew.sh/2023/02/16/homebrew-4.0.0/)
- **CocoaPods**
  - CDN support arrived in 1.7, was finalised in 1.7.2, and became the default spec source in 1.8 (2019-08-05). "CocoaPods no longer requires cloning the now huge master specs repo." The source changed from `https://github.com/CocoaPods/Specs.git` to `https://cdn.cocoapods.org/`. — [CocoaPods Blog: 1.8 Beta](https://blog.cocoapods.org/CocoaPods-1.8.0-beta/)
  - Users had reported the problem as early as March 2016: "Issues Cloning Spec repo – GitHub taking a very long time to download changes to the Specs Repo." — [CocoaPods issue #4989](https://github.com/CocoaPods/CocoaPods/issues/4989)
  - The CocoaPods blog currently carries a banner: "CocoaPods trunk is moving to be read-only … there are 10 months to go." — [CocoaPods Blog](https://blog.cocoapods.org/CocoaPods-1.8.0-beta/)
- **Go modules**
  - The module mirror (proxy.golang.org), index and checksum database launched as production defaults for Go 1.13 on 2019-08-29. — [Go Blog: Module Mirror and Checksum Database Launched](https://go.dev/blog/module-mirror-launch)
  - The checksum database is a global, tamper-evident log of go.sum lines. Clients check "'inclusion' proofs (that a specific record exists in the log) and 'consistency' proofs (that the tree hasn't been tampered with)." The server exposes signed tree heads (`/lookup`) and tree "tiles" (`/tile`). Third-party auditors can iterate the log. — [Go Blog](https://go.dev/blog/module-mirror-launch)
  - The mirror can also "serve source code that is no longer available from the original locations." — [Go Blog](https://go.dev/blog/module-mirror-launch)
- **Hugging Face client**: in June 2022 (huggingface_hub v0.8.1), the HTTP Commit API (`create_commit()`) removed the need for git and git LFS installs. "We were no longer building a Git wrapper for transformers; we were building purpose-built infrastructure for machine learning artifacts." — [HF Blog: huggingface_hub v1.0](https://huggingface.co/blog/huggingface-hub-v1)
- **vcpkg**: has not moved off git. The curated registry at github.com/Microsoft/vcpkg "is an implementation of a Git registry." Git registries need a `versions/` directory and a `versions/baseline.json`. — [Microsoft Learn: vcpkg registries](https://learn.microsoft.com/vcpkg/concepts/registries)

### Inferences

- The recurring flaw is a full-history clone as the client's database. Every consumer downloads every package's full history to answer "what is the latest version of X". Every fix had three parts:
  1. Per-key files fetched over HTTP.
  2. A CDN in front.
  3. An append-only per-key format: Cargo's newline-delimited JSON adds one line per version.
- Go's checksum database is the template closest to ParleyInn's needs. It is an append-only Merkle transparency log with signed tree heads and tile-based retrieval. It gives tamper evidence without git, and its integrity proofs need no network peer. The same structure works locally.
- These moves were driven mainly by read and fetch load: clone size and host CPU. Write rate was not the driver. Package registries publish only thousands of versions a day.

### Gaps

- I could not retrieve the commonly cited 2016 GitHub-engineer comments about CocoaPods and Homebrew shallow fetches burning GitHub CPU. The fetched issue page did not contain them, and the GitHub API was not enabled for those repositories in this session, so no numbers are quoted.
- I found no vcpkg statement on clone size or performance pain, nor any plan to leave git.

## Data versioning: Hugging Face git LFS → Xet, Oxen.ai, DVC, lakeFS

### Takeaway

Git LFS deduplicates whole files, so any edit re-uploads a multi-gigabyte file in full. Hugging Face bought XetHub in August 2024. It replaced LFS storage with content-defined chunking (about 64 KB chunks packed into about 64 MB blocks, held in S3 behind a content-addressed store) and migrated all 77 PB+ across more than 6 million repositories without users noticing. Oxen.ai built a git-like VCS from scratch on Merkle trees. DVC and lakeFS keep git-like semantics but put the data outside git.

### Cited Findings

- LFS "deduplicates at the file level. Even tiny edits create a new revision to upload in full." Xet "uses content-defined chunking (CDC) … at the level of bytes (~64KB chunks)." — [HF Blog: Xet on the Hub](https://huggingface.co/blog/xet-on-the-hub)
- Example: an internal ~5 GB SQLite database gets about 1 MB appends. An upload takes about "a tenth of a second (at 50Mb/s) instead of the 13 minutes if the database were backed by LFS." — [HF Blog: Xet on the Hub](https://huggingface.co/blog/xet-on-the-hub)
- Xet architecture. Source: [HF Blog: Xet on the Hub](https://huggingface.co/blog/xet-on-the-hub)
  - The client chunks data and dedups locally, then aggregates chunks into ~64 MB blocks.
  - A content-addressed store (CAS) enforces chunk dedup and provides an LFS bridge for old clients.
  - S3 stores the blocks and the "shards" of reconstruction metadata.
- First migration, on February 20 (2025): 4.5 TB moved, shifting "~6% of the Hub's download traffic." It used rollback tooling to return repositories to LFS if needed. — [HF Blog: Xet on the Hub](https://huggingface.co/blog/xet-on-the-hub)
- Post-migration issue: the CAS downloaded "four times more data than it was returning to clients." The cause was 10 MB range requests from hf_transfer that did not align with block boundaries. — [HF Blog: Xet on the Hub](https://huggingface.co/blog/xet-on-the-hub)
- Scale: "starting with 20 petabytes across over 500,000 repositories … One year later, all 77PB+ over 6,000,000 repositories have been migrated to the Xet backend … with no user intervention." — [HF Blog: huggingface_hub v1.0](https://huggingface.co/blog/huggingface-hub-v1)
- Hub files are "Large — model or dataset files are in the range of GB and above. We have a few TB-scale files!" and binary (Safetensors, Parquet). — [HF Docs: Xet](https://huggingface.co/docs/hub/xet)
- Timeline: XetHub was acquired in August 2024, Xet became the default for new users in May 2025, and the full migration finished around November 2025. — [AIM summary](https://analyticsindiamag.com/ai-features/hugging-face-is-replacing-git-lfs-with-xet-storage-heres-why) (secondary; dates not checked against a primary source)
- Oxen.ai's position (vendor) is that git "isn't suited to version data" and that LFS is "like trying to fill a swimming pool with a straw … tied to the limitations of the git protocols." Oxen is "built taking inspiration from Git" using "Merkle trees, smart network protocols and fast hashing," aimed at millions of files. — [Oxen.ai blog: Best AI data version control tools](https://ghost.oxen.ai/the-best-ai-data-version-control-tools/); [Oxen docs: 1M files benchmark](https://docs.oxen.ai/features/performance)
- Same vendor source on alternatives: DVC (2017) stores file contents outside git but "leaves collaboration primitive." lakeFS offers git-like semantics over petabytes in object storage, but "you have to handle file storage yourself." — [Oxen.ai blog](https://ghost.oxen.ai/the-best-ai-data-version-control-tools/) (competitor's view; biased)

### Inferences

- Hugging Face kept git refs and commits as the user-facing model but moved bytes into chunked, content-addressed object storage on S3. This is the same split Cursor's Origin makes: a git-shaped interface with storage that is not git.
- Content-defined chunking matters most for append-heavy large files, such as Hugging Face's own SQLite example. An append-only session log stored as one growing file would hit exactly this LFS failure mode. Segmented or chunked storage avoids it.

### Gaps

- No primary DVC or lakeFS engineering source was fetched. The characterisations come from a competitor.
- I found no independent benchmark of Oxen against git, LFS or Xet.

## AI-agent era (2025–2026): agent workloads overwhelming git hosts

### Takeaway

GitHub's CTO said publicly in April 2026 that agentic workflows, which "accelerated sharply" from late December 2025, forced GitHub to raise its capacity target from 10x (October 2025) to 30x (February 2026). Peaks reached 1.4B commits, 90M merged PRs and 20M new repositories a month. Git storage, MySQL, Actions, search and webhooks all buckled. Availability suffered through 2026. In August 2026 Cursor launched Origin, git hosting built for agents. It uses a write-ahead log in S3 instead of consensus-replicated on-disk repositories and claims 120–300+ pushes per second.

### Cited Findings

- GitHub CTO Vlad Fedorov, 2026-04-28. Source for all points below: [GitHub Blog: "An update on GitHub availability"](https://github.blog/news-insights/company-news/an-update-on-github-availability/)
  - "Agentic development workflows have accelerated sharply" since the second half of December 2025.
  - Growth was exponential across repository creation, PR activity, API usage and large-repository workloads.
  - Peaks shown: 90M PRs merged, 1.4B commits and 20M new repositories a month.
  - GitHub started a 10x capacity plan in October 2025. By February 2026 it needed to "design for … 30X today's scale."
  - Stressed subsystems: Git storage, MySQL, branch protection, Actions, Elasticsearch search, notifications, permissions, webhooks and caching.
  - "Small inefficiencies compound: queues deepen, cache misses become database load, indexes fall behind."
  - Mitigations: moved webhooks off MySQL, isolated Git and Actions from other workloads, accelerated the Ruby→Go migration, used the Azure migration for more compute, and began multi-cloud.
  - Incidents: on April 23 a merge-queue regression hit 658 repositories and 2,092 PRs. On April 27 Elasticsearch overloaded.
- The Register, 2026-06-12. Source for all points below: [The Register: "GitHub outages persist as AI coding drives traffic surge"](https://www.theregister.com/software/2026/06/12/github-outages-persist-as-ai-coding-drives-traffic-surge/5255125)
  - GitHub moved from about 1 billion commits a year (2025) to about 1.4 billion a month.
  - SVP Jakub Oleksy: "We're now serving 40 percent of monolith traffic from Azure (up from 8 percent in February)." Git traffic was 30% on Azure, repository replication 99%, and "We've more than doubled our effective capacity in four months."
  - May 2026: GitHub reported 9 incidents, while its status history lists 23.
  - The unofficial "Missing GitHub Status Page" project measured 90-day uptime at 87.26%, with April at 78.33%.
  - The Register points to [GitHub Availability Report, May 2026](https://github.blog/news-insights/company-news/github-availability-report-may-2026/).
- August 17–18, 2026: a global GitHub outage of about 7 h 47 min, attributed to a capacity shortage rather than a code change. — [BigGo Finance](https://finance.biggo.com/news/8384c465-f827-420f-97c0-4a4761db5e31); [Kraviona](https://kraviona.com/blog/github-down-august-2026-outage-what-happened) (secondary; not checked against GitHub's own report)
- Lower-confidence aggregator claims: about 275 million commits a week; Actions minutes rising from 500M in 2023 to 2.1B in one week in early 2026; 257 incidents in 12 months. — [techlogstack](https://techlogstack.com/explore/github-ai-load-30x-architecture-2026/) (aggregator; unverified)
- **Cursor Origin / Continuity** ("Git at any scale", 2026-08-18). Source for all points below: [Cursor Blog: Git at any scale](https://cursor.com/blog/git-at-any-scale)
  - Motivation: "Agents … made this situation worse. More code, more PRs, more CI runs." Also: "A company can grind to a halt if its developers cannot push or pull."
  - History: GitHub tried NFS ("Git makes a lot of assumptions about filesystem semantics"), then block replication (GFS/DRBD: "terrible to operate"), then RPC remotes. It settled on Spokes (about 2013): plain git on NVMe with three-phase commit across replicas, "a push is only accepted if a majority of the nodes acknowledge it."
  - Spokes' weakness: "The latency of every step is bound by the slowest of all the servers … The more replicas you add … the worse push throughput gets." Repositories are "pets, not cattle," with a routing database and constant checksums.
  - Spokes cannot fit both extremes. Monorepos need more than 3 replicas, while "millions of throwaway agent repos" waste mandatory triple replication.
  - Continuity design: S3 is the source of truth through a per-push write-ahead log. "We never acknowledge a push until it has been fully persisted." Disk repositories are "a warm cache." There is no consensus: "Any server can be the primary," with WAL updates through atomic compare-and-swap on S3.
  - Replicas check freshness with ETag conditional GETs (a 304 takes about 10 ms) and use UDP gossip for hints.
  - Only the primary repacks. Replicas download compacted packs.
  - Performance: "up to 120 pushes/s while compacting and replicating" on S3 Standard, and "more than 300 pushes/s" on S3 Express One Zone, bottlenecked by git compaction. Reads scale linearly to 100 replicas.
  - Consistency: "The Git client really doesn't play well with eventual consistency." "We linearize all pushes."
- Origin launched as a beta for paid Cursor plans. — [Cursor's Origin coverage, The Register 2026-08-23](https://www.theregister.com/devops/2026/08/23/how-cursor-beat-gits-scalability-shortcomings/5291421)

### Inferences

- The agent-era bottleneck at hosts is the server-side write path and all the indexing around it, not git's client data model. Push consensus, ref updates, repacking, webhooks, search indexes and MySQL metadata all strain. GitHub's own list of stressed subsystems is mostly derived indexes. That supports I10-style thinking in ParleyInn: keep one append-only log authoritative and make every index rebuildable from it.
- Cursor kept git's wire protocol but rebuilt storage as a linearised write-ahead log (WAL) on object storage. This is a strong signal that an append-only log with compaction is the scalable core, and git is a compatibility view on top of it.
- Agents create many tiny, short-lived repositories (GitHub saw 20M new repositories a month, and Cursor names "throwaway agent repos"). A design that assumes long-lived repositories pays fixed per-repository overhead each time.

### Gaps

- GitHub has not published a per-subsystem breakdown separating git-storage load from agent-specific traffic, or a share of commits by agent identity.
- Cursor's pushes-per-second figures are self-reported, with no independent benchmark.
- I found no 2025–2026 primary report of an individual large repository (as opposed to a host) collapsing under agent commit volume. Reports of agent PR floods on open-source projects were not researched in depth here.

## Recurring failure modes of git at scale, and the replacement pattern that worked

### Takeaway

Six failure modes recur, and each has a known replacement. Almost every successful system keeps something git-shaped at the edge but stores history in a centralised, append-only, linearised log or database. Indexes are derived from that log, and clients fetch lazily or by key.

### Cited Findings

- **Working-tree and file-count scaling (status or checkout is O(files))**
  - Meta found "Git examines every file … slower and slower as the number of files increases." — [Meta 2014](https://engineering.fb.com/2014/01/07/core-infra/scaling-mercurial-at-facebook/)
  - Windows commands took "30 minutes up to hours" before GVFS. — [MS DevBlogs](https://devblogs.microsoft.com/bharry/the-largest-git-repo-on-the-planet/)
  - Replacement: virtual or overlay filesystems (CitC with under 10 local files, EdenFS, VFS for Git) or sparse-checkout plus fsmonitor (Scalar). — [Wikipedia: Piper](https://en.wikipedia.org/wiki/Piper_(source_control_system)); [Story of Scalar](https://github.blog/open-source/git/the-story-of-scalar/)
- **Full-history clone as the database (object count and history size)**
  - The crates.io git clone is 215 MiB against a 16 MiB gzipped tarball. — [Cargo HackMD](https://hackmd.io/@rust-cargo-team/rkm1ttkns)
  - CocoaPods' "now huge master specs repo." — [CocoaPods](https://blog.cocoapods.org/CocoaPods-1.8.0-beta/)
  - Replacement: per-key HTTP files behind a CDN (Cargo sparse, Homebrew JSON, CocoaPods CDN), or lazy object fetch (remotefilelog, partial clone, the GVFS protocol).
- **Write contention and push throughput (replica consensus, ref updates)**
  - Spokes' 3PC is bound by the slowest replica, so throughput falls as replicas are added. — [Cursor](https://cursor.com/blog/git-at-any-scale)
  - Google notes centralized commits "can block other writers." — [SWE book ch. 22](https://abseil.io/resources/swe-book/html/ch22.html)
  - Replacement: a linearised write-ahead log with compare-and-swap on durable storage (Cursor Continuity), Paxos-replicated database storage (Piper on Bigtable/Spanner), and a server built for high commit rates (Mononoke).
- **Large atomic changes and monorepo-wide edits**
  - "The largest atomic change possible counterintuitively decreases" as a codebase grows. — [SWE book ch. 22](https://abseil.io/resources/swe-book/html/ch22.html)
  - Replacement: sharded small commits with admission control (Rosie caps outstanding shards and runs at low priority) and batched testing (TAP train).
- **Large binary files**
  - LFS re-uploads whole files. Studios need locks on assets that cannot be merged. — [HF Xet blog](https://huggingface.co/blog/xet-on-the-hub); [Diversion](https://www.diversion.dev/knowledge-center-articles/perforce-vs-git-for-game-development)
  - Replacement: content-defined chunking into a content-addressed store on object storage (Xet), or a centralised depot with exclusive locks (Perforce).
- **Query and index needs (ancestry, search, metadata)**
  - Meta built Segmented Changelog for O(merges) ancestry queries. — [Sapling](https://engineering.fb.com/2022/11/15/open-source/sapling-source-control-scalable/)
  - GitHub's load fell on search, MySQL, permissions and webhooks: "indexes fall behind." — [GitHub availability update](https://github.blog/news-insights/company-news/an-update-on-github-availability/)
  - Replacement: purpose-built indexes derived from the log, and moving hot metadata off the primary store (webhooks off MySQL).
- **Integrity and verifiability without git's hash chain**
  - Go's checksum database is a Merkle transparency log with inclusion and consistency proofs and signed tree heads. — [Go Blog](https://go.dev/blog/module-mirror-launch)

### Inferences

- For ParleyInn, where sessions are small, appended at a high rate and need tamper evidence, the evidence points away from a git repository as the session store:
  - The patterns that survived scale are an append-only segmented log, content-addressed chunks, a Merkle or transparency-log structure for integrity (as in Go's sumdb), and indexes rebuildable from the log.
  - Git can remain an optional export or compatibility view, which is what Meta, Microsoft (GVFS protocol), Hugging Face and Cursor all do.
- The cleanest precedents for "machines write more than humans" are Google (24k bot commits a day against 16k human changes in 2016, absorbed by a centralised, Paxos-replicated store plus Rosie's throttled sharding) and Cursor Continuity (a WAL on object storage, 120–300 pushes a second).
- Moving ParleyInn's own source-code version control off git (as opposed to session history) has much weaker support. The organisations that left git ran repositories with millions of files and tens of thousands of engineers. Microsoft shows git stretching to 300 GB and 8,400 pushes a day. A small Go repository edited by agents is far below any documented git limit. The pain at GitHub is host-side capacity, not repository-format limits.

### Gaps

- No source quantifies the ref-churn failure mode alone (millions of branches or refs per repository) with numbers. Cursor and GitHub discuss it only implicitly through push throughput and repack cost.
- I found no published measurement of git behaviour under sustained machine-rate commits (for example, more than 10 commits a second) to one local repository, which is ParleyInn's single-machine case.
