# Git-native collaboration data stores and their Go tooling (as of 2026-10)

Scope: how git-bug, git-appraise, git notes, Radicle, Fossil, Forgejo/Gitea and a handful of
git-like systems model append-only, multi-writer data, to inform ParleyInn's candidate design of
"per-origin append-only log segments stored/transported via git, plus a local SQLite index".
Local working copies of git-bug and go-git sources inspected in this session are under
`scratchpad/gnc/`.

## git-bug: data model, multi-writer sync, cache/index, bridges, scale

### Takeaway

git-bug is the closest prior art to ParleyInn's design. It stores each entity as a hash-chained DAG of
git commits under `refs/<namespace>/<id>`. Each commit carries one JSON "operation pack" blob plus
Lamport clocks encoded in tree-entry names, and can be OpenPGP-signed. Concurrent edits become
merge commits, and the DAG is replayed in a deterministic order. Search goes through a rebuildable
on-disk excerpt cache plus a bleve full-text index under `.git/git-bug/`. It is pure Go on top of
go-git.

### Cited Findings

- Entities are stored as a series of edit `Operation`s, explicitly modelled on operation-based CRDTs, rather than as current state. The final state is "compiled" by replaying the operations on an empty state. — [git-bug data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md)
- An Operation holds a type id, an author reference, a wall-clock timestamp, one or two Lamport times, type-specific data and a random nonce. The nonce gives entropy because "the operation identifier is a hash of that data". — [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md)
- Operations are grouped into an `OperationPack`, which "represents an edit session". It is stored as a git Blob holding a JSON array, with the author stored once per pack. A Tree references the blob under `ops` and optional media blobs under `media`. A Commit references the Tree, and each new pack adds a commit to the chain. — [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md)
- The commit chain is published as the ref `refs/<namespace>/<id>`, and a git push carries all reachable data including media. The bug entity namespace is `"bugs"`, so refs live at `refs/bugs/<id>`. — [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md); [entities/bug/bug.go](https://github.com/git-bug/git-bug/blob/master/entities/bug/bug.go)
- Wall-clock time is used "just for display". Ordering uses Lamport clocks. The first commit of an entity carries both a create clock and an edit clock, and later commits carry only an edit clock. The values are encoded in tree entry names such as `create-clock-14` and `edit-clock-137`, each pointing at the empty blob `e69de29…`, so they cost no network transfer. — [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md)
- IDs: `id = hash(json(op))`, and an entity's id is the hash of its first operation as serialized on disk. In code, an operationPack id is derived from the exact JSON bytes written. — [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md); [entity/dag/operation_pack.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/operation_pack.go)
- Merge model:
  - A fast-forward pull or push simply moves the ref.
  - On concurrent edits, git-bug "creates the equivalent of a merge commit". The result is a DAG with a single root.
  - Ordering algorithm: (1) load all commits and packs; (2) check that Lamport clocks respect the DAG, refusing or discarding any commit whose parent has a clock greater than or equal to its own; (3) order by Lamport edit clock, then by the lexicographic order of the OperationPack id for concurrent packs, which is "unbiased and hard to abuse".
  - "This - coupled with signed commits - has the nice property of limiting how this data model can be abused."
  - Sources: [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md)
- Anti-abuse in code: reading rejects "lamport clock ordering doesn't match the DAG". It also rejects clocks "jumping too far in the future, likely an attack", which guards against pushing clocks toward uint64 rollover. Accepted clocks are then "witnessed" into the local clock. — [entity/dag/entity.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/entity.go)
- Signing: if the author identity has a signing key, the pack commit is stored via `StoreSignedCommit` with an OpenPGP entity. On read, the signature is checked with `openpgp.CheckDetachedSignature` (ProtonMail go-crypto). — [entity/dag/operation_pack.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/operation_pack.go)
- Each operation pack has exactly one author, and validation fails if an operation's author differs from the pack author. — [operation_pack.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/operation_pack.go)
- Identities are themselves entities, each a series of `Version`s stored in git. A legacy "Bare" identity was embedded directly in operations, which is a superseded design. — [doc/design/architecture.md](https://github.com/git-bug/git-bug/blob/master/doc/design/architecture.md)
- Sync refspecs: fetch uses `refs/<ns>/*:refs/remotes/<remote>/<ns>/*`, which leaves local state unchanged. Push uses `refs/<ns>/*:refs/<ns>/*`. Pull is fetch plus `MergeAll`, which merges each remote-tracking entity into the local ref. — [repository/gogit.go](https://github.com/git-bug/git-bug/blob/master/repository/gogit.go); [entity/dag/entity_actions.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/entity_actions.go)
- Local storage: the git-bug side store lives in `<repo>/.git/git-bug`, with subdirectories `clocks` (persisted Lamport clocks) and `indexes` (bleve). — [repository/gogit.go](https://github.com/git-bug/git-bug/blob/master/repository/gogit.go)
- Cache layer:
  - It keeps loaded entities in memory, one instance per entity, "avoiding loss of data that we could have with multiple copies in the same process".
  - It maintains "in memory and on disk a pre-digested excerpt for each bug" for fast query, filter and sort.
  - It "lock[s] the git repository for its own usage, by writing a lock file"; plain git operations are unaffected.
  - Sources: [architecture.md](https://github.com/git-bug/git-bug/blob/master/doc/design/architecture.md); [cache/repo_cache.go](https://github.com/git-bug/git-bug/blob/master/cache/repo_cache.go)
- Index consistency check: the cache records, per entity, the commit that its excerpt and bleve document were "builtFrom". At load it compares the index's `BuiltFrom` against the excerpts and treats any mismatch as an error ("mismatch between bleve and … excerpts"), which triggers a rebuild. The rebuild wipes the index and re-indexes in batches of 75, a number chosen experimentally. The comment's benchmark table shows a bug index of about 26 MB built in about 1.3–1.5 s for its test set. — [cache/subcache.go](https://github.com/git-bug/git-bug/blob/master/cache/subcache.go)
- Full-text search uses bleve v2.6.1 with a static mapping and an English analyzer. The git layer is go-git v5.19.2, and the module targets Go 1.26. — [go.mod](https://github.com/git-bug/git-bug/blob/master/go.mod); [repository/index_bleve.go](https://github.com/git-bug/git-bug/blob/master/repository/index_bleve.go)
- git-bug still imports `golang.org/x/sys/execabs`, used here to locate a text editor. The only use seen in the inspected code is editor lookup, not git. — [repository/gogit.go](https://github.com/git-bug/git-bug/blob/master/repository/gogit.go)
- Bridges: git-bug can "import from and export to Github, Gitlab, Jira and Launchpad" via `git bug bridge pull/push`, and it works offline. Other surfaces are a GraphQL API (used by the embedded web UI), a CLI built on cobra, and a terminal UI built on gocui. — [README](https://github.com/git-bug/git-bug/blob/master/README.md); [architecture.md](https://github.com/git-bug/git-bug/blob/master/doc/design/architecture.md)
- The data model is generic. `entity/dag` is a reusable package, and the repository ships an example of defining your own entity type. — [data-model.md](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md); [entity/dag/example_test.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/example_test.go)
- Scale, historical and probably superseded: a git-bug issue reported `git bug ls` taking about 25–29 s for 1,000 bugs and 2–4 s for 100. The cause was that each bug was fully compiled because the title was not in the cache. The maintainer measured fully reading 10k bugs at about 40 s. This predates the excerpt and bleve cache described above. — [git-bug issue mirror "git bug ls should be faster"](https://git.secluded.site/git-bug/bug/f8e9e6cfda83ab8dc958d4211d6c699d8ab61594aa6beac0c1012f72526cd65c)

### Inferences

- git-bug's layout maps almost one-to-one onto "per-origin append-only segments":
  - one ref per entity, which ParleyInn could make one ref per (origin, session);
  - one commit per append batch;
  - the content hash chain comes free from git parent links;
  - per-pack signatures.
- The main difference is that git-bug lets many writers converge on a single ref through merge commits. ParleyInn's per-origin refs would avoid merge commits entirely, because each ref has exactly one writer, and leave the total-ordering problem to the index.
- git-bug's "excerpt plus index, built from commit X, verified on load, wiped and rebuilt on mismatch" is a working precedent for ParleyInn's I10, where everything derived is rebuildable from the record.
- One ref per entity means ref count grows with entity count. ParleyInn creating one ref per session per origin would hit the same many-refs scaling issues as Gerrit; see the git notes section and reftable.

### Gaps

- No current (2025–2026) benchmark of git-bug at 10k–100k entities using the bleve cache was found. The only numbers found are the old pre-cache issue and the batch-size table in source.
- The latest git-bug release tag and date could not be confirmed: the GitHub API was blocked in this session.

## git-appraise: reviews in git notes, cat_sort_uniq merging

### Takeaway

git-appraise stores code-review metadata as one JSON object per line in git notes under
`refs/notes/devtools/*`. Notes are merged with git's `cat_sort_uniq` strategy, so concurrent
writers' lines are unioned, sorted and de-duplicated. "Current state" comes from the latest
timestamped line. It is a simple, server-less multi-writer design. Order and causality are
weak: timestamps are trusted, and there is no hash chain beyond the notes ref history.

### Cited Findings

- Storage refs:
  - `refs/notes/devtools/reviews` holds review requests, annotating the first revision.
  - `refs/notes/devtools/discuss` holds human comments.
  - `refs/notes/devtools/ci` holds CI results.
  - `refs/notes/devtools/analyses` holds robot or static-analysis comments.
  - Source: [git-appraise README](https://github.com/google/git-appraise)
- Data is JSON with "at most one [item] per line," which enables automatic merging. — [git-appraise README](https://github.com/google/git-appraise)
- Sync uses `git appraise push [<remote>]` and `git appraise pull [<remote>]`. Pull merges notes with the `cat_sort_uniq` strategy. — [git-appraise README](https://github.com/google/git-appraise)
- Multiple requests on one commit are "sorted by timestamp and the final request is treated as the current one". — [git-appraise README](https://github.com/google/git-appraise)
- The repository shows about 5.3k stars, 148 forks, 26 open issues and 6 PRs. The README and repository page give no sign of recent active development, so this is a weak signal of reduced maintenance. — [GitHub repo page](https://github.com/google/git-appraise)

### Inferences

- `cat_sort_uniq` is a grow-only set of lines (G-Set) under lexical order. It converges, but lexical sorting destroys append order, so any order must be rebuilt from in-line timestamps or sequence numbers.
- It works for small annotation records keyed by a commit. It is a poor fit for high-volume event streams: a whole note blob is rewritten on each append, and the merged result is a full re-sort.
- Lines that are not byte-identical never de-duplicate, so any non-canonical serialization, such as differing JSON key order, produces duplicates.

### Gaps

- No primary source was found on git-appraise's last release or commit date in 2025–2026, or on documented scale limits.

## git notes in general: layout, fanout, merge strategies, push/fetch, problems

### Takeaway

A notes ref is an ordinary commit history whose tree maps annotated-object ids to note blobs,
using a 2-hex-digit directory fanout. Notes are not pushed or fetched by default and need
explicit refspecs. They have five merge strategies, of which only `union` and `cat_sort_uniq` are
automatic and lossless. Notes are known to be awkward at scale and in UX.

### Cited Findings

- The default ref is `refs/notes/commits`. Each notes change creates a new commit on the notes ref, so the history can be inspected with `git log -p notes/commits`. — [git-notes(1)](https://git-scm.com/docs/git-notes)
- Fanout: a note's path is "a sequence of directory names of two hexadecimal digits each followed by a filename with the rest of the object ID" (e.g. `bf/fe/30/…/680d5a…`), "for performance reasons". — [git-notes(1)](https://git-scm.com/docs/git-notes)
- Merge strategies:
  - `manual` (default): conflicts are checked out to `.git/NOTES_MERGE_WORKTREE` and finished with `--commit` or `--abort`.
  - `ours` and `theirs`: keep one side.
  - `union`: concatenates both sides.
  - `cat_sort_uniq`: concatenates, sorts lines and removes duplicates.
  - Configuration: `notes.mergeStrategy` and `notes.<name>.mergeStrategy`, or `git notes merge -s`.
  - Source: [git-notes(1)](https://git-scm.com/docs/git-notes)
- Notes are not synced by default. They need explicit refspecs such as `git push origin refs/notes/*:refs/notes/*`, or a configured `fetch = +refs/notes/*:refs/notes/*`. — [git-notes(1)](https://git-scm.com/docs/git-notes)
- On rewrite (amend, rebase), notes are copied only when `notes.rewriteRef` is set. `notes.rewriteMode` can be overwrite, concatenate (the default), cat_sort_uniq or ignore. — [git-notes(1)](https://git-scm.com/docs/git-notes)
- Real-world scaling pain:
  - GHC stores performance metrics in git notes, and a merge request titled "Work around perf note fetch failure" proposed a treeless partial-clone filter to fetch note blobs lazily. The MR page itself was blocked by an anti-bot wall; only the title and search snippet were seen.
  - Separately, a tool with 800+ notes timed out because it loaded every note at startup, and the fix was lazy loading.
  - Sources: [GHC MR 11391 (title/snippet only)](https://gitlab.haskell.org/ghc/ghc/-/merge_requests/11391); [codeberg memo PR #9](https://codeberg.org/antoniorodr/memo/pulls/9)
- Many-refs scaling in general:
  - Git 3.0 will default to the reftable ref backend, which "fixes longstanding issues that cannot be fixed with the 'files' format". One of those issues: deleting a ref under the files backend rewrites the whole `packed-refs` file, which "can be dozens of megabytes or even gigabytes".
  - A reported benchmark at 10,000 refs shows git-fetch 22x faster and git-push 18x faster with reftable, against 1.25x and 1.21x for the files backend.
  - Sources: [LWN on Git 3.0](https://lwn.net/Articles/1029364/); [git commit announcing the reftable switch](https://timplab.syktsu.ru/admin/git/commit/d0b94577dda3a50c1833626a70ebefd478bfcbf9); [cybersecuritynews on Git 2.51](https://cybersecuritynews.com/git-2-51-released/amp/). The benchmark is from a secondary news source.

### Inferences

- Notes suit "one small mutable annotation per existing commit". ParleyInn's events are not annotations on code commits. Using notes would add the fanout tree rewrite and a full blob rewrite per append, with no gain over a plain ref-plus-commits chain like git-bug's.
- If ParleyInn uses git transport, it needs custom refspecs anyway, the same as notes. Non-default refs such as `refs/parinn/*` are invisible to ordinary `git clone`, which helps keep the record out of normal code repositories.

### Gaps

- No quantitative benchmark of git notes at 100k+ notes was found.
- The details of the GHC failure could not be read because of the access wall.

## Radicle (Heartwood): COBs, per-peer namespaces, signed refs, gossip

### Takeaway

Radicle stores every peer's view of a repository in one bare git repository, with the peer's
Ed25519 Node ID as a git namespace. Issues and patches are Collaborative Objects (COBs): commit
DAGs under `refs/cobs/<typename>/<id>`. Sync takes the union of peers' DAGs and reduces them in
topological order, which makes merging CRDT-like and conflict-free. Authenticity comes from a
per-peer `rad/sigrefs` commit that signs a snapshot of all of the peer's refs. Near-real-time
spread comes from signed gossip "reference announcements" that trigger git-protocol fetches.
In March 2026 Radicle disclosed a replay flaw in sigrefs, which is directly relevant to any
"signed ref snapshot" design.

### Cited Findings

- COBs are identified by a reverse-DNS type name plus an Object ID. Built-in types are `xyz.radicle.issue`, `xyz.radicle.patch` and `xyz.radicle.id` (the identity document). Each COB is a commit DAG kept apart from source branches. — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Merge: "When the histories of two peers are synchronized, the commit graphs are simply unioned with each other in a non-destructive, idempotent way". The union graph is "reduced in topological order, starting from the root of the graph and going up to the tips". — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Refs are named `refs/cobs/<typename>/<object id>`. COB change commits carry a manifest and a `Rad-Resource` trailer, and non-COB parents, such as source commits referenced by a patch, carry a `Rad-Related` trailer. The crate describes COBs as "graphs of CRDTs" with create, get, list and update operations, and "no delete". — [docs.rs radicle-cob](https://docs.rs/radicle-cob/latest/radicle_cob/); [search snippets of radicle-cob/heartwood source](https://docs.rs/radicle-cob/^0.9.0)
- Per-peer namespaces: "each peer has its own namespaced references (eg. refs/heads, and refs/tags), while sharing the underlying objects", keyed by Node ID. — [Radicle protocol guide](https://radicle.dev/guides/protocol); [Radicle user guide](https://radicle.dev/guides/user)
- Signed refs:
  - Radicle "automatically signs the entirety of a node's references every time they change". This is a commit at `refs/namespaces/<nid>/refs/rad/sigrefs` with a `refs` file, listing every ref and its OID in lexicographic order, and a `signature` file holding an Ed25519 signature over `refs`.
  - Vulnerability, disclosed 2026-03-30: the signature covered only `refs`, with "no nonce or further replay protection". An attacker could replay an old (refs, signature) pair as a new child commit, for example reverting to an empty state or undoing security fixes.
  - Fix in v1.7.0: a signed `refs/rad/sigrefs-parent` entry binds each snapshot to its parent commit hash.
  - v1.8.0 added "feature levels" `none`, `root` (graft protection since 1.1.0) and `parent` (since 1.7.0), enforceable via `node.fetch.signedReferences.featureLevel.minimum = "parent"`. The team says signed pushes "would be more sustainable" long-term.
  - Sources: [Radicle disclosure](https://radicle.dev/2026/03/30/disclosure-of-vulnerability-in-signed-references); [protocol guide](https://radicle.dev/guides/protocol)
- Identity: each user has an Ed25519 key pair whose public half is shared as a DID. Repository IDs derive from the identity document, using SHA-1 and multibase with a `rad:` prefix. — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Gossip: three signed and timestamped announcement types, for nodes, inventory and refs. After connecting, a node runs a git fetch negotiation. — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Real-time: after `git push rad`, peers that seed the repository get a reference announcement and then fetch via git ("✓ Synced with 1 node(s)"). `rad sync --fetch` forces a fetch. `git-remote-rad` handles `rad://` remotes. — [Radicle user guide](https://radicle.dev/guides/user)
- Seeding policy: by default a node subscribes to "repository delegates plus any node explicitly followed". `--scope all` subscribes to every peer. — [Radicle user guide](https://radicle.dev/guides/user)
- Network limitations: the network uses gossip plus "the Git v2 smart transfer protocol". It has no NAT punching and relies on public seed nodes, which "can't modify the data". Identity was one key per node, without multi-device or organization support, as of LWN's 2024 article. — [LWN](https://lwn.net/Articles/966869)
- Releases: 1.0.0 shipped September 2024. 1.2.0 (June 2025) brought "huge improvements in initializing larger repositories" because libgit2 was slower than git for file-protocol push and fetch. — [Radicle history](https://radicle.dev/history); [heartwood CHANGELOG mirror](https://git.hswro.org/mab122/radicle-heartwood-lfs/src/commit/570bfc3bbd7692f7aa4fa28fae0ccd5c348b5532/CHANGELOG.md)
- Implementation language: Rust (heartwood), not Go. Earlier Radicle designs were superseded by Heartwood. — [Radicle history](https://radicle.dev/history)

### Inferences

- Radicle's "one namespace per writer, shared objects, union-then-reduce" is the strongest precedent for ParleyInn's "per-origin segments". Single-writer refs make merging trivial, and the reader builds the global order.
- The sigrefs replay bug is a concrete warning for ParleyInn. Any signed snapshot or segment head must bind to its predecessor hash, with monotonic, chained signatures, or an attacker can roll back or truncate an origin's log while the signatures still verify. A hash chain inside each segment, as ParleyInn plans, plus a signature over the chain head with a parent link avoids this.
- Radicle's real-time behaviour still depends on a network daemon and gossip, which is incompatible with ParleyInn's I4 (no network in shipped code). ParleyInn would need an external transport, such as a user-run git push or pull, outside the binary.

### Gaps

- No published Radicle scale numbers were found (COBs per repository, refs per repository, announcement latency).
- The exact COB commit tree layout (manifest blob, change blob, embeds) could not be read from primary docs. docs.rs gave only the high-level API.

## Fossil SCM: SQLite repository with artifacts, tickets, wiki, forum and chat

### Takeaway

A Fossil repository is a single SQLite file holding an unordered set of immutable, hash-named,
optionally PGP-clearsigned artifacts. Sync is a G-Set CRDT exchange over HTTP using
`igot`/`gimme`/`file` cards. All query tables, such as tickets, are rebuildable projections of
the artifacts. Chat is deliberately outside the sync model: it lives in a local table, expires
after 7 days and uses HTTP long-polling. Fossil is the closest prior art to "append-only record
plus SQLite index in one file".

### Cited Findings

- "The global state of a fossil repository is an unordered set of _artifacts_." There are eight structural types: manifest, cluster, control (tag), wiki, ticket change, attachment, technote and forum post. — [Fossil file format](https://fossil-scm.org/home/doc/trunk/www/fileformat.wiki)
- Artifacts are named by SHA1 or SHA3-256. Structural artifacts are UTF-8 "cards", one per line, each starting with a type letter, with cards in strict lexicographic order. They "may be PGP clearsigned". Storage as "delta- and zlib-compressed blobs in an SQLite database" is "an implementation detail". — [Fossil file format](https://fossil-scm.org/home/doc/trunk/www/fileformat.wiki)
- Tickets have no base artifact: "A ticket exists if it contains one or more changes". Each ticket-change artifact sets fields with J cards, and when values clash "the tag with the latest (most recent) date is used". — [Fossil file format](https://fossil-scm.org/home/doc/trunk/www/fileformat.wiki)
- "only the low-level ticket change artifacts are synced. The content of the two ticket tables can always be reconstructed from the ticket change artifacts".
  - The TICKET table holds current state. TICKETCHNG holds one row per change artifact.
  - State is computed by applying changes "in time stamp order".
  - Reconstruction "happens automatically whenever new ticket change artifacts are received".
  - Source: [Fossil tickets](https://fossil-scm.org/home/doc/trunk/www/tickets.wiki)
- Sync:
  - Sync runs over HTTP POST, HTTPS, SSH or file transport, zlib-compressed.
  - `igot` cards announce artifacts held, `gimme` cards request them, and `file`/`cfile` cards transfer content, possibly as deltas.
  - The model is a "Conflict-Free Replicated Datatype (G-Set)", and "the client will keep sending HTTP requests until it holds all artifacts that exist on the server".
  - Clusters are made when more than 100 artifacts are unclustered, which shrinks the `igot` traffic.
  - Unversioned files and private artifacts have separate card types and permissions.
  - Source: [Fossil sync protocol](https://fossil-scm.org/home/doc/trunk/www/sync.wiki)
- Chat:
  - Chat "is not synced". Messages live in the `repository.chat` table with columns msgid, mtime, xfrom, xmsg and an attachment, are "automatically deleted after a configurable delay (default: 7 days)", and the table can be dropped safely.
  - Delivery uses long-polling, where `/chat-poll` blocks until a message arrives.
  - Messages allow one file each and cannot be edited.
  - Chat is "ephemeral" by design, while the forum is the "persistent and durable" record.
  - Source: [Fossil chat](https://fossil-scm.org/home/doc/trunk/www/chat.md)

### Inferences

- Fossil confirms that ParleyInn's I10 pattern works in production: immutable content-addressed records, with SQL tables as derived projections rebuilt automatically on receipt.
- Fossil orders ticket changes by trusted wall-clock timestamps, which is weaker than Lamport or DAG order. ParleyInn should not copy that.
- Fossil's sync is a pure set union (G-Set) by hash, with no per-writer refs. The equivalent for ParleyInn would be "union all segments, deduplicate by hash", which needs no merge logic at all.
- Splitting chat from forum is a useful pattern for ParleyInn too: ephemeral real-time data stays out of the replicated record. Fossil reaches real-time only through a server endpoint, which ParleyInn's I4 forbids in-binary.

### Gaps

- No primary Fossil scale numbers were captured this session, such as SQLite repository size or artifact counts for large repositories like SQLite's own.
- Autosync behaviour is not covered in the sync document.

## Forgejo/Gitea and go-git

### Takeaway

Forgejo and Gitea (both Go) store repositories as bare git on disk, but issues, comments and PR
data live in a SQL database (SQLite, MySQL or Postgres), not in git. Real-time notification is
through HMAC-signed webhooks. Forgejo's ForgeFed federation remains partial in 2026; federated
stars are the shipped feature found. go-git is pure Go and builds with CGO disabled. Measured in
this session: its root package pulls in `net`, `net/http`, `crypto/tls`, `os/exec` and
`x/crypto/ssh` through the transport registry. Importing only `plumbing/object` and
`storage/filesystem` keeps all of those out, which matters for ParleyInn's I4.

### Cited Findings

- Forgejo databases: `DB_TYPE` is one of mysql, postgres or sqlite3. Repositories live under `[repository] ROOT`, default `%(APP_DATA_PATH)s/forgejo-repositories`, as bare repositories. — [Forgejo config cheat sheet](https://forgejo.org/docs/latest/admin/config-cheat-sheet/)
- Forgejo webhooks: `QUEUE_LENGTH` defaults to 1000, `DELIVER_TIMEOUT` to 5 s and `ALLOWED_HOST_LIST` to `external`. The `[federation] ENABLED` setting gates ActivityPub. `[api]` has `MAX_RESPONSE_ITEMS` and `DEFAULT_PAGING_NUM`. — [Forgejo config cheat sheet](https://forgejo.org/docs/latest/admin/config-cheat-sheet/)
- Gitea webhook events cover:
  - repository: create, delete, fork, push, wiki, repository, release, package, status;
  - issues: issues, issue_assign, issue_label, issue_milestone, issue_comment;
  - pull requests: pull_request plus its assign, label, milestone, comment, review, review_approved, review_rejected, review_comment, sync and review_request variants;
  - workflows: workflow_run, workflow_job.
  - Delivery is asynchronous HTTP. Requests are signed with `X-Gitea-Signature`, a hex HMAC-SHA256 of the body, and the GitHub-compatible `X-Hub-Signature-256`.
  - The documentation states no retry or delivery guarantee.
  - Source: [Gitea webhooks docs](https://docs.gitea.com/usage/webhooks)
- Federation:
  - ForgeFed is an ActivityPub-based federation protocol for forges. Forgejo's federation work was funded by NLnet/NGI0, and federated repository stars shipped as a first step.
  - The Forgejo v15.0 release (2026-04-16) announcement does not mention federation; it focuses on UI, repository-scoped access tokens and Actions.
  - Sources: [forgefed.org](https://forgefed.org/); [NLnet Federated Forgejo](https://nlnet.nl/project/Federated-Forgejo/); [Forgejo v15.0 announcement](https://forgejo.org/2026-04-release-v15-0/)
- go-git describes itself as "a highly extensible git implementation library written in pure Go". It "covers the majority of the plumbing read operations and some of the main write operations, but lacks the main porcelain operations such as merges". It supports a pluggable `Storer` (memory, filesystem via billy, custom). Users include Keybase, Gitea and Pulumi. v5.19.2 was published 2026-07-29, under Apache-2.0. — [pkg.go.dev go-git/v5](https://pkg.go.dev/github.com/go-git/go-git/v5)
- go-git v6 is still pre-release: the module proxy lists `v6.0.0-alpha.1` to `alpha.5`, and `main` declares `module github.com/go-git/go-git/v6` with go 1.26.0. In v6 the root `remote.go` imports `plumbing/client` and `plumbing/transport`, and the v5 `plumbing/transport/client` package is gone. — [proxy.golang.org v6 list](https://proxy.golang.org/github.com/go-git/go-git/v6/@v/list); [go-git main go.mod](https://github.com/go-git/go-git/blob/main/go.mod); [go-git main remote.go](https://github.com/go-git/go-git/blob/main/remote.go)
- In v5, `plumbing/transport/client` imports the `file`, `git`, `http` and `ssh` transports and fills a package-level `Protocols` map with them. Entries can be swapped or removed with `InstallProtocol(scheme, nil)`. The root `remote.go` imports that client package, so any import of the root `git` package links all four transports. — [go-git v5.19.2 plumbing/transport/client/client.go](https://github.com/go-git/go-git/blob/v5.19.2/plumbing/transport/client/client.go); [v5.19.2 remote.go](https://github.com/go-git/go-git/blob/v5.19.2/remote.go)
- **Measured in this session** with `go list -deps` against go-git v5.19.2, Go 1.26.0:
  - Importing the root package `github.com/go-git/go-git/v5` brings in `net`, `net/http`, `crypto/tls`, `os/exec` and `golang.org/x/crypto/ssh`.
  - Importing only `plumbing/object` and `storage/filesystem` brings in none of those five.
  - `CGO_ENABLED=0 go build` succeeds for both.
  - Method: a scratch module at `scratchpad/gnc/closure/` (no URL; local reproduction).
- git-bug uses go-git v5 and bleve. Its Fetch and Push go through go-git remotes, so they use the network transports. — [git-bug go.mod](https://github.com/git-bug/git-bug/blob/master/go.mod); [repository/gogit.go](https://github.com/git-bug/git-bug/blob/master/repository/gogit.go)

### Inferences

- With go-git, a ParleyInn binary can write commits, trees and refs into a local bare repository using only `plumbing/*` and `storage/filesystem`, staying inside I4 and its depguard and import-closure test. It must never import the root `git` package or `plumbing/transport/*`.
- Transport (push and pull) would then be left to the user's own `git` CLI or a separate, non-shipped tool. That matches I4 better than embedding a transport.
- Forgejo and Gitea show that even Go forges built on go-git do not keep collaboration data in git; they use SQL plus webhooks. Their architecture is evidence for "git for code, database for chatty structured data". Forge webhooks would mean network listeners, which ParleyInn's I4 rules out.

### Gaps

- No primary 2026 source was found giving a precise status of Forgejo federation beyond stars, such as federated issues or PRs.
- No Gitea or Forgejo API rate-limit documentation was found. The config sheet has no dedicated rate-limit section, so this remains unconfirmed.
- No go-git performance benchmarks against C git were found this session.

## Other designs: jj operation log, Dolt, Pijul, Irmin, Automerge, gitbase

### Takeaway

- Jujutsu's operation log is the cleanest lock-free multi-writer append design. Operations and views are content-addressed and written without locks. A head is advertised by an empty file named after its ID. Divergent heads are merged by a 3-way merge of views.
- Dolt applies git's model to SQL tables with prolly trees.
- Pijul gets a CRDT from commutative, additive patches.
- Irmin is an OCaml library for mergeable, git-format stores.
- Automerge-repo has pluggable storage, but no git backend was found.
- gitbase (SQL over git via go-git) was an alpha project that did not mature.

### Cited Findings

- jj: "each commit object is instead an 'operation' and each tree object is instead a 'view'". A view holds the visible heads, bookmarks, tags and per-workspace working-copy commit. Operation and view objects are "stored in content-addressed storage just like Git commits are. That makes them safe to write without locking." A head is published by adding a file whose name is the operation ID and which "has no contents", then removing the old head file. Multiple heads trigger "a 3-way merge of the view objects based on their common ancestor". Conflicted bookmarks are kept rather than causing errors. — [jj concurrency docs](https://docs.jj-vcs.dev/latest/technical/concurrency/)
- Dolt: a "Git-style version controlled SQL database". Table data sits in prolly trees, which are content-addressed B-trees with structural sharing, and versions form a Merkle-DAG commit graph. Diffs cost O(d), where d is the size of the change, against O(n) for plain B-trees. — [Dolt storage engine](https://www.dolthub.com/docs/architecture/storage-engine)
- Pijul: independently produced patches commute. Conflicts are first-class. The graph is append-only, and deletions relabel edges as "dead". This makes it a CRDT because it "only performs additive operations on a graph structure". — [Pijul theory](https://pijul.org/manual/theory.html)
- Irmin: an "OCaml library for building mergeable, branchable distributed data stores". It supports custom merge functions and has backends for git (`irmin-git`, "bidirectional compatibility with the Git on-disk format"), pack, fs and chunk. — [irmin.org](https://irmin.org/)
- Automerge-repo wraps the Automerge CRDT with pluggable StorageAdapters (IndexedDB, NodeFS) and NetworkAdapters (WebSocket sync protocol). — [Automerge Repo blog](https://automerge.org/blog/automerge-repo/); [automerge-repo README](https://cdn.jsdelivr.net/npm/@creately/automerge-repo@2.0.2/README.md)
- gitbase: a Go SQL interface to git repositories over the MySQL wire protocol, built on go-mysql-server. Its README calls it "alpha … still lacking performance in a number of cases" and folds it into source{d} Community Edition. — [gitbase repo](https://github.com/src-d/gitbase)

### Inferences

- jj's "empty file named by head ID" trick suits ParleyInn's local multi-process writers: writes are content-addressed and lock-free, and the head set is the union of head files. It is an alternative to SQLite-WAL write locking for the record itself.
- Dolt and Irmin are heavyweight: Dolt is a full SQL engine, and Irmin is OCaml. They add little for an append-only log with a separate SQLite index.

### Gaps

- No maintained "Automerge over git" storage adapter or "event sourcing on git" Go library with production use was found.
- gitbase's archived status was not confirmed: the README does not say "archived", and source{d} is defunct, which is my own knowledge and is unsourced here.
- Dolt's merge semantics and embedded Go driver were not covered by the fetched page.

## Cross-system comparison (what, where, merge, identity, real-time, search, scale)

### Takeaway

Every git-native system that survives multi-writer use avoids conflicting writes to one mutable
location. The patterns are:

- per-writer refs (Radicle namespaces, git-bug tracking refs);
- per-entity commit DAGs merged by deterministic replay (git-bug Lamport plus pack-id order; Radicle topological reduction);
- set union of immutable hashed records (Fossil G-Set; git-appraise cat_sort_uniq lines).

Every one keeps a rebuildable index outside git: git-bug's excerpts and bleve, Fossil's SQL
tables, Radicle's node-side cache (not verified here), and Forgejo's SQL as the primary store.

### Cited Findings

| System             | What is stored                                             | Where                                                                                                                                                                                                                                                                | Merge / conflict model                                                                                                                                                              | Identity / signing                                                                                                                                                 | Real-time                                                                                                    | Retrieval / search                                                                                                                 | Known scale notes                                                                                                                                                                         |
| ------------------ | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| git-bug (Go)       | Operation packs (JSON) per edit session                    | Commit chain per entity at `refs/bugs/<id>` etc.; side store `.git/git-bug/{clocks,indexes}` — [data-model](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md), [gogit.go](https://github.com/git-bug/git-bug/blob/master/repository/gogit.go) | Merge commits; DAG-checked Lamport order, then pack-id tiebreak — [data-model](https://github.com/git-bug/git-bug/blob/master/doc/design/data-model.md)                             | Identity entities; OpenPGP-signed commits — [operation_pack.go](https://github.com/git-bug/git-bug/blob/master/entity/dag/operation_pack.go)                       | None (manual push/pull; bridges) — [README](https://github.com/git-bug/git-bug/blob/master/README.md)        | Excerpt cache + bleve FTS, verified by builtFrom — [subcache.go](https://github.com/git-bug/git-bug/blob/master/cache/subcache.go) | Pre-cache: 10k bugs ≈ 40 s full read — [issue](https://git.secluded.site/git-bug/bug/f8e9e6cfda83ab8dc958d4211d6c699d8ab61594aa6beac0c1012f72526cd65c)                                    |
| git-appraise (Go)  | JSON-per-line review/comment/CI records                    | git notes `refs/notes/devtools/*` — [README](https://github.com/google/git-appraise)                                                                                                                                                                                 | `cat_sort_uniq` line union; latest timestamp wins — [README](https://github.com/google/git-appraise)                                                                                | Git author only (no signing documented)                                                                                                                            | None                                                                                                         | Notes lookup per commit                                                                                                            | Not documented                                                                                                                                                                            |
| git notes          | Blob per annotated object                                  | Notes ref commit history, 2-hex fanout tree — [git-notes](https://git-scm.com/docs/git-notes)                                                                                                                                                                        | manual/ours/theirs/union/cat_sort_uniq — [git-notes](https://git-scm.com/docs/git-notes)                                                                                            | Commit signing only                                                                                                                                                | None; not fetched by default                                                                                 | By object id                                                                                                                       | Load-all problems at hundreds+; GHC lazy-fetch workaround — [memo PR](https://codeberg.org/antoniorodr/memo/pulls/9), [GHC MR](https://gitlab.haskell.org/ghc/ghc/-/merge_requests/11391) |
| Radicle (Rust)     | COB change DAGs, code, identity doc                        | One bare repo; per-peer git namespaces; `refs/cobs/<type>/<id>`; `rad/sigrefs` — [protocol](https://radicle.dev/guides/protocol)                                                                                                                                     | Union of DAGs, topological reduction (CRDT) — [protocol](https://radicle.dev/guides/protocol)                                                                                       | Ed25519 DIDs; signed refs snapshot with parent binding since 1.7.0 — [disclosure](https://radicle.dev/2026/03/30/disclosure-of-vulnerability-in-signed-references) | Gossip ref announcements → git fetch — [user guide](https://radicle.dev/guides/user)                         | Node-side (not verified)                                                                                                           | No public numbers; seed nodes required behind NAT — [LWN](https://lwn.net/Articles/966869)                                                                                                |
| Fossil (C)         | Immutable card artifacts (check-ins, tickets, wiki, forum) | Single SQLite file; derived tables — [file format](https://fossil-scm.org/home/doc/trunk/www/fileformat.wiki)                                                                                                                                                        | G-Set union by hash; tickets replayed by timestamp — [sync](https://fossil-scm.org/home/doc/trunk/www/sync.wiki), [tickets](https://fossil-scm.org/home/doc/trunk/www/tickets.wiki) | Optional PGP clearsign — [file format](https://fossil-scm.org/home/doc/trunk/www/fileformat.wiki)                                                                  | Chat via long-poll, not synced, 7-day expiry — [chat](https://fossil-scm.org/home/doc/trunk/www/chat.md)     | SQL over derived tables                                                                                                            | Not captured                                                                                                                                                                              |
| Forgejo/Gitea (Go) | Code in git; issues/PRs/comments in SQL                    | Bare repos on disk + sqlite/mysql/postgres — [cheat sheet](https://forgejo.org/docs/latest/admin/config-cheat-sheet/)                                                                                                                                                | DB transactions (central)                                                                                                                                                           | Accounts; HMAC-signed webhooks — [webhooks](https://docs.gitea.com/usage/webhooks)                                                                                 | Webhooks, queue 1000, 5 s timeout — [cheat sheet](https://forgejo.org/docs/latest/admin/config-cheat-sheet/) | SQL/API with paging                                                                                                                | Rate limits not found                                                                                                                                                                     |
| jj op log (Rust)   | Operation + view objects                                   | Content-addressed store; head = empty file named by id — [jj](https://docs.jj-vcs.dev/latest/technical/concurrency/)                                                                                                                                                 | 3-way merge of views; conflicts retained — [jj](https://docs.jj-vcs.dev/latest/technical/concurrency/)                                                                              | n/a                                                                                                                                                                | n/a                                                                                                          | n/a                                                                                                                                | n/a                                                                                                                                                                                       |

### Inferences

- For ParleyInn's model (single-writer per origin, many origins), the safest and simplest synthesis is:
  1. Each origin appends to its own ref chain, `refs/parinn/<origin>/<session>` or one ref per origin with a segment per commit. This copies Radicle's per-peer namespace and git-bug's commit-per-pack, with no merge commits.
  2. Each segment is hash-chained inside its own commit and signed with a signature that binds the parent hash, avoiding Radicle's 2026 replay bug.
  3. Global order is computed only in the SQLite index, from (Lamport or causal links, then origin id and segment hash), as git-bug does. Wall-clock time is used only for display, unlike Fossil's tickets.
  4. The index records "built from" commit ids per ref and rebuilds on mismatch, as git-bug's excerpt-plus-bleve check does. This satisfies I10.
  5. ParleyInn code uses only go-git `plumbing/*` and `storage/filesystem`. Push and fetch are delegated to an external `git`, keeping I4.
- Watch ref-count growth: one ref per session across many machines becomes 10^4–10^5 refs. Prefer one ref per origin, or rely on reftable, which becomes the default in Git 3.0.
- None of the git-native systems offer real-time, sub-second cross-machine propagation without a daemon (Radicle) or a server (Fossil chat, Forgejo webhooks). Git transport is inherently batch or pull. Near-real-time multi-agent visibility will need something outside the git record, which under I4 must sit outside the ParleyInn binary.

### Gaps

- No head-to-head benchmark across these systems exists in the sources found.
- Radicle's and Fossil's on-node indexes and search were not verified from primary docs.
