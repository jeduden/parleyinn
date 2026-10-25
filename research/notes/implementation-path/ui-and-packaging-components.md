# UI stacks and one-binary packaging

Scope: the components that could deliver ParleyInn's surfaces and ship them
as one executable, read from primary sources on 4 October 2026. The
surfaces are the three-pane room page served by a loopback-only
component (the SRS's lane view, B1), a TUI for any terminal, and a
phone surface: first the web view through an owner tunnel, then a
paired device that signs with its own key. The stakeholder asked for
one binary and named a VS Code plugin, Electron, extending Ghostty and
Kotlin Multiplatform, with TypeScript or WebAssembly (Rust, Go, Zig).
Sections A to E cover packaging, browser stacks, desktop and mobile
shells, editor shells and TUI libraries. Each claim links its source.
Claims marked "measured here" come from builds run for this note (see
[How this note measured](#how-this-note-measured)). Claims marked
(unverified) rest on reasoning or a secondary source. The frame these
options answer is in [constraints.md](constraints.md); the terminal
parts of the run component are in
[terminal-components.md](terminal-components.md).

## The SRS rows that decide most of this

- CON-01 fixes Go; CON-02 ships the core as one statically linked
  `CGO_ENABLED=0` binary for linux/amd64, linux/arm64 and
  darwin/arm64, and leaves one executable or several open (OQ-32) —
  [02-context.md](../../../docs/srs/02-context.md).
- CON-03 bars a runtime dependency on Python, Node.js or a database
  server for any P0 feature —
  [02-context.md](../../../docs/srs/02-context.md).
- SEC-01: every process runs exactly one component, fixed at start;
  the core opens no socket; packaging must not change a boundary; and
  build-time evidence must cover "dependencies and code that runs as
  the process starts" per component —
  [06-security.md](../../../docs/srs/06-security.md).
- ENG-18 caps direct dependencies at ten, licenses on Apache-2.0, MIT,
  BSD or ISC; ENG-20 signs releases with Sigstore and SLSA L3
  provenance —
  [10-engineering-quality.md](../../../docs/srs/10-engineering-quality.md).
  ParleyInn already holds three direct Go dependencies (godog,
  cucumber messages, testify) in `go.mod`.
- VIEW-03 makes every view an optional read-only client; VIEW-14 lets
  the TUI and phone be reduced surfaces that say what they left out;
  VIEW-17 forbids a vendor push service for notifications —
  [05b-lane-requirements.md](../../../docs/srs/05b-lane-requirements.md).
- OWN-16 and OWN-17 limit a phone credential and a paired phone to
  reading and to allowing or denying held requests — [05c owner and
  peer requirements](../../../docs/srs/05c-owner-and-peer-requirements.md).

## A. Single-binary methods

### A1. Go embed.FS of a prebuilt SPA

- `embed` is a standard-library package since Go 1.16; an `embed.FS`
  is "a read-only collection of files", filled by `//go:embed`, and
  `http.FileServer(http.FS(content))` serves it —
  [pkg.go.dev/embed](https://pkg.go.dev/embed). Files starting with
  `.` or `_` are left out unless the pattern starts with `all:`.
- Go 1.21.0 was "the first Go toolchain with perfectly reproducible
  builds"; for programs without cgo, "a reproducible build is as
  simple as compiling with `CGO_ENABLED=0 go build -trimpath`" —
  [go.dev/blog/rebuild](https://go.dev/blog/rebuild).
- Measured here: a multi-call binary that embeds a 405 KB `dist/` and
  serves it with `net/http` on `127.0.0.1` is 6.2 MB stripped for
  linux/amd64, 5.9 MB for darwin/arm64 and 6.4 MB for windows/amd64,
  all cross-built from Linux with `CGO_ENABLED=0`. Two builds in
  different directories with a cleaned cache gave the same SHA-256.
- The SPA itself comes from a JavaScript build step that runs before
  `go build`. Its reproducibility is that toolchain's, not Go's: pin
  it and commit or verify the bundle (unverified for any bundler).
- Fit: adds no Go dependency and keeps CON-01 and CON-02. Risk: the
  JS toolchain enters the release path that ENG-20 must attest.

### A2. Go multi-call binary

- BusyBox's `main()` sets `applet_name` to `argv[0]` and dispatches to
  the applet — [BusyBox FAQ](https://www.busybox.net/FAQ.html).
  u-root's Go Busybox compiles many Go commands into one binary that
  "uses its invocation arguments (`os.Args`) to determine which command
  is being called"; cgo is not supported —
  [gobusybox README](https://github.com/u-root/gobusybox).
- For ParleyInn the switch would be a subcommand (`parinn mcp`,
  `parinn view`), chosen once at start, as SEC-01 asks.
- Measured here: the core-only program is 1.6 MB stripped; the same
  binary with the view branch is 6.2 MB. The extra 4.6 MB is
  `net/http` and the embedded assets. That code sits in every core
  process even when the core never calls it.
- So SEC-01's evidence cannot be the import closure of `main`. It
  must be per entry point: one package per component, a `main` that
  only dispatches, and a sandbox that denies sockets at run time
  (ENG-12). On Linux, Landlock ABI 4 restricts TCP bind and connect
  and ABI 6 scopes abstract Unix sockets —
  [kernel Landlock docs](https://docs.kernel.org/userspace-api/landlock.html).
  Landlock does not stop socket creation itself (same source).
- Fit: the cheapest way to meet "one binary" in Go. Risk: the per-
  component evidence and the sandbox carry the boundary, not the
  linker.

### A3. Rust: rust-embed and include_dir

- rust-embed 8.12.0 (MIT) "loads files into the rust binary at compile
  time during release and loads the file from the fs during dev" —
  [docs.rs/rust-embed](https://docs.rs/rust-embed/latest/rust_embed/);
  13 normal dependencies, 3 non-optional —
  [crates.io](https://crates.io/crates/rust-embed).
- include_dir 0.7.4 (MIT) embeds a directory tree at compile time —
  [include_dir README](https://github.com/Michael-F-Bryan/include_dir);
  its last release is from 17 June 2024 —
  [crates.io](https://crates.io/crates/include_dir).
- Reproducibility: `--remap-path-prefix` rewrites paths in "all
  compiler generated output", but it "does not affect these
  linker-generated paths" on Windows MSVC and Apple platforms —
  [rustc book](https://doc.rust-lang.org/rustc/remap-source-paths.html).
  Cargo's `trim-paths` profile setting is specified in
  [RFC 3127](https://rust-lang.github.io/rfcs/3127-trim-paths.html);
  whether it is stable today was not verified.
- Measured here: a stripped Rust 1.97.0 hello world is 343 KB, and two
  builds in different directories matched byte for byte.
- Fit: breaks CON-01 unless OQ-32 reopens the language.

### A4. Zig with embedded assets

- `@embedFile(path)` returns a compile-time pointer to the file's bytes,
  "equivalent to a string literal with the file contents" —
  [Zig 0.16.0 language reference](https://ziglang.org/documentation/0.16.0/#embedFile).
- Zig is pre-1.0. 0.16.0 is current and changed the language again,
  for example deprecating `@cImport` for `b.addTranslateC()` —
  [0.16.0 release notes](https://ziglang.org/download/0.16.0/release-notes.html).
- Reproducibility and binary size: not verified here.
- Fit: Zig earns its place through libghostty-vt and OpenTUI's core
  (sections D and E), not as ParleyInn's packaging language.

### A5. Bun build --compile

- Targets: linux x64 and arm64 (glibc and musl), windows x64 and
  arm64, darwin x64 and arm64, chosen with `--target` —
  [Bun executables docs](https://bun.com/docs/bundler/executables).
- Importing an HTML file in server code makes Bun bundle the frontend
  and embed it in the executable. `with { type: "file" }` embeds any
  file, `--asset` embeds directories, and `bun:sqlite` works inside a
  compiled binary (same source).
- The docs say "Bun's binary is still way too big and we need to make
  it smaller" (same source). Measured here with Bun 1.3.14: 94.6 MB
  for linux-x64 and 63.4 MB when cross-compiled for darwin-arm64.
- Reproducibility: the docs say profile-guided layout keeps "builds
  stay reproducible" (same source). Measured here: two builds with the
  same `--outfile` name, in different directories, matched; builds
  with different output names differed, so the name is embedded.
- License: "Bun itself is MIT-licensed", but it "statically links
  JavaScriptCore (and WebKit), which is LGPL-2 licensed", which obliges
  shipping a relinkable form — [Bun license](https://bun.com/docs/project/license).
  LGPL-2 is not on ENG-18's allow-list.
- Open issues on compiled binaries include a truncated macOS code
  signature in 1.3.12 —
  [oven-sh/bun#29120](https://github.com/oven-sh/bun/issues/29120) —
  and 1.4.0 binaries killed on macOS 27 —
  [oven-sh/bun#39764](https://github.com/oven-sh/bun/issues/39764).
- Fit: the best single-binary story for a TypeScript program. Risk:
  size, the LGPL-2 engine, a JavaScript runtime inside the core
  binary, and macOS signing regressions.

### A6. Deno compile

- Cross-compiles to Windows, macOS and Linux on x86_64 and ARM64;
  `--include` embeds files or directories; "Runtime flags ... must be
  specified at compilation time. This includes permission flags";
  macOS binaries get an ad-hoc signature —
  [Deno compile docs](https://docs.deno.com/runtime/reference/cli/compile/).
  The docs say nothing about reproducibility.
- Size: every compiled program carries `denort`. Measured here,
  `denort` 2.9.7 for linux x86_64 unpacks to 104.6 MB.
- License: MIT — [deno LICENSE](https://github.com/denoland/deno/blob/main/LICENSE.md).

### A7. Node single executable applications

- Stability 1.1, "Active development". `node --build-sea` arrived in
  v25.5.0. Assets go in the config's `assets` map and are read with
  `sea.getAsset()`; a read-only virtual file system arrived in
  v26.9.0. Cross-platform builds must set `useCodeCache` and
  `useSnapshot` to false. Tested platforms include macOS on arm64
  only — [Node SEA docs](https://nodejs.org/api/single-executable-applications.html).
- Size: the output is a copy of `node`; measured here, the Node
  22.22.0 linux x64 binary is 123 MB.
- Fit: CON-03 names Node.js; poor fit.

### A8. WASM modules inside the binary: wazero and wasmtime

- wazero is "the zero dependency WebAssembly runtime for Go
  developers" and "doesn't rely on CGO"; its only dependency is
  `golang.org/x/sys`. Its compiler runs on amd64 and arm64, and an
  interpreter runs everywhere else —
  [wazero README](https://github.com/tetratelabs/wazero), Apache-2.0
  ([LICENSE](https://github.com/tetratelabs/wazero/blob/main/LICENSE)).
  It reached 1.0 in March 2023 — [wazero.io](https://wazero.io/).
- Measured here: wazero v1.12.0 instantiated `ghostty-vt.wasm` from
  ghostty-web 0.4.0 inside a pure-Go binary. The module imports only
  `env.log`, which a stub satisfied. It exports 76 functions, among
  them `ghostty_terminal_*` and `ghostty_render_state_*`. The stripped
  binary with the 423 KB module is 5.4 MB. Only instantiation was
  tested, not terminal output.
- wasmtime 49.0.2 is "Apache-2.0 WITH LLVM-exception", with 14
  non-optional dependencies — [crates.io](https://crates.io/crates/wasmtime).
  That license string is not literally on ENG-18's allow-list.
- Fit: lets Zig or Rust code (libghostty-vt, a diff engine) ride
  inside a `CGO_ENABLED=0` Go binary for one direct dependency. Risk:
  the compiler mode maps executable memory, so its effect under the
  ENG-12 sandboxes and macOS hardened runtime needs testing
  (unverified).

### A9. cgo and zig cc cross-compilation

- `zig cc` "ships with source code only and cross-compiles on-the-fly"
  for musl, glibc and mingw-w64 targets —
  [Andrew Kelley, 2020](https://andrewkelley.me/post/zig-cc-powerful-drop-in-replacement-gcc-clang.html).
- Uber uses it as a hermetic C toolchain for Go with cgo, choosing the
  glibc version (`-target aarch64-linux-gnu.2.28`) and cross-building
  Mach-O from Linux —
  [Uber, 2023](https://www.uber.com/blog/bootstrapping-ubers-infrastructure-on-arm64-with-zig/).
- With cgo, Go builds are reproducible only with "a specific host C
  toolchain version" — [go.dev/blog/rebuild](https://go.dev/blog/rebuild).
- A webview shell on macOS links Apple frameworks (WebKit, AppKit);
  whether `zig cc` can do that without Apple's SDK was not verified.
- Fit: CON-02 forbids cgo in the core binary. Anything needing cgo,
  such as a webview, lives in another executable or changes CON-02.

### A10. Static linking on macOS and Windows

- Apple: "Apple does not support statically linked binaries on Mac OS
  X", because the kernel system-call interface carries no
  compatibility guarantee —
  [Apple QA1118](https://developer.apple.com/library/archive/qa/qa1118/_index.html).
- Since Go 1.11, "on macOS and iOS, the runtime now uses
  libSystem.dylib instead of calling the kernel directly" —
  [Go 1.11 release notes](https://go.dev/doc/go1.11). Measured here,
  the darwin/arm64 binary from A1 carries four `LC_LOAD_DYLIB`
  commands, `/usr/lib/libSystem.B.dylib` among them, and an
  `LC_CODE_SIGNATURE`.
- So CON-02's "statically linked" holds on Linux only. On darwin the
  core is a cgo-free binary that links libSystem dynamically. This is
  a wording finding for the SRS, not a design problem.
- Windows: Go loads system DLLs at run time without cgo (unverified
  here; the windows/amd64 build in A1 needed no C toolchain).

## B. Browser client stacks

### B1. Frameworks

Sizes are min+gzip. "Measured here" sizes come from `bun build
--minify` of a file that imports the named parts, then `gzip -9`.

| Stack                     | License           | Version                      | npm deps                | Size                                       | Live updates                                             | Source                                                                                            |
| ------------------------- | ----------------- | ---------------------------- | ----------------------- | ------------------------------------------ | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Datastar                  | MIT               | 1.0.4                        | 0 (one file)            | 13.4 KB measured; site says 11.90 KiB      | SSE: `datastar-patch-elements`, `datastar-patch-signals` | [data-star.dev](https://data-star.dev/), [SSE events](https://data-star.dev/reference/sse_events) |
| htmx                      | 0BSD              | 2.0.11 on npm; 4.0 announced | 0                       | 16.8 KB measured; site says ~16k           | SSE and WebSocket attributes                             | [htmx.org](https://htmx.org/)                                                                     |
| Preact                    | MIT               | 11.0.0                       | 0                       | 5.6 KB measured with hooks; site says 3kB  | your own EventSource                                     | [preactjs.com](https://preactjs.com/)                                                             |
| SolidJS                   | MIT               | 1.9.15                       | 3                       | 5.1 KB measured with `render`              | your own EventSource                                     | [solid README](https://github.com/solidjs/solid)                                                  |
| Lit                       | BSD-3-Clause      | 3.3.3                        | 3 (own packages)        | 9.8 KB measured; site says ~5 KB           | your own EventSource                                     | [lit.dev](https://lit.dev/)                                                                       |
| Svelte 5                  | MIT               | 5.57.1                       | 15, mostly the compiler | not measured                               | your own EventSource                                     | [Svelte 5 post](https://svelte.dev/blog/svelte-5-is-alive)                                        |
| go-app (Go to WASM)       | MIT               | v11                          | 5 Go modules            | 740 KB gzip for a Go `fmt` hello, measured | Go code in WASM                                          | [go-app README](https://github.com/maxence-charriere/go-app)                                      |
| Leptos (Rust)             | MIT               | 0.8.21                       | 31 non-optional crates  | not stated                                 | fine-grained signals                                     | [Leptos README](https://github.com/leptos-rs/leptos)                                              |
| Dioxus (Rust)             | MIT OR Apache-2.0 | 0.7.10                       | 3 non-optional crates   | "about 50kb" hello claimed                 | signals; web, desktop, mobile                            | [Dioxus README](https://github.com/DioxusLabs/dioxus)                                             |
| Compose Multiplatform web | Apache-2.0        | Beta on Kotlin/Wasm          | Gradle stack            | not measured                               | Kotlin code in WASM                                      | [supported platforms](https://kotlinlang.org/docs/multiplatform/supported-platforms.html)         |
| Flutter web               | BSD-3-Clause      | Wasm since 3.24              | Dart SDK                | not measured                               | Dart code                                                | [Flutter Wasm](https://docs.flutter.dev/platform-integration/web/wasm)                            |

npm data is from each package's registry record
([npm registry](https://registry.npmjs.org/)); crate data from
[crates.io](https://crates.io/).

- Datastar 1.0 shipped with "We are done. Done like dinner"; v1.0.3
  "Added an opt-in CSP mode that lets Datastar run without
  `unsafe-eval`" —
  [Datastar releases](https://github.com/starfederation/datastar/releases).
  The release page shows days without years. For v1, "a handful of
  convenience plugins" moved into a paid Pro tier; the core stays MIT —
  [Greedy Developer?](https://data-star.dev/essays/greedy_developer).
- Datastar's SSE format is plain `data:` lines; "you can format them
  yourself" without an SDK —
  [SSE events](https://data-star.dev/reference/sse_events). The Go
  SDK requires four modules (httpcompression, klauspost/compress,
  jsonschema, bytebufferpool) —
  [datastar-go go.mod](https://github.com/starfederation/datastar-go/blob/main/go.mod).
  Writing the events with the standard library adds no dependency.
- Svelte is "a compiler that takes your declarative components and
  converts them into efficient JavaScript" —
  [Svelte README](https://github.com/sveltejs/svelte). Solid "compiles
  its templates to real DOM nodes" with fine-grained updates and no
  virtual DOM — [solid README](https://github.com/solidjs/solid).
- go-app builds PWAs in Go and WebAssembly —
  [go-app README](https://github.com/maxence-charriere/go-app). It
  requires `webpush-go`, `x/net`, `uuid`, `gomarkdown` and `testify` —
  [go-app go.mod](https://github.com/maxence-charriere/go-app/blob/master/go.mod).
  Measured here, a Go `fmt` hello compiled to `js/wasm` is 2.58 MB, or
  740 KB gzipped.
- Leptos claims "smaller WASM binary sizes" than Dioxus and a focus on
  web performance; Dioxus claims the better desktop story —
  [Leptos README](https://github.com/leptos-rs/leptos).
- Compose Multiplatform for web went Beta in 1.9.0 —
  [JetBrains blog](https://blog.jetbrains.com/kotlin/2025/09/compose-multiplatform-1-9-0-compose-for-web-beta/).
- Flutter's Wasm build needs WasmGC, and "Flutter compiled to Wasm
  can't run on the iOS version of any browser" —
  [Flutter Wasm](https://docs.flutter.dev/platform-integration/web/wasm).
  That rules it out for the phone's web stage.
- Without HTTP/2, SSE is limited to six connections per browser and
  domain, across tabs — [MDN EventSource](https://developer.mozilla.org/en-US/docs/Web/API/EventSource).
  A plain-HTTP loopback page has this limit, so the room page should
  multiplex its live panes over one stream (reasoning).

### B2. Diff viewers

| Viewer                                         | License | Size                                    | Mobile                                 | Source                                                                                                     |
| ---------------------------------------------- | ------- | --------------------------------------- | -------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| CodeMirror 6 merge (`MergeView`, unified view) | MIT     | 134 KB gzip measured, with `basicSetup` | native selection and editing on phones | [@codemirror/merge README](https://github.com/codemirror/merge), [codemirror.net](https://codemirror.net/) |
| Monaco diff editor                             | MIT     | npm package unpacks to 101.7 MB         | "No"                                   | [Monaco README](https://github.com/microsoft/monaco-editor)                                                |
| diff2html                                      | MIT     | `diff2html-ui-slim` 97 KB gzip measured | static HTML (unverified on phones)     | [diff2html](https://github.com/rtfpessoa/diff2html)                                                        |

- `@codemirror/merge` 6.12.2 needs five packages, and `codemirror`
  pulls seven more; the bundle measured here resolved to 14 packages
  in total ([npm registry](https://registry.npmjs.org/@codemirror%2Fmerge)).
- CodeMirror lists "Mobile Support: Use the platform's native
  selection and editing features on phones" and screen-reader support
  — [codemirror.net](https://codemirror.net/).
- Monaco answers "Is the editor supported in mobile browsers or mobile
  web app frameworks?" with "No." —
  [Monaco README](https://github.com/microsoft/monaco-editor).

### B3. Terminal panes

| Pane              | License | Size                                             | Source                                              |
| ----------------- | ------- | ------------------------------------------------ | --------------------------------------------------- |
| xterm.js 6.0.0    | MIT     | 88 KB gzip measured                              | [xterm.js](https://github.com/xtermjs/xterm.js)     |
| ghostty-web 0.4.0 | MIT     | 193 KB gzip measured; the 423 KB WASM is inlined | [ghostty-web](https://github.com/coder/ghostty-web) |

- ghostty-web offers "xterm.js API compatibility", a "WASM-compiled
  parser from Ghostty—the same code that runs the native app", and
  "Zero runtime dependencies, ~400KB WASM bundle"; Coder built it for
  Mux — [ghostty-web README](https://github.com/coder/ghostty-web).
  Measured here, its dist JavaScript holds a 564 KB base64 string that
  decodes to a WASM header, so the module ships inline.
- Both have zero npm runtime dependencies
  ([npm registry](https://registry.npmjs.org/ghostty-web)).

## C. Desktop and mobile shells

| Shell                     | Language                  | License           | Status                        | Mobile                   | One binary                                                          | Source                                                                                                                                                     |
| ------------------------- | ------------------------- | ----------------- | ----------------------------- | ------------------------ | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri 2                   | Rust + web                | Apache-2.0 OR MIT | 2.12.1; 2.0 stable 2 Oct 2024 | iOS, Android             | assets embedded in the Rust executable; sidecars are separate files | [Tauri 2.0](https://v2.tauri.app/blog/tauri-20/)                                                                                                           |
| Wails v3                  | Go (cgo) + web            | MIT               | v3.0.0-beta.27, 1 Oct 2026    | iOS, Android documented  | Go binary with embedded assets                                      | [Go module proxy](https://proxy.golang.org/github.com/wailsapp/wails/v3/@latest), [mobile guide](https://docs-preview.wails.io/guides/mobile/)             |
| Electron                  | JS + Chromium             | MIT               | 44.5.1                        | none                     | no: a framework directory                                           | [distribution docs](https://www.electronjs.org/docs/latest/tutorial/application-distribution)                                                              |
| Electrobun                | TS + Zig + system webview | MIT               | 2.0.2                         | not documented           | self-extracting bundle                                              | [Electrobun README](https://github.com/blackboardsh/electrobun)                                                                                            |
| Capacitor                 | TS + native               | MIT               | 8.5.2                         | iOS, Android             | an app per store                                                    | [Capacitor docs](https://capacitorjs.com/docs)                                                                                                             |
| PWA                       | web                       | n/a               | platform feature              | install from the browser | none needed                                                         | [WebKit web push](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/)                                                                  |
| gomobile                  | Go                        | BSD-3-Clause (Go) | `golang.org/x/mobile`         | iOS, Android             | a library or app                                                    | [Go wiki: Mobile](https://go.dev/wiki/Mobile)                                                                                                              |
| Dioxus mobile             | Rust                      | MIT OR Apache-2.0 | 0.7.10                        | `.ipa`, `.apk`           | app bundles                                                         | [Dioxus README](https://github.com/DioxusLabs/dioxus)                                                                                                      |
| Compose Multiplatform iOS | Kotlin                    | Apache-2.0        | iOS stable since 1.8.0        | iOS, Android             | desktop needs a JVM installer                                       | [1.8.0 post](https://blog.jetbrains.com/kotlin/2025/05/compose-multiplatform-1-8-0-released-compose-multiplatform-for-ios-is-stable-and-production-ready/) |

- Tauri: "A minimal Tauri app can be less than 600KB in size" because
  it uses the system webview — [Tauri start](https://v2.tauri.app/start/).
  That is WebView2 on Windows, WKWebView on macOS and iOS,
  `webkit2gtk` on Linux and the system WebView on Android —
  [webview versions](https://v2.tauri.app/reference/webview-versions/).
  A relative `frontendDist` is embedded in the binary; a URL makes the
  app load that URL instead —
  [Tauri config](https://v2.tauri.app/reference/config/). So a Tauri
  window could simply open the loopback room page.
- Tauri sidecars are separate files named by target triple, and
  running one needs a `shell:allow-execute` capability —
  [Tauri sidecar](https://v2.tauri.app/develop/sidecar/). The tauri
  crate has 39 non-optional dependencies —
  [crates.io](https://crates.io/crates/tauri).
- Wails v3: iOS uses "WKWebView + UIKit host. Assets served via a
  custom `wails://` scheme — no open ports"; Android compiles Go to
  `libwails.so` through the NDK —
  [Wails mobile guide](https://docs-preview.wails.io/guides/mobile/).
  A release summary calls mobile experimental —
  [newreleases.io](https://newreleases.io/project/github/wailsapp/wails/release/v3.0.0-beta.0)
  (secondary). The v3 module lists 35 direct requirements, CLI
  tooling included —
  [wails v3 go.mod](https://github.com/wailsapp/wails/blob/master/v3/go.mod).
  Its webview needs cgo, which CON-02 keeps out of the core binary.
- Electron: the app goes into Electron's `resources` directory, often
  as `app.asar`, and the whole directory ships —
  [Electron distribution](https://www.electronjs.org/docs/latest/tutorial/application-distribution).
  Measured here (HTTP `Content-Length` of the release assets),
  Electron 44.5.1 zips are 123 MB for linux-x64, 130 MB for
  darwin-arm64 and 158 MB for win32-x64.
- Electrobun uses system webviews and its JSC-based Cottontail runtime
  "instead of a bundled Chromium and Node", ships bsdiff updates, and
  runs on macOS, Windows and Linux —
  [Electrobun docs](https://framework.blackboard.sh/electrobun/),
  [README](https://github.com/blackboardsh/electrobun).
- Capacitor is a "cross-platform native runtime" for web apps on iOS
  and Android, with plugins in Swift, Java and JavaScript —
  [Capacitor docs](https://capacitorjs.com/docs).
- PWA on iOS: since iOS 16.4, "A web app that has been added to the
  Home Screen can request permission to receive push notifications" —
  [WebKit](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/).
  Push goes through the browser's push service: the app server posts
  to the subscription's endpoint URL —
  [MDN Push API](https://developer.mozilla.org/en-US/docs/Web/API/Push_API).
  VIEW-17 forbids a vendor push service, so ParleyInn's PWA gets no web
  push.
- Apple kept Home Screen web apps in the EU after a 17.4 beta removed
  them — [Apple developer support](https://developer.apple.com/support/dma-and-apps-in-the-eu#dev-qaa).
- Background work: Periodic Background Sync is experimental and not
  Baseline; Chrome grants it only to an installed app —
  [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Web_Periodic_Background_Synchronization_API).
  A phone page therefore learns of held requests only while open.
- Service workers and the Push API need a secure context; loopback
  hosts count as trustworthy, a phone reaching the tunnel needs HTTPS —
  [MDN secure contexts](https://developer.mozilla.org/en-US/docs/Web/Security/Secure_Contexts).
- Safari deletes a site's script-writable storage after seven days of
  Safari use without interaction; Home Screen apps keep "their own
  counter of days of use" —
  [WebKit](https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/).
  A phone's device key held in IndexedDB is safer installed than in a
  tab.
- WebCrypto can generate non-extractable keys and lists Ed25519, but
  "not all browsers support this algorithm" —
  [MDN generateKey](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/generateKey).
  Ed25519 on iOS Safari was not verified.
- gomobile: "Only a subset of Go types are currently supported" in
  bindings — [Go wiki: Mobile](https://go.dev/wiki/Mobile). A gomobile
  library could reuse ParleyInn's Go signing code inside a thin native app
  (reasoning).
- Compose Multiplatform 1.8.0 made iOS stable; it adds "~9 MB" to an
  iOS app against SwiftUI —
  [JetBrains](https://blog.jetbrains.com/kotlin/2025/05/compose-multiplatform-1-8-0-released-compose-multiplatform-for-ios-is-stable-and-production-ready/).
  Desktop packaging uses `jpackage` with a bundled JDK, gives
  installers rather than one executable, and "Cross-compilation is
  currently not supported" —
  [native distributions](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html).

## D. Editor and terminal shells

### VS Code webview extension

- A webview is "an `iframe` within VS Code that your extension
  controls", scripts are off by default, and the guide recommends a
  `default-src 'none'` CSP —
  [webview guide](https://code.visualstudio.com/api/extension-guides/webview).
- `portMapping` remaps localhost ports inside the webview, but "port
  mappings only work for `http` or `https` urls. Websocket urls ...
  cannot be mapped" —
  [vscode.d.ts](https://github.com/microsoft/vscode/blob/main/src/vscode-dts/vscode.d.ts).
  SSE passes; WebSocket does not.
- When the extension runs remotely, `asExternalUri` "automatically
  establishes a port forwarding tunnel from the local machine to
  `target` on the remote" (same source). That tunnel carries the
  loopback page off the node, so it needs review against SEC-01's
  "local endpoint only the same local user can reach" (reasoning).
- Fit: a thin TypeScript VSIX that frames the loopback room page. It
  is a second artifact beside the binary, also in ENG-20's scope.

### JetBrains

- Plugins embed Chromium through JCEF: check `JBCefApp.isSupported()`,
  then load a URL with `JBCefBrowser`; Kotlin or Java —
  [JCEF docs](https://plugins.jetbrains.com/docs/intellij/embedded-browser-jcef.html).
  Loading `http://127.0.0.1` is not stated there (unverified).

### Zed

- Zed extensions provide "Languages, Debuggers, Themes, Icon Themes,
  Snippets, and MCP Servers", written in Rust compiled to WebAssembly
  — [Zed extension docs](https://zed.dev/docs/extensions/developing-extensions).
  No webview or custom panel is offered.
- External agents speak ACP, and "Zed hosts the thread in the Agent
  Panel" — [Zed external agents](https://zed.dev/docs/ai/external-agents).
  ACP uses "JSON-RPC over stdio" for local agents and has types for
  diffs — [ACP introduction](https://agentclientprotocol.com/overview/introduction).
- Fit: in Zed, ParleyInn appears as its MCP server; the room stays in the
  browser.

### Ghostty

- Ghostty has no plugin API. 1.3.0 (9 March 2026) added AppleScript on
  macOS as a preview, and 1.4 plans to make Ghostty scriptable —
  [1.3.0 release notes](https://ghostty.org/docs/install/release-notes/1-3-0).
  The release-notes index lists nothing past 1.3.1 today —
  [release notes](https://ghostty.org/docs/install/release-notes).
- libghostty-vt parses terminal sequences and keeps terminal state,
  with "zero external dependencies (not even libc)"; it is in "public
  alpha" — [Mitchell Hashimoto](https://mitchellh.com/writing/libghostty-is-coming).
  1.3.0 says no versioned release is tagged yet (release notes above).
  License MIT — [Ghostty LICENSE](https://github.com/ghostty-org/ghostty/blob/main/LICENSE).
- Fit: "extending Ghostty" in practice means reusing libghostty-vt:
  ghostty-web in the room page (B3) or its WASM under wazero in a
  Go TUI (A8). Detail is in
  [terminal-components.md](terminal-components.md).

## E. TUI libraries

| Library                      | Language         | License | Version          | Deps                               | Source                                                                                                                            |
| ---------------------------- | ---------------- | ------- | ---------------- | ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Bubble Tea v2 + Lip Gloss v2 | Go               | MIT     | 2.0.10 and 2.0.6 | 8 and 10 direct requirements       | [bubbletea](https://github.com/charmbracelet/bubbletea), [module proxy](https://proxy.golang.org/charm.land/bubbletea/v2/@latest) |
| Ratatui                      | Rust             | MIT     | 0.30.2           | 3 non-optional crates              | [ratatui.rs](https://ratatui.rs/), [crates.io](https://crates.io/crates/ratatui)                                                  |
| libvaxis                     | Zig 0.16         | MIT     | 0.6.0            | zigimg, uucode (lazy)              | [libvaxis](https://github.com/rockorager/libvaxis)                                                                                |
| Ink                          | TS (React)       | MIT     | 8.0.0            | 23 npm                             | [Ink](https://github.com/vadimdemedes/ink)                                                                                        |
| OpenTUI                      | Zig core, TS API | MIT     | 0.5.14           | 5 npm + 8 native platform packages | [OpenTUI](https://github.com/anomalyco/opentui)                                                                                   |

- Bubble Tea follows "The Elm Architecture", now at import path
  `charm.land/bubbletea/v2`, with "a high-performance cell-based
  renderer" — [bubbletea](https://github.com/charmbracelet/bubbletea).
  Its `go.mod` pins `charmbracelet/ultraviolet` at a pseudo-version —
  [module proxy](https://proxy.golang.org/charm.land/bubbletea/v2/@v/v2.0.10.mod).
  Bubble Tea plus Lip Gloss would take ParleyInn to five of ten direct
  dependencies.
- Ratatui offers "immediate-mode rendering"; Netflix, OpenAI and AWS
  are listed as users — [ratatui.rs](https://ratatui.rs/).
- libvaxis "does not use terminfo"; it detects features by querying
  the terminal and supports the Kitty keyboard and graphics protocols
  — [libvaxis README](https://github.com/rockorager/libvaxis).
- Ink is "React for CLIs" and lays out with Yoga; Claude Code and
  Gemini CLI use it — [Ink README](https://github.com/vadimdemedes/ink).
- OpenTUI is "written in Zig" with TypeScript, React and Solid APIs;
  "OpenCode uses OpenTUI in production" —
  [OpenTUI README](https://github.com/anomalyco/opentui). Its npm
  engines field asks for Bun 1.3 or Node 26.4 or later
  ([npm registry](https://registry.npmjs.org/@opentui%2Fcore)).
- Fit: Bubble Tea is the only Go choice in the list. A reduced TUI
  (VIEW-14) could also be hand-written on `golang.org/x/term` for one
  dependency (reasoning). Ratatui, libvaxis, Ink and OpenTUI each
  bring a second language or a JS runtime.

## How this note measured

All builds ran in a scratch directory on linux/amd64 on 4 October
2026; none touched ParleyInn's tree or any real home.

- Go 1.26 (toolchain go1.26.0 selected by `go.mod`),
  `CGO_ENABLED=0 go build -trimpath -ldflags="-s -w"`; repeat build in
  a copied directory after `go clean -cache`; `sha256sum` compared.
- Bun 1.3.14, `bun build --compile --minify`, native and
  `--target=bun-darwin-arm64`; hashes compared across directories and
  output names.
- Rust 1.97.0, `cargo build --release` with `strip = true`, two
  directories.
- Bundles: `bun build --minify --target=browser` of an entry that
  re-exports the named parts, then `gzip -9 | wc -c`. Shipped dist
  files (htmx, Datastar, diff2html, ghostty-web) were gzipped as
  published.
- Runtime sizes: `denort` 2.9.7 unzipped; local Node 22.22.0 binary;
  Electron 44.5.1 release asset `Content-Length`.
- wazero v1.12.0 with a stub `env.log`, listing exported functions.

## Ways to ship one binary

| Method                                          | Languages                      | How the UI gets in                                                     | Reproducible                                                                                    | Size                                                                | Caveats                                                                                        |
| ----------------------------------------------- | ------------------------------ | ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Go multi-call + `embed.FS` of a prebuilt SPA    | Go, plus a JS build step       | `//go:embed dist`, served by `http.FS` in the view subcommand          | Yes for Go (≥1.21, `CGO_ENABLED=0 -trimpath`, measured); the JS build must be pinned separately | 6.2 MB linux/amd64 with 405 KB assets; core alone 1.6 MB (measured) | `net/http` code sits in core processes; SEC-01 evidence must be per entry point plus a sandbox |
| Go multi-call + server-rendered HTML + Datastar | Go                             | `html/template` and one vendored 33.5 KB JS file in `embed.FS`         | Yes, no JS toolchain in the release path                                                        | as above                                                            | SSE only; Datastar needs `unsafe-eval` unless its CSP mode (v1.0.3) is on                      |
| Go + wazero + embedded WASM                     | Go host; Zig, Rust or C guests | `//go:embed *.wasm`, run in-process                                    | Host yes; the guest WASM is a prebuilt input to pin or rebuild (unverified)                     | 5.4 MB with ghostty-vt.wasm (measured)                              | one direct dep; compiler maps executable memory (sandbox effect unverified)                    |
| Rust + rust-embed or include_dir                | Rust                           | derive macro or `include_dir!` at compile time                         | Hello matched across directories (measured); Apple and MSVC linker paths not remapped           | 343 KB hello (measured)                                             | breaks CON-01; rust-embed reads disk in dev builds                                             |
| Zig + `@embedFile`                              | Zig                            | one builtin per file                                                   | not verified                                                                                    | not measured                                                        | pre-1.0 language, breaking releases                                                            |
| Bun `build --compile`                           | TS, JS                         | HTML imports bundled; `type: "file"`; `--asset`; `bun:sqlite` built in | Matched for the same output name (measured); the name is embedded                               | 94.6 MB linux-x64, 63.4 MB darwin-arm64 (measured)                  | LGPL-2 JavaScriptCore off the ENG-18 list; macOS signing bugs                                  |
| Deno `compile`                                  | TS, JS                         | `--include`, `--include-as-is`                                         | not stated by Deno                                                                              | at least 104.6 MB (`denort`, measured)                              | permissions fixed at compile time                                                              |
| Node SEA                                        | JS                             | `assets` map, `sea.getAsset()`, VFS since v26.9                        | not stated by Node                                                                              | at least 123 MB (node binary, measured)                             | stability 1.1; CON-03 bars Node.js                                                             |
| Tauri 2 executable                              | Rust + web                     | `frontendDist` embedded by codegen, or a URL loaded                    | not verified                                                                                    | "less than 600KB" minimal (claimed)                                 | system webview; sidecars are separate files; cannot be the core                                |
| Wails v3 executable                             | Go with cgo + web              | `embed.FS`                                                             | needs a pinned C toolchain (cgo)                                                                | not measured                                                        | beta.27; cgo conflicts with CON-02 if the core shares it                                       |
| Electrobun bundle                               | TS + Zig                       | bundled into a self-extracting app                                     | not verified                                                                                    | "megabytes, not hundreds" (claimed)                                 | desktop only; young                                                                            |
| cgo + `zig cc` (enabler)                        | Go or Rust with C              | any of the above                                                       | only with the toolchain pinned                                                                  | n/a                                                                 | Apple frameworks without Apple's SDK unverified                                                |
| Not one binary: Electron, Compose Desktop       | JS; Kotlin                     | framework directory; `jpackage` installer                              | n/a                                                                                             | 123 to 158 MB zipped Electron (measured)                            | no single executable; Compose cannot cross-compile                                             |

## Client stack per surface

| Surface                       | Strongest candidate                                                              | Runner-up                                                                                | Why                                                                | Main risk                                                                 |
| ----------------------------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| Room page (B1, loopback)      | Go-rendered HTML + Datastar over one SSE stream, with web-component islands      | Preact or Solid SPA (about 5 KB) built once and embedded                                 | no SPA toolchain, zero npm deps, SSE passes VS Code's port mapping | six-connection SSE limit on plain HTTP; Datastar CSP mode must be on      |
| Diff pane                     | CodeMirror 6 `@codemirror/merge`                                                 | diff2html (read-only)                                                                    | MIT, 134 KB gzip, works on phones                                  | 14 npm packages to vendor and review                                      |
| Terminal and test-run pane    | ghostty-web                                                                      | xterm.js                                                                                 | libghostty-vt fidelity, xterm.js API, zero deps                    | pre-1.0 (0.4.0); libghostty untagged                                      |
| Running-app preview           | sandboxed `iframe` to the run component's loopback port                          | none                                                                                     | no library                                                         | CSP and frame isolation from the room page (not researched here)          |
| TUI                           | Bubble Tea v2 + Lip Gloss v2                                                     | hand-written on `x/term`; ghostty-vt.wasm under wazero for live panes                    | Go, MIT, mature                                                    | two to three direct deps of ten                                           |
| Phone, stage one              | the same room page through the owner tunnel, installed as a PWA                  | same page in a browser tab                                                               | one UI; CodeMirror works on phones                                 | HTTPS needed; no web push under VIEW-17; nothing in the background        |
| Phone, stage two (device key) | Tauri 2 mobile or Capacitor wrapping the same page, key in the platform keystore | PWA with a non-extractable WebCrypto key; gomobile or Wails v3 mobile to reuse Go crypto | reuses the web UI, holds a key the browser cannot lose             | a second language (Rust) or store packaging; Ed25519 in Safari unverified |
| VS Code                       | thin TS webview extension framing the loopback page                              | none                                                                                     | `portMapping` covers HTTP and SSE                                  | a second signed artifact; remote tunnels widen the boundary               |
| JetBrains                     | JCEF panel loading the loopback page                                             | none                                                                                     | Chromium inside the IDE                                            | loopback loading unverified                                               |
| Zed                           | ParleyInn's MCP server as a Zed extension                                        | ACP, if ParleyInn ever fronts an agent                                                   | Zed has no webviews                                                | no room UI inside Zed                                                     |
| Ghostty                       | libghostty-vt reused (ghostty-web, wazero)                                       | AppleScript preview on macOS                                                             | Ghostty has no plugin API                                          | libghostty alpha, untagged                                                |
| Desktop window (optional)     | the system browser opening the loopback page                                     | Tauri 2 window pointed at the loopback URL                                               | no extra artifact                                                  | a Tauri shell is a second binary                                          |
