# Chat interface for AI agent sessions: systems and protocols

Scope: chat and messaging systems and protocols that could act as the "chat interface" for agent sessions in ParleyInn. Current as of 2 October 2026. Archived, deprecated and superseded items are marked **[ARCHIVED]**, **[DEPRECATED]** or **[SUPERSEDED]**.

## Q1. Which systems already present agent sessions as chat?

### Takeaway

Two kinds of design show up. Some products make the agent session itself a chat room with a durable log: Block's Buzz (a Nostr relay with agents as keyed members, git events and a hash-chained audit log), Zed Delta (threads tied to CRDT edits), and GitHub's Copilot agent session logs. Others add a mailbox between sessions: Claude Code agent teams and cross-session messaging, MCP Agent Mail and Gas Town mail. Of all of these, Buzz comes closest to the stakeholder's "chat plus git plus signed log" architecture. It is Rust and pre-1.0, and its deletion support is relay-local.

### Cited Findings

**Block Buzz (Nostr relay, launched July 2026)**

- Block launched Buzz on 21 July 2026 as a free, open-source workspace for teams of humans and AI agents. It has channels, threads, DMs and voice. Agents join as members with their own permissions. It works with Claude Code, Codex and goose through the Agent Client Protocol, and is licensed Apache 2.0 — [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/); [Block announcement](https://block.xyz/inside/introducing-buzz-where-humans-and-agents-work-together) (we could not fetch the announcement because of a header overflow, so this rests on its search snippet).
- Every agent has its own Nostr keypair. A second signature ties the agent to its human owner, which SiliconANGLE calls a "verifiable passport" and "cryptographic paper trail" — [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/) (search snippet).
- "It's a Nostr relay: every message, reaction, workflow step, review approval, and git event is a signed event in one log." One relay hosts one community by default. Hosted multi-tenant mode shares Postgres, Redis and object storage between communities — [block/buzz README](https://github.com/block/buzz).
- "Branch as room": opening a feature branch creates a channel. Patches land there as NIP-34 events, CI posts results, an agent reviews, and "the merge decision lands in the same room as the evidence" — [block/buzz README](https://github.com/block/buzz).
- Stack: `buzz-relay` (Rust, Axum over WebSocket plus REST), Postgres for events and full-text search, Redis for pub/sub, presence and typing, S3 or MinIO for media (Blossom), NIP-42/98 Schnorr auth, and `git-sign-nostr` / `git-credential-nostr` for Nostr-signed git — [block/buzz README](https://github.com/block/buzz).
- Status table. "Works today": relay, channels, threads, DMs, canvases, media, search, audit log, desktop app, ACP harness, YAML workflows, NIP-34 git events and the git hosting backend. "Being wired up": mobile clients and workflow approval gates. The README also says: "Please do not plan your compliance program around the 💭 column yet" — [block/buzz README](https://github.com/block/buzz).
- Kind ranges: kinds 20000–29999 are "Ephemeral events — not stored, not audited". Kind 9 is a NIP-29 group chat message. Custom kinds cover message edits (40003), agent job requests (43001), forum posts and workflow runs (46001–46012). There are 127 kinds in all — [Buzz ARCHITECTURE.md](https://github.com/block/buzz/blob/main/ARCHITECTURE.md).
- `buzz-audit` is a "hash-chain tamper-evident log". Each entry stores `prev_hash`, and `verify_chain()` recomputes the chain. Its ten actions include `EventCreated` and `EventDeleted`. Audit writes are fire-and-forget, so a failed audit write does not fail the event submission — [Buzz ARCHITECTURE.md](https://github.com/block/buzz/blob/main/ARCHITECTURE.md).
- Deletion: an authorized kind:5 deletion commits the deletion event and the removal in one transaction. Channel membership is soft-deleted through `removed_at`. Whole-community deletion is an operator flow (`buzz-admin deletions submit/drain`) — [Buzz ARCHITECTURE.md](https://github.com/block/buzz/blob/main/ARCHITECTURE.md).
- The e2e interop tests cover NIP-17 gift wraps, NIP-10 threads and NIP-50 search — [Buzz ARCHITECTURE.md](https://github.com/block/buzz/blob/main/ARCHITECTURE.md).
- Sources disagree on dates and stars. One blog says v0.5.18 shipped on 21 August 2026, the project "launched March 6, 2026", and it has about 30k stars — [carlos.lat](https://carlos.lat/en/blog/buzz-block-workspace-2026/). That contradicts the 21 July 2026 public launch in [SiliconANGLE](https://siliconangle.com/2026/07/21/block-launches-buzz-open-source-workspace-humans-ai-agents/). The March date may mark the first commit rather than the launch.
- The git forge UI is described as early, with an incomplete feature set — [TFTC](https://www.tftc.io/buzz-block-nostr-ai-agent-workspace-launch) (secondary source).

**Zed Delta threads and DeltaDB**

- "Software Is Made Between Commits" (11 June 2026) introduced DeltaDB. It "breaks your work into a stream of fine-grained deltas", gives each operation "a stable identity", and uses "conflict-free replicated worktrees". "A message and the edit it produced are recorded side by side." The post does not say how storage, sync, deletion or licensing work — [Zed blog: introducing DeltaDB](https://zed.dev/blog/introducing-deltadb).
- "Introducing Delta" (12 August 2026) was followed by "Replace PRs with Delta – Now in Public Beta" (16 September 2026) — [Zed blog index](https://zed.dev/blog).
- In Delta, "You invite teammates directly into your conversations with agents. When someone joins your thread, they see the same worktrees you do." It keeps "edits between commits alongside messages from humans and agents". It is free during the beta, and Zed disabled PRs on Delta's own repository. The post does not say whether Delta is open source or how it handles encryption — [Zed blog: Delta public beta](https://zed.dev/blog/delta-public-beta).
- The Zed editor's thread types are Zed Agent threads, External Agent (ACP) threads and Terminal Threads, all shown in the Threads Sidebar — [Zed docs: agents](https://zed.dev/docs/ai/agents).

**Slack-style agent channels**

- Claude Code in Slack (research preview): you tag Claude in a thread. Claude collects recent channel and thread messages as context, picks a repository, posts status updates back into the thread, and links to the full session — [Salesforce news](https://salesforce.com/news/stories/claude-code-in-slack); [AlternativeTo, Dec 2025](https://alternativeto.net/news/2025/12/slack-gains-agentic-coding-capabilities-with-anthropic-s-claude-code-integration).
- Cursor: mentioning @Cursor launches a background (cloud) agent. It reads the whole Slack thread and can open GitHub PRs — [Cursor docs: Slack](https://docs.cursor.com/slack). One user reports that in Slack's Assistant pane "every message spawns a new cloud agent, with no way to follow up on an existing one" — [Cursor forum](https://forum.cursor.com/t/slack-assistant-pane-every-message-spawns-a-new-cloud-agent-with-no-way-to-follow-up-on-an-existing-one/165947).
- Devin: @Devin in a channel starts a session, and Devin answers in-thread with updates and questions — [Devin docs: Slack](https://docs.devin.ai/integrations/slack).

**MCP Agent Mail**

- Messages are stored twice: "Human-readable markdown in a per-project Git repo for every canonical message and per-recipient inbox/outbox copy", plus "SQLite with FTS5" for search and file reservations. Messages are GFM with JSON frontmatter under `messages/YYYY/MM/`, threaded by `thread_id`. Leases are advisory file reservations, with an optional pre-commit guard. Cross-project messaging needs a contact approval (`request_contact`/`respond_contact`). A human "overseer" composer sends priority messages. It is written in Python and has about 2.2k stars — [Dicklesworthstone/mcp_agent_mail](https://github.com/Dicklesworthstone/mcp_agent_mail).

**Gas Town mail**

- Steve Yegge released Gas Town on 1 January 2026. It is a Go orchestrator for 20–30 parallel Claude Code agents, coordinated through Beads, a git-backed issue tracker, and "an internal mail system". Mail can go to one agent, to a queue where the first consumer takes it, or to every member of a channel — [Exploring Gas Town](https://embracingenigmas.substack.com/p/exploring-gas-town) and the [search result set](https://dehora.net/journal/2026/2/initial-thoughts-on-welcome-to-gas-town) (secondary sources).
- The repo README mentions `gt mail check --inject` for runtimes without hooks, and Beads/Dolt as the persistence layer. It is MIT-licensed, has about 18.2k stars and is active — [steveyegge/gastown](https://github.com/steveyegge/gastown).
- `docs.gastownhall.ai/glossary` now redirects with a 301 to `docs.gascity.com` (seen on 2 October 2026) — [docs.gastownhall.ai](https://docs.gastownhall.ai/glossary).

**Claude Code agent teams and cross-session messaging**

- Agent teams are "experimental and disabled by default" (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`). "Each agent's mailbox is a JSON file at `~/.claude/teams/{team-name}/inboxes/{agent-name}.json`." A message counts as sent only when the write to that file succeeds. Malformed entries are reported and removed. The team config directory is removed at session end, while task lists persist. Limits: one team per session, no nested teams, and teammates are not restored on resume — [Claude Code docs: agent teams](https://code.claude.com/docs/en/agent-teams).
- Trust model: a message from another agent is marked as coming from another session, "never counts as your consent", and cannot approve prompts or change configuration. In auto mode a classifier reviews every inter-agent message before delivery — [agent teams](https://code.claude.com/docs/en/agent-teams); [cross-session messaging](https://code.claude.com/docs/en/cross-session-messaging).
- Cross-session messaging (v2.1.224 and later) runs over a per-session Unix domain socket or named pipe on the same machine, "never through Anthropic servers". Messages to other machines and cloud sessions go "through Anthropic servers" over Remote Control, and a message to an offline session waits until it reconnects. Messages are plain text only. Each message is capped at about one million characters, at most 50 accepted messages queue for Claude to read, and at most 100 are held. The `crossSessionInbound` setting takes accept, hold or refuse — [Claude Code docs: cross-session messaging](https://code.claude.com/docs/en/cross-session-messaging).

**Happy (mobile client for Claude Code and Codex)**

- Happy wraps the CLI (`happy claude`), pairs devices by QR code, and uses `happy-server` as a relay. Data is end-to-end encrypted before it leaves the device ("Your code never leaves your devices unencrypted"). It ships Expo-based iOS, Android and web apps, is MIT-licensed and has about 24k stars. The README does not name the cipher or key schedule — [slopus/happy](https://github.com/slopus/happy); [Show HN](https://hn.svelte.dev/item/44904039).

**GitHub Copilot agent session views**

- An Agents tab in each repository ("mission control") lists, creates and switches agent sessions (26 January 2026) — [GitHub changelog](https://github.blog/changelog/2026-01-26-introducing-the-agents-tab-in-your-repository).
- Since 19 March 2026, session logs group tool calls, show inline diffs, show setup and firewall steps, and collapse subagent activity — [GitHub changelog](https://github.blog/changelog/2026-03-19-more-visibility-into-copilot-coding-agent-sessions/); [docs: track sessions](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/track-copilot-sessions).
- JetBrains IDEs got a unified sessions view on 13 May 2026 — [GitHub changelog](https://github.blog/changelog/2026-05-13-introducing-copilot-cli-agent-and-unified-sessions-view-in-github-copilot-for-jetbrains-ides/).

**Fossil chat**

- Messages live in a CHAT table that "is not cross-linked with any other tables in the repository schema" and can be removed with `DROP TABLE chat;`. They are "automatically deleted after a configurable delay (default: 7 days)" and "do not sync to peer repositories". Delivery is long polling (`/chat-poll`). The `chat-timeline-user` setting announces repository timeline changes in chat. Bots post with `fossil chat send` — [Fossil chat docs](https://fossil-scm.org/home/doc/trunk/www/chat.md).

**Keybase chat and encrypted git**

- Keybase git is a git remote helper backed by KBFS: "your data is encrypted—not even Keybase can see what's in there (nor its name, the filenames...)". Every write is verified by device keys, and the repository is locked to prevent concurrent overwrites — [Keybase book: git](https://book.keybase.io/git).
- Zoom acquired Keybase on 7 May 2020, and the staff joined Zoom's security engineering team — [TechCrunch](https://techcrunch.com/2020/05/07/zoom-acquires-keybase-to-get-end-to-end-encryption-expertise/).

### Inferences

- Buzz already implements most of the stakeholder's proposed shape (one signed event log, git as events, agents as keyed principals, rooms per branch), but on Rust, Postgres and Redis. ParleyInn could read Buzz as a reference design or as an optional bridge target. It cannot be an embedded dependency, because I4 bars ParleyInn from talking to the network.
- Three designs decouple chat from durable history in a useful way. Fossil puts chat in a separate, unsynced, auto-expiring table. Claude Code teams delete the team mailbox at session end. Buzz never stores ephemeral kinds. Each of these lets the conversational channel be erased without touching versioned artifacts.
- Claude Code treats a message from another agent as data, never as consent. That matches ParleyInn's I2: a chat surface that agents read must envelope peer messages as untrusted.
- Most "agent in Slack" integrations, such as Cursor's, do not give a chat-native session identity: each message can spawn a new session. A persistent thread-to-session binding, which Delta and Buzz both have, is the stronger model.

### Gaps

- We found no primary-source description of Gas Town's mail storage format. It is presumably Beads/Dolt rows, but that is unverified. The rename to "Gas City" is inferred only from the docs redirect.
- Buzz: we found no documented answer to whether a kind:5 deletion hard-deletes the Postgres row and its search vector, or what an `EventDeleted` audit entry contains (a hash or content). We could not fetch Block's own announcement. Channel E2EE is not described, beyond DM gift wraps tested for interop.
- Zed DeltaDB/Delta: the storage backend, licence, encryption and erasure model are undisclosed.
- Keybase: no 2026 maintenance status found. We have no source on whether Keybase chat still announces git pushes.
- Happy: the encryption algorithm and key schedule are not in the README.

## Q2. Protocols to build on: model, storage, delivery, encryption, deletion, Go availability, maturity

### Takeaway

Matrix is the only mature protocol that standardizes a verifiable event DAG with redaction: event IDs are hashes of the redacted form, so content can be stripped while the DAG stays verifiable. Its Go server, Dendrite, now lives at Element in reduced form, and MLS for Matrix is still a proof of concept. Nostr gives signed events, a git NIP (NIP-34) and simple relays, but deletion is only advisory and its common encryption (NIP-44, NIP-17) has no forward secrecy. MLS (RFC 9420) is the standard group-key layer, but the Go implementations are immature and unaudited.

### Cited Findings

**Matrix**

- The current spec is v1.19. It defines rooms up to Room Version 12, the "Olm & Megolm" algorithms, threading (`GET /_matrix/client/v1/rooms/{roomId}/threads`, `m.thread` relations) and the Application Service API. The room encryption algorithm field lists `m.megolm.v1.aes-sha2` — [Matrix Client-Server API v1.19](https://spec.matrix.org/latest/client-server-api/).
- Redaction semantics: "Since some events cannot be simply deleted, e.g. membership events, we instead 'redact' events. This involves removing all keys from an event that are not required by the protocol. This stripped down event is thereafter returned anytime a client or remote server requests it. Redacting an event cannot be undone, allowing server owners to delete the offending content from the databases. Servers should include a copy of the `m.room.redaction` event under `unsigned` as `redacted_because`" — [Matrix C-S API: Redactions](https://spec.matrix.org/latest/client-server-api/#redactions).
- Room v11 redaction algorithm: "the server must strip off any keys not in the following list: event_id type room_id sender state_key content hashes signatures depth prev_events auth_events origin_server_ts". Content is stripped too, except for whitelisted keys on membership, create, join-rule and power-level events — [Room Version 11](https://spec.matrix.org/latest/rooms/v11/).
- The design that makes redaction verifiable: "The event ID is the reference hash of the event" — [Room v11](https://spec.matrix.org/latest/rooms/v11/). The reference hash is computed after "The event is put through the redaction algorithm". "The content hash of an event covers the complete event including the unredacted contents". If the content-hash check fails, "it is assumed that this is because we have only been given a redacted version of the event", and the server uses the redacted copy. Signature checks "should succeed whether we have been sent the full event or a redacted copy" — [Matrix Server-Server API](https://spec.matrix.org/latest/server-server-api/).
- Send-to-device messages give exactly-once delivery to each device "without them being stored permanently as part of a shared communication history" — [Matrix C-S API](https://spec.matrix.org/latest/client-server-api/).
- Servers in the wild (TWIM, April 2026): Synapse 15,082 instances (80.1%), Continuwuity 1,356 (7.2%), Conduit 611 (3.2%), Dendrite 369 (2.0%) — [This Week in Matrix 2026-04-07](https://matrix.org/blog/2026/04/07/this-week-in-matrix-2026-04-07/).
- **Dendrite (Go)**: the matrix-org repo now says "Dendrite is now maintained at element-hq/dendrite". The Foundation "is unable to resource maintenance", and "it continues to be developed by Element". Licence is Apache-2.0 — [matrix-org/dendrite](https://github.com/matrix-org/dendrite) **[MOVED; the matrix-org repo is no longer the home]**.
- **conduwuit [ARCHIVED]**: archived and unmaintained. Continuwuity is "the official community continuation", and tuwunel is another fork; both are Rust — [continuwuity.org](https://continuwuity.org/introduction); [comparison post](https://peachpie.theatl.social/post/23774).
- **mautrix-go**: a Go Matrix framework with appservice support (Intent API, state store), E2EE (key backup, cross-signing, verification) and bridge building. Licence is MPL-2.0. It has about 660 stars and is used by gomuks and the mautrix bridges — [mautrix/go](https://github.com/mautrix/go).
- **MLS in Matrix**: still a proof of concept (page last updated 20 June 2025). MSC2883 is the draft. There is a Rust `matrix-dmls` library, and Element Web has a proof of concept that "does not persist state". The core obstacle: "MLS assumes that epochs...have a linear ordering. However, Matrix being a decentralized system... it is difficult to enforce a linear ordering" — [arewemlsyet.com](https://arewemlsyet.com/); [Matrix blog 2023](https://matrix.org/blog/2023/07/a-giant-leap-with-mls/).

**MLS (RFC 9420)**

- MLS was published in July 2023 on the Standards Track. It "provides efficient asynchronous group key establishment with forward secrecy (FS) and post-compromise security (PCS) for groups in size ranging from two to thousands". Section 9.2 requires secrets from the secret tree to be "deleted according to a deletion schedule". The Delivery Service is "largely untrusted" — [RFC 9420](https://www.rfc-editor.org/rfc/rfc9420.html).
- NIP-EE argues that MLS moves group operations from linear to logarithmic cost compared with pairwise Signal-style schemes — [NIP-EE](https://github.com/nostr-protocol/nips/blob/master/EE.md).
- **Go implementations**:
  - `github.com/emersion/go-mls`: MIT, work in progress, "has not yet been audited for security issues", 32 stars, 220 commits, ships RFC test vectors — [emersion/go-mls](https://github.com/emersion/go-mls).
  - The SourceHut module path is **[DEPRECATED]** ("use github.com/emersion/go-mls"). It is an untagged pseudo-version from May 2024 with zero importers — [pkg.go.dev](https://pkg.go.dev/git.sr.ht/~emersion/go-mls).
  - `github.com/BitravenS/go-mls`: MIT, published June 2025 — [pkg.go.dev](https://pkg.go.dev/github.com/BitravenS/go-mls).

**Nostr**

- **NIP-09 deletion** (`draft`): kind 5 is a "deletion request". "Relays SHOULD delete or stop publishing any referenced events". Relays "SHOULD continue to publish/share the deletion request events indefinitely", which leaves the deletion request behind as a tombstone. "Clients MAY choose to inform the user that their request for deletion does not guarantee deletion because it is impossible to delete events from all relays and clients" — [NIP-09](https://github.com/nostr-protocol/nips/blob/master/09.md).
- **NIP-44 v2**: versioned payload encryption, audited by Cure53 in December 2023. Stated limits: "No deniability", "No forward secrecy: when a key is compromised, it is possible to decrypt all previous conversations", "No post-quantum security" — [NIP-44](https://github.com/nostr-protocol/nips/blob/master/44.md).
- **NIP-17 private DMs** (`draft`): an unsigned kind 14 "rumor" is sealed in kind 13 and gift-wrapped in kind 1059 by a fresh random key for each message, with `created_at` randomized up to two days back. The set of `pubkey` plus `p` tags defines a room, and a membership change starts "a new room... with a clean message history". Forward secrecy is "optional", through disappearing messages (an `expiration` tag). Messages are "Fully Recoverable... with the user's private key". "Group chats with more than 10 participants should find a more suitable messaging scheme." Relays SHOULD serve kind 1059 only to the p-tagged user after NIP-42 AUTH — [NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md).
- **NIP-29 relay-based groups** (`draft`): a group is an id plus the relay that enforces its rules. Groups can migrate or fork by copying events to another relay — [NIP-29](https://github.com/nostr-protocol/nips/blob/master/29.md).
- **NIP-EE (MLS over Nostr)**: `final` but **[SUPERSEDED / "unrecommended"]** by the Marmot Protocol. It says NIP-17 "doesn't solve forward secrecy or post compromise security" — [NIP-EE](https://github.com/nostr-protocol/nips/blob/master/EE.md); [Marmot](https://github.com/marmot-protocol/marmot). NIP-04 DMs are "not recommended" because they leak metadata — [NIP-EE](https://github.com/nostr-protocol/nips/blob/master/EE.md).
- **NIP-34 git** (`draft`): the event kinds are:
  - repository announcement, kind 30617, with `clone`, `web` and `relays` tags;
  - repository state, kind 30618;
  - patch, kind 1617, whose content is `git format-patch` output and which carries an `r` tag holding the "earliest-unique-commit-id-of-repo";
  - pull request, kind 1618, and PR update, kind 1619, both carrying `clone` URLs;
  - issue, kind 1621;
  - statuses open, applied or merged, closed and draft, kinds 1630–1633.

  Replies use NIP-22 comments. `nostr://` URLs work with `git clone` through a `git-remote-nostr` helper. Git objects stay on ordinary git servers, which the clone URLs point to — [NIP-34](https://github.com/nostr-protocol/nips/blob/master/34.md).

**XMPP**

- XEP-0424 Message Retraction is "Proposed", with Council Last Call through 20 July 2026. A retraction "can only be considered an unenforceable request". Retraction messages must be kept in MAM so offline clients learn of them. Services that support tombstones replace content with `<retracted/>` and keep evidence that the message existed. "There can never be a guarantee that a retracted message was never seen". XEP-0425 covers moderator removal — [XEP-0424](https://xmpp.org/extensions/xep-0424.html).

**AT Protocol / Bluesky**

- The spring 2026 roadmap makes "permissioned data", meaning "non-public data with explicit access control", a major focus. It will need "new protocol concepts, sync mechanisms, and data flows", and details are not final. The roadmap does not mention DMs, E2EE or MLS — [AT Protocol Roadmap, Spring 2026](https://atproto.com/blog/2026-spring-roadmap).
- Bluesky DMs still depend on centralized services run by Bluesky Social PBC — [ecosistemastartup](https://ecosistemastartup.com/?p=93564) (secondary source). Matrix Live S12E04 was titled "Matrix DMs in Bluesky", but the post gives no detail — [TWIM 2026-04-07](https://matrix.org/blog/2026/04/07/this-week-in-matrix-2026-04-07/).

**Zulip topics**

- Two levels, channel and topic: "Lots of conversations can happen in the same channel at the same time, each in its own topic." Topics can be renamed later — [Zulip help: topics](https://zulip.com/help/introduction-to-topics).

**SimpleX and Signal**

- SimpleX: when a message is deleted, "Receiving clients MUST implement it as soft-delete, replacing the original chat item with a special chat item indicating that 'message is deleted.'" Users can remove it fully later. SMP queues drop messages after delivery, groups have no global identifier, and PQ keys are added to the agent envelope — [SimpleX Chat protocol](https://simplex.chat/docs/protocol/simplex-chat.html).
- Signal Double Ratchet: forward security ("Output keys from the past appear random to an adversary who learns the KDF key") and post-compromise security. The spec advises per-session limits on stored skipped message keys (for example 1000) and deleting them "after an appropriate interval" — [Signal Double Ratchet spec](https://signal.org/docs/specifications/doubleratchet/).

### Inferences

- Matrix pairs a content hash, which is lost on redaction, with a reference hash computed over the redacted form, which is the event ID. This is the strongest standardized precedent for "append-only, verifiable structure with erasable content". It is also lightweight to adopt as a data-model idea without running a Matrix server.
- A Matrix redacted event keeps the `hashes` key, which holds an unsalted SHA-256 over the full original event. For low-entropy content, someone holding a guess could confirm it against that hash. A ParleyInn design should commit to content with a keyed or salted hash, or commit to ciphertext, so that a shredded key leaves nothing to confirm guesses against.
- Nostr's deletion is cooperative: it is a SHOULD, nothing guarantees it, and a tombstone stays. Static identity keys combined with NIP-44 also mean one key compromise exposes all past DMs. Neither is compatible with crypto-shredding as a guarantee unless ParleyInn controls the only relay and the keys.
- For ephemeral cloud sandboxes, Matrix (homeserver-held history, sync tokens), Nostr (relay-held events, re-subscribe by filter) and NIP-17 ("fully recoverable" with the key) all work for offline or stateless clients. MLS and Double Ratchet clients must carry ratchet state, which is hard for disposable sandboxes. Matrix's MLS proof of concept losing state on reload shows the problem.
- Go availability, from best to worst:
  1. Matrix client and appservice through mautrix-go (MPL-2.0).
  2. Matrix server through Dendrite (Element-maintained, about 2% of servers).
  3. MLS through go-mls (unaudited).

  Nostr Go libraries were not checked in this session (see Gaps).

### Gaps

- Megolm's exact security properties (ratchet-forward-only, no PCS within a session, key sharing and backup) were not fetched from the spec page, <https://spec.matrix.org/latest/olm-megolm/megolm/.>
- Dendrite's current release cadence and feature completeness under Element were not established.
- Matrix threads and appservice details beyond endpoint names were not extracted.
- Nostr Go libraries (for example go-nostr and the khatru relay framework) are known from background knowledge but were not verified in this session.
- XMPP OMEMO (Signal-based E2EE) and MIX were not researched. Neither was Go XMPP server or library status.
- Signal's "delete for everyone" semantics and SimpleX's server language and licence were not fetched.
- The Marmot Protocol (the NIP-EE successor) and its maturity were not researched.
- Scale numbers (messages per second, room sizes) for Matrix, Nostr relays and Buzz were not found in primary sources. The one exception is RFC 9420's "two to thousands".

## Q3. Cross-cutting comparison: event model, deletion, offline clients, links to code

### Takeaway

Three event models recur. Hash-linked DAGs (Matrix) and signed-event logs (Nostr, Buzz) support verification. Operation logs and CRDTs (DeltaDB) support real-time co-editing with links from chat to code. Plain mailboxes (files, SQLite or git) are what most agent tools actually ship today. Content "deletion" almost always leaves a tombstone: Matrix's redacted skeleton, Nostr's kind 5 request, the XMPP `<retracted/>` element and SimpleX's soft-delete marker.

### Cited Findings

| System            | Event model                                                                                                  | Storage                                                                                    | Real-time                                                                                                       | Encryption                                                                                               | Deletion: what remains                                                                                                            | Code link                                                                                                           | Go                                                                                   | Licence / maturity                                                                                              |
| ----------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| Matrix            | Room DAG; event ID = reference hash of redacted form ([v11](https://spec.matrix.org/latest/rooms/v11/))      | Homeserver DB                                                                              | Sync API, federation push ([S-S API](https://spec.matrix.org/latest/server-server-api/))                        | Olm/Megolm (`m.megolm.v1.aes-sha2`); MLS proof of concept only ([arewemlsyet](https://arewemlsyet.com/)) | Redacted skeleton, `hashes` and `redacted_because` stay ([C-S API](https://spec.matrix.org/latest/client-server-api/#redactions)) | None native                                                                                                         | mautrix-go; Dendrite at Element ([dendrite](https://github.com/matrix-org/dendrite)) | Spec v1.19; Synapse 80% of servers ([TWIM](https://matrix.org/blog/2026/04/07/this-week-in-matrix-2026-04-07/)) |
| Nostr             | Independent signed events; relay filters                                                                     | Relay DB                                                                                   | WebSocket subscriptions                                                                                         | NIP-44 v2, NIP-17 gift wrap ([NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md))         | Kind 5 request kept "indefinitely"; deletion not guaranteed ([NIP-09](https://github.com/nostr-protocol/nips/blob/master/09.md))  | NIP-34 patches, PRs, status; `r` = root commit ([NIP-34](https://github.com/nostr-protocol/nips/blob/master/34.md)) | Not verified (gap)                                                                   | NIPs mostly `draft`                                                                                             |
| Buzz              | Nostr events plus hash-chained audit log ([ARCH](https://github.com/block/buzz/blob/main/ARCHITECTURE.md))   | Postgres + FTS, Redis, MinIO                                                               | WebSocket, Redis pub/sub                                                                                        | NIP-17 interop for DMs; channel E2EE not documented                                                      | Kind 5 removal in one transaction; `EventDeleted` audit entry                                                                     | NIP-34, branch-as-room, nostr-signed git ([README](https://github.com/block/buzz))                                  | No (Rust)                                                                            | Apache-2.0, pre-1.0                                                                                             |
| Zed Delta         | Fine-grained deltas with stable op IDs; CRDT worktrees ([DeltaDB](https://zed.dev/blog/introducing-deltadb)) | Undisclosed                                                                                | Real-time multiplayer                                                                                           | Undisclosed                                                                                              | Undisclosed                                                                                                                       | Message stored "side by side" with its edit                                                                         | No                                                                                   | Public beta, free ([Delta beta](https://zed.dev/blog/delta-public-beta))                                        |
| Claude Code teams | Per-agent JSON mailbox files ([teams](https://code.claude.com/docs/en/agent-teams))                          | `~/.claude/teams/...`, removed at session end                                              | Automatic delivery; UDS for cross-session ([xsession](https://code.claude.com/docs/en/cross-session-messaging)) | Local OS permissions; relayed through Anthropic servers across machines                                  | Team directory deleted at session end                                                                                             | None                                                                                                                | n/a                                                                                  | Experimental                                                                                                    |
| MCP Agent Mail    | Threads, inbox and outbox copies                                                                             | Git (markdown) + SQLite FTS5 ([repo](https://github.com/Dicklesworthstone/mcp_agent_mail)) | MCP/HTTP                                                                                                        | None stated                                                                                              | Kept in git history                                                                                                               | File leases, pre-commit guard                                                                                       | No (Python)                                                                          | Active, about 2.2k stars                                                                                        |
| Fossil chat       | Flat table, not versioned ([docs](https://fossil-scm.org/home/doc/trunk/www/chat.md))                        | CHAT table, not synced                                                                     | Long poll                                                                                                       | None                                                                                                     | Auto-deleted after 7 days by default; `DROP TABLE`                                                                                | Timeline announcements                                                                                              | No (C)                                                                               | Stable                                                                                                          |
| XMPP              | Stanzas plus MAM archive                                                                                     | Server archive                                                                             | Push stream                                                                                                     | (OMEMO, not researched)                                                                                  | `<retracted/>` tombstone; "unenforceable request" ([XEP-0424](https://xmpp.org/extensions/xep-0424.html))                         | None                                                                                                                | Not researched                                                                       | XEP-0424 Proposed                                                                                               |
| SimpleX           | Unidirectional queues                                                                                        | Queue deleted after delivery                                                               | Push                                                                                                            | Double ratchet + PQ ([spec](https://simplex.chat/docs/protocol/simplex-chat.html))                       | Soft-delete placeholder                                                                                                           | None                                                                                                                | No                                                                                   | Production                                                                                                      |
| Keybase           | Encrypted KBFS plus chat                                                                                     | KBFS                                                                                       | App                                                                                                             | E2E; names and filenames encrypted ([book](https://book.keybase.io/git))                                 | Not researched                                                                                                                    | Encrypted git remote                                                                                                | Client is Go (not verified here)                                                     | Zoom-owned since 2020                                                                                           |

### Inferences

- For links from messages to code, Nostr NIP-34 (root-commit `r` tag plus patch content) and DeltaDB (op-level anchors that "survive any code transformation") are the two concrete models. NIP-34 fits a design built on a git interface. DeltaDB-style anchors would need ParleyInn to record operations between commits.
- Every system that offers real deletion (Fossil, Claude Code teams, SimpleX queues, Buzz ephemeral kinds) gets it by not making the content part of a verifiable history. Every system with verifiable history (Matrix, Nostr, the Buzz audit chain) keeps a tombstone or skeleton. That is the trade-off ParleyInn has to resolve explicitly.

### Gaps

- No primary source gives throughput or latency for any of these systems.
- How Buzz, Delta and Copilot session views behave with offline or ephemeral cloud-sandbox clients is undocumented. Claude Code says only that a message to an offline Remote Control session is delivered after it reconnects.

## Q4. Which designs keep an append-only, verifiable history while allowing content erasure?

### Takeaway

Matrix redaction is the most direct prior art. The identity and DAG hashes cover a redacted projection of each event, so stripping content leaves every link and signature valid. The Buzz hash-chained audit log, Nostr's signed kind 5 tombstones and XMPP's MAM-stored retractions show the "log the deletion as an event" pattern. MLS and Double Ratchet deletion schedules show the key-deletion half of crypto-shredding. None of the surveyed chat systems combines all three: a hash commitment that leaks nothing, per-key encryption, and key destruction as the erasure act.

### Cited Findings

- Matrix computes the event ID over the redacted event, and the content hash covers the unredacted event. Servers accept a redacted copy when the content-hash check fails, and signatures verify on either form. "Redacting an event cannot be undone, allowing server owners to delete the offending content from the databases" — [S-S API](https://spec.matrix.org/latest/server-server-api/); [Room v11](https://spec.matrix.org/latest/rooms/v11/); [C-S API](https://spec.matrix.org/latest/client-server-api/#redactions).
- Nostr keeps deletion requests forever as signed events, and relays SHOULD delete the target. Deletion "does not guarantee" removal everywhere — [NIP-09](https://github.com/nostr-protocol/nips/blob/master/09.md).
- Buzz's audit log is a SHA-256 hash chain over canonical JSON with a single writer (`pg_advisory_lock`). It records `EventDeleted` as an action, and it is per-community in multi-tenant mode — [Buzz ARCHITECTURE.md](https://github.com/block/buzz/blob/main/ARCHITECTURE.md).
- XMPP keeps retractions in MAM, and tombstone-capable servers replace the content — [XEP-0424](https://xmpp.org/extensions/xep-0424.html).
- MLS requires deleting tree-derived secrets on a schedule, so past messages cannot be decrypted after a compromise — [RFC 9420](https://www.rfc-editor.org/rfc/rfc9420.html). Double Ratchet message keys can be deleted one by one, because no other key derives from them — [Signal Double Ratchet](https://signal.org/docs/specifications/doubleratchet/).
- NIP-17 gives "optional forward secrecy" only through disappearing messages. Otherwise messages are recoverable with the long-term key — [NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md). NIP-44 itself has no forward secrecy — [NIP-44](https://github.com/nostr-protocol/nips/blob/master/44.md).
- Fossil keeps chat out of the versioned artifact graph entirely, so it can be erased without touching history — [Fossil chat](https://fossil-scm.org/home/doc/trunk/www/chat.md).

### Inferences

- A pattern for ParleyInn, synthesized from the above:
  1. Make each log entry an envelope: sender key, room or thread, parent hashes, timestamp, kind, and a commitment to the ciphertext, or a keyed MAC of the plaintext.
  2. Hash-chain or Merkle-link the envelopes, which is Matrix's reference-hash idea.
  3. Encrypt the content under a per-subject or per-session data key.
  4. Erase by destroying the key and appending a signed "shredded" event, which follows Nostr kind 5, XMPP retraction and Buzz `EventDeleted`.

  The log stays verifiable end to end, and the content becomes irrecoverable without rewriting history.
- Do not copy Matrix's unsalted content hash into the envelope. After shredding it lets anyone confirm a guessed message. Commit to ciphertext, or use HMAC under the data key, so the commitment dies with the key.
- Chat fan-out to external clients (Matrix, Nostr or Slack bridges) defeats erasure guarantees, because copies leave ParleyInn's custody, as NIP-09 and XEP-0424 both admit. It also conflicts with I4 if ParleyInn itself does the networking. Any chat interface should be a separate, opt-in process reading a projection, with erasure guaranteed only for the custody ParleyInn controls.
- MLS gives forward secrecy and PCS for live group delivery, but it needs ordered epochs, which Matrix's proof of concept has not solved in a decentralized setting. A single-authority ParleyInn store would satisfy MLS's linear-epoch assumption trivially. The Go MLS implementations, however, are unaudited.

### Gaps

- We found no surveyed chat product that documents crypto-shredding (per-key erasure of chat content) as a feature.
- We did not establish whether Buzz's `EventDeleted` audit entries embed content or only a hash, which decides whether its audit chain itself defeats erasure.
- We found no formal analysis of how Matrix redaction interacts with GDPR erasure or with backups.
