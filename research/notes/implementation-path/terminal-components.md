# Terminal components for the run component

Scope: components that could host a coding-agent harness in a pty,
model its screen headlessly, render it in a browser, or adapt an
existing multiplexer, for ParleyInn's run component, as of 4 October 2026.
Read from source where possible: shallow clones of
[herdrdev/herdr](https://github.com/herdrdev/herdr) at e35f3937b0ef
(4 Oct 2026), its vendored libghostty-vt (Ghostty commit 44f2a44df),
[coder/ghostty-web](https://github.com/coder/ghostty-web) at
1858a5947767, [seruman/hauntty](https://github.com/seruman/hauntty) at
ddcef3d71401, [neurosnap/zmx](https://github.com/neurosnap/zmx) at
2d23c0d44058, [charmbracelet/x](https://github.com/charmbracelet/x) at
ad85c59fdf4e, [owenthereal/upterm](https://github.com/owenthereal/upterm)
at b0d835954b2a, and trees of zellij, wezterm, tmate, libvaxis and
libapps (hterm); versions come from crates.io, npm and the Go module
proxy on the same day. Anything not read at its source is marked
"unverified".

## What ParleyInn asks of these parts

- Terminal hosting and takeover live only in the run component, on the
  harness's machine; no takeover from another machine before PEER —
  [OWN-19](../../../docs/srs/05c-owner-and-peer-requirements.md).
- Owner steering reaches an agent only through the harness's own input,
  "the terminal the run component hosts" among them —
  [OWN-03](../../../docs/srs/05c-owner-and-peer-requirements.md).
  Text typed at the hosted terminal is the harness's channel, recorded
  as `user`, never an owner act (OWN-02, same file).
- The run component is a separate component, off by default, the only
  one that starts programs, with no network beyond loopback to the
  lane-view component; any listener meets SEC-20 or is a local endpoint
  that refuses another UID —
  [SEC-29, SEC-01](../../../docs/srs/06-security.md).
- T21 names the attack these parts must not widen: an unsandboxed agent
  "opens its own pty", "reads a terminal's screen (`tmux capture-pane`
  …)", "injects keystrokes (`tmux send-keys`)" —
  [T21](../../../docs/srs/06-security.md).
- Every direct Go dependency needs an ADR and an Apache-2.0, MIT, BSD
  or ISC license, with at most ten in total —
  [DEPENDENCIES.md](../../../DEPENDENCIES.md). The proposed SQLite
  driver is already a cgo-free wasm2go translation —
  ADR-2609302341.

Slots used below: **pty host** (spawns the harness on a pseudo-terminal),
**headless VT model** (parses output into a screen, scrollback and
state), **browser renderer** (draws that screen for the web room or a
phone), **multiplexer adapter** (drives an external multiplexer instead
of owning the pty).

## herdr

- Rust, version 0.9.3 (29 Sep 2026), "terminal workspace manager for
  AI coding agents" —
  [Cargo.toml](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/Cargo.toml),
  [CHANGELOG](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/CHANGELOG.md).
- License: **Apache-2.0 today**. Release 0.8.0 (3 Aug 2026) says
  "Relicensed Herdr from AGPL-3.0-or-later to Apache-2.0"; releases
  before it stay AGPL-3.0-or-later —
  [CHANGELOG line 297](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/CHANGELOG.md),
  [LICENSE](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/LICENSE).
  Secondary sources still report AGPL with a commercial option —
  [LinuxLinks](https://www.linuxlinks.com/herdr-terminal-based-agent-multiplexer/).
- Popularity: herdr.dev shows about 41.7k stars (not checked on GitHub)
  — [herdr.dev](https://herdr.dev).
- Platforms: Linux, macOS and Windows (named pipes, ConPTY; Windows
  ships as a preview) —
  [Cargo.toml](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/Cargo.toml),
  [README](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/README.md).

### Architecture

- A background server owns every pane's pty and keeps running when the
  client detaches; `herdr` reattaches —
  [README](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/README.md).
- Clients talk to the server over a second socket,
  `herdr-client.sock`, with a length-prefixed binary protocol
  (version 22, 2 MB frame cap) that sends either semantic frames or
  pre-diffed ANSI — [session.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/session.rs),
  [protocol/wire.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/protocol/wire.rs).
- Multi-client attach since 0.5.0; since 0.9.0 clients can view
  different tabs, and "the last one to interact with it controls its
  size" — [CHANGELOG](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/CHANGELOG.md).
  All clients are the same OS user; there is no per-person identity.
- "Mobile" is a narrow-terminal layout for SSH from a phone, not a web
  client — [CHANGELOG](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/CHANGELOG.md).
- Remote machines attach over SSH —
  [README](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/README.md).

### Socket API

- Newline-delimited JSON over a Unix socket (default
  `~/.config/herdr/herdr.sock`, or `HERDR_SOCKET_PATH`), or a named
  pipe on Windows — [socket API docs](https://herdr.dev/docs/socket-api/).
- Auth is the file mode only: the socket is chmod `0o600`; the Windows
  pipe's SDDL grants SYSTEM and the owner —
  [api/server.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/api/server.rs),
  [ipc.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/ipc.rs).
  No token, no peer check.
- Verbs in the API schema include `pane.send_input`, `pane.send_text`,
  `pane.send_keys`, `agent.prompt` (send and wait), `agent.send_keys`,
  `pane.read` (visible, recent, recent-unwrapped, detection),
  `agent.read`, `agent.wait`, `pane.wait_for_output`,
  `events.subscribe`, `agent.start`, `pane.split`, `worktree.create`,
  `integration.install` and `server.stop` —
  [herdr-api.schema.json](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/docs/next/api/herdr-api.schema.json).
- The README sells this surface to agents: "agents drive herdr through
  the cli and socket api: they can spawn panes, prompt each other" —
  [README](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/README.md).

### Agent state detection

- States are idle, working, blocked and unknown, from "terminal tail
  pattern matching" —
  [detect/mod.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/detect/mod.rs).
- Per-agent TOML manifests hold prioritised regex rules over screen
  regions (OSC title, bottom N lines, the line above the prompt box);
  Claude's matches spinner glyphs and "esc to interrupt" —
  [manifests/claude.toml](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/detect/manifests/claude.toml).
- Installed agent hooks report state back over the socket
  (`pane.report_agent`); the Claude hook is a shell script that runs
  Python — [claude/herdr-agent-state.sh](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/integration/assets/claude/herdr-agent-state.sh).
  Hook reports, screen matches and PTY activity are arbitrated as
  "authority" sources —
  [terminal/state.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/src/terminal/state.rs).

### Use of libghostty-vt (verified)

- A workspace crate `ghostty-vt`, "Herdr's binding to its vendored
  libghostty-vt" —
  [crates/ghostty-vt/Cargo.toml](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/crates/ghostty-vt/Cargo.toml).
- The vendored source is Ghostty commit 44f2a44df, version
  `1.3.2-HEAD`, with two local patches (a modifyOtherKeys query, a Wuffs
  build fix) —
  [vendor.json](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/vendor/libghostty-vt.vendor.json),
  [patches.md](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/vendor/libghostty-vt.patches.md).
- `build.rs` runs `zig build -Demit-lib-vt` and demands Zig 0.16.0; it
  maps Rust targets for Linux (gnu, musl), macOS, iOS and Windows MSVC —
  [build.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/crates/ghostty-vt/build.rs).
- The binding calls the C ABI: `ghostty_terminal_vt_write`,
  `ghostty_render_state_*`, `ghostty_key_encoder_encode`,
  `ghostty_snapshot_decoder_*`, `ghostty_kitty_graphics_*` and others —
  [crates/ghostty-vt/src](https://github.com/herdrdev/herdr/tree/e35f3937b0ef/crates/ghostty-vt/src).
- The pty layer is a patched, vendored `portable-pty` pinned to 0.9.0
  — [Cargo.toml](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/Cargo.toml).

### herdr against ParleyInn

- As a run component: no. Any same-UID process can type into any pane
  and read its screen through `pane.send_input` and `pane.read`, which
  is T21 as a feature. Input sent that way is not owner-exclusive, so a
  herdr pane is not "the terminal the run component hosts" (OWN-03).
- As a follow-only adapter: possible, with herdr's events and
  `pane.read` treated as untrusted, and the controls it cannot secure
  shown unavailable (OWN-15).
- As a design and code source: Apache-2.0 now permits reuse with
  notice. The detection manifests and the libghostty-vt build recipe
  are the useful parts. Copy nothing from AGPL-era releases.

## libghostty-vt

- A C library "extracted from the Ghostty terminal emulator": VT
  parsing, terminal state, scrollback, reflow, input encoding —
  [include/ghostty/vt.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt.h).
  Written in Zig, exposed through a C ABI and a Zig module.
- License: MIT, "Copyright (c) 2024 Mitchell Hashimoto, Ghostty
  contributors" —
  [LICENSE](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/LICENSE).
  The build pulls in uucode (graphemes), Wuffs (Kitty graphics) and,
  for SIMD builds, simdutf and Highway —
  [GhosttyZig.zig](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/src/build/GhosttyZig.zig),
  [GhosttyLibVt.zig](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/src/build/GhosttyLibVt.zig).
  Their licenses are permissive by repute; unverified here.
- Announced 22 Sep 2025 as "zero-dependency" and "doesn't even require
  libc", with the C API then still to come —
  [Mitchell Hashimoto](https://mitchellh.com/writing/libghostty-is-coming).

### What is exposed

- API groups: Terminal, Render State (incremental, for custom
  renderers), Formatter (plain text, VT or HTML), Terminal Snapshot,
  Search, OSC and SGR parsers, Paste, Unicode, key, mouse and focus
  encoders, allocators, byte-stream I/O and WebAssembly utilities —
  [vt.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt.h),
  [API docs](https://libghostty.tip.ghostty.org/).
- Snapshot: "an ordered, CRC-protected record stream" whose READY
  marker comes before older scrollback, so a late joiner can render
  first and fill history after —
  [snapshot.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt/snapshot.h).
  That is the late-join primitive a multiplayer room needs.
- Formatter output as plain text gives the recall path a text form of
  the screen without screen scraping —
  [vt.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt.h).
- A tmux control-mode parser exists in Zig (`src/terminal/tmux/`), and
  build info reports whether it is compiled in; it has no C API at this
  commit — [build_info.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt/build_info.h).

### Build targets

- `zig build -Demit-lib-vt` builds the static and shared libraries;
  herdr builds it for Linux, macOS, iOS and Windows —
  [herdr build.rs](https://github.com/herdrdev/herdr/blob/e35f3937b0ef/crates/ghostty-vt/build.rs).
- WebAssembly: `-Dtarget=wasm32-freestanding` yields
  `ghostty-vt.wasm`, about 0.8 MB at ReleaseSmall; it uses `simd128`
  by default (`-Dcpu=generic` turns it off), and `-Dvt-features` can
  trim it, e.g. render-state only for "a read-only terminal viewer" —
  [PACKAGING.md](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/PACKAGING.md).
  Freestanding means no WASI imports, so a Go host needs no system
  interface shim.

### Maturity

- The header says "WARNING: This is an incomplete, work-in-progress
  API. It is not yet stable and is definitely going to change" —
  [vt.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt.h).
- No separate libghostty-vt release exists; Ghostty's latest tag is
  v1.3.1 (13 Mar 2026) —
  [release notes](https://ghostty.org/docs/install/release-notes/1-3-1).
  Consumers vendor a commit, as herdr and zmx do.
- The core is the one Ghostty ships, fuzz-tested for 1.3 —
  [heise](https://heise.de/-11205553).
- Platforms: Linux, macOS, Windows, iOS and wasm32.

### Bindings

- Go: [go-libghostty](https://pkg.go.dev/go.mitchellh.com/libghostty)
  by Mitchell Hashimoto, MIT, **cgo**, static by default through
  pkg-config, cross-built with Zig for Linux, macOS and Windows on
  x86_64 and aarch64; pseudo-version of 4 Oct 2026; "I'm not promising
  any API stability yet". Source now lives on
  [tangled.org](https://tangled.org/mitchellh.com/go-libghostty).
- Go without cgo: hauntty translates `ghostty-vt-small.wasm` to Go with
  wasm2go and mirrors a subset of go-libghostty's API — see
  [Go components](#go-components).
- Rust, C++, .NET, Dart, Python, Node-API
  ([coder/libghostty-vt-node](https://github.com/coder/libghostty-vt-node),
  npm `@coder/libghostty-vt-node` 0.1.0-beta.0) and TypeScript bindings
  are listed in
  [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty);
  each unverified beyond that list.

### libghostty-vt against ParleyInn

- The strongest headless VT model found: correct, maintained, MIT, one
  core usable natively, in Go via WASM and in the browser via WASM.
- Risk: unstable API, so pin a commit and wrap it behind a ParleyInn
  interface. Building it needs Zig 0.16 in the release pipeline, or a
  checked-in, hash-verified `.wasm`, as hauntty does.

## Browser renderers on libghostty

- **ghostty-web** (Coder): MIT, TypeScript, npm 0.4.0 (9 Dec 2025),
  last commit 28 Jun 2026; an xterm.js-compatible API, a canvas
  renderer and "~400KB WASM bundle"; written for Coder's Mux —
  [README](https://github.com/coder/ghostty-web/blob/1858a5947767/README.md),
  [CHANGELOG](https://github.com/coder/ghostty-web/blob/1858a5947767/CHANGELOG.md).
  It builds from a pinned Ghostty submodule plus a 1,620-line patch
  that adds its own `terminal.h` —
  [patch](https://github.com/coder/ghostty-web/blob/1858a5947767/patches/ghostty-wasm-api.patch).
  Its WASM ABI is therefore not upstream's; sharing one `.wasm` between
  a Go host and ghostty-web would mean adopting that patch.
- **Restty**: MIT, TypeScript, npm 0.3.0, "powered by libghostty-vt in
  WASM, WebGPU with WebGL2 fallback", with an xterm.js-style wrapper;
  "early-release software" —
  [wiedymi/restty](https://github.com/wiedymi/restty). Whether it uses
  the upstream ABI is unverified.
- browstty, vscode-bootty, webterm and RemoteTTYs are further
  libghostty WASM terminals, unverified beyond
  [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty).
- A ParleyInn-owned renderer is also feasible: upstream
  `ghostty-vt.wasm` built with render-state only, drawn to a canvas or
  DOM grid; the `wasm.h` notes cover memory growth for JS hosts —
  [wasm.h](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/include/ghostty/vt/wasm.h).

## xterm.js and hterm

- **xterm.js**: MIT, TypeScript, about 21.3k stars, npm `@xterm/xterm`
  6.0.0 (22 Dec 2025), 6.1 in beta; used by VS Code —
  [xtermjs/xterm.js](https://github.com/xtermjs/xterm.js),
  [npm](https://www.npmjs.com/package/@xterm/xterm).
  `@xterm/headless` is "a stripped down version of xterm.js that runs
  headless in Node.js"; `@xterm/addon-serialize` turns the buffer into
  VT sequences or HTML for reconnects — same source.
  Zellij's web client and ttyd ship it (see below).
  Fit: the safest browser renderer; headless needs a JS runtime on the
  server, which only a TypeScript build has.
- **hterm**: BSD-3-Clause (ChromiumOS authors), JavaScript, in
  `libapps`, last commit 29 Sep 2026; "a JS library that provides a
  terminal emulator", the renderer behind Secure Shell; browser-only —
  [hterm/README.md](https://chromium.googlesource.com/apps/libapps/+/617b74be7be0/hterm/README.md).
  The npm mirror `hterm-umdjs` stopped at 1.4.1 in 2022. Fit: viable
  but less used outside Chrome OS; no headless twin.

## Rust components

- **alacritty_terminal** 0.26.0 (6 Apr 2026), Apache-2.0: grid,
  parser, and a `tty` module with Unix and Windows ConPTY back ends —
  crate source from [crates.io](https://crates.io/crates/alacritty_terminal).
  Fit: solid headless model and pty host for a Rust build.
- **vt100** 0.16.2 (12 Jul 2025), MIT: parser plus in-memory screen,
  built for "programs like `screen` or `tmux`"; `contents_formatted()`
  returns a VT redraw — [README](https://crates.io/crates/vt100).
  Fit: small and simple; fewer modern sequences than Ghostty
  (unverified).
- **avt** 0.18.0, Apache-2.0: asciinema's VT, used by its CLI, player
  and server; parsing and buffer only — [README](https://crates.io/crates/avt).
  Fit: one Rust core for server and browser (via the player's WASM
  build, unverified).
- **wezterm-term**, MIT, "The Virtual Terminal Emulator core from
  wezterm" — [term/Cargo.toml](https://github.com/wezterm/wezterm/blob/cab25161054c/term/Cargo.toml).
  Not published on crates.io (a git dependency). WezTerm's last tagged
  release is 20240203-110809; `main` is active (29 Sep 2026).
  `wezterm-mux-server` serves unix, SSH and TLS domains —
  [multiplexing](https://wezterm.org/multiplexing.html); whether several
  clients share one mux at once is unverified. Fit: untagged for over
  two years; `wezterm cli send-text` is the same T21 path as tmux
  (unverified).
- **portable-pty** 0.9.0 (11 Feb 2025), MIT, from WezTerm: Unix ptys
  and ConPTY — [crates.io](https://crates.io/crates/portable-pty).
  herdr pins and patches it. Fit: the default Rust pty host.
- **zellij** 0.45.1 (28 Aug 2026), MIT; 0.46.0 on `main`. Its own grid
  on the `vte` 0.15 crate; plugins run as WASM on `wasmi` —
  [Cargo.toml](https://github.com/zellij-org/zellij/blob/4ba55d724837/Cargo.toml),
  [zellij-server/Cargo.toml](https://github.com/zellij-org/zellij/blob/4ba55d724837/zellij-server/Cargo.toml).
  Multiplayer since 0.23: several users in one session, each with a
  coloured cursor — [news](https://zellij.dev/news/multiplayer-sessions/).
  Web client since 0.43, on `127.0.0.1:8082` by default; login tokens
  stored hashed; read-only tokens "can view sessions but cannot send
  input"; HTTPS required off loopback; "authenticated users are
  trusted" — [web client docs](https://zellij.dev/documentation/web-client.html),
  [0.43 announcement](https://dev.to/imsnif/zellij-043-brings-the-terminal-to-your-browser-57lg).
  The browser side ships `xterm.js` —
  [zellij-client/assets](https://github.com/zellij-org/zellij/tree/4ba55d724837/zellij-client/assets).
  Fit: the closest existing model of ParleyInn's room (read-only versus
  input credentials), but a same-user `zellij action write-chars` path
  exists (unverified here) and its tokens are not owner acts.

## Shared-terminal relays

- **tmate**: fork of tmux, ISC/BSD, last tag 2.4.0, last commit 29 Jul
  2026; default server `ssh.tmate.io`; clients can join read-only —
  [README](https://github.com/tmate-io/tmate/blob/985ab6140c4a/README.md),
  [options-table.c](https://github.com/tmate-io/tmate/blob/985ab6140c4a/options-table.c),
  [tmate-protocol.h](https://github.com/tmate-io/tmate/blob/985ab6140c4a/tmate-protocol.h).
- **upterm**: Go, Apache-2.0, v0.34.0; shares a pty through an SSH
  relay, `uptermd.upterm.dev` by default or self-hosted; joiners limited
  by `--authorized-keys` or GitHub users —
  [README](https://github.com/owenthereal/upterm/blob/b0d835954b2a/README.md).
- Fit: both route terminals through a relay a third party runs by
  default, against I4, and enable takeover from another machine, which
  OWN-19 forbids before PEER. Design references for B2 only.

## Go components

- **creack/pty** v1.1.24 (31 Oct 2024), MIT, about 2.1k stars; Linux,
  macOS, the BSDs, Solaris, z/OS. On Windows `StartWithSize` returns
  `ErrUnsupported` —
  [start_windows.go](https://github.com/creack/pty/blob/master/start_windows.go).
  Fit: the standard Unix pty host; needs a ConPTY partner on Windows.
- **charmbracelet/x/conpty** and **x/xpty**: MIT; xpty wraps creack/pty
  and conpty behind one API — [xpty/go.mod](https://github.com/charmbracelet/x/blob/ad85c59fdf4e/xpty/go.mod).
  The repo holds "experimental packages with no promises of backwards
  compatibility" — [README](https://github.com/charmbracelet/x/blob/ad85c59fdf4e/README.md).
- **aymanbagabas/go-pty** v0.2.3 (17 May 2026), MIT, with Windows
  sources — [go-pty](https://github.com/aymanbagabas/go-pty), module
  zip from the Go proxy. An alternative cross-platform pty.
- **charmbracelet/x/vt**: MIT, pure-Go emulator with screen,
  scrollback, damage tracking and key/mouse handling; pseudo-versions
  only, same "experimental" caveat —
  [x/vt](https://github.com/charmbracelet/x/tree/ad85c59fdf4e/vt).
  Fit: no cgo and no WASM, but a second VT core whose output would
  differ from the browser's.
- **hinshun/vt10x**: MIT-style (James Gray, 2013), last change March
  2022 — [LICENSE](https://github.com/hinshun/vt10x/blob/master/LICENSE),
  [pkg.go.dev](https://pkg.go.dev/github.com/hinshun/vt10x). Fit: stale.
- **gotty** (sorenisanerd fork, v1.8.0, May 2026, MIT) and **ttyd** (C,
  MIT, 1.7.7): one command served to a browser over WebSocket with
  xterm.js; ttyd is read-only unless `-W`, with basic auth —
  [ttyd README](https://github.com/tsl0922/ttyd/blob/main/README.md),
  [gotty LICENSE](https://github.com/sorenisanerd/gotty/blob/master/LICENSE).
  Fit: reference designs; their auth is weaker than SEC-20.
- **wazero** v1.12.0 (28 May 2026), Apache-2.0: "zero dependencies, and
  doesn't rely on CGO"; passes Core 1.0 and 2.0 tests (2.0 includes
  SIMD); compiler on amd64 and arm64, interpreter elsewhere —
  [README](https://github.com/tetratelabs/wazero/blob/main/README.md).
  Fit: runs upstream `ghostty-vt.wasm` with SIMD, as one direct
  dependency.
- **wasm2go** v0.4.16 (29 Sep 2026), MIT: translates a module into "a
  single Go source file, with no dependencies beyond the standard
  library"; its feature list omits SIMD —
  [README](https://github.com/ncruces/wasm2go/blob/main/README.md).
  Fit: no runtime dependency at all; build ghostty-vt with
  `-Dcpu=generic`, or follow hauntty.
- **hauntty**: Go, MIT, 7 stars, "Terminal session persistence using
  Ghostty's VT parser compiled to WASM"; a daemon with creack/pty, a
  checked-in, SHA-256-pinned `ghostty-vt-small.wasm` turned into a 3.6
  MB generated Go file by a forked wasm2go under `GOEXPERIMENT=simd`,
  and a Unix-socket client —
  [README](https://github.com/seruman/hauntty/blob/ddcef3d71401/README.md),
  [go.mod](https://github.com/seruman/hauntty/blob/ddcef3d71401/go.mod),
  [wasm.go](https://github.com/seruman/hauntty/blob/ddcef3d71401/libghostty/wasm.go),
  [Justfile](https://github.com/seruman/hauntty/blob/ddcef3d71401/Justfile).
  That the fork adds SIMD is inferred, unverified. Fit: proof that the
  cgo-free path works; a hobby project, not a dependency.

## tmux control mode

- tmux is ISC — [COPYING](https://github.com/tmux/tmux/blob/master/COPYING);
  latest tags 3.7c and 3.8-rc3 —
  [tags](https://github.com/tmux/tmux/tags).
- `-CC` turns the client into a line protocol: command replies between
  `%begin` and `%end`, pane bytes as `%output %<pane> <octal-escaped>`,
  flow control through `pause-after` with `%pause` and
  `%extended-output`, format subscriptions through `refresh-client -B`;
  iTerm2 is the main user —
  [Control Mode wiki](https://github.com/tmux/tmux/wiki/Control-Mode).
- Fit: a usable multiplexer adapter for people already running agents
  in tmux, and it pairs with any headless VT model fed by `%output`.
  But tmux's server socket lets any same-user process `send-keys` and
  `capture-pane`, which T21 names; input through tmux cannot count as
  the run component's terminal, so the adapter should be follow-only,
  with input shown unavailable under OWN-15.

## Zig options

- Ghostty's Zig module used directly: **zmx** (MIT, 0.8.1, Zig 0.16)
  depends on Ghostty commit 8af6897c, spawns shells with `forkpty`, and
  keeps one Unix socket per session with several clients —
  [build.zig.zon](https://github.com/neurosnap/zmx/blob/2d23c0d44058/build.zig.zon),
  [daemonize.zig](https://github.com/neurosnap/zmx/blob/2d23c0d44058/src/daemonize.zig),
  [repo](https://github.com/neurosnap/zmx). Linux and macOS only.
- **libvaxis** 0.6.0, MIT, a TUI library with its own terminal widget
  (parser, screen, `Pty.zig`), Zig 0.16 —
  [widgets/terminal](https://github.com/rockorager/libvaxis/tree/173a890d1394/src/widgets/terminal).
- Fit: Zig gives the most direct use of Ghostty, but Zig is pre-1.0;
  Ghostty's packaging notes cite Zig 0.14 while herdr and zmx now need
  0.16 — [PACKAGING.md](https://github.com/ghostty-org/ghostty/blob/44f2a44df7e8/PACKAGING.md).
  In any language, Zig is also the toolchain that builds libghostty-vt
  and cross-compiles cgo.

## TypeScript runtimes

- **Bun**: `Bun.spawn({terminal})` and `Bun.Terminal` give a built-in
  pty since v1.3.5 (December 2025), "only available on POSIX systems"
  — [Bun v1.3.5](https://bun.com/blog/bun-v1.3.5). Bun is MIT but
  "statically links JavaScriptCore and WebKit, which are LGPL-2"; a
  `bun build --compile` binary carries that relinking duty —
  [Bun license](https://bun.com/docs/project/license). Whether
  `Bun.Terminal` works inside a compiled binary is unverified.
- **node-pty** 1.1.0, MIT, a native addon
  ([npm](https://www.npmjs.com/package/node-pty)); bundling it into one
  executable needs per-platform prebuilds (unverified).
- Fit: Bun plus `@xterm/headless` server-side and `@xterm/xterm` in the
  browser gives one VT core on both sides, at the cost of an LGPL
  runtime, no Windows pty and a far larger binary.

## Fit against T21 and the boundaries

- Every external multiplexer here (herdr, tmux, zellij, WezTerm mux,
  zmx, hauntty) publishes a same-user control socket that can type and
  read. A 0600 file mode or a UID check stops other users, not an
  unsandboxed agent running as the owner. Only a run component that
  holds the pty master itself, and accepts input only from the SEC-20
  credentialed lane-view or the local terminal, keeps typing
  owner-exclusive.
- Holding the pty in-process does not stop an agent that can ptrace or
  open its own pty (T21 lists both); it only avoids adding an easier
  path. The residual risk stays where SEC-29 and OWN-22 put it.
- A pty host needs `os/exec`; SEC-01 and SEC-29 allow it only in the
  run component, so the VT model should sit in a package without
  `os/exec` or `net`, importable by the run and lane-view components.
- Every relay or cross-machine takeover (tmate, upterm, Zellij web off
  loopback) is out until PEER (OWN-19, I4).

## Summary

| Component                         | Slot                                   | Language          | License                        | Embed path                                                   | Fit                                                                    |
| --------------------------------- | -------------------------------------- | ----------------- | ------------------------------ | ------------------------------------------------------------ | ---------------------------------------------------------------------- |
| herdr 0.9.3                       | multiplexer (pty host, VT, TUI)        | Rust              | Apache-2.0 (AGPL before 0.8.0) | separate process over Unix socket                            | Poor: open same-user input and read API is T21; borrow detection ideas |
| libghostty-vt (Ghostty 44f2a44df) | headless VT model                      | Zig, C ABI        | MIT                            | static lib (cgo/FFI), WASM via wazero or wasm2go, Zig module | Best model; unstable API, pin a commit                                 |
| go-libghostty                     | headless VT model binding              | Go (cgo)          | MIT                            | cgo static link, Zig toolchain                               | Good if cgo is acceptable                                              |
| ghostty-web 0.4.0                 | browser renderer                       | TypeScript + WASM | MIT                            | JS asset                                                     | Good; patched Ghostty ABI, xterm.js API                                |
| Restty 0.3.0                      | browser renderer                       | TypeScript + WASM | MIT                            | JS asset                                                     | Early release; WebGPU                                                  |
| xterm.js 6.0.0 / headless         | browser renderer; headless model in JS | TypeScript        | MIT                            | JS asset; Node or Bun process                                | Safest renderer                                                        |
| hterm                             | browser renderer                       | JavaScript        | BSD-3-Clause                   | JS asset                                                     | Viable, little used outside Chrome OS                                  |
| alacritty_terminal 0.26.0         | headless VT model and pty host         | Rust              | Apache-2.0                     | Rust crate                                                   | Good for a Rust build                                                  |
| vt100 0.16.2                      | headless VT model                      | Rust              | MIT                            | Rust crate                                                   | Simple; fewer sequences                                                |
| avt 0.18.0                        | headless VT model                      | Rust              | Apache-2.0                     | Rust crate                                                   | Same core in asciinema player                                          |
| wezterm-term / mux                | headless VT model; multiplexer         | Rust              | MIT                            | git dependency; mux socket                                   | No release since Feb 2024                                              |
| portable-pty 0.9.0                | pty host                               | Rust              | MIT                            | Rust crate                                                   | Default Rust pty, ConPTY included                                      |
| zellij 0.45.1                     | multiplexer with web client            | Rust (+ xterm.js) | MIT                            | separate process                                             | Good room model; same-user CLI path, not owner-scoped                  |
| tmate / upterm                    | shared-terminal relay                  | C / Go            | ISC-BSD / Apache-2.0           | separate process plus relay                                  | Out: third-party relay, remote takeover                                |
| creack/pty 1.1.24                 | pty host                               | Go                | MIT                            | Go module                                                    | Unix standard; no Windows                                              |
| x/conpty, x/xpty, go-pty          | pty host (Windows too)                 | Go                | MIT                            | Go module                                                    | Experimental but cover ConPTY                                          |
| charmbracelet/x/vt                | headless VT model                      | Go                | MIT                            | Go module                                                    | Pure Go; experimental, second core                                     |
| hinshun/vt10x                     | headless VT model                      | Go                | MIT-style                      | Go module                                                    | Stale since 2022                                                       |
| gotty / ttyd                      | web terminal server                    | Go / C            | MIT                            | separate process                                             | Reference only                                                         |
| wazero 1.12.0                     | WASM runtime for the VT model          | Go                | Apache-2.0                     | Go module, no cgo                                            | Runs upstream WASM with SIMD                                           |
| wasm2go 0.4.16                    | WASM-to-Go translator                  | Go                | MIT                            | build tool; generated code                                   | Zero runtime deps; precedent in go-sqlite3                             |
| tmux -CC                          | multiplexer adapter                    | C                 | ISC                            | separate process, line protocol                              | Follow-only; send-keys is T21                                          |
| zmx / libvaxis                    | session host; TUI terminal widget      | Zig               | MIT                            | Zig build                                                    | Direct Ghostty use; Zig pre-1.0                                        |
| Bun.Terminal / node-pty           | pty host                               | TypeScript        | MIT (Bun links LGPL JSC)       | runtime built-in / native addon                              | POSIX only; LGPL duty in compiled binary                               |

No candidate is GPL or AGPL today. Watch three things: herdr releases
before 0.8.0 (AGPL-3.0-or-later), Bun's statically linked LGPL-2
JavaScriptCore, and the unverified licenses of libghostty-vt's bundled
simdutf, Highway, uucode and Wuffs.

## Combinations that work

1. **Go, no cgo, one VT core everywhere.** creack/pty on Unix plus
   x/conpty or go-pty on Windows; upstream `ghostty-vt.wasm`, pinned by
   hash, translated with wasm2go (zero runtime dependencies, the
   go-sqlite3 precedent) or hosted by wazero (one Apache-2.0
   dependency, SIMD kept). The snapshot API serves late joiners, the
   formatter gives recall a text form, the render-state API drives
   diffs. In the browser, the same `.wasm` behind a small ParleyInn
   renderer, or ghostty-web if its patched ABI is adopted on both sides.
   hauntty shows the Go half working.
2. **Go with cgo.** go-libghostty linked statically, Zig as the C cross
   compiler; native speed, but cgo and Zig enter the reproducible,
   signed release pipeline. Browser as in 1.
3. **Go, pure, two cores.** charmbracelet/x/vt on the server, xterm.js
   in the browser. Simplest build; two emulators may disagree on what
   the screen shows, and x/vt is experimental.
4. **Rust.** portable-pty plus libghostty-vt built the way herdr's
   `build.rs` does, or alacritty_terminal; ghostty-web or xterm.js in
   the browser. herdr's Apache-2.0 code becomes a reusable reference.
5. **TypeScript on Bun.** `Bun.Terminal` plus `@xterm/headless` on the
   server and `@xterm/xterm` in the browser: one core on both sides,
   but POSIX only and an LGPL runtime inside the single binary.
6. **Zig.** Ghostty's Zig module and `forkpty` as in zmx, the same core
   compiled to WASM for the browser; most direct, least stable
   toolchain.
7. **Adapter only, beside any of the above.** tmux `-CC` or herdr's
   events feeding the headless model for a follow-only view of agents
   the run component did not start, treated as untrusted, with input
   shown unavailable (OWN-15).
