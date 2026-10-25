# Implementation path: the options

Scope: step 1 of answering OQ-32, revision 2, written 4 October 2026.
Each option is one complete way to build ParleyInn: it fills every slot of
the [evaluation frame](constraints.md) in the same template, so step 2
can [compare](comparison.md) them criterion by criterion. The facts
come from the component notes on
[terminals](terminal-components.md),
[protocols and data](protocol-and-data-components.md),
[UI and packaging](ui-and-packaging-components.md),
[five-platform apps](five-platform-apps.md),
[phone reach and live delivery](phone-reach-and-live-delivery.md),
[Windows](windows-support.md) and [Zig](zig-option.md), from
[smalt's Zig tooling](zig-tooling-smalt.md),
[Bun's compile times](compile-times-bun.md) and the
[prior-art survey](prior-art-stacks.md). Nothing here is decided; a
choice lands as an ADR and an SRS change.

## What changed since revision 1

- **The target experience.** The stakeholder wants the same experience
  on Linux, Windows, macOS, iOS and Android: start ParleyInn, then see your
  stuff. ParleyInn becomes an app on five platforms. A desktop app is a
  node; a phone app is a client of the owner's nodes.
- **Harnesses connect directly.** Following the [pitch][pitch]
  (decision 8), a harness the person starts as usual joins through the
  plugin; the app is one more client, never a required hop. Hosting
  every agent in the run component is not the default.
- **A software factory builds ParleyInn.** Agents write, port, review and
  maintain the code, so a missing library weighs less than a good source
  to port from and an oracle to test against, as mdsmith's vendored
  goldmark fork shows. The language choice is reversible: a factory
  ported Bun's 535k lines from Zig to Rust in 11 days against its test
  suite ([compile times](compile-times-bun.md)).
- **Memory safety is a hard constraint** for hostile-input and trust
  code (HC-23, below).
- **New options.** G puts a Flutter UI over a Rust core. H takes
  Ghostty's shape: a Zig core library that renders on the GPU, under a
  thin native host per platform, with TigerStyle discipline and smalt's
  tooling. Where Ghostty renders the terminal, ParleyInn's core renders the
  whole room: chat, diff and terminal, and an embedded browser shows
  results, as Steam's client embeds Chromium. I applies the same shape
  to a Rust core. Zig leaves "options left out".

## How the options were cut

The options differ on three questions: which language holds the core,
where the core runs, and what kind of shell draws the UI. In A, B and C
the core runs only on the owner's desktop nodes, and every shell is a
thin window over the node's page. In D, G, H and I one core library is
linked into every shell, the phone's included, so a phone can verify,
sync and sign with the same code as the node. The shell is a web page
in a webview (A to D), one widget toolkit everywhere (F, G), or, in
Ghostty's shape (H, I): the core draws the whole UI on the GPU into a
surface a thin native host provides on each platform, and an embedded
browser shows results. Everything else is held equal in
[common ground](#common-ground), so a difference in step 2 traces to
the option.

## Common ground

Every option takes these positions; each answers an
[open point](constraints.md#open-points) or a finding of the notes that
does not depend on the language.

1. **One download per platform.** On a desktop, one app carries two
   executables: a static core executable (hooks, MCP server, CLI, TUI,
   kernel worker) and the app executable (window, lane view, run
   component, peer component). The plugin points hooks and MCP at the
   core executable, which starts in milliseconds and links no window
   system. Every process runs one component, fixed at start (SEC-01).
   On a phone, one app: view, sync and the device key, with no core
   B0 role. Whether "one binary" may mean "one download" is the
   stakeholder's ruling.
2. **Starting ParleyInn opens the app** on all five platforms. Its window
   reaches its backend through in-process IPC where the shell allows
   it, so the app's own view opens no loopback listener. A loopback page
   stays available for browsers and editor panels, under SEC-20 with a
   per-launch token and no cookie, since cookies cross ports.
3. **Harnesses connect through the plugin.** The owner's message reaches
   an agent through the harness's own input where the harness offers
   one: Claude Code's channels (a research preview, reached by ParleyInn's
   own stdio MCP server) and Codex's local daemon. Elsewhere the agent
   gets a fixed notice and reads the message when it asks. Neither path
   is on OWN-03's list yet. `parinn run` hosts a harness for takeover and
   witness runs.
4. **One view model.** Statuses, the Needs you order and the vocabulary
   are computed once in the core and sent to every surface as data
   (VIEW-04, VIEW-05, VIEW-14).
5. **Phone reach.** Off the home network, a phone reaches a node the
   owner keeps reachable: a home server, a rented host, or a
   self-hosted Headscale. Android notifies through UnifiedPush to an
   ntfy server the owner runs; an iPhone learns of a held request only
   when opened, unless VIEW-17 admits Apple's push service.
6. **A pty as fallback, with a headless VT model** for late joiners and
   takeover, owned by the run component; owner input only (OWN-19).
7. **Storage.** SQLite with FTS5 in every option. SQLite is in the
   public domain, which is not on ENG-18's allow-list; that ruling
   applies to every option.
8. **Device keys.** Both phones hold P-256 in hardware, not Ed25519,
   and Windows key stores hold no Ed25519. Every option wraps a software
   Ed25519 key or asks for P-256 device keys (open point 13).
9. **Running-app preview.** The app under test opens on its own origin,
   never inside the view's; a mark on it is a snapshot the run component
   takes. It needs a requirement and a SEC-21 decision in any option.
10. **Kernel.** A hermetic, metered sandbox in the `parinn
    kernel-worker` child process (CMP-03 to CMP-05); spike S5 decides
    the guest language (OQ-10).
11. **Vendored forks.** A library ParleyInn changes, or ports into its
    language, lives inside the repository as mdsmith keeps goldmark:
    never a `replace`, the upstream version recorded, every divergence
    listed with a guard test, an equivalence job in CI against the
    original, and an advisory watch the factory runs. ENG-18 counts a
    fork as a direct dependency.
12. **HC-23, memory safety.** Code that parses content from outside the
    trusted boundary, and code that decides trust, cannot perform an
    out-of-bounds access, a use after free, a double free, an
    uninitialised read or a data race, by construction or by stopping
    before the access; every exception (an unsafe block, a foreign
    library such as SQLite) is listed with its fuzz target, and CI
    fails on an unlisted one ([Zig note](zig-option.md)).
13. **Windows.** Native Claude Code on Windows runs commands
    unsandboxed, so widening owner acts there need the owner's recorded
    risk acceptance (OWN-22). Authenticode signing is effectively
    required, so ENG-19's comparison becomes "identical except the
    embedded signature". WebView2 and Android's WebView send vendor
    diagnostics by default; that needs an I4 ruling in every option
    that uses a system webview.

## The options at a glance

| Slot                   | A. Go node, hypermedia                 | B. Go node, TS page                  | C. Go node, TS page, WASM cores                   | D. Rust core, Tauri                    | E. TypeScript on Bun         | F. Kotlin Multiplatform    | G. Rust core, Flutter     | H. Zig core, Ghostty shape                                                                                       | I. Rust core, Ghostty shape                                                               |
| ---------------------- | -------------------------------------- | ------------------------------------ | ------------------------------------------------- | -------------------------------------- | ---------------------------- | -------------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Core                   | Go                                     | Go                                   | Go                                                | Rust library                           | TypeScript on Bun            | Kotlin/Native              | Rust library              | Zig library, C ABI                                                                                               | Rust library, hand-written C ABI                                                          |
| Where the core runs    | desktop nodes                          | desktop nodes                        | desktop nodes; seal and sync modules everywhere   | every shell, phones included           | desktop; phone via Capacitor | every shell                | every shell               | every shell                                                                                                      | every shell                                                                               |
| Desktop shell          | thin Tauri window over the node page   | thin Tauri window over the node page | thin Tauri window over the node page              | Tauri, page over IPC                   | Electron                     | Compose Desktop (JVM)      | Flutter                   | thin native host (window, input, menus, key store); the core draws the UI on the GPU                             | thin native host; the core draws the UI (GPUI on desktop, Makepad, or Vello with Masonry) |
| Phone app              | wrapper over the node page, key plugin | as A                                 | as A, with the shared WASM modules in the webview | Tauri mobile, core in process          | Capacitor                    | Compose on iOS and Android | Flutter, core through FFI | thin native host on iOS and Android; the same core-drawn UI                                                      | thin native host; the same core-drawn UI (Makepad, or Vello with Masonry)                 |
| Page                   | server-rendered HTML + Datastar        | TS SPA                               | TS SPA plus shared WASM modules                   | TS SPA                                 | TS SPA                       | Compose for Web            | Flutter widgets           | drawn by the core: chat, diff, terminal (Ghostty's font and glyph stack, sokol); an embedded browser for results | drawn by the core: chat, diff, terminal; an embedded browser for results                  |
| Run component          | Go pty; ghostty-vt.wasm via wasm2go    | as A                                 | as A, the same `.wasm` in the page                | portable-pty; libghostty-vt native     | `Bun.Terminal`; xterm        | POSIX pty by interop       | as D                      | Ghostty's pty and libghostty-vt, native Zig                                                                      | as D                                                                                      |
| TUI                    | Bubble Tea                             | Bubble Tea                           | Bubble Tea                                        | ratatui                                | OpenTUI or Ink               | Mosaic or own              | ratatui                   | libvaxis                                                                                                         | ratatui                                                                                   |
| Peer                   | Go transports, own range sync          | as A                                 | as A; CRDT as WASM                                | iroh or p2panda, own relays            | Hypercore or js-libp2p       | iroh-ffi (MPL-2.0) or own  | as D                      | own Noise port; QUIC and CRDT linked from C or Rust                                                              | as D                                                                                      |
| Memory safety          | GC, `unsafe` banned, race design       | as A                                 | as A                                              | `forbid(unsafe_code)` per crate        | GC in JS; the runtime is Zig | GC                         | as D                      | ReleaseSafe, TigerStyle allocation, simulation                                                                   | as D                                                                                      |
| Reach evidence         | import closure per entry               | as A                                 | as A, plus each module's imports                  | crate per component, lints, symbols    | runtime permissions only     | per-module deps, unproven  | as D                      | separate single-threaded core executable, own `Io`, symbol check                                                 | as D                                                                                      |
| Languages in the build | Go, a Rust shell                       | Go, TS, a Rust shell                 | Go, TS, Zig and Rust as WASM, a Rust shell        | Rust, TS                               | TS                           | Kotlin                     | Rust, Dart                | Zig, Swift, Kotlin, a Windows UI language                                                                        | Rust, Swift, Kotlin, a Windows UI language                                                |
| SRS language rows      | none                                   | none                                 | none                                              | CON-01, ENG-01 to 09, ENG-16, two ADRs | as D, and CON-03             | as D                       | as D                      | as D                                                                                                             | as D                                                                                      |

## Option A: Go node, hypermedia

**Thesis.** Keep Go for every process on the node and render the page
on the server, so the node's release path has no JavaScript build.

- **Core.** ncruces/go-sqlite3, `crypto/ed25519` and `crypto/sha256`
  from the standard library, an own stdio MCP server (the Go SDKs import
  `net/http`), the kernel as Starlark in Go or a WASM guest under
  wazero.
- **Page.** `html/template` fragments over one SSE stream with Datastar
  (CSP mode on), with CodeMirror 6 and ghostty-web as islands.
- **Shells.** A thin Tauri window on each desktop loads the node's
  loopback page; on the phone a wrapper loads the same page through the
  owner's tunnel and adds a native key plugin. The shells carry no
  logic, so the phone cannot verify or sync on its own.
- **Run component and TUI.** creack/pty with ConPTY packages on
  Windows; upstream `ghostty-vt.wasm` translated by wasm2go; Bubble Tea.
- **Peer.** quic-go or Noise and ParleyInn's own sealed-range exchange; no
  cgo-free CRDT.
- **Evidence and HC-23.** The import closure per entry point, checked
  per function on Windows where `x/sys/windows` mixes calls; `unsafe`
  banned in hostile-input packages, races excluded by a single-writer
  design and the race detector.
- **Precedent.** Crush and Claude Squad ship Go as static binaries;
  Tailscale and Lantern reach phones from a Go core
  ([prior art][prior], [five platforms](five-platform-apps.md)).
- **Biggest risk.** The phone stays a window onto a node; the moment it
  must be a peer, A needs a second core language on the phone.

## Option B: Go node, TypeScript page

**Thesis.** As A, with a small TypeScript SPA built once in the release
pipeline, embedded in the node and loaded by the thin shells.

- **Core, run component, TUI, peer, evidence.** As A.
- **Page.** Preact or Solid with CodeMirror 6 merge and ghostty-web,
  fed the core's view model as JSON over SSE or a WebSocket.
- **Shells.** As A.
- **Precedent.** A compiled core serving an embedded web UI is the
  survey's most common shape ([prior art][prior]).
- **Biggest risk.** As A, plus npm in the release path.

## Option C: Go node, TypeScript page, shared WASM cores

**Thesis.** As B, and the algorithms that must agree across surfaces
enter as WebAssembly modules that run in the Go node and, unchanged, in
every shell's webview: the VT model, the seal verifier and canonical
encoding, later a CRDT.

- **Core, run component, TUI, evidence.** As A, plus each module's
  import list, which is all a module can call.
- **Shared modules.** `ghostty-vt.wasm` (Zig); a seal and canonical
  encoding module; later Loro or Automerge (Rust).
- **Shells.** As A, but the phone's webview runs the same seal and sync
  modules, so a phone can check what it shows and, later, sync.
- **Peer.** Go transports; the CRDT as a module.
- **Precedent.** go-sqlite3 runs C inside Go by WASM translation, the
  route the SQLite ADR takes; Warp ships a WASM web terminal.
- **Biggest risk.** Modules built by other toolchains; their
  reproducibility and the phone webview's storage for a peer are
  unverified.

## Option D: Rust core, Tauri everywhere

**Thesis.** One Rust core library, linked into a Tauri shell on all
five platforms, with the TypeScript page talking to it over IPC.

- **Core.** rusqlite with FTS5, `rmcp` with HTTP off,
  `ed25519-dalek`, `sha2`, `serde_json_canonicalizer`, the `regex` crate
  for RE2 semantics, the kernel as Starlark in Rust or a WASM guest
  under wasmtime, whose fuel meters steps.
- **Shells.** Tauri 2 (2.12) on desktop and phone; the phone runs the
  same core in process, so it verifies, syncs and signs like a node. A
  key plugin in Swift and Kotlin is ParleyInn's own; Tauri 3's bundled
  Chromium is the way out of WebKitGTK's GPU problems on Linux.
- **Run component and TUI.** portable-pty (ConPTY included) and
  libghostty-vt linked natively, as herdr builds it; ratatui.
- **Peer.** iroh, p2panda or Willow with ParleyInn's own relays (CON-06).
- **Evidence and HC-23.** One crate per component, clippy bans on
  `std::net` and `std::process`, a symbol check; `forbid(unsafe_code)` in
  every hostile-input and trust crate, unsafe only in listed shims.
- **Precedent.** Vibe Kanban, Codex, Goose and herdr are Rust; Block
  Buzz ships Tauri on desktop; Bun moved from Zig to Rust over memory
  bugs.
- **Biggest risk.** Rust compile times in the factory's inner loop, and
  the cost of rewording the Go-specific rows.

## Option E: TypeScript on Bun

**Thesis.** One language from core to page.

- **Core.** `bun:sqlite`, WebCrypto, the reference MCP SDK; the kernel
  as a WASM guest.
- **Shells.** Electron on desktop, Capacitor on phones; no shell covers
  all five as one framework.
- **Evidence.** `fetch`, `Bun.listen` and `Bun.spawn` are globals, so
  no build-time reach evidence exists (HC-3).
- **Biggest risk.** It fails HC-3, HC-13 (LGPL-2 JavaScriptCore) and
  HC-22 (backtracking regex), and Bun itself now runs on Rust.

## Option F: Kotlin Multiplatform

**Thesis.** One Kotlin codebase with native Compose apps.

- **Core.** Kotlin/Native with SQLDelight; no canonical JSON or Starlark
  for Kotlin/Native.
- **Shells.** Compose Desktop runs on the JVM, with no single binary and
  no cross-builds; Kotlin/Native on Windows is Tier 3.
- **Biggest risk.** Weak core slots and a desktop shell that cannot meet
  CON-02.

## Option G: Rust core, Flutter shells

**Thesis.** As D's core, with the UI written once in Flutter instead of
a web page, stable on all five platforms.

- **Core, run component, TUI, peer, evidence, HC-23.** As D; the shells
  reach the core through `flutter_rust_bridge`.
- **Page.** Flutter widgets. The terminal pane (xterm.dart, last
  released February 2024), the diff widgets and the Linux webview for
  the running-app preview are the weak parts.
- **Precedent.** AppFlowy and RustDesk ship Flutter over a Rust core;
  Block Buzz's Flutter phone app is still being wired up.
- **Biggest risk.** The room's panes, which are web-native, rebuilt in a
  widget toolkit with thin libraries.

## Option H: Zig core library, Ghostty's shape

**Thesis.** Build ParleyInn the way Ghostty is built, one step further.
Ghostty's core library does terminal emulation, font handling and GPU
rendering, and each platform only hosts it: Swift with AppKit and
SwiftUI on macOS over the libghostty C API, Zig against GTK 4 on Linux
([About Ghostty][ghostty]). ParleyInn's core renders the whole room the same
way, chat and diff as well as the terminal, so there is one UI, drawn
once, on five platforms. The core
is built with smalt's coverage, lint and gate tooling and TigerStyle's
discipline, and every client links the same library, as TigerBeetle's
six client languages do ([Zig note](zig-option.md)).

- **Core.** SQLite with FTS5 built statically by Zig; `std.crypto`;
  canonical JSON, the MCP server, RE2 redaction (a port of Go's
  `regexp`) and the git reader as vendored ports with equivalence jobs
  against their originals; QUIC and a CRDT linked from C or Rust
  libraries when P2 needs them.
- **Hosts.** A thin native host per platform owns the window or
  view, input and IME, menus, the key store and the embedded browser,
  and hands the core a GPU surface: Swift on macOS and iOS, Zig on
  Linux, a Kotlin activity on Android, Win32 on Windows. smalt already
  draws through sokol, whose app layer covers the desktops, iOS and
  Android, with Metal, Direct3D 11 and OpenGL backends.
- **The UI.** Drawn by the core on the GPU: the room's conversation,
  the rooms list, the diff and the terminal, with Ghostty's font
  discovery, shaping and glyph atlas reused for chat text, and
  libghostty for the terminal. One UI on every platform keeps VIEW-14's
  one vocabulary by construction.
- **Results.** An embedded browser shows what an agent presents: the
  running app, an HTML report, a test page. On desktop it is the system
  webview or a bundled Chromium through CEF, as Steam's client does; on
  iOS it must be WKWebView. It runs on its own origin with no bridge
  into the core.
- **What a drawn UI must supply.** Accessibility through AccessKit's C
  API (screen readers on all five platforms); IME, selection and copy,
  links and Markdown with code highlighting in the core; mobile
  scrolling that feels native. The SRS has no accessibility
  requirement yet; any option needs one.
- **Security gain.** Untrusted agent and tool text is never parsed as
  HTML: the core lays out text it drew itself, so inert rendering
  (SEC-21) holds by construction. Only the results browser runs a web
  engine, sandboxed and apart.
- **Run component and TUI.** Ghostty's pty with ConPTY and libghostty-vt
  natively, no FFI; libvaxis.
- **Memory safety.** ReleaseSafe in shipped builds (TigerBeetle ships
  that way); hooks allocate at start and free at exit within NFR-09's
  50 MiB; long-lived components use per-request arenas with hard
  limits; assertion rules in the zlint fork; a VOPR-style deterministic
  simulator for I10, ENG-06 and PEER-03; the thread sanitizer and a
  single-threaded core build.
- **Evidence.** Zig's standard I/O links socket and spawn code into
  every binary (measured), so the core is its own single-threaded
  executable with an own `Io` for files and clocks, lint rules per
  component root and a symbol check of the artifact.
- **Precedent.** Ghostty, cmux, TigerBeetle, herdr's terminal core and
  the stakeholder's smalt (266k lines of Zig 0.16 built by agents).
- **Biggest risk.** Long-lived components rest on arena discipline,
  not a guarantee; every release breaks code (0.17 regressed an iOS
  library build); upstream bans LLM-written reports, so the factory
  cannot file a compiler bug itself. A drawn UI owes accessibility,
  IME and text editing that a webview gives for free.

## Option I: Rust core library, Ghostty's shape

**Thesis.** Ghostty's shape with a Rust core: the core draws the whole
UI on the GPU under a thin native host per platform, with an embedded
browser for results, as in H. Rust has the GPU UI toolkits Zig lacks.

- **Core, run component, TUI, peer, evidence, memory safety.** As D:
  rusqlite, `rmcp` with HTTP off, the `regex` crate, portable-pty,
  ratatui, iroh or p2panda, `forbid(unsafe_code)` in hostile-input and
  trust crates.
- **The UI.** GPUI, Zed's GPU framework, Apache-2.0 and published
  standalone as open-gpui, on macOS, Linux and Windows but not phones
  ([open-gpui][gpui]); Makepad, MIT, GPU-rendered on the desktops, iOS,
  Android and the web ([Makepad][makepad]); or Vello with Masonry from
  the Linebender project. The terminal is libghostty linked from Rust
  for its renderer, or libghostty-vt with the toolkit drawing the cells.
- **Hosts, results, accessibility, security.** As H; AccessKit is
  native Rust here ([AccessKit][accesskit]). Accessibility splits the
  toolkits, as of October 2026: Masonry and Xilem, drawn with Vello,
  are built on AccessKit ([Xilem][xilem]); egui integrates AccessKit
  too; GPUI's support is experimental behind `ZED_EXPERIMENTAL_A11Y`,
  with menus but not the editor reachable and none in practice on
  Windows ([Zed a11y][zed-a11y]); Makepad has no screen reader support,
  its accessibility issue open since August 2023 ([Makepad
  #196][makepad-a11y]). Vello is a renderer only and carries no
  accessibility of its own. So I's toolkit is Masonry on Vello, or
  egui, unless Makepad's phone reach outweighs a screen reader.
- **Precedent.** Zed draws its whole editor with GPUI; herdr links
  libghostty-vt from Rust; Element X shows a Rust core under room UIs on
  iOS and Android, though with native views ([Element X iOS][elementx-ios],
  [Element X Android][elementx]).
- **Biggest risk.** The toolkit: GPUI has no phones, and Makepad and
  Masonry are younger than Flutter; plus Rust's build times.

## Shells and hosts any option can add

| Addition                       | What it gives                                       | What it costs                                                                                                   | Verdict                                      |
| ------------------------------ | --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| VS Code webview extension      | the room beside the code                            | a second signed artifact; `asExternalUri` tunnels the loopback page off a remote machine, against SEC-01 and I4 | later, view only, local editors only         |
| VS Code layout takeover        | an IDE-shaped ParleyInn                             | fights the host; the agent often runs in the same extension host (T21)                                          | no                                           |
| Electron                       | a desktop window                                    | 120 MB or more; CON-03 bars Node.js; no phones                                                                  | no; Tauri covers the desktop shell           |
| Ghostty fork or plugin         | ParleyInn inside a terminal people use              | no plugin API; single-player                                                                                    | no; reuse libghostty-vt (A to D, G, H)       |
| herdr or tmux as followed host | agents the run component did not start, in the room | their sockets let any same-user process type and read (T21)                                                     | follow-only adapter, input shown unavailable |
| JetBrains JCEF panel           | the room inside JetBrains IDEs                      | a second artifact                                                                                               | later                                        |
| Zed                            | recall inside Zed                                   | no webviews                                                                                                     | MCP server only                              |
| WSL 2 on Windows               | the Linux node unchanged                            | Windows users run ParleyInn inside WSL                                                                          | a cheaper first Windows stage                |

## Options left out

- **Node or Deno executables.** Bun dominates them for option E's
  shape; Node falls under CON-03.
- **Wails.** Desktop only in practice, with cgo on macOS and Linux.
- **React Native, Capacitor or Electrobun alone.** Each misses
  platforms ([five platforms](five-platform-apps.md)).
- **Dioxus.** Watched: native phone APIs arrive only in 0.8.

[pitch]: ../../../plan/2610012322_parinn-for-agent-fleets/pitch.md
[ghostty]: https://ghostty.org/docs/about
[elementx]: https://github.com/element-hq/element-x-android
[elementx-ios]: https://github.com/element-hq/element-x-ios
[gpui]: https://docs.rs/crate/open-gpui/0.2.0
[makepad]: https://github.com/makepad/makepad
[accesskit]: https://github.com/AccessKit/accesskit
[xilem]: https://github.com/linebender/xilem
[zed-a11y]: https://github.com/zed-industries/zed/discussions/6576
[makepad-a11y]: https://github.com/makepad/makepad/issues/196
[prior]: prior-art-stacks.md#patterns
