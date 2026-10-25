# Tools that store AI coding-agent session history with the code repository (survey, as of 2026-10-01)

## Q1. What is "buzz" (mentioned alongside Beads and Gas Town)?

### Takeaway

The best match is **Buzz by Block** (Jack Dorsey's company), announced 21 July 2026. It is an open-source, self-hostable workspace where humans and agents share one log of **signed Nostr events** (messages, workflow steps, approvals, git patches, agent actions). The log is held in **Postgres** with full-text search, and git hosting runs on the relay (NIP-34 plus smart HTTP). It is not a git-native session store: git is one event type inside a relay database, not the place transcripts live. I found no "buzz" component in Beads, Gas Town or Gas City.

### Cited Findings

- Buzz was announced 21 July 2026 as an open-source workspace that puts employees, AI agents, conversations and repositories behind one identity system. Apache-2.0, at github.com/block/buzz — [runtimewire](https://runtimewire.com/article/jack-dorsey-block-buzz-team-chat-ai-agents-git); [winbuzzer](https://winbuzzer.com/2026/07/24/block-launches-buzz-to-unite-team-chat-ai-agents-and-git-xcxwbn/); [Block announcement](https://block.xyz/inside/introducing-buzz-where-humans-and-agents-work-together) (the page itself failed to fetch: header overflow)
- README: "It's a Nostr relay: every message, reaction, workflow step, review approval, and git event is a signed event in one log." Architecture: **Postgres (events + FTS search)**, **Redis** (pub/sub, presence), **S3/MinIO (Blossom)** for media. Git events use **NIP-34** (patches, repo announcements, status). Auth is NIP-42/98 Schnorr. Search is in the `buzz-search` crate on Postgres FTS — [github.com/block/buzz](https://github.com/block/buzz)
- Agent harnesses: `buzz-acp` (Agent Client Protocol harness), `buzz-cli` (agent-first CLI, JSON in and out) and `buzz-agent`. They integrate Claude Code, Codex and Goose. Agents get their own keypairs, channel memberships and audit trail — [github.com/block/buzz](https://github.com/block/buzz); [flaviocopes](https://flaviocopes.com/buzz/)
- README status markers (as fetched): relay, channels and clients "works today". Workflows and push notifications are "in progress". The "git hosting backend" is still listed under open opinions — [github.com/block/buzz](https://github.com/block/buzz)
- The relay "exposes standard Git smart HTTP endpoints, so normal Git clients can clone and push". A committed event is searchable "through the same database row". "Signatures do not encrypt their content". There is no end-to-end encryption by default — [flaviocopes.com/buzz](https://flaviocopes.com/buzz/) (published 3 Aug 2026)
- Gas Town README fetch: "no mention of 'buzz'" — [github.com/steveyegge/gastown](https://github.com/steveyegge/gastown)

### Inferences

- Buzz fits "a tool that stores agent data" next to Beads and Gas Town: it is a 2026 multi-agent coordination system with a durable, signed, append-only event log. Its storage model is "relay DB + signed events", **not** "inside the git repo". For ParleyInn it is a reference for signed append-only events and per-agent identity, not for git-as-storage.
- Buzz treats agent messages as signed but unencrypted data searchable by the workspace. That is the same risk surface ParleyInn's I2 addresses (recalled content reaching a model). I found no Buzz statement on prompt-injection handling for searched history.

### Gaps

- No other credible "Buzz" candidate in the agent-tool space turned up in searches. A small or unrelated project of that name cannot be ruled out. Confirm with the stakeholder.
- I could not read Block's own announcement page (fetch error). I did not verify whether Buzz stores full agent transcripts or only the messages agents post.

## Q2. Beads and Gas Town (Yegge) — storage model

### Takeaway

Beads began as git-committed JSONL. Its current docs describe a **Dolt** (version-controlled SQL) database under `.beads/`, gitignored, which syncs through Dolt remotes. These can be the existing git remote, with history under **`refs/dolt/data`**, separate from code branches. Only small config files are committed to git. Gas Town builds on Beads, uses git worktrees ("hooks") for agent work, and keeps `.events.jsonl` activity logs. Gas City (April 2026) is described as its successor.

### Cited Findings

- Beads is a "git-backed issue-tracking and agent-memory system". Beads are units of work "stored as JSON in Git alongside your code". Gas Town (released 1 Jan 2026) is built on it — [maggieappleton.com/gastown](https://maggieappleton.com/gastown); [rywalker.com/research/gastown](https://rywalker.com/research/gastown)
- Current Beads data locations: `.beads/embeddeddolt/` (default embedded mode, single writer), `.beads/dolt/` (server mode, multiple writers), `.beads/issues.jsonl` (passive export), and `.beads/metadata.json` plus `.beads/config.yaml` (tracked in git). "Only `.beads/metadata.json` and `.beads/config.yaml` are tracked in git; the database and runtime files are gitignored" — [beads.gascity.com/architecture](https://beads.gascity.com/architecture)
- Dolt remotes can be DoltHub, S3, GCS, a filesystem or an existing git remote: "issue history rides under `refs/dolt/data`, separate from code branches". `bd init` auto-configures git origin as the Dolt remote — [beads architecture](https://beads.gascity.com/architecture); [beads FAQ](https://beads.gascity.com/reference/faq)
- Merge: Dolt "merges at the cell level, so most concurrent changes resolve automatically". Hash IDs (`bd-a1b2`, 3–8 chars) exist because sequential IDs "collide the moment two agents or two branches create issues concurrently" — [beads FAQ](https://beads.gascity.com/reference/faq)
- Scale: "fast at the thousands-of-issues scale". For 100k+ issues, split into several databases. `bd gc` compacts closed issues and squashes old Dolt commits — [beads FAQ](https://beads.gascity.com/reference/faq)
- Known hazard: race conditions during simultaneous push and pull across several git clones. The docs advise stopping the server before switching clones — [beads architecture](https://beads.gascity.com/architecture)
- The JSONL export "do[es] not capture Dolt branches, commit history, working-set state, or non-issue tables" — [beads Dolt docs](https://beads.gascity.com/architecture/dolt)
- Gas Town inter-agent mail is built from beads (to/subject/body fields, inbox polling, ack to mark read), and "all mail is git-committed" — [X post, al_from_koii](https://x.com/al_from_koii/status/2007299975781200358) (secondary social-media source)
- Gas Town "hooks" are git worktrees: "Persistent state … Version control – All changes tracked in git". It has town-level and rig-level beads, TOML formula "molecules", and `.events.jsonl` logs used by the "Seance" feature to find and query previous sessions — [github.com/steveyegge/gastown](https://github.com/steveyegge/gastown)
- Gas City, announced 24 April 2026, is described as the SDK successor to Gas Town (composable "packs") — search-result snippet via [dsebastien.net](https://www.dsebastien.net/tag/news/page/7/) (unverified secondary source)

### Inferences

- Beads moved from flat JSONL in the working tree to a database under a custom ref (`refs/dolt/data`). That is the strongest signal from a heavily used multi-agent tool that working-tree files strain under concurrent agents. Their stated reasons are ID collisions, merge semantics and the need for SQL queries.
- The custom-ref pattern (data rides on the code remote but off code branches) recurs in Beads (`refs/dolt/data`) and Entire (`refs/entire/checkpoints/*`).

### Gaps

- I found no dated changelog entry for the JSONL→Dolt switch. Mintlify-mirrored docs describe only the Dolt design. My memory of the older design (committed `issues.jsonl`, SQLite cache, merge driver) is not verified here.
- I did not verify whether Gas Town stores full Claude Code transcripts. The README mentions only `.events.jsonl`.

## Q3. Per-tool storage models (named tools)

### Takeaway

Tools fall into five storage patterns:

1. **Working-tree files**: SpecStory `.specstory/history/*.md`; aider `.aider.chat.history.md`, auto-gitignored; agent-sessions `.claude/sessions/`; Cline `memory-bank/`.
2. **Git notes**: git-ai `refs/notes/ai`, attribution only, with transcripts kept outside git.
3. **Custom refs or orphan branches**: Entire `refs/entire/checkpoints/<shard>/<id>` or the `entire/checkpoints/v1` branch; claude-git-sessions orphan branch `@ccgs/<name>`; Beads `refs/dolt/data`.
4. **Separate local content-addressed store**: re_gent `.regent/`; Cline's shadow git checkpoints.
5. **External service or app-local DB**: Amp threads (server); Cursor `state.vscdb`; Codex `~/.codex/sessions`; Cline globalStorage.

Only Entire and claude-git-sessions put **full verbatim transcripts** into git objects by default once enabled. Entire redacts on a best-effort basis. claude-git-sessions does not redact.

### Cited Findings

**Entire (entire.io) / Checkpoints CLI — active, MIT**

- Launched 10 Feb 2026 by former GitHub CEO Thomas Dohmke, with a $60M seed at a $300M valuation (led by Felicis). The CLI "captures AI agent sessions (prompts, reasoning, transcripts) and indexes them alongside git commits" — [rywalker.com/research/entire](https://rywalker.com/research/entire)
- Agent hooks capture prompts, transcript, tool activity, changed files and token usage. Git hooks link these to a commit with an **`Entire-Checkpoint` trailer** — [docs.entire.io capture-checkpoints](https://docs.entire.io/guides/checkpoints/capture-checkpoints). An `Entire-Attribution` trailer records agent vs human percentages — [docs.entire.io core-concepts](https://docs.entire.io/core-concepts)
- Storage: the default backend from CLI 0.10.0 is refs at **`refs/entire/checkpoints/<shard>/<id>`**, where shard is the last two chars of the ID ("keeps the refs evenly distributed"). The alternate backend is the branch **`entire/checkpoints/v1`**. **Shadow branches** `entire/<commit>-<worktree>` hold work in progress locally and are "never pushed" — [core-concepts](https://docs.entire.io/core-concepts); [github.com/entireio/cli](https://github.com/entireio/cli)
- Each checkpoint ref points to "a commit whose tree is that checkpoint — `metadata.json`, the per-session transcript files, and any subagent task records" — [entireio/cli](https://github.com/entireio/cli)
- "Entire never creates commits on the active branch". Concurrent sessions are tracked separately. "Each worktree has independent session tracking" — [entireio/cli](https://github.com/entireio/cli)
- Push: auto-push is on by default, to an elected remote (`strategy_options.checkpoint_push_remote`, then a habit-derived remote, then `origin`, then the sole remote). It can be disabled with `"push_sessions": false`. A separate "checkpoint remote" option exists — [entireio/cli](https://github.com/entireio/cli); [core-concepts](https://docs.entire.io/core-concepts)
- Redaction: "best-effort", "detected secrets (API keys, tokens, credentials)" redacted before writing a checkpoint. Shadow-branch code snapshots are unredacted ("raw blobs of your working tree") — [entireio/cli](https://github.com/entireio/cli). PII redaction only if the developer enabled it — [errata-bench issue #16](https://github.com/zanwenfu/errata-bench/issues/16) (via search snippet)
- Agents supported: antigravity, claude-code, codex, copilot-cli, cursor, factoryai-droid, opencode, pi — [entireio/cli](https://github.com/entireio/cli)
- Reported incident, 26 Sep 2026: Entire 0.11.0 found that a stale committed `.entire/settings.json` named a `checkpoint_remote` with a different owner. It logged "ignoring checkpoint_remote that appears to belong to another owner; pushing checkpoints to the push remote instead" and pushed `refs/entire/checkpoints/8Z/…` to the **public** origin. Only a WARN log hinted at it — [trost-systems/emotely#199](https://github.com/trost-systems/emotely/issues/199)
- Also reported, via search snippets: a committed `.entire/settings.json` from a hostile repo was copied verbatim into shell commands (`acme/foo;id`, `$(id)` …), a shell-injection vector. Task descriptions were once copied into checkpoints unredacted (since fixed) — [search snippets re entireio/cli PR mirror](https://github.com/rozsazoltan-forks/entire-cli/pull/5) (not independently verified)
- Researchers mined public Entire checkpoints through GitHub code search. Code search now finds only 221 repos with `.entire/settings.json` — [errata-bench#16](https://github.com/zanwenfu/errata-bench/issues/16) (snippet)

**git-ai — active, open source**

- A Git extension that links "every AI-written line to the agent, model, and transcripts or prompts". Attribution lives in **Git Notes under `refs/notes/ai`**. View it with `git log --show-notes="ai"`. Notes sync on push and fetch — [usegitai.com get-started](https://usegitai.com/docs/get-started)
- Prompts and transcripts: "These sessions are scanned and redacted, and saved outside of Git" — [usegitai.com](https://usegitai.com/docs/get-started)
- Standard v3.0.0: a note has an attestation section (line ranges), then `---`, then JSON metadata (`schema_version: "authorship/3.0.0"`, `base_commit_sha`, `prompts`, optional `sessions`/`humans`). An optional `messages_url` points to an external transcript. "The `messages` array field was removed as of v1.3.4 and MUST NOT appear in new notes." Rewrites: rebase/cherry-pick copy notes and recompute lines; squash consolidates prompt records; stash uses **`refs/notes/ai-stash`**. The divider allows reading attestation without loading metadata — [git_ai_standard_v3.0.0.md](https://github.com/git-ai-project/git-ai/blob/main/specs/git_ai_standard_v3.0.0.md)
- Survives `rebase`, `cherry-pick`, `stash`, `merge --squash`, `reset`, `commit --amend` by recomputing on the final state ("eventually-consistent"). Detects 12+ local and 4 background agents — [usegitai.com](https://usegitai.com/docs/get-started)

**Agent Trace — spec, RFC v0.1.0 (Jan 2026)**

- A JSON "trace record" with `version`, `id` (UUID), `timestamp`, `files`, plus optional `vcs`, `tool`, `metadata`. Files contain conversations, and conversations contain line ranges. Contributor type is human, AI, mixed or unknown. Conversations are referenced by **URL** (`url`, `related[]`), not embedded — [agent-trace.dev](https://agent-trace.dev/); [morphllm explainer](https://www.morphllm.com/agent-trace-spec)
- "Storage mechanisms are implementation-defined. The spec is unopinionated about where traces live". There is no privacy guidance. Shaped by Amp, Amplitude, Cline, Cloudflare, Cognition, git-ai, Jules, OpenCode, Tapes, Vercel — [agent-trace.dev](https://agent-trace.dev/); [cognition.ai blog, 29 Jan 2026](https://cognition.ai/blog/agent-trace)
- Has been combined with Jujutsu for lineage tracking — [classmethod](https://dev.classmethod.jp/en/articles/agent-trace-jujutsu-ai-code-tracking/)

**SpecStory — active, Apache-2.0**

- Saves sessions as **Markdown in `.specstory/history/`** in the project. It covers Cursor and Copilot (IDE extensions) and CLI agents including Claude Code, Codex, Cursor CLI, Droid, OpenCode, Gemini CLI and others, through `specstory run <agent>`. It is local-first. "Nothing syncs to SpecStory Cloud without an optional sign-up and login first." Full-text search runs locally or in the cloud. "Lore" mines history into skills — [github.com/specstoryai/getspecstory](https://github.com/specstoryai/getspecstory)
- Commit policy: docs do not state auto-commit. Users configure `.gitignore` if they want history out of VCS — [getspecstory](https://github.com/specstoryai/getspecstory)
- SpecStory ships a "Guard" skill: a pre-commit hook that scans `.specstory/history` for secrets (AWS keys, Bearer tokens, private keys, `ghp_`/`github_pat_`, `password=`) and blocks the commit. It cites pasted API keys, env vars in command output and tokens in error messages as the leak sources — [specstory-guard on skills.sh](https://www.skills.sh/specstoryai/agent-skills/specstory-guard); [tessl](https://tessl.io/skills/github/specstoryai/agent-skills/specstory-guard)
- For Cursor, SpecStory reads the local `state.vscdb` SQLite — [cursor forum / SpecStory FAQ](https://docs.specstory.com/faqs)

**aider — active**

- Defaults: `.aider.input.history` (input history), `.aider.chat.history.md` (Markdown transcript), and optional `--llm-history-file` (raw LLM I/O). `--restore-chat-history` defaults to False. **`--gitignore` defaults to True: it adds `.aider*` to `.gitignore` automatically** — [aider options](https://aider.chat/docs/config/options.html)
- The file lives at `<git_root>/.aider.chat.history.md` — [agent-safehouse aider analysis](https://agent-safehouse.dev/docs/agent-investigations/aider)

**Claude Code session-sharing tools (commit `~/.claude/projects` JSONL)**

- Claude Code natively writes every session as JSONL under `~/.claude/projects/` (tool calls, token usage, subagent traces, hook events) — [frederick-douglas-pearce/claude-code-sessions](https://github.com/frederick-douglas-pearce/claude-code-sessions); messages form a git-like DAG via parent UUIDs — [piebald.ai](https://piebald.ai/blog/messages-as-commits-claude-codes-git-like-dag-of-conversations)
- **claude-git-sessions** (npm, Node 20+, git 2.5+): an orphan branch `@ccgs/<name>` (default `@ccgs/default`). It holds `sessions/<session-id>.jsonl` (verbatim) plus `<id>.meta.json` (name, author, cwd), and `memory/<fact>.md` plus a meta sidecar. Files are keyed by session UUID, so "transcripts from different authors never collide". It uses plumbing (`hash-object`/`update-index`/`write-tree`/`commit-tree`) on a **private temporary index**, so the working tree, index and branch are never touched. A non-fast-forward push is handled by re-fetching and replaying. Pull rewrites `cwd` to the repo root so sessions resume. **No redaction**: "transcripts are pushed verbatim … `--redact` … Not implemented today" — [github.com/ingram-technologies/claude-git-sessions](https://github.com/ingram-technologies/claude-git-sessions)
- **agent-sessions** (`/sessions:init|push|pull`): commits to `.claude/sessions/<date>_<slug>_<author>_<id>/` with `summary.md` (≤120 lines), `transcript.md` (tool calls collapsed) and `meta.json`. Redaction: file-read results stubbed, `.env`/secrets/certs withheld, and pattern redaction of AWS, GitHub, JWT and URL credentials. The export **refuses** if a high-confidence match survives. A preview is shown. It is opt-in per session. Pull reads summaries first (3 by default) and loads the full transcript only with `--full` — [github.com/prajwalgajakesari/agent-sessions](https://github.com/prajwalgajakesari/agent-sessions)

**re_gent ("Git for AI agents", Go) — new, Show HN 2026**

- `.regent/` directory: content-addressed blobs (BLAKE3), refs as session pointers, and a SQLite query index. Every tool-using turn is a Step. Steps form a DAG, each session gets its own branch, and common ancestors deduplicate. `rgt log/blame/show` answer "which prompt wrote each line". Works with Claude Code. Cursor, Cline, Continue and Aider are planned — [sourcepulse re_gent](https://www.sourcepulse.org/projects/29579175); [pkg.go.dev regent](https://pkg.go.dev/github.com/regent-vcs/regent); [HN thread mirror](https://brianlovin.com/hn/48063548)

**Codex CLI (OpenAI)**

- Rollouts are JSONL under `~/.codex/sessions/YYYY/MM/DD/rollout-<ts>-<id>.jsonl[.zst]`. The envelope is `{timestamp, type, payload}`, with types `session_meta`, `turn_context`, `response_item`, `event_msg`, `compacted`. `codex resume` rebuilds state from the rollout — [codex.danielvaughan.com](https://codex.danielvaughan.com/2026/04/11/codex-cli-session-lifecycle-resume-fork-rollouts); [rollout audit trails](https://codex.danielvaughan.com/2026/04/29/codex-cli-rollout-files-session-recording-replay-audit-trails/). Kept in the home directory, not the repo. The zstd compression claim comes from this single third-party blog.

**Cline**

- Task history lives in VS Code globalStorage (`…/globalStorage/saoudrizwan.claude-dev/`) and `~/.cline/data`. `state/taskHistory.json` is the index. `tasks/<id>/` holds `api_conversation_history.json`, `ui_messages.json` and `task_metadata.json`. It is not in the repo — [Cline task-history recovery](https://docs.cline.bot/troubleshooting/task-history-recovery.md); [codeburn cline provider doc](https://raw.githubusercontent.com/getagentseal/codeburn/main/docs/providers/cline.md)
- Checkpoints use a **shadow Git repository** separate from the project's git. It commits after every tool use. For large repos "checkpoints may use significant storage and slow down Cline" — [docs.cline.bot checkpoints](https://docs.cline.bot/core-workflows/checkpoints)
- The Memory Bank is in-repo Markdown under `memory-bank/` (`projectbrief.md`, `productContext.md`, `activeContext.md`, `systemPatterns.md`, `techContext.md`, `progress.md`), driven by `.clinerules/memory-bank.md`, and updated manually or at milestones. These are summaries, not transcripts — [docs.cline.bot memory-bank](https://docs.cline.bot/features/memory-bank)

**Cursor chat exporters**

- Cursor stores chats in SQLite `state.vscdb` (per workspace plus global). Exporters such as `cursor-chat-export` (Python) and MCP readers query it and write Markdown — [eslco/cursor-chat-export](https://github.com/eslco/cursor-chat-export); [Cursor forum guide](https://forum.cursor.com/t/guide-5-steps-exporting-chats-prompts-from-cursor/2825)

**Amp (Sourcegraph/Amp) — server-side threads**

- Threads hold messages, context and tool calls, stored server-side (ampcode.com/feed). Visibility is Unlisted, Workspace-shared, Group-shared (Enterprise) or Private. Threads in workspace-owned projects are shared by default — [ampcode thread sharing](https://ampcode.com/docs/collaborate/thread-sharing)

**Git-based agent memory MCP servers**

- `memory-mcp`: a user-owned git repo of Markdown memories, read-only by default, with `MEMORY_MCP_MODE=read-write` to capture. `agent-memory`: "Markdown is the source of truth and git is the sync". `robo-cortex`: memory grounded in git history — [glama memory-mcp](https://glama.ai/mcp/servers/credp/memory-mcp); [glama agent-memory](https://glama.ai/mcp/servers/xChuCx/agent-memory); [robo-cortex](https://www.getdrio.com/mcp/io-github-robotel-limited-robo-cortex/md)

### Inferences

- The only tools that make full transcripts travel with git by default are Entire and claude-git-sessions. Both chose refs or branches **off the code branch**, to avoid merge noise in code history and working-tree churn. Entire shards refs. claude-git-sessions keys files by UUID, so concurrent appends never conflict.
- The attribution tools (git-ai, Agent Trace) deliberately keep transcripts **out** of git: notes hold line ranges, and transcripts are referenced by URL or held locally after redaction. git-ai removed embedded `messages` in v1.3.4. This is evidence that transcript-in-notes was tried and abandoned.
- Every tool that pushes transcripts relies on regex or pattern secret redaction (Entire, agent-sessions, SpecStory Guard). None claims completeness. Entire calls it "best-effort", and its shadow branches stay unredacted.

### Gaps

- Continue's session storage (believed to be `~/.continue/sessions/*.json`) was not verified this session.
- ccusage and similar readers were not researched. They read `~/.claude/projects` locally and do not store anything in git (unverified).
- I did not get a full Entire checkpoint tree layout (per-session filenames). I found no published size limits for Entire, git-ai or claude-git-sessions.
- Amp backend storage and retention details were not checked.

## Q4. Other projects that explicitly merge repo + session history

### Takeaway

2026 saw a wave of these. In storage terms they split into notes (git-ai), custom refs (Entire, Beads/Dolt), orphan branches (claude-git-sessions), working-tree folders (SpecStory, agent-sessions) and parallel VCS (re_gent, Jujutsu-based flows). HN reaction is mixed. Many argue the "why" belongs in commit messages and that raw sessions are intermediate output.

### Cited Findings

- Show HN "Git for AI Agents" (re_gent): commenters cite history bloat and GC, credential removal and rebase complications, and the difficulty of cross-session reasoning retrieval. Others say "agents are pretty good with git" and only need "skills and hooks". They point to UseGitAI, Entire.io, Jujutsu auto-commit and Triblespace — [brianlovin.com HN mirror 48063548](https://brianlovin.com/hn/48063548)
- An industry summary: many see sessions as "messy intermediate output". Git notes are an option because they do not change commit objects. Some prefer a separate VCS for agents. Chats "contain proprietary code, internal URLs, credentials-adjacent snippets", making chat history "a poor default place to store project memory" — [sesamedisk](https://sesamedisk.com/ai-code-generation-commit-practices/); [shardstitch radar](https://shardstitch.com/radar/ai-coding-chat-history-risk/)
- Other names surfaced but not examined: GitMemo ([Product Hunt](https://www.producthunt.com/p/gitmemo)), ChatCommit (Chrome extension, [webstore](https://chromewebstore.google.com/detail/dhefohpdnmkhiceflikfkjajbfmplflp)), an "agent-history-hygiene" skill ([skills.sh](https://www.skills.sh/daviddwlee84/agent-skills/agent-history-hygiene)), and a personal "permanent archive of every AI conversation" ([cengizhan.com](https://www.cengizhan.com/p/building-a-permanent-archive-of-every))

### Inferences

- Nobody found so far combines (a) full verbatim transcripts, (b) many agents, worktrees and machines, (c) an append-only, tamper-evident record, and (d) treating recalled content as untrusted. Entire comes closest on (a) and (b). Buzz comes closest on signed append-only events.

### Gaps

- I did not inspect GitMemo, ChatCommit or the cengizhan archive.

## Q5. Reported problems: bloat, secret leaks, merge noise, prompt injection

### Takeaway

Secret and PII leakage is the most concretely documented problem: Entire's push to a public origin, 2.4% of AI-config repos holding verified secrets, and the unredacted claude-git-sessions push. Bloat is reported for shadow-git checkpoints (Cline) and raised on HN. Merge noise is mostly *designed around*: off-branch refs, UUID-keyed files, cell-level merge, auto-gitignore. Prompt injection from committed transcripts is a recognised class (repo files and chat history as indirect injection vectors), but I found no tool that defends against it at recall time.

### Cited Findings

- Secrets: an analysis of AI-tool config dirs (`.claude/`, `.cursor/`, `.continue/`) in public repos found ~2.4% hold verified exposed secrets. The author's own committed `.claude/settings.local.json` contained whitelisted commands with secrets passed as env vars (17 Feb 2026) — [ironpeak.be](https://ironpeak.be/blog/leaking-secrets-from-the-claud/)
- Entire leaked transcripts to a public repo through a silent remote fallback (26 Sep 2026) — [emotely#199](https://github.com/trost-systems/emotely/issues/199)
- claude-git-sessions pushes verbatim, with no redaction — [claude-git-sessions](https://github.com/ingram-technologies/claude-git-sessions)
- Aider and SpecStory history contain pasted keys and env output, hence aider's auto-gitignore and SpecStory Guard's pre-commit scan — [aider options](https://aider.chat/docs/config/options.html); [specstory-guard](https://www.skills.sh/specstoryai/agent-skills/specstory-guard)
- Claude Code issue: a "conversation history leak via FileChanged notifications bypasses guard hooks and gitignore" — [claudeissues.com #44909](https://claudeissues.com/issue/44909-conversation-history-leak-via-filechanged-notifications-bypasses-guard-hooks-and) (title only seen; not read)
- Bloat: Cline's shadow repo commits after every tool use and "may use significant storage" — [Cline docs](https://docs.cline.bot/core-workflows/checkpoints). HN commenters raise history bloat and GC — [HN mirror](https://brianlovin.com/hn/48063548). Beads needs `bd gc` to squash old Dolt commits — [beads FAQ](https://beads.gascity.com/reference/faq)
- Hostile repo config: a committed `.entire/settings.json` was reported as a shell-injection vector — [search snippet, entire-cli PR mirror](https://github.com/rozsazoltan-forks/entire-cli/pull/5) (unverified)
- Prompt injection: when agents clone a repo "every file becomes part of the agent's context", and instructions hidden in files hijack the agent. Malicious text can sit "inside a shared note, a chat transcript" until revisited. Memory poisoning through indirect injection is documented — [Unit 42](https://unit42.paloaltonetworks.com/?p=160017); [wiz.io](https://www.wiz.io/api/md/academy/ai-security/prompt-injection-attack); ["Your AI agent's chat history is user input", dev.to](https://dev.to/y11t0/your-ai-agents-chat-history-is-user-input-fl6)

### Inferences

- Committed transcripts carry the worst of both: they are secrets-bearing (they leak outward) and instruction-bearing (they inject inward when another agent reads them). In-repo formats (SpecStory Markdown, agent-sessions `transcript.md`, Cline memory bank) sit in the working tree where agents read files by default. That is the automatic untrusted→model path ParleyInn's I2 forbids. Off-branch refs (Entire, claude-git-sessions, Beads) at least keep transcripts out of the checkout.
- Silent fallback behaviour, as in Entire's remote election, breaks a ParleyInn-style "no silent failures" (I6) rule. ParleyInn should treat push destination ambiguity as a hard stop, not a warning.

### Gaps

- I found no quantitative repo-growth figures (MB per session or per month) for Entire, SpecStory or claude-git-sessions.
- I found no documented real-world incident of prompt injection *through a committed agent transcript* specifically. The risk is argued by analogy to repo-file and memory injection.
