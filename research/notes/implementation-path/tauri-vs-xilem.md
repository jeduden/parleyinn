# Tauri 2, Electron and Xilem, for five platforms and the browser

Scope: a full comparison of the two Rust UI paths for ParleyInn's app,
written 4 October 2026 for the stakeholder's target of Linux, Windows,
macOS, iOS, Android and a browser (ParleyInn hosted as a web service).
Tauri 2 is option D's shell: the UI is a web page in the platform's
webview. Xilem on Masonry and Vello is option I's toolkit: the core
draws the UI on the GPU. Licences and crate counts were measured here
with `cargo metadata` on a scratch crate per stack (Tauri 2.12.1,
Xilem 0.4.0) for each target; build times were measured on a 4-core
Linux container. The Tauri binary only constructs a builder, with no
page and no `generate_context!`, so its size understates a real app;
the Xilem binary opens a window with a label and a button. The rest is
sourced.

## Summary

| Question                       | Tauri 2                                                                                                                 | Xilem on Masonry and Vello                                                                                                   |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| What draws the UI              | the platform's webview renders HTML, CSS and JavaScript                                                                 | the app: Masonry's widget tree, laid out with Parley, drawn by Vello on the GPU through wgpu                                 |
| Framework licence              | Apache-2.0 OR MIT (tao: Apache-2.0)                                                                                     | Apache-2.0 (Xilem, Masonry); Apache-2.0 OR MIT (Vello, wgpu, Parley, AccessKit)                                              |
| Off ENG-18's list, all targets | MPL-2.0: five crates, four only at build time inside the proc-macro, `option-ext` shipped; Unicode-3.0: 18; Zlib: 2     | BSL-1.0: 2 (Windows clipboard); CC0-1.0: 1 (`hexf-parse`, wgpu's shader parser); Unicode-3.0: 6; Zlib: 3                     |
| System components              | WebKitGTK (LGPL-2.1, Linux), the WebView2 runtime (Microsoft), WKWebView, Android System WebView                        | GPU drivers only (Vulkan, Metal, Direct3D 12, GL)                                                                            |
| Crates per target (measured)   | Linux 284, Windows 246, macOS 240, iOS 261, Android 258; 350 across all                                                 | Linux 265, Windows 192, macOS 186, iOS 189, Android 191; 348 across all                                                      |
| Desktops                       | stable                                                                                                                  | through `masonry_winit`                                                                                                      |
| iOS and Android                | supported since 2.0, "not on par" with desktop                                                                          | Android "not yet generally usable"; iOS not stated (winit supports it; unverified)                                           |
| Browser                        | **the same page**, served by ParleyInn's own server                                                                     | **a second UI:** `xilem_web` renders the DOM with its own views (0.4.0, 1,458 downloads)                                     |
| Maturity                       | 2.12.1, 34 million downloads, 3.0 in alpha                                                                              | "experimental"; last release 0.4.0 on 29 October 2025                                                                        |
| Accessibility                  | the webview's own: every platform and the browser                                                                       | AccessKit on Windows, macOS, Linux, iOS and Android; no web adapter, so a canvas in a browser has none                       |
| Text input and IME             | the webview's own                                                                                                       | Parley's editor and winit's IME; mobile soft keyboards unproven                                                              |
| Terminal pane                  | ghostty-web or xterm.js in the page                                                                                     | an own widget drawing libghostty-vt's cells with Vello                                                                       |
| Diff pane                      | CodeMirror 6 merge view                                                                                                 | an own widget                                                                                                                |
| Results browser                | a second webview on its own origin                                                                                      | wry (Tauri's webview library) embedded beside the drawn UI                                                                   |
| Untrusted text                 | parsed by an HTML engine; needs CSP, text-only rendering and the IPC capability allow-list                              | laid out by Parley as text; never parsed as HTML                                                                             |
| Vendor diagnostics             | WebView2 and Android WebView send them by default (I4 ruling)                                                           | none; only in the results browser                                                                                            |
| Clean release build (measured) | 99 s for the dependency graph; 0.6 s after touching `main.rs`; 2.0 MB binary (1.3 MB stripped), plus the system webview | about 154 s for the dependency graph; 3.9 s after touching `main.rs`; 25.0 MB binary (18.2 MB stripped) for a counter window |

## Licences in detail

- **Tauri.** The MPL-2.0 crates are `cssparser`, `cssparser-macros`,
  `dtoa-short` and `selectors`, reached through `dom_query` from
  `tauri-utils` inside `tauri-macros`, a proc-macro that runs at build
  time; `option-ext`, through `dirs`, ships in the binary. MPL-2.0 is
  file-level copyleft and off ENG-18's list; using an unmodified crate
  is usually fine in law, but the list forbids it as written.
- **Xilem.** Nothing copyleft. BSL-1.0, CC0-1.0, Zlib and Unicode-3.0
  are permissive but also off the list as written.
- **Both** need ENG-18's list widened to the permissive licences they
  carry (Zlib, Unicode-3.0), and Tauri needs a ruling on MPL-2.0 or a
  replacement for `dirs`.
- **System libraries.** Tauri links the platform's webview: WebKitGTK
  is LGPL-2.1 and loaded dynamically on Linux; WebView2 and WKWebView
  are vendor components. Xilem ships none.

## The browser decides most of it

Hosting ParleyInn as a web service means a sixth surface, the browser.

- **Tauri's page is already a web app.** ParleyInn's own server (Rust, in
  the core's boundary) serves the same bundle, and the calls the page
  makes through Tauri's IPC go over HTTP and a WebSocket instead. One
  UI codebase serves all six surfaces, with the browser's
  accessibility, IME, find and copy for free.
- **Xilem needs two UIs.** Masonry draws natively; in a browser,
  `xilem_web` builds DOM elements with its own views, so the room is
  written twice. Running Masonry and Vello on a WebGPU canvas instead
  would have no accessibility, since AccessKit has no web adapter.
- **Either way the hosted service is a new boundary.** A server that
  people reach from a browser is B3, or an owner-run node under B2
  rules: the per-launch token, SEC-20's credential rules, WebAuthn and
  rate limits move from loopback to the network. CON-06 allows it only
  when the owner runs it, never as a central service.

## Electron against Tauri 2

Electron keeps the web page, so the browser comes free as with Tauri,
but it changes what the page runs in on the desktop.

| Question               | Tauri 2                                                                           | Electron                                                                                                                                                                                                              |
| ---------------------- | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Engine on the desktop  | the system's: WebKit on macOS, WebView2 (Chromium) on Windows, WebKitGTK on Linux | one bundled Chromium on every desktop, so the page renders the same everywhere                                                                                                                                        |
| Phones                 | the same shell, Tauri mobile                                                      | none: a second shell such as Capacitor, on WKWebView and Android WebView, so WebKit is a test target anyway                                                                                                           |
| Browser                | the same page                                                                     | the same page                                                                                                                                                                                                         |
| Process with the logic | Rust, in process with the core                                                    | Node.js in the main process; the Rust or Go core runs as a sidecar or a native addon                                                                                                                                  |
| Size                   | a few MB plus the system webview                                                  | 123 MB (linux-x64), 130 MB (darwin-arm64), 158 MB (win32-x64) zipped for Electron 44.5.1 alone ([UI note](ui-and-packaging-components.md)); products ship 150 to 950 MB installers ([prior art](prior-art-stacks.md)) |
| One binary             | yes                                                                               | no: a framework directory with `app.asar`                                                                                                                                                                             |
| Licence                | Apache-2.0 OR MIT                                                                 | MIT for Electron, Chromium's BSD-3 and its third-party licences; Chromium's ffmpeg ships as a shared library under LGPL (unverified)                                                                                  |
| SRS fit                | Rust shell, reach evidence by crate                                               | CON-03 bars a Node.js runtime for P0 features; Node's `fetch` and `child_process` leave no build-time reach evidence (HC-3), so the shell may hold no logic                                                           |
| Calls home by default  | WebView2's and Android WebView's vendor diagnostics                               | none from Electron itself, but spellcheck dictionaries download "from a Google CDN by default" until the app sets its own URL ([Electron docs][el-spell])                                                             |
| Security updates       | follow the operating system's webview                                             | ParleyInn ships each Chromium fix: a major every 8 weeks, the latest three supported ([Electron timelines][el-time])                                                                                                  |
| Precedent              | Vibe Kanban, Conductor, Block Buzz                                                | VS Code, Cursor, Goose, Superset; OpenCode moved from Tauri to Electron, reportedly over WebKit differences (secondary)                                                                                               |

Electron's one real gain is a single rendering engine on the
desktops. It does not remove WebKit from the test matrix, because iOS
allows only WKWebView, and it costs a Node runtime in the trusted path,
CON-03, the reach evidence, a 120 MB-plus download and a Chromium
release train. Tauri 3's bundled-Chromium runtime, in alpha, may give
the same consistency without Node ([five platforms](five-platform-apps.md)).

## What this means for the options

- **D (Rust core, Tauri) fits five platforms plus the browser with one
  UI.** Its costs are the system webviews (vendor diagnostics,
  WebKitGTK on Linux), HTML handling of untrusted text, and MPL-2.0 in
  the build.
- **I (Rust core, drawn UI) loses its main promise in the browser:**
  "one UI, drawn once" becomes two UIs, and Xilem's phones and maturity
  lag. Its gains (clean licences, no HTML parsing of agent text, no
  webview frame) do not outweigh a second UI unless the browser is a
  reduced surface.
- **Electron** buys one desktop engine at the cost of Node, CON-03,
  size and a release train; Tauri covers more platforms with less, and
  Tauri 3's bundled Chromium is the better answer to WebKit
  differences once it leaves alpha.
- **The Go family (A to C) and H** follow the same split: a web page
  reaches the browser for free; a drawn UI does not.

[el-spell]: https://www.electronjs.org/docs/latest/tutorial/spellchecker
[el-time]: https://www.electronjs.org/docs/latest/tutorial/electron-timelines
