---
id: ADR-2610042341
title: "The app shell: Tauri 2 on five platforms, the same page in the browser"
status: accepted
summary: >-
  ParleyInn's app is a Tauri 2 shell on Linux, Windows, macOS, iOS and
  Android around one web page, and the same page serves the browser
  when ParleyInn is hosted. It answers OQ-32's UI and packaging half;
  ADR-2610050528 answers the language. The stakeholder accepted it on
  14 October 2026.
---
# ADR-2610042341: The app shell

## Context

The stakeholder's target for ParleyInn is the same experience on Linux,
Windows, macOS, iOS and Android: "start ParleyInn, then you see your
stuff". It also wants a browser when ParleyInn is hosted as a web service.
OQ-32 in
[§13](../srs/13-open-questions-and-risks.md) asks which language,
runtime and storage fit the complete SRS; the shell that draws the UI
is half of that question.

The research behind this record is in the [implementation-path
notes][notes]. They compare nine options on twenty-one criteria and
memory safety. Sourced notes cover five-platform shells, phone reach,
Windows, Zig, compile times and 27 agent products. The deciding note
compares [Tauri 2, Electron and Xilem][shells]. A web page reaches the
browser unchanged. A UI the core draws needs a second UI there.

## Decision

- **Tauri 2 is the app shell** on all five platforms. Starting ParleyInn
  opens it; on a desktop the app carries the static core executable
  that hooks and the MCP server run as (common ground item 1 of the
  options). The app shell, the app and the hosted browser surface
  name packaging under OQ-32, not new concepts: how ParleyInn's
  components ship. The app shell and the app change no boundary
  (§6.3); the hosted browser surface needs its own row (below).
- **The UI is one web page**, written once. In the app it reaches its
  backend through Tauri's IPC; in a browser the same page is served by
  ParleyInn's own server and makes the same calls over HTTP and a
  WebSocket, behind one interface.
- **Presentations open in a second webview** on their own origin,
  with no bridge into the core: an HTML report, a test page. A dev
  server's URL stays inert text the person opens in their own browser
  (VIEW-22).
- **Untrusted text is rendered as text.** The page never inserts agent
  or tool output as HTML, under a CSP with no inline script and no
  `eval` (SEC-21), and the IPC exposes only the commands a capability
  allow-list names.
- **The core is Rust, in process.** Tauri hosts the Rust core library
  on every platform; [ADR-2610050528][lang] records the language.

## Alternatives

- **Electron, with Capacitor on phones.** One Chromium on every
  desktop, but no phones, a Node.js runtime in the trusted path against
  CON-03 and HC-3, about 120 to 160 MB per desktop download, and a
  Chromium release train ParleyInn would have to ship; WebKit stays a test
  target through iOS anyway.
- **A UI the core draws on the GPU** (options H and I: Zig, or Rust
  with Xilem on Vello), in Ghostty's shape. Cleaner licences, no HTML
  parsing of agent text and no webview frame, but the browser needs a
  second UI, Xilem's phones are "not yet generally usable", and
  screen readers, IME and rich text become ParleyInn's work.
- **Flutter** (option G). Stable on five platforms, but its terminal,
  diff and Linux webview are weak for the room's panes, and the browser
  build is a canvas.
- **Compose Multiplatform** (option F). The desktop runs on the JVM,
  with no single binary and Kotlin/Native on Windows at Tier 3.
- **Wails, Dioxus, Capacitor or React Native alone.** Each misses
  platforms.

## Consequences

- **SRS changes, each the stakeholder's:** Windows in CON-02 and
  NFR-10; decision 9's loopback page becomes the shared UI, not the
  way in; a phone that shows rooms and chats changes OWN-16 and
  OWN-17; the hosted browser surface, packaging under OQ-32, is a new
  boundary row (B3, or a node its principal runs), with SEC-20's
  credential rules on the network.
- **Licences (ENG-18).** Tauri is Apache-2.0 OR MIT. Its tree carries
  MPL-2.0 in five crates: four only at build time inside the
  `tauri-macros` proc-macro, and `option-ext`, through `dirs`, in the
  shipped binary. It also carries Zlib and Unicode-3.0. The allow-list
  needs widening, or `dirs` replacing. WebKitGTK on Linux is LGPL-2.1,
  loaded dynamically.
- **Calls home (I4).** WebView2 and Android's WebView send vendor
  diagnostics by default; the stakeholder rules on each, and ParleyInn
  turns off what it can.
- **Engines.** The page runs on WebKit (macOS, iOS), Chromium (Windows,
  Android) and WebKitGTK (Linux), so the test matrix covers all three.
  WebKitGTK's GPU problems on Linux are a known risk; Tauri 3's bundled
  Chromium runtime, in alpha, is the way out when it is stable.
- **Phones.** Tauri's mobile support is "not on par" with desktop;
  ParleyInn writes its own key plugin in Swift and Kotlin, and the iPhone's
  hardware keys are P-256, not Ed25519.
- **Accessibility, IME and copy** come from each webview and the
  browser.
- **The bake-off becomes a proving slice** of the Rust core in this
  shell ([ADR-2610050528][lang]).
- The stakeholder accepted this record on 14 October 2026. Windows
  joined CON-02 and NFR-10 in SRS 2.34-draft. The other SRS changes
  above, the hosted browser surface's boundary row among them, land
  with the plans that build those surfaces; the packaging stays open
  under OQ-32.

[notes]: ../../research/notes/implementation-path/options.md
[shells]: ../../research/notes/implementation-path/tauri-vs-xilem.md
[lang]: ADR-2610050528-language-rust-typescript.md
