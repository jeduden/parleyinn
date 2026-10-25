# Interesting ideas digest: storing and sharing agent sessions

Source: [lessons-compact.md](../notes/git-alternatives-for-agent-sessions/lessons-compact.md),
194 entries. That is about 175 distinct systems,
because some appear more than once (GitHub Copilot's cloud agent five times,
NIP-34 three times). The "where" and "lesson" fields are cut short in the
file, so some claims below rest on the first few hundred characters of an
entry. Weak entries are marked *(weak)*.

## 1. Top 15 ideas, ranked by how much they would change ParleyInn's design

1. **Git holds the pointer, not the live record.** Every big vendor that owns
   both git and an agent keeps transcripts out of git. In git they leave only
   a trailer or URL. Hugging Face ran exactly this workload, raw Claude Code
   JSONL synced from many machines, and its docs say git "is not meant to work
   as a database with a lot of writes". Performance drops after a few thousand
   commits, and HF says to batch commits at least 5 minutes apart. Git refs can
   still archive sealed segments, but they cannot be the real-time path.
   Shown by: HF Agent Traces (<https://huggingface.co/docs/hub/en/agent-traces>),
   GitHub Copilot cloud agent (<https://docs.github.com/en/copilot/concepts/agents/coding-agent/about-coding-agent>),
   Claude Code cloud sessions (<https://code.claude.com/docs/en/claude-code-on-the-web>),
   Cursor Cloud Agents (<https://cursor.com/docs/cloud-agent>),
   Letta (<https://www.letta.com/blog/context-repositories>).
2. **One writer per log, merged only when read.** Every system that carries
   multi-writer history without losing data has each writer append only to
   its own ref or stream. Readers take the union into an index they can
   rebuild. Two refinements matter. Name the writer by a key hash, not by
   hostname: gitmem's `devices/$HOSTNAME` clobbered history. And make the
   writer the process, not the session: NTM's per-session chain forked as soon
   as two processes appended. Shown by: Rekal (<https://github.com/rekal-dev/rekal-cli>),
   GrayCodeAI/trace *(weak: the repo has held two unrelated codebases)*
   (<https://github.com/GrayCodeAI/trace>), agentsview (<https://github.com/kenn-io/agentsview>),
   NTM (<https://github.com/Dicklesworthstone/ntm>).
3. **Chain the hashes, store the payloads apart.** Hash each payload, keep the
   bytes in a separate store, and chain only the hashes. Retention or redaction
   can then delete bytes and every proof still verifies. Hash the ciphertext,
   so a mirror without the key can still check and serve blocks. Use a keyed
   commitment, not a bare SHA-256 of the prompt, which only confirms guesses.
   This lets append-only coexist with deletion. Shown by: immudb (<https://github.com/codenotary/immudb>),
   Hypercore (<https://github.com/holepunchto/hypercore>), C2SP (<https://c2sp.org/tlog-tiles>),
   OpenFab (<https://github.com/Open-fab-ai/openfab>).
4. **Write segments in a standard log format any carrier can move.** C2SP
   tlog-tiles uses files that never change once written plus one small signed
   checkpoint. That gives Merkle inclusion and consistency proofs, not just a
   linear chain, and any dumb carrier can move it without merging: a git
   branch, a bucket, rsync, a zip. A network-free subset needs about three Go
   modules. Following Litestream, the core writes segments and a separate
   process ships them. Shown by: Tessera (<https://github.com/transparency-dev/tessera>),
   torchwood (<https://pkg.go.dev/filippo.io/torchwood>), sumdb/tlog
   (<https://pkg.go.dev/golang.org/x/mod/sumdb/tlog>), Litestream (<https://litestream.io/>).
5. **The relay accepts an append only if it states where it lands.** Each
   append names the expected sequence number or the producer's id, epoch and
   sequence number. A duplicate counts as success, a gap is refused, and a
   writer with an old epoch is locked out. Ephemeral sandboxes then get
   exactly-once appends with no merge step. Change notices are only hints, and
   a receiver that misses one reads the durable record again. Shown by: S2
   (<https://s2.dev/>), Durable Streams (<https://github.com/durable-streams/durable-streams>),
   NATS JetStream (<https://docs.nats.io/nats-concepts/jetstream>),
   LiveStore (<https://livestore.dev/>), Coder Agents (<https://coder.com/docs/ai-coder/agents>).
6. **Every pull is untrusted input, and the trust label goes with the
   record.** Re-check hashes and run size, parser and secret checks on import,
   and quarantine anything that fails. Set the trust class from the capture
   path, never from the content, and apply it again at every recall. Hermes
   wraps live tool output but returns the same text unwrapped through
   `session_search`. LangGraph had four CVEs from rebuilding typed objects
   out of stored bytes. Shown by: bettermemory (<https://github.com/0Mattias/bettermemory>),
   OpenClaw (<https://docs.openclaw.ai/concepts/memory>), Hermes
   (<https://hermes-agent.nousresearch.com/docs/user-guide/features/memory>),
   LangGraph (<https://docs.langchain.com/oss/python/langgraph/persistence>).
7. **Redact when events are captured, not when they are pushed.** A plain
   `git push` skips claude-nomad's scan. claude-context-sync's `--auto` mode
   silently falls back to plaintext. Sharing should be a separate step that
   fails closed, with each verdict cached by content hash plus policy hash.
   Shown by: claude-nomad (<https://github.com/funkadelic/claude-nomad>),
   claude-context-sync (<https://github.com/Daniel-de-Oliveira-Trindade/claude-context-sync>),
   pi-share-hf (<https://github.com/badlogic/pi-share-hf>).
8. **Prove nothing was skipped, not just that nothing changed.** A hash chain
   cannot show that no source bytes were skipped. Tie each captured chunk to
   the source file: (file, generation, [start,end), sha256), plus a digest of
   the whole prefix. Before each append, re-hash the old prefix. If the source
   shrank or was rewritten, quarantine the session instead of reconciling it.
   Shown by: gitmemory (<https://github.com/doronp/gitmemory>), Opaline
   (<https://github.com/opalinehq/cli>), Xum ADR 0005 (<https://github.com/coder/xum>),
   kcap (<https://www.kurrent.io/how-it-works/>).
9. **The relay sees only ciphertext, and sandboxes can write but not read.**
   Each session gets its own data key, wrapped to an account public key. A
   cloud sandbox can then append history but not decrypt anyone else's. Happy
   also shows that confidentiality is not integrity: its ciphertexts have no
   associated data and no chain. Shown by: Happy (<https://github.com/slopus/happy>),
   C3Sync (<https://getc3.app/sync>), Nimbalyst (<https://github.com/Nimbalyst/nimbalyst>).
10. **Link sessions to commits from what the record already holds.** A Claude
    Code transcript already contains the commit SHA from `git commit` output
    and the patch of every edit. That is enough to attribute commits after
    squash or rebase, with no trailer. Treat a timing or diff match as "not
    proven". Shown by: Lore (<https://github.com/tt-a1i/lore>),
    ctx (<https://github.com/ctxrs/ctx>), SmolForge (<https://forge.smol.ai>).
11. **Keep the trailer as a hint, written by a hook and pointing at a
    hash.** Trailers survive squash and forges, but Claude-Session is
    model-written, and only 3 of 24 Copilot agent commits carried theirs.
    Kernel maintainers strip or rewrite trailers. Write it from a commit-msg
    hook as chain id plus record hash, and never depend on it. Shown by:
    Semantica (<https://github.com/semanticash/cli>), Linux kernel
    (<https://docs.kernel.org/process/coding-assistants.html>), Copilot
    (<https://github.blog/changelog/2026-03-20-trace-any-copilot-coding-agent-commit-to-its-session-logs>).
12. **A fork starts a new chain with a deterministic id.** Only a strict
    prefix merges automatically. When two copies diverge, set fork id =
    H(parent chain ‖ first differing event), so every machine names the fork
    the same way without coordinating. Factory never merges a session: fork,
    compact and rewind all start a new one. Shown by: agent-sync
    (<https://github.com/lidongpeng36/agent-sync>), Factory
    (<https://docs.factory.com/droid-cli/settings>), jj op log
    (<https://docs.jj-vcs.dev/latest/operation-log/>).
13. **Fix the canonical bytes before the first record ships.** Oak's
    newline-joined hash input caused three integrity bugs in six months.
    Go's `encoding/json` HTML-escapes `<>&` by default, which breaks
    JCS-style canonical bytes. Shown by: Oak (<https://oak.space>), in-toto
    predicate *(weak: still an open PR)* (<https://github.com/in-toto/attestation/pull/588>).
14. **Group by repo identity, not by path.** Keep a growing set of
    normalized remotes, or the earliest unique commit, so worktrees, clones,
    forks and renamed repos group together. Join the per-checkout stores
    only when reading. Shown by: lcm (<https://github.com/lossless-claude/lcm>),
    NIP-34 (<https://github.com/nostr-protocol/nips/blob/master/34.md>).
15. **Compaction is an event: the record stays lossless and only the view
    shrinks.** Record compaction as its own event type, and apply limits only
    on the read path. That matches ParleyInn's restore-after-compaction job.
    Shown by: Cloudflare Agents SDK (<https://developers.cloudflare.com/agents/api-reference/agents-api/>),
    GitLab DAP (<https://docs.gitlab.com/user/duo_>agent_platform/sessions/),
    OpenHands (<https://docs.openhands.dev/sdk/guides/convo-persistence.md>).

## 2. Surprises that push against the current proposal

- **Git cannot be the real-time path.** HF's 5-minute batching guidance and
  the commit-count ceiling mean git refs suit sealed, batched segments only.
  The relay has to carry live traffic. The proposal allows for this, but it
  should say so outright.
- **Projects keep moving off git.** MCP Agent Mail ended with SQLite as the
  source of truth. ByteRover dropped git within months. GitButler deleted
  prompt-in-commit. Sculptor dropped two-way git sync, and Task Master moved
  its high-churn data out of the tree. Not one system moved toward git for
  raw history.
- **A plain forge cannot enforce one writer per ref.** Any token with push
  access can rewrite or delete another writer's ref. agentdiff notes that
  custom refs escape branch protection. Code.Storage and Tangled sell exactly
  the server-side rules GitHub lacks. A forge can also fork a log without
  anyone noticing (gittuf). ParleyInn therefore needs signed checkpoints and some
  out-of-band witness, not just per-writer refs.
- **Whatever reaches a forge stays there.** whogitit's "retention delete"
  leaves every deleted prompt in the notes history on every remote. Raw
  segments on GitHub can never be redacted, which argues for pushing only
  hashes or ciphertext (idea 3).
- **Refspecs lose and leak data.** A forced `+refs/notes/*` fetch destroyed
  unpushed records (CommitLore #417, gitwhy). Of 30 concurrent `git notes
  add` calls, all exited 0 and only 2–5 notes survived. A wildcard push sent
  memento's private transcripts to the remote.
- **`git gc` breaks code anchors.** A tree hash written into the record
  dangles after `gc --prune` unless ParleyInn pins it (OpenCode, Kilo).
- **Segment size needs a cap.** One 512 MB `events.jsonl` bricked a Copilot
  session. Syncestra's 2.6 GB mirror caused an out-of-memory kill. Roo's
  checkpoints grew from 40 GB into terabytes.

## 3. Storage locations by count

These are rough keyword counts over the shortened "where" fields, hand-fixed
for negations such as "not git notes". A system can count in several rows.

| Where data lives                           | Approx. systems |
| ------------------------------------------ | --------------- |
| Hosted server or cloud                     | ~75             |
| Local database (SQLite, DuckDB, redb, …)   | ~60             |
| Working-tree files in the code repo        | ~30             |
| Separate git repo (including shadow repos) | ~27             |
| Plain local files outside git              | ~20             |
| P2P or transparency log                    | ~15             |
| Git notes                                  | ~13             |
| Custom refs or orphan branches             | ~12             |
| Commit trailer as the only git footprint   | ~12             |
| CRDT or operation-log VCS                  | ~9              |

Git notes and shared custom refs make up nearly all the documented cases of
silent data loss. A separate repo or working-tree files work only for small,
curated Markdown, such as Letta, Kiro, Junie and Serena.

## 4. Watch list: read these in depth first

1. **Rekal**: per-writer refs carrying framed segments. This is closest to the
   current proposal (<https://github.com/rekal-dev/rekal-cli>).
2. **agentsview**: arrived at a per-origin journal plus SHA-256-named
   segments after trying every other option (<https://github.com/kenn-io/agentsview>).
3. **C2SP tlog-tiles plus torchwood**: the candidate segment format
   (<https://c2sp.org/tlog-tiles,> <https://pkg.go.dev/filippo.io/torchwood>).
4. **Durable Streams**: the relay write contract
   (<https://github.com/durable-streams/durable-streams>).
5. **MemPalace**: version vectors and per-origin range pulls for live sync
   (<https://github.com/MemPalace/mempalace>).
6. **Happy**: blind relay and write-only sandboxes (<https://github.com/slopus/happy>).
7. **gitmemory**: the byte-range tiling proof (<https://github.com/doronp/gitmemory>).
8. **HF Agent Traces**: hard numbers on git limits for this exact workload
   (<https://huggingface.co/docs/hub/en/agent-traces>).

Weak or unverified: GNAP and the in-toto predicate are only announced.
whogitit's repo returns 404, so that entry rests on crates.io. GrayCodeAI/trace
has held two unrelated codebases. Copilot Memory's "+7pp merge rate" is the
vendor's own figure. Many systems are beta, and most entries are cut short in
the source file.
