# Integrated platforms: git + chat + encryption/deletion on one storage/network layer

Research date: 2026-10-02. Every claim carries a URL. Search-snippet-only claims are flagged.
Fetch-tool summaries were cross-checked against primary pages where possible; contradictions are noted.

## Q1. Integrated systems: architecture, identity/keys, code-to-conversation linkage, sync, encryption, deletion, Go availability, licence, maturity

### Takeaway

Only **Keybase** ever shipped all three properties (git, chat, E2E encryption with per-device key revocation and forward-secret deletion) on one identity and key system. It is now a maintenance-mode "zombie" under Zoom. The 2026 newcomers that put git and chat on one log for agents (**Block Buzz**, **Zed Delta**) both dropped end-to-end encryption: Buzz uses TLS plus storage-layer at-rest encryption, and Delta documents no encryption at all. The git-native p2p systems (**Radicle**, **Tangled**, **gitpear**) have signed identity but **no encryption at rest**. The encrypted local-first systems (**Anytype/any-sync**, **Peergos**, **Keet**, **SimpleX**) have chat and encryption but **no git interface**. Nothing current combines all three.

### Cited Findings

**Keybase (keybase/client, Go; BSD-3 per repo, licence not re-verified here)**

- Architecture: one identity per user, with a signature chain (sigchain). Each device publishes its own encryption and signing keys, linked to the user's other devices by mutual signatures. Chat uses the same 32-byte symmetric key as the private KBFS folder for that chat — [Keybase Book: Chat Crypto](https://book.keybase.io/docs/chat/crypto)
- Revocation: when a device is removed, "their other devices will create and share a new encryption key. That guarantees the removed device can't read new messages." — [Keybase Book: Chat Crypto](https://book.keybase.io/docs/chat/crypto)
- Deletion: a "delete" message makes the server delete the body. Headers persist to keep conversation context. Normal messages have no forward secrecy: "the keys to read them still exist on your devices, and they never get deleted." — [Keybase Book: Chat Crypto](https://book.keybase.io/docs/chat/crypto)
- Exploding messages give real crypto-shredding. Each device makes a daily ephemeral keypair signed by its long-term key, and messages live at most one week. "Keys are deleted one week after the following generation is issued." A device that publishes no new key for three months goes "stale" and cannot receive exploding messages, so "one mothballed device won't compromise the forward secrecy of an entire group forever." User and team ephemeral keys mirror the long-term key hierarchy — [Keybase Book: Exploding messages](https://book.keybase.io/docs/chat/ephemeral)
- Encrypted git: E2E, signed by device keys that never leave the device. Repository and branch names are encrypted, while the server still knows team membership and device details. The git layer is separate from KBFS to avoid conflicted HEADs, with locking. The remote helper `git-remote-keybase` is built on go-git (`keybase://private/chris/docs.git`). Each account or team gets 100 GB — [Keybase blog: Encrypted git for everyone](https://keybase.io/blog/encrypted-git-for-everyone)
- Code–conversation link: team chat and team git share the same team key and membership. I found no first-class "thread per PR" feature (see Gaps) — inferred from the shared team key model above.
- Go availability: KBFS and the client are Go packages (`github.com/keybase/client/go/kbfs`) — [pkg.go.dev](https://pkg.go.dev/github.com/keybase/client/go/kbfs)
- Zoom acquired Keybase on 7 May 2020 as an acquihire for Zoom's E2E encryption effort. Zoom said it had no intention of continuing Keybase as-is — [Decrypt](https://decrypt.co/28121/keybase-users-revolt-following-zoom-acquisition); [Training Industry press release](https://trainingindustry.com/press-release/remote-learning/zoom-acquires-keybase-and-announces-goal-of-developing-the-most-broadly-used-enterprise-end-to-end-encryption-offering/)
- Status as of 2026: servers still run, chats and encrypted git still work, and KBFS mounts. Feature development is dead, with only dependency bumps and critical fixes. keybase.pub was "abruptly taken offline" in March 2023, and the mobile apps struggle on current iOS/Android. The 2019 Stellar airdrop (2B XLM) brought bot spam — [schulz.dk, Apr 2026](https://schulz.dk/2026/04/06/the-cryptographic-zombie-how-keybase-went-from-privacy-darling-to-zooms-cleanup-crew/) (opinion blog; status claims are consistent with [Wikipedia](https://en.wikipedia.org/wiki/Keybase) but not independently verified)
- **Status label: maintenance-only / effectively deprecated (no official EOL notice found).**

**Block Buzz (block/buzz, Rust, Apache-2.0, launched 21 Jul 2026)**

- Architecture: one Nostr relay is "the workspace". Every message, reaction, workflow step, review approval and git event is a signed event in one log. The Rust Axum relay speaks WebSocket and REST, with Postgres for events and full-text search, Redis for pub/sub and presence, and S3/MinIO for media via Blossom. Multi-community mode is tenant-scoped — [github.com/block/buzz](https://github.com/block/buzz)
- Identity: secp256k1 Nostr keypairs. Humans authenticate with NIP-42 Schnorr auth and agents with NIP-98. Agents are first-class members with their own keys, memberships and audit trail. The `buzz-acp` harness supports Goose, Codex and Claude Code — [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md); [README](https://github.com/block/buzz)
- Code–conversation link: "branch as room". A feature branch creates a channel, patches land as NIP-34 events, and CI, review and merge happen there. After merge the channel becomes an archived record of why the change exists. Git hosting uses Smart HTTP ("standard git clone, git push") from object storage. Tooling includes `git-sign-nostr` and `git-credential-nostr` — [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md); [README](https://github.com/block/buzz)
- **Encryption: not E2E.** "TLS in transit. At-rest encryption delegated to the storage layer (e.g., Postgres TDE, volume encryption)." The server-managed model covers every channel, DM and event, and NIP-44 E2E is only a future consideration for DMs — [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md). NOSTR.md says "NIP-04/NIP-44 not implemented". NIP-17 gift-wrap (kind 1059) is accepted but "stored community-globally" — [NOSTR.md](https://raw.githubusercontent.com/block/buzz/main/NOSTR.md). Block-hosted communities are "not end-to-end encrypted", and the operator can access content (search snippet only, attributed to Buzz terms; not fetched) — [search result](https://github.com/block/buzz/blob/main/VISION.md)
- Deletion: standard NIP-09 kind 5, self-authored only. Admin delete uses kind 9005 within a channel. This is relay-side deletion; nothing is crypto-shredded — [NOSTR.md](https://raw.githubusercontent.com/block/buzz/main/NOSTR.md)
- Supported NIPs: 01, 09, 10, 11, 17, 25, 29 (relay-based groups, native), 42 and 50. NOSTR.md does not list NIP-34, even though the README and VISION do — [NOSTR.md](https://raw.githubusercontent.com/block/buzz/main/NOSTR.md) (internal doc inconsistency)
- Maturity: relay, channels, threads, DMs, canvases, search and audit log work. Mobile, desktop, workflows and **git hosting** are "being wired up". There is no federation and no Go SDK; the SDK is Rust (`buzz-sdk`). The clients are Tauri+React and Flutter — [README](https://github.com/block/buzz); git "is new… the least mileage on it" — [mager.co, 24 Jul 2026](https://www.mager.co/blog/2026-07-24-buzz-explainer/)
- **Status label: early beta (buzz.xyz hosted beta).**

**Generic Nostr git and chat (NIP-34, NIP-17, NIP-44)**

- NIP-34 defines repositories, patches, issues and PRs as Nostr events. NIP-17 private DMs use NIP-44 encryption inside NIP-59 seals and gift wraps. The inner events are unsigned, which makes them deniable. This replaces NIP-04, which leaked metadata — [d-central NIP reference](https://d-central.tech/nostr-nips-reference/); [NIP-17 mirror](https://nips.4rs.nl/nips/17)
- An academic paper presents practical attacks on Nostr — [Kimura et al., IACR ePrint 2025/1459](https://eprint.iacr.org/2025/1459.pdf) (title only seen; contents not reviewed)

**Radicle (Rust, heartwood; 1.0 Sept 2024; 1.10.0 Aug 2026)**

- Architecture: p2p and local-first on Git. Each node has an Ed25519 key, encoded as a `did:key` DID. Gossip carries three signed message types: Node, Inventory and Reference announcements. Peers verify signatures before relaying. Storage uses git namespaces per Node ID over a shared object database. Seed nodes can be public or community-selective — [Radicle protocol guide](https://radicle.dev/guides/protocol)
- Conversation: Collaborative Objects (COBs) are CRDTs stored as git commits inside the repo. They hold issues, patches, reviews and identities, and are replayed in a deterministic causal order. They are extensible — [Radicle protocol guide](https://radicle.dev/guides/protocol); [Radicle 1.0.0](https://radicle.xyz/2024/09/10/radicle-1.0.0). Discussion is asynchronous patch and issue comments, not real-time chat.
- Encryption: transport uses Noise XK with forward secrecy, plus Tor support. Private repos use allow-list selective replication, but "the data is not encrypted at rest." — [Radicle protocol guide](https://radicle.dev/guides/protocol); [FAQ](https://radicle.xyz/faq)
- Deletion: the CLI has `rad patch delete` and `rad patch redact` — [rad-patch(1)](https://man.archlinux.org/man/rad-patch.1.en). Copies that peers have already replicated cannot be recalled. I found no source on this; it is inferred from the protocol design.
- 1.10.0 (5 Aug 2026) focused on identity-document revision evaluation (Active, Accepted, Rejected, Redacted states) ahead of 2.0. Of "several thousand" repos scanned, about 15 changed — [Radicle 1.10.0](https://radicle.dev/2026/08/05/radicle-1.10.0). That gives a rough network-size signal of thousands of repos.
- **Status: active, stable 1.x, pre-2.0. Not Go.**

**Tangled (AT Protocol; Go + Rust monorepo, MIT, alpha v1.16.1-alpha)**

- Architecture: "knots" are lightweight headless git servers, single- or multi-tenant. The appview at tangled.org aggregates across knots. "Spindles" are CI runners. Identity is atproto DIDs, and social records use `sh.tangled.*` lexicons — [Introducing Tangled, 2 Mar 2025](https://blog.tangled.org/intro); [tangled.org/core](https://tangled.org/tangled.org/core)
- Encryption and privacy: "By design, all data stored on the protocol today is public." Atproto Spaces (formerly "permissioned data") is now in alpha to support non-public data — [atproto Spaces alpha](https://atproto.com/blog/atproto-spaces-alpha). A Tangled issue states "ATProto itself is not enough for tangled's needs… shared-private records" — [tangled core issue #356](https://tangled.org/tangled.org/core/issues/356). The fetch summary of the repo page claimed private-repo and encryption support. I could not verify that and treat it as unreliable.
- Real-time chat: none found. Discussion is issues and PR comments as public atproto records (inferred from above).
- **Status: alpha, active. Language: Go (42%) + Rust (43%).**

**Holepunch Pear / Keet + gitpear**

- Keet: p2p chat over Hyperswarm with E2E encryption and no hosted servers. Identity comes from a 24-word seed phrase — [App Store listing](https://apps.apple.com/us/app/keet-private-encrypted-chat/id6443880549); [Snap listing](https://snapupdates.popey.com/snap/keet)
- Hypercore is a signed, distributed append-only log — [docs.pears.com Hypercore](https://docs.pears.com/building-blocks/hypercore)
- gitpear: a third-party p2p git transport (CLI, daemon and remote helper) on Holepunch. It shares regenerated packfiles via ephemeral Hyperdrive and connects peers over Hyperswarm, keyed by a public key (`git pear key`) — [npm gitpear](https://npmjs.com/package/gitpear)
- No integration exists between Keet chat and gitpear git. They share the transport stack only. The stack is JavaScript/Bare, so there is no Go — inferred from sources above.

**Iroh (n0, Rust)**

- Iroh is a modular p2p framework. iroh-gossip uses epidemic broadcast trees. iroh-docs is an eventually consistent KV store over iroh-blobs. A Router dispatches by ALPN — [docs.iroh.computer](https://docs.iroh.computer/what-is-iroh); [iroh-gossip docs](https://docs.rs/iroh-gossip)
- Chat apps on iroh include Dash Chat (for internet shutdowns) and a PoC combining automerge keyhive/beelay, iroh and tauri — [awesome-iroh](https://awesome.ecosyste.ms/lists/n0-computer%2Fawesome-iroh?topic=gui). There is an unofficial Go binding: [tmc/go-iroh](https://github.com/tmc/go-iroh/wiki). I found no iroh app that combines git and chat.

**Zed Delta / DeltaDB (Zed Industries)**

- DeltaDB was announced 11 Jun 2026, Delta on 12 Aug 2026, and the Delta public beta on 16 Sep 2026 ("Replacing pull requests is our first step toward replacing GitHub") — [zed.dev/blog](https://zed.dev/blog)
- Architecture: an operation-based VCS with CRDT "conflict-free replicated worktrees". Every operation between commits gets a stable identity. "A message and the edit it produced are recorded side by side." References anchor to deltas, not line numbers — [Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- Git compatibility: "DeltaDB works with the git repository you already have. Every edit and conversation is captured between your commits". Teammates without Delta see a normal git repo. The database "replicates the conversation and the worktree together, in real time, for everyone in a thread". Work can move to a cloud runner. The Wasm/WebGL browser client connects to Claude Code — [Introducing Delta](https://zed.dev/blog/introducing-delta)
- From a line in a past conversation you can jump to the code as it is now or as it was when written, and back again — [AlphaSignal](https://alphasignal.ai/news/zed-s-deltadb-rebuilds-version-control-around-ai-agent-conversations)
- **Encryption, licence, retention and self-hosting: not documented** in either blog post — [Introducing Delta](https://zed.dev/blog/introducing-delta); [Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- **Status: public beta, hosted. Not Go (Zed is Rust).**

**Fossil (C, BSD-2)**

- A single-file SQLite repo bundles VCS, wiki, tickets, forum and chat. Chat lives in a separate CHAT table: "Chat messages do not sync to peer repositories, and they are automatically deleted after a configurable delay (default: 7 days)." Local and global delete exist, editing does not (by design). Chat uses long polling (CGI), needs capability C, and supports robot accounts for notifications. No chat encryption is mentioned — [Fossil chat docs](https://fossil-scm.org/home/doc/trunk/www/chat.md)
- **Status: mature and active.** Lesson: Fossil deliberately kept chat ephemeral and outside the replicated, append-only history.

**Forgejo / Gitea + Matrix**

- The integration is only a webhook: a Matrix webhook type (homeserver URL, room ID, access token) posts issue, PR and Actions events to a room — [Forgejo webhook source, matrix.go mirror](https://git.fb1.uni-bremen.de/davrot/forgejo_backup/src/branch/upload_with_path_structure/services/webhook/matrix.go). Federation work follows ForgeFed, an ActivityPub extension, not Matrix — [forgefed.org](https://forgefed.org/); [Forgejo federation FAQ](https://forgejo.codeberg.page/2023-01-10-answering-forgejo-federation-questions/)
- Forgejo's own Matrix room suffered "state resets which are a known bug in Matrix" (profile and permission resets) and moved to a new room — [Forgejo monthly report](https://forgejo.org/2025-05-monthly-update/)
- Matrix redaction strips non-essential keys "forever", but "some server implementations retain the redacted fields for a short time" — [Matrix room v9 spec](https://spec.matrix.org/v1.19/rooms/v9/). MSC4117 proposes reinstating redacted events for legal appeals — [MSC4117](https://git.sudo.is/matrix/matrix-doc/src/branch/travis/reinstated-events/proposals/4117-reinstating-events.md). Forgejo and Gitea are Go.

**Anytype / any-sync (Go, MIT)**

- any-sync is a local-first, p2p, E2E-encrypted protocol. Data lives in encrypted DAGs with ACLs. It has sync, file, consensus (watches ACL changes) and coordinator nodes, and powers Anytype chat — [github.com/anyproto/any-sync](https://github.com/anyproto/any-sync); [pkg.go.dev](https://pkg.go.dev/github.com/anyproto/any-sync@v0.4.9)
- Each change has two encryption layers (object linkage, then payload AES-CFB). The key is generated locally and never sent; there is no central user registry — [Anytype privacy & encryption](https://doc.anytype.io/anytype/data/privacy-and-encryption)
- Deletion: an account has 30 days to cancel, then deletion is permanent. On key loss, "Your Vault will remain encrypted on our backup nodes without anybody being able to access it." "Anytype cannot recover a lost Key." Local copies must be removed device by device — [Data erasure & recovery](https://doc.anytype.io/anytype/data/data-erasure-and-recovery.md)
- **No git interface.** It is the closest Go library precedent for an encrypted, ACL'd, multi-device sync layer.

**Peergos (Java, AGPL per repo — licence not re-verified)**

- A p2p E2E-encrypted filesystem using cryptree, a tree of symmetric keys. The host cannot see file sizes, names, topology, the social graph or who has access — [peergos.org technology](https://peergos.org/technology); [Peergos book](https://book.peergos.org/)
- Chat is an app on top of the filesystem. Each member appends to their own append-only chat file, and messages form a Merkle DAG for causal order. Removed members are revoked from future messages. "Your Peergos server doesn't even know you're sending chat messages" — [Peergos decentralized chat](https://peergos.org/posts/decentralized-chat)
- I found no git integration.

**SimpleX (Haskell, AGPL)**

- Secret groups have no user identifiers. XFTP relays store file fragments temporarily (48 h on preset servers). Senders can revoke files before receipt, and relays are self-hostable — [SimpleX FAQ mirror](https://forgejo.multed.com/android-apps-mirrors/SimpleX-Chat/src/commit/41cb734d5681e6bff70a59bb100edc9e566334b1/docs/FAQ.md). There is no git interface.

### Inferences

- Integration grid: (1) **git + chat + E2E + shredding**: Keybase only, now in maintenance. (2) **git + chat, signed, server-trusted**: Buzz, Delta, Fossil, Forgejo+Matrix. (3) **git + discussion, signed, p2p, plaintext at rest**: Radicle, Tangled, NIP-34. (4) **chat + E2E, no git**: Keet, any-sync, Peergos, SimpleX.
- The agent-first entrants of 2026 (Buzz, Delta) both chose operator-readable storage, apparently to get search, moderation and server-side agents. ParleyInn's I2 and crypto-shredding goals would put it in a quadrant nobody currently occupies.
- Keybase's exploding-message design is the best documented crypto-shredding precedent for multi-device users: daily per-device ephemeral keys, delete one generation later, and exclude stale devices. Fossil's "chat is not replicated, auto-expires" is a cheaper alternative: keep conversation out of the immutable record.
- Signed append-only logs (Nostr, Radicle COBs, Hypercore, Peergos chat files) cannot delete what peers already replicated. Only per-key encryption can retract content, and only from parties who never received the key.

### Gaps

- Keybase: no official Zoom EOL or roadmap statement found. Whether team chat could reference git commits natively (beyond push notifications in chat) is unverified.
- Buzz: the operator-access wording of the hosted terms was seen only as a search snippet. I found no adoption numbers or HN thread.
- Zed Delta: licence, encryption, self-host and retention are not published.
- Gitea's "chat plans": no source found. Element/Matrix "spaces for code": no product or MSC found.
- Radicle COB deletion semantics beyond CLI verbs: no doc found.
- Tangled and Radicle adoption numbers beyond the 1.6k-star repo and "several thousand" repos: not found.
- No iroh-based git+chat app found.

## Q2. Recurring failure modes and lessons

### Takeaway

The recurring failures are mostly organisational and operational, not cryptographic. A single company stewarding the identity and key system can be acquired and frozen (Keybase). Losing the user's key irrecoverably locks them out (Anytype, Keet seed phrase). Signed public logs cannot really delete (Nostr, atproto, Radicle). Relay or appview centralisation creeps back (Buzz single relay, Tangled appview, Anytype backup nodes). Metadata leaks even under E2E (Keybase team membership, Nostr relays).

### Cited Findings

- **Vendor capture**: Keybase was an acquihire. Development stopped, keybase.pub was shut down abruptly in 2023, and mobile apps rot — [schulz.dk](https://schulz.dk/2026/04/06/the-cryptographic-zombie-how-keybase-went-from-privacy-darling-to-zooms-cleanup-crew/); users revolted on acquisition — [Decrypt](https://decrypt.co/28121/keybase-users-revolt-following-zoom-acquisition)
- **Distraction and spam**: the Stellar airdrop brought bot spam and conspiracy theories to Keybase — [schulz.dk](https://schulz.dk/2026/04/06/the-cryptographic-zombie-how-keybase-went-from-privacy-darling-to-zooms-cleanup-crew/)
- **Key loss = lockout**: "Anytype cannot recover a lost Key"; the vault stays encrypted on backup nodes forever — [Anytype data erasure](https://doc.anytype.io/anytype/data/data-erasure-and-recovery.md); Keet identity is a 24-word seed — [App Store](https://apps.apple.com/us/app/keet-private-encrypted-chat/id6443880549)
- **No forward secrecy by default**: Keybase's non-exploding chat keys "never get deleted", so device theft exposes all history — [Keybase Chat Crypto](https://book.keybase.io/docs/chat/crypto)
- **Stale devices undermine group FS**: Keybase had to add a three-month staleness cutoff — [Keybase ephemeral](https://book.keybase.io/docs/chat/ephemeral)
- **Metadata leakage**: Keybase servers know team membership and devices — [Keybase git blog](https://keybase.io/blog/encrypted-git-for-everyone); NIP-04 leaked metadata, which motivated NIP-17/44/59 — [d-central](https://d-central.tech/nostr-nips-reference/); Peergos and SimpleX were designed specifically to hide the social graph — [peergos.org](https://peergos.org/technology); [SimpleX FAQ](https://forgejo.multed.com/android-apps-mirrors/SimpleX-Chat/src/commit/41cb734d5681e6bff70a59bb100edc9e566334b1/docs/FAQ.md)
- **Public-by-design protocols block private use**: "all data stored on the protocol today is public" on atproto — [atproto Spaces](https://atproto.com/blog/atproto-spaces-alpha); Radicle private repos are "not encrypted at rest" — [Radicle protocol](https://radicle.dev/guides/protocol)
- **Relay centralisation and trust**: Buzz's relay is the single source of truth with server-managed encryption, and the operator can read content — [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md); Tangled's appview aggregates the network — [Tangled intro](https://blog.tangled.org/intro)
- **Federated state bugs**: Matrix state resets corrupted Forgejo's room permissions — [Forgejo report](https://forgejo.org/2025-05-monthly-update/)
- **Soft deletion**: Matrix servers may keep redacted fields for a while, and MSC4117 adds un-redaction — [Matrix spec](https://spec.matrix.org/v1.19/rooms/v9/); [MSC4117](https://git.sudo.is/matrix/matrix-doc/src/branch/travis/reinstated-events/proposals/4117-reinstating-events.md)
- **Git in sync folders breaks**: Keybase built a separate locked git layer because concurrent pushes over a sync FS give "a conflicted HEAD" — [Keybase git blog](https://keybase.io/blog/encrypted-git-for-everyone)
- **Mobile is the adoption blocker**: Buzz reviewers stopped short of migrating for lack of mobile — [mager.co](https://www.mager.co/blog/2026-07-24-buzz-explainer/)

### Inferences

- For ParleyInn: local-first, single-binary, no-network (I4) avoids vendor capture and relay centralisation by construction. The cost is that no hosted recovery path exists, so key-loss UX (escrow, paper keys, multi-device) must be designed up front.
- Crypto-shredding in an append-only record needs Keybase-style key generations plus a rule for stale holders, or Fossil-style separation of ephemeral conversation from the durable record.
- Git over a custom sync layer needs its own ref-locking or CRDT layer (Keybase, DeltaDB). Putting a bare repo on generic synced storage is a known failure.

### Gaps

- I found no quantitative user-lockout data for any system.
- HN threads on these failures were not fetched; [Ask HN: still using Keybase?](https://hn.nuxtjs.org/item/27064102) exists but its content was not reviewed.

## Q3. Any system designed specifically for AI agent sessions with the full combination?

### Takeaway

No. Zed Delta links agent conversations to code at operation granularity, Block Buzz gives agents keys and puts git and chat in one log, and Entire (Go, MIT) stores agent session transcripts in git refs. **None of the three offers end-to-end encryption or crypto-shredding.** Entire's redaction is explicitly "best-effort", which bears directly on ParleyInn's I2 and I6.

### Cited Findings

- **Zed Delta**: threads record agent conversations next to the edits they caused, replicated in real time, git-compatible. Encryption and licence are not stated — [Introducing Delta](https://zed.dev/blog/introducing-delta); [Introducing DeltaDB](https://zed.dev/blog/introducing-deltadb)
- **Block Buzz**: agents are first-class keyholders with audit trails, branch-as-room links git and chat, and `buzz-acp` drives Claude Code, Codex and Goose. Storage is TLS plus at-rest encryption only, with E2E a future consideration — [README](https://github.com/block/buzz); [VISION.md](https://raw.githubusercontent.com/block/buzz/main/VISION.md)
- **Entire CLI (entireio/cli, Go, MIT)**: captures prompts, responses, files touched, token usage and tool calls per session. Checkpoints live in `refs/entire/checkpoints/<shard>/<id>`, outside branch history, and are linked by an `Entire-Checkpoint:` commit trailer. They push to origin or to a separate `checkpoint_remote`. Secret redaction is "best-effort", and working-tree snapshots can contain raw secrets. No encryption is mentioned. It supports Claude Code, Codex, Copilot CLI, Cursor, OpenCode and others — [Entire README](https://cdn.jsdelivr.net/gh/entireio/cli@main/README.md); [pkg.go.dev](https://pkg.go.dev/github.com/entireio/cli@v0.7.3); [Waydev AI checkpoints](https://docs.waydev.co/docs/ai-checkpoints)

### Inferences

- The market has converged on linking code to agent conversation (Delta: operation anchors; Buzz: branch channels; Entire: commit trailers and git refs). Security properties have not followed. ParleyInn could claim the security-first niche: a pull-only recall envelope (I2), encrypted per-key records with shredding, and no network (I4).
- Entire is the nearest Go precedent and potential interop target: sessions as git refs plus commit trailers. ParleyInn could adopt the trailer/ref linkage pattern while adding encryption, which Entire lacks.

### Gaps

- I did not check whether Entire has a hosted service with encryption, or whether Delta's backend encrypts at rest; neither is in the fetched docs.
- Other agent-session recorders (e.g. SpecStory, git-ai, Agent Trace) were not surveyed in this pass.
