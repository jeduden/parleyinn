# Prior art: the stacks agent UIs and orchestrators chose

Scope: the implementation stacks of 27 agent-UI, agent-orchestration and
terminal products, surveyed on 4 October 2026 to inform ParleyInn's
implementation path (Go core, one binary, a room UI). For each: core
language, UI technology per surface, how it ships, its size, how it hosts
agents, multiplayer, license, and one lesson for ParleyInn. Method:

- Open repositories were shallow-cloned at the commit named in each
  section. Language shares are bytes of code files by extension at that
  commit. They exclude vendored code, test data, lockfiles, docs and
  media, so they approximate GitHub's language bar but are not it.
- Build files (`package.json`, `Cargo.toml`, `go.mod`, `build.zig`,
  Tauri, Electron and goreleaser configs) were read from the clone.
- Sizes come from the npm registry (`unpackedSize`, uncompressed) or
  from release-asset `Content-Length` (download, usually compressed).
  Two native binaries were unpacked to check for Bun's embedded
  filesystem marker (`/$bunfs/`) and its version string.
- Claims from secondary sources are marked (secondary). Claims no source
  confirmed are marked (unverified). GitHub's API was not reachable
  from this session, so star counts and API language bars are absent.

Earlier notes are cited, not redone: [T3 Code](../t3code/t3code.md),
[agent UX](../agent-ux/agent-ux.md), [Amp orbs](../amp-orbs/amp-orbs.md),
[OpenAI agent UI](../openai-agent-ui/openai-agent-ui.md) and the
[competitor review](../live-pr-pitch-review/competitors.md).

## OpenCode

[anomalyco/opencode](https://github.com/anomalyco/opencode) (formerly
`sst/opencode`), commit 907b3bc of 2 October 2026, v1.18.34, MIT.

- **Core:** TypeScript (TS 78%, TSX 18%), run on Bun. One package holds
  the agent and an HTTP server; the server publishes an OpenAPI 3.1 spec
  that generates the SDK. "When you run opencode it starts a TUI and a
  server. Where the TUI is the client that talks to the server"
  (`packages/web/src/content/docs/server.mdx`).
- **Surfaces:** TUI on [OpenTUI](https://github.com/anomalyco/opentui)
  with its Solid renderer; OpenTUI's core "is written in Zig" with
  TypeScript bindings (Zig 26%, TS 68% of its code). A Solid web app
  (`opencode web`), an Electron desktop app, an ACP mode
  (`opencode acp`, JSON-RPC over stdio), a Slack app and an SDK.
- **History:** the first OpenCode was Go with Bubble Tea; its repository
  [opencode-ai/opencode](https://github.com/opencode-ai/opencode) is
  archived and says the project "continued under the name Crush". SST's
  1.0 replaced its Go and Bubble Tea TUI with OpenTUI in Zig and Solid
  ([v1.0.0 notes](https://newreleases.io/project/github/sst/opencode/release/v1.0.0),
  secondary). The desktop app moved from Tauri to Electron: its README
  says "built with Electron", and a `tauri-linux` build container and a
  "Tauri Icons" README remain. Reported reasons were WebKit rendering
  and CLI start-up (secondary, unverified).
- **Ships:** `script/build.ts` runs `Bun.build` with `compile` for 13
  targets (musl and no-AVX2 variants) and embeds the web UI bundle in
  the binary. Distributed by a curl installer, npm
  ([opencode-ai](https://www.npmjs.com/package/opencode-ai), 12
  optional platform packages), Homebrew, Scoop and Chocolatey. The
  Linux x64 package is 185.6 MB unpacked; the release tarball is
  60.7 MB, the macOS arm64 zip 45.5 MB, the desktop DMG 151.1 MB and
  the AppImage 159.2 MB.
- **Agent hosting:** it is the agent; other UIs drive it over HTTP or
  ACP. `opencode serve` binds `127.0.0.1:4096`; basic auth only when
  `OPENCODE_SERVER_PASSWORD` is set, and `opencode web` warns that the
  server is otherwise "unsecured".
- **Multiplayer:** none live. `/share` syncs a session to OpenCode's
  servers behind a public `opncd.ai/s/<id>` link.
- **Lesson:** the client/server split from day one let one engine carry
  a TUI, web, desktop, SDK and ACP. The price of TypeScript everywhere
  was a 186 MB binary and two UI rewrites.

## Crush

[charmbracelet/crush](https://github.com/charmbracelet/crush), commit
8da3490 of 4 October 2026, v0.97.1, FSL-1.1-MIT (source-available,
converting to MIT later).

- **Core:** Go (99% of code). Bubble Tea v2, Lip Gloss v2, Ultraviolet,
  Charm's `fantasy` agent library and `catwalk` model catalog; SQLite
  through pure-Go drivers (`ncruces/go-sqlite3`, `modernc.org/sqlite`).
  `go.mod` lists 197 modules, direct and indirect.
- **Surfaces:** terminal only. An opt-in client/server mode
  (`CRUSH_CLIENT_SERVER=1`) serves HTTP over a Unix socket in
  `$XDG_RUNTIME_DIR`, with multi-client tests (`internal/server`); a
  `--host` flag reaches a TCP server.
- **Ships:** goreleaser with `CGO_ENABLED=0` to Homebrew, Scoop, npm
  ([@charmland/crush](https://www.npmjs.com/package/@charmland/crush)),
  AUR, deb and rpm, Nix, and Android targets. The Linux x86_64 tarball
  is 26.7 MB; the static binary inside is 90.8 MB.
- **Agent hosting:** it is the agent. `internal/herdr` reports idle,
  working and blocked to herdr's Unix socket "without screen scraping".
- **Multiplayer:** none found.
- **Lesson:** Go with no cgo cross-compiles one static binary to every
  OS. Even so, SDKs for many providers push it to 91 MB, so a size
  budget needs watching.

## Claude Squad

[smtg-ai/claude-squad](https://github.com/smtg-ai/claude-squad), commit
ce1ffb4 of 20 August 2026, v1.0.20, AGPL-3.0.

- **Core:** Go (90%), Bubble Tea v1 and `creack/pty`.
- **Surfaces:** one TUI listing sessions; `web/` is a Next.js project
  site, not a client.
- **Ships:** Homebrew and a curl `install.sh`; needs tmux installed. The
  Linux tarball is 1.9 MB; the static binary 4.8 MB.
- **Agent hosting:** each agent runs in its own tmux session and git
  worktree. Status comes from hashing the captured pane and matching the
  string "No, and tell Claude what to do differently"; auto-yes mode
  sends Enter to the pane (`session/tmux/tmux.go`).
- **Multiplayer:** none.
- **Lesson:** the smallest stack in this survey works with any agent,
  but its state detection breaks when an agent rewords a prompt.

## herdr

[herdrdev/herdr](https://github.com/herdrdev/herdr) (formerly
`ogulcancelik/herdr`), commit e35f393 of 4 October 2026, v0.9.3,
Apache-2.0.

- **Core:** Rust (95%): ratatui, crossterm, tokio and a vendored
  `portable-pty`. Terminal state comes from a vendored `libghostty-vt`;
  `crates/ghostty-vt/build.rs` runs `zig` to build it.
- **Surfaces:** a TUI inside the user's own terminal, with a sidebar of
  agent states; a CLI and a socket API that agents themselves call.
- **Ships:** "one rust binary, no electron": a curl installer, Homebrew,
  mise and PowerShell. Release binaries, uncompressed: 30.0 MB on Linux
  x86_64, 22.2 MB on macOS arm64. A secondary source's "~10MB" is
  out of date.
- **Agent hosting:** a real PTY per pane, kept by a background server
  across client detach and SSH loss. For 17 agents herdr "reads their
  state from what they draw on screen"; agents may instead report
  `idle`, `working` or `blocked` with `herdr pane report-agent`
  ([agents docs](https://herdr.dev/docs/agents/)).
- **Multiplayer:** none; one person across several SSH machines.
- **Lesson:** a three-state vocabulary that agents push beats scraping,
  and libghostty-vt is now a reusable terminal-state engine.

## Vibe Kanban

[BloopAI/vibe-kanban](https://github.com/BloopAI/vibe-kanban), commit
d5cbb53 of 19 September 2026, Apache-2.0.

- **Core:** Rust (50%) with a React front end (TSX 31%, TS 15%). The
  server crate uses axum, sqlx on SQLite, `rust-embed` for the web
  assets and tokio-tungstenite. Other crates cover relays (WebSocket,
  WebRTC, tunnel), embedded SSH, `trusted-key-auth` and worktrees.
- **Surfaces:** the web UI served by the binary; a Tauri 2 desktop
  wrapper (`crates/tauri-app`); a hosted kanban for teams.
- **Ships:** `npx vibe-kanban`. The npm package is an esbuild-bundled
  Node script that downloads a per-platform zip named in a
  `manifest.json` on an R2 bucket and caches it
  (`npx-cli/src/download.ts`). Binary size: unverified.
- **Agent hosting:** structured, per harness: Claude's stream protocol,
  Codex through the `codex-app-server-protocol` crate pinned to
  `rust-v0.124.0`, ACP (`agent-client-protocol` 0.8), the OpenCode SDK,
  and executors for Gemini, Cursor, Copilot, Droid and Amp.
- **Status:** sunsetting since 10 April 2026: "the vast majority are
  free users", remote services removed after 30 days, the code
  community-maintained and "fully local"
  ([shutdown post](https://www.vibekanban.com/blog/shutdown)).
- **Multiplayer:** team kanban in the hosted service, now removed.
- **Lesson:** a Rust server with an embedded React UI ships as one
  download. The lasting cost was the executors, one per harness.

## Conductor

Proprietary, macOS ([docs](https://www.conductor.build/docs)).

- **Core and surfaces:** "Conductor is built in Tauri. It has a Rust
  backend and uses the native Mac renderer" (Charlie Holtz, co-founder,
  [17 July 2025](https://threadreaderapp.com/thread/1945870105109246401.html)).
- **Ships:** a Mac app; size unverified.
- **Agent hosting:** runs Claude Code, Codex, Cursor and OpenCode in
  per-branch worktrees; whether through SDKs or terminals is
  unverified. Local workspaces send messages "directly to your model
  provider" and keep data under
  `~/Library/Application Support/com.conductor.app`
  ([privacy](https://www.conductor.build/docs/reference/privacy)).
- **Multiplayer:** Conductor Cloud (GA 30 July 2026) stores session
  inputs and outputs on Conductor's servers so members "can
  collaborate"; see the [competitor review](../live-pr-pitch-review/competitors.md).
- **Lesson:** Tauri made a small native-feeling Mac app, but
  multiplayer arrived only with a hosted cloud.

## Crystal and Nimbalyst

[stravu/crystal](https://github.com/stravu/crystal) (MIT) was an Electron
37 app (TS 54%, TSX 43%) with better-sqlite3 and node-pty. It ran
Claude with `--output-format stream-json` and was deprecated in
February 2026 for its successor.

[Nimbalyst/nimbalyst](https://github.com/Nimbalyst/nimbalyst), commit
ee72524 of 4 October 2026, v0.79.1, MIT.

- **Core:** TypeScript (TS 69%, TSX 22%) in Electron 43, with the Claude
  Agent SDK, node-pty, Lexical editors and Yjs.
- **Surfaces:** desktop (macOS, Windows, Linux); a native SwiftUI iOS
  app (`packages/ios`); an Android package in Kotlin; a browser
  extension.
- **Ships:** installers of 491.8 MB (macOS arm64 DMG), 418.7 MB
  (Windows) and 783.8 MB (Linux AppImage), the largest found here.
- **Agent hosting:** Claude Code and Codex through SDKs, OpenCode and
  Copilot in alpha; node-pty terminals.
- **Multiplayer:** team inbox, documents and tracker over a WebSocket
  "sync server"; `packages/collab-protocol` holds the wire types, and
  the server's source was not found in the repository.
- **Lesson:** Electron plus an agent SDK is the fastest route to a rich
  workspace, and the installer size is the bill.

## opcode

[winfunc/opcode](https://github.com/winfunc/opcode) (formerly
`getAsterisk/claudia`), commit d1ca30a of 18 September 2026, AGPL-3.0.

- **Core:** a Rust backend (22%) with a React UI (TSX 63%, TS 13%):
  Tauri 2, rusqlite and tokio. A second binary, `opcode-web`, serves
  the same UI through axum with WebSockets.
- **Agent hosting:** spawns the `claude` CLI with
  `--output-format stream-json` (`src-tauri/src/commands/claude.rs`).
- **Ships:** build from source; the README still says "Release
  Executables Will Be Published Soon" although a v0.2.0 tag exists.
- **Multiplayer:** none.
- **Lesson:** one Rust core can serve both a Tauri window and a browser
  over loopback, which is the shape ParleyInn's B1 room UI would take.

## Sculptor

[imbue-ai/sculptor](https://github.com/imbue-ai/sculptor), commit
f847102 of 18 September 2026, MIT.

- **Core:** Python (56%): FastAPI, SQLAlchemy, Python 3.14, frozen into
  an executable with PyInstaller. Workspaces can run in Docker.
- **Surfaces:** an Electron 42 desktop app (Electron Forge, React; TS
  and TSX 38%) for Apple Silicon and Linux x64 and arm64.
- **Agent hosting:** "integrated harnesses" (Claude Code and Pi), which
  Sculptor controls for lifecycle and events, plus "any terminal-based
  agents" ([integrated harnesses](https://github.com/imbue-ai/sculptor/blob/main/docs/help/integrated_harnesses.md)).
- **Multiplayer:** none found; several agents may share a workspace.
- **Lesson:** Sculptor names the per-harness cost openly; in return it
  gets queued messages, turns that survive a crash and compaction shown
  as an event.

## Zed and Delta

[zed-industries/zed](https://github.com/zed-industries/zed), commit
a846890 of 3 October 2026, v1.22.0. GPL-3.0-or-later for the editor
and collab server, Apache-2.0 for the GPUI crates.

- **Core:** Rust (98%), 249 crates, with its own GPU UI framework, GPUI:
  Metal on macOS and wgpu elsewhere. `crates/gpui_web` is a browser
  platform: "one document-owned canvas", WebGPU preferred with a WebGL2
  fallback.
- **Surfaces:** a desktop editor (macOS, Linux, Windows) and a headless
  remote server for SSH projects.
- **Ships:** installers; the macOS DMG is 118.5 MB, the Linux tarball
  125.5 MB and the remote server 40.1 MB compressed.
- **Agent hosting:** a native agent plus external agents over ACP
  (`acp_thread`, `agent_servers`); Codex arrives through the
  `codex-acp` adapter ([OpenAI note](../openai-agent-ui/openai-agent-ui.md)).
- **Multiplayer:** channels, shared projects and calls through a Rust
  collab server (axum, Postgres through sea-orm, LiveKit). Delta is a
  separate, closed app for macOS, Linux, Windows and the web
  ([public beta](https://zed.dev/blog/delta-public-beta)). Its backend
  is on Cloudflare R2, Durable Objects, KV and D1
  ([security](https://delta.dev/docs/privacy-and-security/security)).
  Reports that Delta's web client is the Rust app compiled to
  WebAssembly fit `gpui_web` but are secondary.
- **Lesson:** one Rust UI framework now reaches desktop and browser,
  after years of work by a funded team. ACP lets an editor host any
  agent over stdio.

## Warp

[warpdotdev/Warp](https://github.com/warpdotdev/Warp), commit 6398d1e of
3 October 2026; AGPL-3.0, with MIT for `warpui_core` and `warpui`.
The client was open-sourced on 27 April 2026 with OpenAI as "founding
sponsor" ([Help Net Security](https://www.helpnetsecurity.com/2026/04/30/warp-open-source-client/),
secondary; the sponsor line is also in the README).

- **Core:** Rust (98%) with its own UI framework (`warpui`), a terminal
  crate, `secret_redaction`, `mcp`, `remote_server` and
  `warp_multi_agent_client`.
- **Surfaces:** desktop terminal (macOS, Linux, Windows); `warp_tui`,
  the "Warp Agent CLI"; a "web-compiled Warp terminal" (`serve-wasm`)
  on build.warp.dev.
- **Ships:** installers from warp.dev; sizes unverified.
- **Agent hosting:** its own agent plus CLI agents (Claude Code, Codex,
  Gemini CLI) run in its terminal.
- **Multiplayer:** shared agent sessions through Warp's servers
  ([competitor review](../live-pr-pitch-review/competitors.md)).
- **Lesson:** a terminal company ended up with its own GPU UI
  framework, a WebAssembly build and a cloud orchestration layer.

## Superset

[superset-sh/superset](https://github.com/superset-sh/superset), commit
01dff43 of 4 October 2026, desktop v1.35.0, Elastic License 2.0
(source-available).

- **Core:** TypeScript (TS 61%, TSX 27%). `packages/pty-daemon` is a
  "long-lived PTY-owning process"; the host service is "a client over a
  Unix socket", so upgrades do not kill shells.
- **Surfaces:** an Electron 41 desktop app with node-pty and xterm.js;
  an Expo React Native iPhone app with xterm (Pro plan, iOS 26); Next.js
  web, API and admin apps; a CLI, SDK and MCP server.
- **Ships:** macOS DMG 686.5 MB, Linux AppImage 947.0 MB (experimental);
  Windows "not yet available".
- **Agent hosting:** any CLI agent in a PTY, one git worktree per task.
- **Multiplayer:** Pages that teammates comment on; remote access to
  another machine through `apps/relay` and `apps/realtime`, Cloudflare
  Workers deployed with wrangler.
- **Lesson:** a PTY daemon apart from the UI host is worth copying, but
  reaching a phone required a hosted relay.

## Codex CLI and the Codex app

[openai/codex](https://github.com/openai/codex), commit 4ad985e of
4 October 2026, rust-v0.160.0, Apache-2.0.

- **Core:** Rust (97%), built with Cargo and Bazel; TUI on ratatui and
  crossterm.
- **Surfaces:** TUI; the app-server JSON-RPC over stdio, Unix socket or
  WebSocket, which every Codex client drives; IDE extension, desktop
  app, web and mobile ([OpenAI note](../openai-agent-ui/openai-agent-ui.md)).
  The desktop app "is built with Electron and Node.js"
  ([Simon Willison](https://simonwillison.net/2026/Feb/2/introducing-the-codex-app/),
  secondary) and is closed source.
- **Ships:** npm [@openai/codex](https://www.npmjs.com/package/@openai/codex)
  aliases six platform builds; the Linux x64 one is 446.7 MB unpacked
  across 46 files. The Linux musl release tarball is 109.3 MB, macOS
  arm64 95.9 MB, and a separate app-server tarball 84.6 MB.
- **Agent hosting:** it is the harness; the app-server is the hosting
  protocol others adopt (Vibe Kanban, T3 Code, `codex-acp`).
- **Multiplayer:** thin: read-only snapshots and requester-only Slack
  threads ([OpenAI note](../openai-agent-ui/openai-agent-ui.md)).
- **Lesson:** one Rust harness and one protocol serve every surface;
  the GUIs on top are Electron and closed.

## Claude Code

Proprietary; npm
[@anthropic-ai/claude-code](https://www.npmjs.com/package/@anthropic-ai/claude-code)
2.1.289 with eight native platform packages.

- **Core:** the Linux x64 package holds one 246.1 MB executable whose
  bytes carry `/$bunfs/` and "Bun v1.4.3": TypeScript compiled by Bun.
- **Surfaces:** terminal; claude.ai/code and the Claude iOS and Android
  apps through Remote Control.
- **Mobile path:** "Your local Claude Code session makes outbound HTTPS
  requests only and never opens inbound ports"; it "registers with the
  Anthropic API and polls for work", and the transcript "is stored on
  Anthropic servers" while connected
  ([Remote Control](https://code.claude.com/docs/en/remote-control)).
- **Multiplayer:** none in a live session.
- **Lesson:** the harness ParleyInn sits under is Bun-compiled TypeScript,
  and its official phone path is a vendor relay over outbound HTTPS.

## Gemini CLI

[google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli),
commit fb972b2 of 2 October 2026, v0.62.0, Apache-2.0.

- **Core:** TypeScript (TS 80%, TSX 17%) on Node 20 or later, bundled
  with esbuild.
- **Surfaces:** an Ink and React TUI (a fork, `@jrichman/ink`); an ACP
  mode (`packages/cli/src/acp`, `docs/cli/acp-mode.md`); an A2A server
  package; a VS Code companion.
- **Ships:** npm [@google/gemini-cli](https://www.npmjs.com/package/@google/gemini-cli),
  98.4 MB unpacked, with optional node-pty; Homebrew. A Node
  single-executable launcher sits in `sea/`; whether releases ship
  from it is unverified.
- **Agent hosting:** it is the agent; editors host it over ACP.
- **Multiplayer:** none.
- **Lesson:** Ink on Node is the default TypeScript TUI; an ACP mode
  makes the agent embeddable in anyone's UI.

## Goose

[aaif-goose/goose](https://github.com/aaif-goose/goose) (formerly
`block/goose`), commit 591edd4 of 2 October 2026, v1.53.0, Apache-2.0;
part of the Agentic AI Foundation at the Linux Foundation.

- **Core:** Rust (56%): crates `goose`, `goose-cli`, `goose-mcp`,
  `goose-sdk` and others.
- **Surfaces:** a CLI; an Electron 43 desktop app (Electron Forge, React,
  Vite) that talks to the Rust binary over ACP through
  `@aaif/goose-acp-client`; an API.
- **Ships:** a curl installer; npm `@aaif/goose-binary-*` per platform;
  the Linux CLI tarball is 84.1 MB (bz2), the macOS desktop zip
  212.9 MB and the deb 170.3 MB.
- **Agent hosting:** it is the agent, and it can use Claude, ChatGPT or
  Gemini subscriptions through ACP providers.
- **Multiplayer:** none found.
- **Lesson:** a Rust core and an Electron shell joined by a standard
  protocol (ACP), not a private one.

## Cline and Roo Code

[cline/cline](https://github.com/cline/cline), commit 39ff235 of
3 October 2026, Apache-2.0.

- **Core:** TypeScript (TS 79%, TSX 19%), layered as SDK packages
  (`@cline/shared`, `llms`, `agents`, `core`) under host apps.
- **Surfaces:** the VS Code extension with a React webview and a
  protobuf host bridge (`apps/vscode/proto`) that also serves JetBrains;
  a CLI on OpenTUI and React; `cline-hub`, a "browser dashboard for the
  Cline hub: live clients, sessions, streaming chat".
- **Ships:** the Marketplace extension; npm `cline` 3.0.68 with six
  platform binaries built by Bun compile (`apps/cli/script/build.ts`).
  The Linux x64 binary package is 137.2 MB unpacked.
- **Multiplayer:** none found.
- **Roo Code**, the Cline fork
  ([RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code),
  Apache-2.0, TS 79%), shut down its extension on 15 May 2026, the date
  of its last commit, to focus on the cloud product Roomote
  ([report](https://www.distillintelligence.com/news/roomote), secondary).
- **Lesson:** a UI that lives only inside an editor's extension host is
  hostage to it; Cline moved to an SDK with a CLI and a hub, and Roo
  closed.

## Cursor

Proprietary.

- **Surfaces:** the desktop editor, a VS Code fork on Electron
  ([DataCamp](https://www.datacamp.com/blog/cursor-vs-vs-code),
  secondary); Cursor Web at cursor.com/agents, an iOS app, Slack,
  GitHub, Bitbucket, Linear and an API
  ([cloud agents](https://cursor.com/docs/cloud-agent)).
- **Agent hosting:** cloud agents run in "isolated VMs in the cloud";
  self-hosted machines can run the tool execution. The CLI's packaging
  is unverified.
- **Multiplayer:** team follow-ups let several people write to one
  agent, with Cursor's own warning about "lateral movement and secret
  exposure" ([competitor review](../live-pr-pitch-review/competitors.md)).
- **Lesson:** background agents became a cloud product, and the
  surfaces fanned out to wherever people already are.

## Amp

Proprietary. npm `@sourcegraph/amp` now resolves to `@ampcode/cli` with
five platform packages.

- **Core:** the Linux x64 executable is 133.8 MB and carries `/$bunfs/`
  and "Bun v1.4.1": TypeScript compiled by Bun.
- **Surfaces:** terminal, ampcode.com, macOS and iOS apps, and a shared
  tmux session into an orb ([Amp orbs note](../amp-orbs/amp-orbs.md)).
- **Agent hosting:** it is the agent; orbs run it on Amp's machines.
- **Multiplayer:** workspace members join a thread on Amp's servers.
- **Lesson:** another Bun-compiled harness; its multiplayer exists
  only because a server holds the threads.

## T3 Code

[pingdotgg/t3code](https://github.com/pingdotgg/t3code), commit b4381985
of 4 October 2026, MIT. Architecture and security are in the
[T3 Code note](../t3code/t3code.md); this section adds packaging.

- **Core:** TypeScript (TS 80%, TSX 17%): a Node or Bun server.
- **Surfaces:** web, Electron 44 desktop, Expo 58 and React Native 0.88
  mobile.
- **Ships:** npm [t3](https://www.npmjs.com/package/t3) "installs the
  self-contained executable for this platform". The Linux x64 package
  is 222.4 MB unpacked across 2,381 files. Per
  `scripts/build-cli-archive.ts` it holds the binary, the web client
  and native `node_modules` (node-pty, msgpackr-extract), not one file.
- **Agent hosting:** Agent SDK, Codex app-server and ACP.
- **Multiplayer:** none; one person, several devices, through a relay.
- **Lesson:** "self-contained" TypeScript still drags native modules
  beside the binary.

## Agor

[preset-io/agor](https://github.com/preset-io/agor), commit b88f662 of
3 October 2026, Business Source License 1.1.

- **Core:** TypeScript (TS 73%, TSX 22%): a daemon on FeathersJS and
  Socket.IO with Drizzle over libSQL or Postgres.
- **Surfaces:** a React and Ant Design board; a CLI with node-pty.
- **Ships:** npm `agor-live` 0.26.9, 122.7 MB unpacked.
- **Agent hosting:** the Claude Agent SDK, the Codex SDK, Gemini CLI's
  core package, and packages for OpenCode, Copilot and Cursor.
- **Multiplayer:** a real-time board with cursors and shared terminals
  ([competitor review](../live-pr-pitch-review/competitors.md)).
- **Lesson:** a multiplayer agent board fits in one Node daemon, but it
  is a networked server under a source-available license.

## Block Buzz

[block/buzz](https://github.com/block/buzz), commit f0eb557 of
4 October 2026, Apache-2.0: "A workspace where humans and agents build
together, on a relay you own."

- **Core:** a Rust relay (47% of code) speaking Nostr; Postgres, Redis
  and S3 behind it ([competitor review](../live-pr-pitch-review/competitors.md)).
- **Surfaces:** a Tauri 2 React desktop app, a Flutter (Dart) mobile
  app and a web client.
- **Agent hosting:** `buzz-acp`, an "ACP harness that bridges Buzz
  events to AI agents".
- **Multiplayer:** yes; people and agents share rooms on a
  self-hostable relay.
- **Lesson:** the nearest stack to a ParleyInn room (Rust core, ACP bridge,
  signed events), but every room depends on a relay server.

## Happy

[slopus/happy](https://github.com/slopus/happy), commit dafe9a5 of
3 October 2026, MIT.

- **Core:** TypeScript (TS 71%, TSX 25%) in four parts: CLI, app,
  server and shared wire types (`@slopus/happy-wire`).
- **How it reaches the phone:** `happy claude` wraps the CLI. In local
  mode it hands the terminal to Claude (`stdio: 'inherit'`); in remote
  mode it reruns the session through the Agent SDK's `query()`. Messages
  are end-to-end encrypted (tweetnacl, libsodium) and relayed by
  `happy-server`: Fastify, Socket.IO, Prisma on Postgres, Redis and
  MinIO. `happy-server-self-host` bundles the server and web app with
  PGlite for `happy server`.
- **Surfaces:** an Expo React Native app for iOS, Android and the web
  (react-native-web), with a Tauri wrapper; a separate Electron app,
  [slopus/happy-desktop](https://github.com/slopus/happy-desktop).
- **Ships:** npm `happy` 1.2.5, 112.8 MB unpacked; app stores.
- **Multiplayer:** none; one person's devices.
- **Lesson:** a phone needs a rendezvous; an encrypted relay that runs
  as one self-hosted process (PGlite) is the friendliest form.

## Omnara

[omnara-ai/omnara](https://github.com/omnara-ai/omnara), commit 3e9edc1
of 2 October 2026, Apache-2.0.

- **2025:** a Python wrapper. PyPI `omnara` 1.6.0, "Omnara Agent
  Dashboard - MCP Server and Python SDK", depends on `claude-code-sdk`,
  FastAPI and fastmcp; `pip install omnara` synced a Claude Code
  session to a web dashboard and an iOS app.
- **2026:** pivoted to Go (80% of code): "an open source platform for
  running managed agents", with state in Postgres, sandboxes from
  several providers, RBAC and a Slack connector. PyPI 1.7.4
  (22 May 2026) is a "Deprecated PyPI redirect".
- **Lesson:** a mobile remote for someone else's harness proved a thin
  product; Omnara and Vibe Kanban both moved on.

## Other mobile and web clients

- **CloudCLI** ([siteboon/claudecodeui](https://github.com/siteboon/claudecodeui),
  AGPL-3.0): an Express, ws and node-pty server with the Claude Agent
  SDK and better-sqlite3; a React and Vite UI with a responsive mobile
  layout; a hosted cloud tier.
- **VibeTunnel** ([amantus-ai/vibetunnel](https://github.com/amantus-ai/vibetunnel),
  MIT, last commit 5 August 2026): a Swift Mac app plus a Node server
  that proxies terminals into any browser; npm `vibetunnel` on Linux.
- Mobile surfaces already covered above: Superset (Expo), Nimbalyst
  (SwiftUI), T3 Code (Expo), Happy (Expo), Cursor (iOS), Claude
  (Remote Control), Codex (iOS, through OpenAI's relay) and Amp (iOS).

## Ghostty

[ghostty-org/ghostty](https://github.com/ghostty-org/ghostty), commit
5dc28bb of 4 October 2026, v1.3.1, MIT.

- **Core:** Zig (77% of code), built with `build.zig` and per-dependency
  `pkg/*` builds.
- **Surfaces:** "The macOS app is a true SwiftUI-based application" with
  a Metal renderer; "The Linux app is built with GTK".
- **libghostty:** "a cross-platform, zero-dependency C and Zig library";
  its first part, `libghostty-vt`, works on macOS, Linux, Windows and
  WebAssembly, with API signatures "still in flux".
- **Ships:** a 33.8 MB macOS DMG; Linux through distribution packages.
- **Consumers:** herdr vendors `libghostty-vt`;
  [cmux](https://github.com/manaflow-ai/cmux) is a "Ghostty-based macOS
  terminal" for agents, "Built with Swift and AppKit, not Electron"
  (Swift 56% of code; license unverified).
- **Lesson:** a shared Zig core with native shells per platform. Its
  VT library is the component agent multiplexers now reuse.

## Summary table

Sizes: "dl" is a compressed download, "unp" is unpacked.

| Product      | Core              | Surfaces and their tech                                            | Ships as                                         | Agent hosting                                       | Multiplayer                       | License             |
| ------------ | ----------------- | ------------------------------------------------------------------ | ------------------------------------------------ | --------------------------------------------------- | --------------------------------- | ------------------- |
| OpenCode     | TS on Bun         | TUI (OpenTUI, Zig core); web (Solid); desktop (Electron); ACP; SDK | Bun-compiled binary, web UI embedded; 186 MB unp | is the agent; HTTP/OpenAPI server, ACP              | public share links                | MIT                 |
| Crush        | Go                | TUI (Bubble Tea v2); opt-in client/server on a Unix socket         | goreleaser static binary; 27 MB dl, 91 MB unp    | is the agent; reports state to herdr                | none                              | FSL-1.1-MIT         |
| Claude Squad | Go                | TUI (Bubble Tea)                                                   | static binary; 1.9 MB dl, 4.8 MB unp; needs tmux | tmux + worktree; scrapes the pane                   | none                              | AGPL-3.0            |
| herdr        | Rust + Zig VT     | TUI (ratatui) in the user's terminal; socket API                   | one binary; 22–30 MB                             | PTY per pane; screen state or self-report           | none                              | Apache-2.0          |
| Vibe Kanban  | Rust              | web (React, embedded); desktop (Tauri 2)                           | npx wrapper fetches binary from R2               | per-harness executors: stream-json, app-server, ACP | team board (removed)              | Apache-2.0          |
| Conductor    | Rust              | macOS app (Tauri, native renderer)                                 | Mac app                                          | unverified                                          | Conductor Cloud, hosted           | proprietary         |
| Nimbalyst    | TS                | desktop (Electron); iOS (SwiftUI); Android (Kotlin)                | installers; 418–784 MB                           | Agent SDK, node-pty                                 | sync server                       | MIT                 |
| opcode       | Rust + TS         | desktop (Tauri 2, React); browser via axum                         | build from source                                | `claude` stream-json                                | none                              | AGPL-3.0            |
| Sculptor     | Python            | desktop (Electron, React)                                          | PyInstaller backend in Electron                  | integrated harnesses; any terminal agent            | none                              | MIT                 |
| Zed          | Rust              | desktop (GPUI: Metal, wgpu); browser GPUI; remote server           | installers; 119–126 MB                           | native agent; ACP over stdio                        | collab server; Delta (Cloudflare) | GPL-3.0, Apache-2.0 |
| Warp         | Rust              | desktop (warpui); agent TUI; wasm web terminal                     | installers                                       | own agent; CLI agents in PTYs                       | shared sessions, hosted           | AGPL-3.0, MIT       |
| Superset     | TS                | desktop (Electron, xterm.js); iOS (Expo); web (Next.js)            | installers; 687–947 MB                           | PTY daemon; any CLI agent                           | Pages; Cloudflare relay           | ELv2                |
| Codex        | Rust              | TUI (ratatui); app-server; desktop (Electron, closed); IDE; web    | npm platform binaries; 109 MB dl                 | is the harness; app-server JSON-RPC                 | thin                              | Apache-2.0          |
| Claude Code  | TS on Bun         | terminal; web and mobile via Remote Control                        | Bun-compiled binary; 246 MB unp                  | is the harness; Agent SDK                           | none                              | proprietary         |
| Gemini CLI   | TS on Node        | TUI (Ink, React); ACP; VS Code companion                           | npm bundle; 98 MB unp                            | is the agent; ACP                                   | none                              | Apache-2.0          |
| Goose        | Rust              | CLI; desktop (Electron) over ACP                                   | binaries; 84 MB dl CLI, 213 MB desktop           | is the agent; ACP                                   | none                              | Apache-2.0          |
| Cline        | TS                | VS Code (React webview, protobuf bridge); CLI (OpenTUI); hub web   | Marketplace; Bun-compiled CLI 137 MB unp         | is the agent; SDK                                   | none                              | Apache-2.0          |
| Cursor       | TS (VS Code fork) | desktop (Electron); web; iOS; Slack; API                           | installer; cloud VMs                             | cloud agents in VMs                                 | team follow-ups                   | proprietary         |
| Amp          | TS on Bun         | CLI; web; macOS and iOS apps                                       | Bun-compiled binary; 134 MB unp                  | is the agent; orbs                                  | workspace threads, hosted         | proprietary         |
| T3 Code      | TS on Node or Bun | web; desktop (Electron); mobile (Expo)                             | npm archive; 222 MB unp, 2,381 files             | Agent SDK, Codex app-server, ACP                    | none                              | MIT                 |
| Agor         | TS on Node        | web board (React, antd); CLI                                       | npm; 123 MB unp                                  | Claude, Codex and Gemini SDKs                       | real-time board, self-hosted      | BUSL-1.1            |
| Block Buzz   | Rust              | desktop (Tauri 2); mobile (Flutter); web                           | relay plus clients                               | ACP bridge                                          | rooms on own relay                | Apache-2.0          |
| Happy        | TS on Node        | mobile and web (Expo); desktop (Electron); CLI (Ink)               | npm 113 MB unp; app stores                       | wraps CLI; Agent SDK in remote mode                 | none; E2E relay to own devices    | MIT                 |
| Omnara       | Python, then Go   | 2025: iOS and web dashboard; 2026: dashboard, API, Slack           | PyPI (deprecated); install script                | 2025: claude-code-sdk; 2026: own agents             | 2026: RBAC teams                  | Apache-2.0          |
| Ghostty      | Zig               | macOS (SwiftUI, Metal); Linux (GTK); libghostty                    | DMG 34 MB; distro packages                       | n/a (terminal)                                      | none                              | MIT                 |

## Patterns

Seven stack shapes recur. Each proved something and failed at something.

- **Terminal multiplexer over real PTYs** (Claude Squad, herdr, cmux,
  Superset, Warp). It works with every agent unchanged, the binaries
  are small (5–30 MB), and detach and SSH come for free. It fails at
  knowing what the agent is doing: Claude Squad matches a prompt
  string, and herdr reads "what they draw on screen" before adding a
  self-report socket, which Crush, OpenCode and others now call. No
  product in this shape keeps a structured record of tool calls. For
  ParleyInn this is a fallback for unknown agents. Nothing scraped from a
  pane can carry the trust labels I2 needs.
- **Compiled core serving an embedded web client** (Vibe Kanban: Rust,
  axum, `rust-embed`; opcode: Tauri plus `opcode-web`; Codex:
  app-server; Crush: Go server on a Unix socket; Zed's remote server).
  It proved one static download, the server as the only source of
  truth, and many thin clients. It is the shape nearest ParleyInn: a Go
  core with `embed.FS`, and the room UI as a loopback-only B1 listener
  apart from the socket-free B0 core (I4). It failed only where the
  product did: Vibe Kanban's executors and its hosted board, not the
  binary.
- **TypeScript everywhere, compiled with Bun** (OpenCode, Claude Code,
  Amp, Cline's CLI; T3 Code nearly). One language spans TUI, server,
  web and desktop, and the harness vendors chose it. It costs
  134–246 MB executables and native modules beside the binary. T3
  Code ships a 2,381-file archive, and Cline's build pre-installs every
  platform's OpenTUI binary, or cross-compiles "fail to resolve
  @opentui/core's FFI layer". It also brought churn: OpenCode rewrote
  its TUI and swapped its desktop shell within a year.
- **Electron shell around a local server or agent** (Codex app, OpenCode
  desktop, Goose, Nimbalyst, Superset, Sculptor, Happy Desktop, T3
  Code, Cursor). It is the fastest route to a rich desktop UI with one
  rendering engine on every OS. It costs 150–950 MB installers, a Node
  runtime in the trusted path, and a second process to secure.
- **Rust plus Tauri** (Conductor, opcode, Vibe Kanban's desktop, Block
  Buzz, Happy's wrapper). It gives smaller installers and a Rust
  backend in-process. It failed for OpenCode, which left Tauri for
  Electron (WebKit differences, reported in secondary sources). opcode
  never published release builds.
- **Native GPU UI in Rust or Zig** (Zed's GPUI, Warp's warpui, Ghostty
  with Swift and GTK shells). It proved speed, and now a browser target
  (`gpui_web`, Warp's `serve-wasm`, `libghostty-vt` on WebAssembly). It
  took funded teams years, which a small team cannot spend. What ParleyInn
  can reuse is `libghostty-vt` as a component, as herdr does.
- **Editor extension** (Cline, Roo Code, Codex and Gemini companions).
  It meets users where they work, but the host owns the UI: Roo Code
  shut down, and Cline grew an SDK, a CLI and a hub to stand on its own.

Three findings cut across the shapes:

- **Agent hosting has converged on structured protocols.** The Claude
  Agent SDK's stream (Happy, Nimbalyst, Agor, CloudCLI, T3 Code),
  Codex's app-server (Vibe Kanban, T3 Code, `codex-acp`) and ACP (Zed,
  Gemini CLI, Goose, OpenCode, Vibe Kanban, Block Buzz) carry typed
  events. PTYs remain only for "any terminal agent". Each adapter is a
  standing per-harness cost, as Sculptor and Vibe Kanban show.
- **Every phone path is a relay.** Claude Remote Control (outbound HTTPS
  through Anthropic), Codex (OpenAI's relay), Happy (an encrypted
  Socket.IO relay, self-hostable), T3 Code and Superset (Cloudflare),
  Nimbalyst (a sync server) and Cursor (cloud VMs). None is peer to
  peer. Expo React Native is the usual mobile client, with SwiftUI
  (Nimbalyst) and Flutter (Block Buzz) as exceptions. For ParleyInn, mobile
  is a B2 or B3 component the tenant turns on, never part of the core.
- **Every multiplayer product has a server in the middle.** Zed's collab
  server and Delta on Cloudflare, Amp, Conductor Cloud, Cursor, Warp,
  Nimbalyst, Superset, Agor's daemon and Block Buzz's relay all have
  one. Only Block Buzz and Agor can be self-hosted. No surveyed
  product syncs a multi-person, multi-agent room peer to peer, which
  leaves that place open for ParleyInn.
