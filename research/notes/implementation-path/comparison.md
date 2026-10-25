# Implementation path: comparing the options

Scope: step 2 of answering OQ-32, revision 2, written 4 October 2026.
It rates the nine [options](options.md) on the
[evaluation frame's](constraints.md#comparison-criteria) fifteen
criteria and on the criteria the stakeholder's direction added since
revision 1: the five-platform app, memory safety (HC-23) and the
software factory's own needs. It is a desk assessment from the
component notes the [options](options.md) list; where a rating rests on
a measurement, the note it comes from says so, and the rest wait for
the bake-off at the end.

## Decision so far

On 4 October 2026 the stakeholder chose **Tauri 2 as the app shell**
on five platforms, with the same page serving the browser when ParleyInn
is hosted ([ADR-2610042341][adr-shell], proposed). The browser settled
it: a web page reaches the browser unchanged, while a UI the core draws
needs a second UI there ([shells](tauri-vs-xilem.md)). On 5 October
2026 the stakeholder chose **Rust for the core and TypeScript for the
page** ([ADR-2610050528][adr-lang], proposed): option D. With Tauri,
Rust is in every build already, and a Rust core reaches the phone,
which a Go node (C) does not. The bake-off becomes a proving slice of
the Rust core in a Tauri app on one desktop, one phone and the
browser. Options E, F, G, H and I leave
the field as shells; H and I's lessons (drawn text is never parsed as
HTML, a C-ABI core) carry into the core's design.

## Ratings

- **strong**: meets the criterion with maintained parts as they are.
- **workable**: meets it with ParleyInn's own code, a vendored port, or one
  stated caveat.
- **weak**: experimental, unmeasured on a risky point, or a poor fit.
- **fails**: breaks a hard constraint as the SRS words it today.

Since a factory ports what it lacks (common ground item 11), criteria
3, 11, 12 and 13 rate whether a source to port and an oracle to test
against exist, not whether a library does.

## The table

| #   | Criterion                      | A. Go node, hypermedia | B. Go node, TS page | C. Go node, WASM cores | D. Rust, Tauri | E. TS on Bun | F. Kotlin MP | G. Rust, Flutter | H. Zig, Ghostty shape | I. Rust, Ghostty shape |
| --- | ------------------------------ | ---------------------- | ------------------- | ---------------------- | -------------- | ------------ | ------------ | ---------------- | --------------------- | ---------------------- |
| 0   | The app on five platforms      | workable               | workable            | workable               | strong         | weak         | weak         | workable         | workable              | workable               |
| 1   | Hook path                      | strong                 | strong              | strong                 | strong         | weak         | weak         | strong           | strong                | strong                 |
| 2   | Boundary evidence              | strong                 | strong              | strong                 | workable       | fails        | weak         | workable         | weak                  | workable               |
| 3   | Determinism and cryptography   | strong                 | strong              | strong                 | strong         | workable     | weak         | strong           | strong                | strong                 |
| 4   | Store engine                   | strong                 | strong              | strong                 | strong         | strong       | workable     | strong           | workable              | strong                 |
| 5   | Dependency budget and licences | workable               | workable            | workable               | weak           | fails        | weak         | weak             | workable              | weak                   |
| 6   | Reproducible, signed artifact  | strong                 | workable            | workable               | workable       | weak         | weak         | workable         | workable              | workable               |
| 7   | Run component and pty          | strong                 | strong              | strong                 | strong         | workable     | weak         | strong           | strong                | strong                 |
| 8   | View and the three panes       | workable               | strong              | strong                 | strong         | strong       | weak         | workable         | workable              | workable               |
| 9   | Terminal parity                | strong                 | strong              | strong                 | strong         | workable     | weak         | strong           | workable              | strong                 |
| 10  | One view model                 | strong                 | strong              | strong                 | strong         | workable     | strong       | strong           | strong                | strong                 |
| 11  | Git and signatures             | workable               | workable            | workable               | strong         | workable     | weak         | strong           | workable              | strong                 |
| 12  | Kernel sandbox and MCP         | workable               | workable            | workable               | strong         | workable     | weak         | strong           | weak                  | strong                 |
| 13  | P2 reach                       | weak                   | weak                | workable               | strong         | workable     | weak         | strong           | workable              | strong                 |
| 14  | Verification toolchain         | strong                 | strong              | strong                 | workable       | weak         | weak         | workable         | workable              | workable               |
| 15  | Cost of the change             | strong                 | strong              | strong                 | workable       | fails        | weak         | weak             | workable              | workable               |
| 16  | Agent fluency                  | strong                 | strong              | strong                 | strong         | strong       | workable     | workable         | workable              | strong                 |
| 17  | Compiler as reviewer           | workable               | workable            | workable               | strong         | weak         | workable     | strong           | workable              | strong                 |
| 18  | Iteration speed                | strong                 | strong              | strong                 | workable       | strong       | weak         | workable         | strong                | workable               |
| 19  | Toolchain stability            | strong                 | strong              | workable               | strong         | workable     | workable     | workable         | weak                  | strong                 |
| 20  | Accessibility and text input   | strong                 | strong              | strong                 | strong         | strong       | workable     | strong           | weak                  | workable               |
| HC  | HC-23, memory safety           | workable               | workable            | workable               | strong         | workable     | workable     | strong           | weak                  | strong                 |

## Why each row reads as it does

0. **The app on five platforms.** D runs one Rust core in a Tauri shell
   on all five, phones included. G does the same with Flutter, whose
   room panes are weak. H and I take Ghostty's shape: the core draws one
   UI on the GPU under a thin host per platform. Rust has a toolkit for
   that on all five (Makepad; GPUI covers only the desktops); Zig has
   sokol, which smalt draws with, and must build the widgets itself. Zig
   0.17 also regressed an iOS library build. A, B and C give every platform
   a window onto the node, but the phone cannot be a peer without a
   second core language (C narrows that with WASM modules). E and F
   miss platforms or a single binary.
1. **Hook path.** Go, Rust and Zig executables start in milliseconds
   (Zig measured at 0.44 ms); Bun's runtime and the JVM do not.
2. **Boundary evidence.** Go's import closure per entry exists today,
   checked per function on Windows. Rust needs a crate per component,
   lints and a symbol check. Zig's standard I/O links socket and spawn
   code into every binary (measured), so H needs a separate core
   executable with an own `Io`. Bun's globals leave no build-time
   evidence.
3. **Determinism and cryptography.** Go and Zig hold the primitives in
   their standard libraries, and Zig passes clock and randomness in as a
   value; Rust uses audited crates; canonical JSON is a short port
   against RFC vectors everywhere.
4. **Store.** SQLite with FTS5 everywhere; H builds it statically
   (measured) with its own binding.
5. **Dependencies and licences.** Rust's thin standard library pushes D
   and G past ten crates; Zig's broad one keeps H near five plus ports;
   Bun links LGPL-2. SQLite's public domain needs a ruling in every
   option.
6. **Reproducible artifact.** Go builds were measured bit-identical, Zig
   across three targets (measured); Rust's Apple and MSVC paths and the
   phone builds on Xcode and the NDK are unverified for all.
7. **Run component.** Go has creack/pty and ConPTY packages; Rust has
   portable-pty; Zig has Ghostty's own pty and libghostty-vt with no
   FFI.
8. **View and panes.** A web page with CodeMirror and ghostty-web
   carries the panes in B to E; A's server-rendered page and G's widgets
   are weaker. H and I draw everything themselves: the terminal with
   libghostty, the best of any option, and the chat and diff with the
   same glyph stack, while an embedded browser shows results. Agent text
   is never parsed as HTML there, so inert rendering (SEC-21) holds by
   construction; the cost is Markdown, code highlighting, selection and
   links written into the core.
9. **Terminal parity.** Bubble Tea and ratatui are mature; libvaxis is
   younger.
10. **One view model.** Computed once in the core in every option; in
    D, G and H the same library computes it inside every shell.
11. **Git.** gix in Rust; go-git in Go; a port of go-git's plumbing in
    Zig, with `git` itself as the oracle.
12. **Kernel and MCP.** Rust has wasmtime with fuel and `rmcp`; Go writes
    its stdio MCP server and meters a wazero guest by wall clock; Zig has
    no verified metered runtime.
13. **P2 reach.** Rust holds iroh, p2panda, Automerge and Loro natively.
    Zig ports Noise and links QUIC and a CRDT from C or Rust. Go has the
    transports but no CRDT, and A and B leave the phone without a core;
    C carries a CRDT and the seal code as modules.
14. **Verification toolchain.** Go's gates exist today. smalt brings
    coverage, lint and a gate table to Zig, beside a built-in fuzzer and
    the thread sanitizer; Rust has cargo-fuzz and Miri. The gates under
    `internal/` would be ported or kept as black-box drivers.
15. **Cost of the change.** A to C keep CON-01 and every ADR. D, G and H
    reword the Go-specific rows; the factory does the porting, so the
    cost is review, not writing.
16. **Agent fluency.** Models write Go, Rust and TypeScript well; Zig
    needs version-pinned aids, which smalt has (a `zig-reviewer` agent
    and a pinned toolchain); Dart and Kotlin are less common in agent
    work.
17. **Compiler as reviewer.** Rust's compiler rejects memory and data
    race errors that the others leave to review; Zig and Go reject
    ignored errors; TypeScript's types vanish at runtime.
18. **Iteration speed.** Go builds a codebase of ParleyInn's size in
    seconds; Zig rebuilds incrementally in hundreds of milliseconds on
    x86_64 Linux; Rust's incremental builds take seconds to tens of
    seconds, and Bun published none for its Rust port.
19. **Toolchain stability.** Go and Rust keep compatibility; every Zig
    release since 0.15 broke code, and upstream bans LLM-written
    reports, so the factory cannot file a compiler bug itself.

20. **Accessibility and text input.** No SRS row asks for it yet, and
    every option needs one. A webview (A to E) and Flutter's semantics
    tree give screen readers, IME and text selection largely for free.
    A UI the core draws must supply them: AccessKit covers screen readers
    on all five platforms, natively in Rust (I) and through its C API in
    Zig (H), but IME and selection are the toolkit's or ParleyInn's own
    work. Ghostty shows the pattern and its limits: the core hands its
    visible text to the host through the C API, and the macOS host
    exposes one terminal surface as an `AXTextArea` through NSAccessibility
    in `SurfaceView_AppKit.swift`, with selection and change
    notifications; position queries were still missing in 2026
    ([#9932][gh-9932]), and no Linux support was found (unverified).
    ParleyInn needs a whole tree (rooms, messages, buttons), which is
    AccessKit's model. Among Rust GPU toolkits, Masonry and Xilem on
    Vello and egui are built on AccessKit; GPUI's support is
    experimental and absent in practice on Windows; Makepad has none
    ([options](options.md#option-i-rust-core-library-ghosttys-shape)). Accessibility is also a read-and-act channel for
    same-user processes (T21): AT-SPI on Linux needs no permission, so an
    exposed "Allow" action must not count as an owner act without the
    presence check of OWN-11.

HC-23. Rust meets it by construction with `forbid(unsafe_code)`. Go
meets it without `unsafe` and cgo, with races excluded by design and
the race detector. Zig meets it on the hook path, where a process
allocates at start and frees at exit, and only by arena discipline in
the long-lived lane view, run and peer components. Kotlin and the
JavaScript in E are garbage-collected.

## What the comparison says

- **E and F leave the field**, as in revision 1: E fails three hard
  constraints, and F's desktop shell cannot meet CON-02.
- **D leads on the new requirement.** One Rust core in a Tauri shell on
  all five platforms makes a phone a full peer, and it is the only
  option strong on both the app and HC-23. Its costs are compile time
  in the factory's inner loop and a dependency count past ten.
- **The Go family is the cheap path for a node-first release.** A, B
  and C keep every gate and ADR and the fastest loop, and ship the app
  as thin windows onto the node. They fall behind the moment a phone
  must work offline as a peer; C delays that point with WASM modules.
- **H is the factory's own language, with the safety risk Bun left.**
  It is strongest on the terminal, on embedding one core everywhere and
  on the build loop, and smalt pays for its tooling. It is weakest on
  memory safety for long-lived components and on toolchain stability.
- **G is D with a different UI**, not worth the weaker panes unless a
  native-feeling UI outweighs them.
- **H and I are Ghostty's shape**, and the shell choice is separate
  from the core language: the core draws one UI on the GPU over a thin
  host, in Zig (H) or Rust (I), with an embedded browser for results.
  It gives the best terminal, one UI identical on five platforms
  (VIEW-14 by construction), no HTML parsing of agent text, and vendor
  webview diagnostics confined to the results browser. It costs
  accessibility, IME and rich text, which a webview gives for free. I
  has GPU toolkits to start from; H keeps Zig's build and terminal
  advantages with its safety and stability risks.
- **The choice is reversible.** A factory ported Bun in 11 days against
  its tests; ParleyInn's executable SRS, driving the binary through the CLI
  and MCP, is the same kind of oracle if the bindings stay black-box.

## Shortlist

1. **A Rust core: D with Tauri, or I in Ghostty's shape**, if the
   five-platform app with a peer phone is the first release. The core is
   the same; the shell spike below picks between a webview and native
   UIs.
2. **C, Go node with WASM cores**, if a node-first release with phones
   as windows comes first; port the core later if the phone must be a
   peer.
3. **H, Zig in Ghostty's shape**, if the bake-off shows arena discipline
   and simulation hold HC-23 in the long-lived components, and the
   toolchain churn is acceptable.

## The bake-off

The factory builds the same slice in Rust (D), Go (C) and Zig (H), in
parallel lanes, and the numbers decide:

| Part of the slice              | Measures                                                                                                                                                                                                       |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Canonical JSON, the signed log | RFC 8785 vectors pass; first-attempt pass rate; iterations to green                                                                                                                                            |
| Stdio MCP server with recall   | MCP Inspector passes; hook and MCP p95 and RSS on the reference hardware                                                                                                                                       |
| A port of Go's `regexp`        | RE2's test files pass; differential fuzzing against Go finds no divergence in a fixed budget (Go reuses it)                                                                                                    |
| The core in a phone app        | the same library verifies a seal inside a Tauri app on iOS and Android                                                                                                                                         |
| Reach evidence                 | a drift case that adds a socket to the core turns CI red                                                                                                                                                       |
| A long-lived component         | a lane-view loop under fuzzing and a deterministic simulation, run for a fixed budget, with no memory error                                                                                                    |
| The shell                      | the room's chat and terminal as a Tauri page and as a UI the core draws (Makepad or sokol) over the same core, on macOS and Android: code size, frame time, screen reader and IME behaviour, terminal fidelity |
| Factory cost                   | agent time, tokens, CPU per edit–build–test cycle, findings from reviewers, fuzzers and the security review                                                                                                    |

## Decisions for the stakeholder

- **One binary or one download.** Whether a desktop app carrying a
  static core executable beside the app executable meets "one binary".
- **The first release.** Whether the phone must be a peer from the
  start (favouring D or H) or may begin as a window onto a node
  (allowing C).
- **SRS changes in every option.** Windows in CON-02 and NFR-10; a
  phone that chats (OWN-16, OWN-17); peering or the owner's tunnel in
  v1 (NG4); VIEW-17 on iPhone; channels and the Codex daemon on
  OWN-03's list; WebView2 and Android WebView diagnostics under I4;
  SQLite's public domain under ENG-18; vendored forks under ENG-18;
  HC-23 as a requirement; P-256 device keys.
- **Carried from revision 1.** Whether build-only tools count toward
  the ten; the running-app preview; the kernel spike S5 in three arms.

[gh-9932]: https://github.com/ghostty-org/ghostty/issues/9932
[adr-shell]: ../../../docs/adr/ADR-2610042341-app-shell-tauri.md
[adr-lang]: ../../../docs/adr/ADR-2610050528-language-rust-typescript.md
