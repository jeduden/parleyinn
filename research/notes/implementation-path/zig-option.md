# Option H: a Zig core library with per-platform shells

Scope: written 4 October 2026. It assesses Zig as a full option for
OQ-32, "H. Zig core library + per-platform shell", against the
[evaluation frame](constraints.md). The stakeholder now asks for the
same app on Linux, Windows, macOS, iOS and Android: "start ParleyInn, then
you see your stuff". That favours a core built as a library which each
platform's shell links in process, the way Ghostty pairs a Zig core
with native shells. The note extends, and does not repeat, the
[options](options.md), the [comparison](comparison.md), the Zig column
of the [protocol and data matrix](protocol-and-data-components.md#matrix),
the Zig parts of the [terminal note](terminal-components.md) and A4 of
the [UI and packaging note](ui-and-packaging-components.md). Sibling
notes cover [smalt's tooling](zig-tooling-smalt.md),
[Bun's compile times](compile-times-bun.md),
[five-platform frameworks](five-platform-apps.md),
[Windows](windows-support.md) and
[phone reach](phone-reach-and-live-delivery.md). It recommends nothing.

Method:

- Sources were read on 4 October 2026: the Zig 0.15.1, 0.16.0 and
  0.17.0 release notes, the 0.17.0 language reference and standard
  library source, and repositories at named commits: Ghostty 5dc28bb,
  TigerBeetle 6f8e6b5 and Bun 1878660. GitHub's web pages and API
  refused this session; raw files and `git` worked.
- smalt is the stakeholder's own Zig project, read at commit c5903ed.
  It is private, so it is cited as "smalt `path` at c5903ed".
- Claims marked "measured" come from builds with the official Zig
  0.17.0 and 0.16.0 tarballs, SHA-256 checked, in a scratch directory;
  see [how this note measured](#how-this-note-measured).
- "(unverified)" marks a claim no primary source confirmed;
  "(inference)" marks a conclusion drawn from cited facts.

## Key findings

1. Zig 0.17.0 shipped on 1 October 2026 with no 1.0 date. Each of the
   last three releases broke working code, upstream warns that a
   non-trivial project "may require participating in the development
   process", and its code of conduct bans LLM-written issues and code.
2. Zig is the strongest language for two slots: the terminal, where
   libghostty-vt is native Zig, and one core inside five shells, where a
   C-ABI library with no runtime is what Ghostty and TigerBeetle ship.
3. Memory safety is the gap. Safe mode stops out-of-bounds access,
   overflow and null, but not use after free, double free, uninitialised
   reads or data races. Bun left Zig for Rust in 2026 over exactly
   those. TigerBeetle's discipline fits ParleyInn's short-lived hook path
   well and its long-lived components poorly.
4. Measured: Zig's default `std.Io.Threaded` puts socket and
   process-spawn code into every program that uses it. A core can keep
   it out only with its own `Io`, so SEC-01 evidence is buildable but
   not free, and no tool for it exists.
5. MCP, ACP, canonical JSON, git, Starlark and a CRDT have no Zig
   library; QUIC and an RE2 port exist but are months old.
6. iOS is a Tier 3 target with no CI, and 0.17.0 broke an iOS build
   that 0.16.0 accepts (measured). Android and Windows MSVC need the
   vendor's SDK for libc.
7. smalt shows the factory already runs Zig 0.16 at 266k lines with
   coverage, lint and pin gates. Zig is used by 2.1% of developers,
   against 16.4% for Go and 14.8% for Rust.

## 1. Zig's status and the cost of upgrades

### Releases and the road to 1.0

- Release dates: 0.14.0 on 5 March 2025, 0.15.1 on 19 August 2025,
  0.16.0 on 13 April 2026, 0.17.0 on 1 October 2026; master is
  0.18.0-dev ([download index][zig-index]). A breaking release comes
  every five to eight months.
- 0.17.0 "features 5 months of work: changes from 206 different
  contributors, spread among 925 commits" ([0.17.0 notes][rn17]).
- The roadmap lists "Complete and stabilize the language", "Audit the
  Standard Library", "Complete the Build System, in particular Package
  Management features" and a fuzzer "competitive with AFL"
  ([0.17.0 notes][rn17]). Since 0.16 the team accepted about 25 and
  rejected about 125 language proposals; "23 undecided language
  proposals remain open on the Codeberg issue tracker, and 61 undecided
  language proposals remain open on the legacy GitHub issue tracker"
  (same).
- "Even with Zig 0.17.x, working on a non-trivial project using Zig may
  require participating in the development process." "When Zig reaches
  1.0.0, Tier 1 support will gain a bug policy" (same).
- No date for 1.0 exists. JetBrains' write-up of a June 2026 interview
  gives the question the team asks, "What would we regret locking in if
  we shipped it today?", and no timing ([JetBrains][jb-1-0]).

### What 0.15, 0.16 and 0.17 broke

- **0.15.1.** "Writergate" deprecated every `std.io` reader and writer:
  "These changes are extremely breaking." The `async` and `await`
  keywords and `usingnamespace` were removed, and the HTTP client and
  server "have been completely reworked" ([0.15.1 notes][rn15]).
- **0.16.0.** "I/O as an Interface": "all input and output
  functionality requires being passed an Io instance", covering files,
  networking, processes, clocks and randomness. `Io.Evented` is
  "work-in-progress, experimental". Most `std.posix` and
  `std.os.windows` functions were removed, `@cImport` deprecated,
  `Thread.Pool` removed, and arguments and environment stopped being
  global ("Juicy Main") ([0.16.0 notes][rn16]).
- **0.17.0.** `@cImport` is removed in favour of an external
  translate-c package; `@bitCast` changed; the optimise modes were
  renamed (`ReleaseSafe` became `safe`); type reflection became
  struct-of-arrays; `DebugAllocator` gave way to `SafeAllocator`;
  package management moved out of the compiler ([0.17.0 notes][rn17]).
- **Measured.** A 35-line program written for 0.16, reflecting over
  `std.Io.VTable` fields, failed on 0.17.0 because of the
  struct-of-arrays change. A C-ABI library that 0.16.0 builds for
  aarch64-ios fails on 0.17.0 ([iOS](#ios-as-a-zig-target)).

### The package manager

- A dependency is a URL with a content hash and a fingerprint in
  `build.zig.zon`, fetched into a project-local `zig-pkg` directory;
  `--fork` overrides one locally; "It will become an error to have the
  same fingerprint, same version, different hash in your dependency
  tree" ([0.16.0 notes][rn16]).
- In 0.17 the fetching, HTTP, TLS and git protocol code ship as source
  inside the build system ([0.17.0 notes][rn17]).
- No central registry is named in the notes; Ghostty serves its
  dependency tarballs from `deps.files.ghostty.org` (Ghostty
  `build.zig.zon` at 5dc28bb). No advisory database or
  reachability-aware scanner for Zig packages was found (unverified
  absence), which matters for ENG-16's `govulncheck` row.

### How Zig projects cope with upgrades

| Project                                             | Zig it builds with today                 | How it copes                                                                        | Source                                                         |
| --------------------------------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Ghostty 1.3.2-dev                                   | 0.16.0 minimum                           | always a released Zig; its PACKAGING.md still says 0.14.0                           | `build.zig.zon`, `PACKAGING.md` at 5dc28bb                     |
| TigerBeetle                                         | exactly 0.14.1                           | a comptime check refuses any other version; the toolchain is downloaded by checksum | [build.zig][tb-build], [download.sh][tb-dl]                    |
| Bun                                                 | none since 2026                          | ran its own Zig fork, [oven-sh/zig][bun-zig], then ported to Rust                   | [Bun][bun-rust], [compile-times note](compile-times-bun.md)    |
| zmx, libvaxis, zig-android-sdk, zquic, zoptia0regex | 0.16.0 minimum                           | one release behind, three days after 0.17.0                                         | each repository's `build.zig.zon`                              |
| zig-sqlite                                          | 0.14.0 minimum; master tracks Zig master | a branch per Zig release                                                            | [README][zsql]                                                 |
| zig-jni                                             | 0.14.0 minimum                           | last commit 15 May 2025                                                             | [zig-jni][zigjni]                                              |
| smalt                                               | 0.16.0 pinned                            | `mise.toml` pin, the session hook's pin with SHA-256, and a `hook_pin_sync` gate    | smalt `mise.toml`, `.claude/hooks/session-start.sh` at c5903ed |

A Zig ParleyInn would pin one release, carry its dependencies' upgrades
itself, and upgrade once or twice a year (inference).

### Upstream and agents

- Zig moved its canonical repository to Codeberg on 26 November 2025;
  the GitHub repository is read-only ([Zig news][codeberg]).
- Its code of conduct holds a "Strict No LLM / No AI Policy": "No
  LLM-generated content, whether it be code or prose" and "No LLMs for
  finding bugs", for the ziglang organisation on Codeberg, its IRC
  channel and its Zulip ([code of conduct][zig-coc]).
- ParleyInn is built and reviewed by agents (ENG-21). A compiler bug an
  agent finds must reach upstream through a person writing in their own
  words (inference). Ghostty, TigerBeetle and the libraries are not
  bound by the policy.

## 2. Targets

### Support tiers for ParleyInn's targets

From the 0.17.0 support table ([0.17.0 notes][rn17]):

| Target                | Tier       | Standard library | Stack traces | Fuzzer | libc shipped  | CI      |
| --------------------- | ---------- | ---------------- | ------------ | ------ | ------------- | ------- |
| x86_64-linux          | 1          | yes              | yes          | yes    | yes           | yes     |
| aarch64-linux         | 2          | yes              | yes          | yes    | yes           | yes     |
| aarch64-macos         | 2          | yes              | yes          | yes    | yes           | yes     |
| x86_64-windows        | 2          | yes              | yes          | no     | yes           | yes     |
| aarch64-windows       | 2          | yes              | yes          | no     | yes           | partial |
| wasm32-wasi           | 2          | yes              | partial      | no     | yes           | yes     |
| aarch64-ios           | 3          | yes              | yes          | no     | no            | no      |
| aarch64-linux-android | not listed | (an ABI)         | n/a          | n/a    | no (measured) | n/a     |

Tier 1 means "The Compiler can generate machine code for this target
without relying on LLVM" and "The integrated fuzzer works on this
target"; Tier 2 means "Continuous integration machines build the
module tests for this target on every push"; Tier 3 relies "on an
external backend such as LLVM" (same).

### iOS as a Zig target

- Zig compiles aarch64-ios but ships no libc for it. Measured: a static
  library builds from Linux; an executable fails with "undefined
  symbol: `___error`", so app builds need Apple's SDK on a macOS
  builder, as in Go and Rust (inference).
- Measured regression: on 0.17.0 a library holding only SHA-256 and
  Ed25519 code failed for aarch64-ios with "no field named 'fd' in
  struct 'Io.Threaded.NullFile…'", reached through std.debug's default
  `Io`. Overriding `std_options_debug_io` gave a 483 KB library, and
  0.16.0 built the original source. Zig's tracker was not checked.
- Ghostty builds libghostty-vt as an xcframework with iOS and simulator
  slices "gated on SDK detection", and notes that "tvOS, watchOS, and
  visionOS are not yet supported by Zig's standard library"
  (`src/build/GhosttyLibVt.zig` at 5dc28bb). The app-level
  `GhosttyKit.xcframework` there carries macOS slices only
  (`src/build/GhosttyXCFramework.zig`).

### Android as a Zig target

- Zig cannot provide Bionic. Measured: "unable to provide libc for
  target aarch64-linux…android.29". A `libc.txt` must point into the
  NDK, and 0.17 changed how ("users of these targets will likely want
  to set cc_dir to the same path as crt_dir") ([0.17.0 notes][rn17]).
  A Zig library that needs no libc built (measured).
- zig-android-sdk 0.3.0 (MIT, Zig 0.16) builds an APK from one
  `.dynamic` library per ABI and needs the Android SDK, NDK and a JDK
  ([zig-android-sdk][zas]).
- SQLite is C and needs libc, so ParleyInn's Android build needs the NDK
  (inference).

### Windows as a Zig target

- x86_64-windows is Tier 2 with CI and no fuzzer. The MinGW-w64 ABI is
  self-contained: SQLite with FTS5 linked into a 2.4 MB executable
  (measured, not run). The MSVC ABI fails with "unable to provide
  libc" (measured), and Ghostty's MSVC DLL build locates the Windows
  SDK itself (`src/build/GhosttyLib.zig` at 5dc28bb).
- Ghostty has no Windows app. Its runtimes are `none`, which builds
  the library ("On macOS, Xcode is used to build the app that links to
  libghostty"), and `gtk` (`src/apprt/runtime.zig` at 5dc28bb);
  community forks add a Win32 runtime ([WolftacDigital][ghostty-win],
  unverified). libghostty-vt supports Windows (Ghostty `README.md`),
  and Ghostty's `src/pty.zig` implements ConPTY.
- Go and Rust per slot on Windows: [windows note](windows-support.md).

### WebAssembly as a Zig target

- wasm32-wasi is Tier 2; wasm32-freestanding built the seal library at
  532 KB unstripped (measured). The self-hosted WebAssembly backend
  passes "1813/1970 (92%) of behavior tests" ([0.16.0 notes][rn16]).
- libghostty-vt's `ghostty-vt.wasm` is in the
  [terminal note](terminal-components.md); smalt builds browser POCs
  for wasm32-emscripten (smalt `build/web.zig` at c5903ed).

### Static linking per target, measured

| Artifact                                  | Target             | Bytes     | Linking                                    | Two directories |
| ----------------------------------------- | ------------------ | --------- | ------------------------------------------ | --------------- |
| hello, Juicy Main, `-Osafe -fstrip`       | x86_64-linux       | 351,848   | static ELF                                 | identical       |
| same                                      | aarch64-macos      | 256,296   | `libSystem.B.dylib`, ad-hoc code signature | identical       |
| same                                      | x86_64-windows     | 603,136   | system DLLs (unverified which)             | identical       |
| SQLite 3.53.4 with FTS5, a `bm25()` query | x86_64-linux-musl  | 2,285,624 | static ELF; the query ran                  | identical       |
| same                                      | aarch64-linux-musl | 2,031,632 | static ELF; not run                        | not compared    |
| same                                      | aarch64-macos      | 1,997,312 | `libSystem.B.dylib`; not run               | not compared    |
| same                                      | x86_64-windows-gnu | 2,371,072 | not run                                    | not compared    |

Both builds of a pair ran on one Linux host; the two-builder check of
ENG-19 (an ubuntu and a macos builder) was not run. The language
reference lists "Reproducible build" for the safe, fast and small modes
([language reference][langref]).

## 3. Libraries per ParleyInn slot

- **SQLite with FTS5.** The amalgamation compiles inside `zig build`
  with any `-D` option (measured above). The binding is a page of
  `extern fn` declarations or the translate-c package. zig-sqlite (MIT)
  says "the API is still subject to changes" ([README][zsql]). SQLite
  is public domain, which is not on ENG-18's list by name, so vendoring
  it directly needs a ruling (inference). memsys5 gives SQLite a fixed
  heap: with `SQLITE_CONFIG_HEAP` it will "never call system malloc()
  or free()" ([SQLite][sqlite-malloc]).
- **JSON and RFC 8785.** `std.json` (scanner, stringify, dynamic
  values) and `std.zon` are in the standard library
  ([source][std-json]). No RFC 8785 implementation was found; key order
  by UTF-16 code units and ECMAScript number form are own code, as the
  [protocol note](protocol-and-data-components.md) found.
- **Cryptography.** `std.crypto` holds SHA-2, SHA-3, BLAKE2, BLAKE3,
  HMAC, HKDF, Ed25519, X25519, ECDSA over P-256, ML-DSA, ML-KEM, the
  AEADs AES-GCM, AES-GCM-SIV, AES-SIV, AES-OCB, ChaCha20-Poly1305,
  XChaCha20-Poly1305 and Ascon, Argon2, scrypt, a TLS client and X.509
  ([crypto.zig][crypto]). P-256 and ML-DSA are the key types an
  iPhone's Secure Enclave holds
  ([protocol note](protocol-and-data-components.md)).
- **Regex for redaction (HC-22).** zoptia0regex 0.6.0 (Apache-2.0, Zig
  0.16) is "a high-fidelity port of the RE2 engine, with a linear-time
  guarantee and ~54,000 tests proving byte-for-byte parity with Go"
  ([README][zre]): the gitleaks semantics HC-22 asks for. Its history
  is 29 commits by two authors from 25 June to 23 September 2026
  (measured from git). mvzr 0.3.9 (MIT, a Pike VM) and tiehuis'
  zig-regex 0.1.3 (MIT, Zig 0.15.1) are smaller.
- **Loopback server, SSE and WebSocket.** `std.http.Server` "only
  depends on std.Io.Reader and std.Io.Writer" since 0.15
  ([0.15.1 notes][rn15]) and has a `WebSocket` type
  ([Server.zig][http-server]). http.zig and websocket.zig (MIT, Karl
  Seguin) add routing; http.zig dates from March 2023 with 626 commits
  by 53 authors and no versioned release (`0.0.0`)
  ([http.zig][httpz]). zap (MIT) wraps the C library facil.io.
- **MCP and JSON-RPC over stdio.** No Zig SDK is on MCP's list
  ([protocol note](protocol-and-data-components.md)). Zig-written stdio
  MCP servers exist, such as [zig-mcp][zig-mcp], which bridges ZLS. A
  ParleyInn server is own code on `std.json`, as in options A to C. ACP,
  Claude Code's stream-json and the Codex app-server are hand-written
  too.
- **Pty.** Ghostty's `src/pty.zig` (MIT, 519 lines) opens POSIX ptys
  through libc's `openpty` and Windows ones through ConPTY (Ghostty at
  5dc28bb). zmx uses `forkpty` on Linux and macOS
  ([terminal note](terminal-components.md)).
- **VT model.** libghostty-vt as a Zig module: no C ABI, WASM or cgo
  in between; MIT; the API is "not yet stable"
  ([terminal note](terminal-components.md)).
- **TUI.** libvaxis 0.6.0 (MIT, Zig 0.16)
  ([UI note](ui-and-packaging-components.md)).
- **Peer transport.** zquic 1.7.86 (MIT, Zig 0.16) is a "Pure-Zig
  QUIC (RFC 9000 / 9001 / 9002), TLS 1.3, HTTP/3" stack that reports
  passing the quic-interop-runner cases (unverified); its history runs
  from 6 April to 10 July 2026, two authors, 471 commits
  ([zquic][zquic]). No maintained Zig Noise library was verified, but
  `std.crypto` holds every primitive a Noise XX handshake needs
  (inference). iroh reaches other languages only through UniFFI, whose
  runtime is MPL-2.0 ([protocol note](protocol-and-data-components.md)).
- **CRDT.** None found in Zig. automerge-c exposes Automerge through a
  C API (MIT) but builds with Rust ([automerge-c][automerge-c]).
- **Git.** No Zig library reads objects, refs, trees and diffs. Zig's
  build system carries `git.zig` (1,750 lines, MIT): SHA-1 and SHA-256
  object ids, pack indexing and smart-protocol fetch
  ([git.zig][zig-git]), a base for PEER-08's carrier (inference).
  libgit2 is "GPLv2 with a special Linking Exception"
  ([libgit2][libgit2]), off the allow-list. SSH signature checks are
  Ed25519 from `std.crypto` plus the sshsig format as own code;
  OpenPGP is own code (inference).
- **Kernel sandbox.** No Starlark implementation in Zig was found.
  zware (MIT, pure Zig) has "WebAssembly 2.0 supported (apart from the
  vector / SIMD support which is WIP)", passes the official test suite
  and has "Partial WASI support"; fuel metering is unverified
  ([zware][zware]). wasm3 (MIT, C) "will enter a minimal maintenance
  phase" ([wasm3][wasm3]). wasmtime's C API is a Rust build under
  "Apache-2.0 WITH LLVM-exception"
  ([UI note](ui-and-packaging-components.md)).
- **Key store (SEC-10).** Zig calls C frameworks directly when linked
  against Apple's SDK; Ghostty reaches Objective-C through its
  `zig_objc` dependency (`build.zig.zon` at 5dc28bb). There is no cgo
  question, but a macOS core that uses the Keychain needs the SDK on
  the builder (inference).

## 4. Safety and verification

### What safe mode checks, and what it does not

- The modes are debug, safe ("Optimizations on and safety on"), fast
  and small; safe, fast and small list "Reproducible build"
  ([language reference][langref]).
- Safety-checked illegal behaviour covers unreachable code, index out
  of bounds, integer overflow, lossy and invalid casts, null and error
  unwraps, invalid enums, misaligned pointers and the wrong union field.
  "All other Illegal Behavior is unchecked", and then "the optimizer is
  free to make Unchecked Illegal Behavior do anything" (same).
- Use after free, double free, reads of `undefined` memory and data
  races are not on the checked list (inference from the list). "The
  Zig language performs no memory management on behalf of the
  programmer" (same).
- Bun's reason to leave: "A large percentage of bugs from that list are
  use-after-free, double-free, and 'forgot to free' in an error path.
  In safe Rust, these are compiler errors", while "Zig made Bun
  possible" ([Bun][bun-rust]). Bun 1.4.0, the Rust port, shipped in
  August 2026 and "resolved 128 longstanding bugs present in v1.3.14"
  ([InfoQ][infoq-bun]).

### Allocators, fuzzing, races and coverage

- **Allocator checks.** 0.17's `SafeAllocator` "reports all leaks",
  makes "Double frees and operation … races panic or segmentation
  fault", and never reuses memory, so "most writes after free will
  segmentation fault or are eventually detected" ([0.17.0
  notes][rn17]). It is a test-time detector, not a production
  guarantee (inference).
- **Fuzzing.** `-ffuzz` and `zig build --fuzz` are built in; 0.16
  added a structured input generator, multiprocess fuzzing and crash
  dumps ([0.16.0 notes][rn16]); in 0.17 "no changes were made to the
  fuzzer itself" ([0.17.0 notes][rn17]). It runs on x86_64-linux,
  aarch64-linux and aarch64-macos, not on Windows or iOS (tier table
  above).
- **Races.** `-fsanitize-thread` links the libtsan Zig ships with
  LLVM 22 ([0.17.0 notes][rn17]); measured, it reported "WARNING:
  ThreadSanitizer: data race" on a deliberate race. `-fsingle-threaded`
  makes spawning a thread a compile error ("Cannot spawn thread when
  building in single-threaded mode", measured): race freedom by
  construction for a single-threaded executable.
- **Coverage.** No upstream tool. kcov reports zero Zig files because
  libdw rejects Zig 0.16's DWARF-5 line tables; smalt wrote its own
  ptrace tracer on `std.debug.Dwarf`, an INT3 per line, gating "100%
  minus documented waivers" on Linux x86_64 only (smalt
  `docs/coverage.md`, `tools/coverage/`, about 5.3k lines,
  `build/coverage.zig` at c5903ed). Running it at ReleaseSafe "silently
  drops 44% of the coverable lines", and `-fsanitize-coverage-trace-pc-guard`
  "is a SILENT NO-OP on the default backend" (smalt
  `plan/2607311235_coverage-native-speed.md` at c5903ed). More in the
  [smalt tooling note](zig-tooling-smalt.md).

### Static analysis and smalt's gates

- zlint 0.10.0 (MIT, Zig 0.16) is the Zig linter ([zlint][zlint]).
  smalt pins the `jeduden/zlint` fork at 0.13.0 (`mise.toml`) and makes
  `homeless-try`, `no-unresolved`, `no-anonymous-tests` and cognitive
  complexity (15 per function, 100 per file) errors, with
  `unsafe-undefined` a warning (`zlint.json`); CI runs
  `zig fmt --check` (`.github/workflows/ci.yml`), all at c5903ed.
- smalt's factory gates are Zig programs: `tools/check` (one gate
  table, a PASS, FAIL or VOID verdict per gate, about 6.4k lines),
  `tools/skills_drift` (installed frit skills byte-identical to the
  pinned frit) and `tools/hook_pin_sync` (hook pins against
  `mise.toml`), at c5903ed.
- Missing for ENG-16 and ENG-13: a security linter like `gosec`, a
  reachability-aware vulnerability scanner and a mutation tester (none
  found; unverified absence).

### TigerBeetle: TigerStyle, ReleaseSafe and the VOPR

- **TigerStyle** ([TIGER_STYLE.md][tigerstyle]): "All memory must be
  statically allocated at startup. No memory may be dynamically
  allocated (or freed and reallocated) after initialization", which
  "avoids use-after-free"; "The assertion density of the code must
  average a minimum of two assertions per function"; pair assertions;
  compile-time assertions; "Do not use recursion"; "Put a limit on
  everything"; "a hard limit of 70 lines per function"; "All errors
  must be handled"; and "a 'zero dependencies' policy, apart from the
  Zig toolchain".
- **Enforcement.** `StaticAllocator` is "An allocator wrapper which can
  be disabled at runtime", used "for allocating at startup and then
  disable it to prevent accidental dynamic allocation at runtime"
  ([static_allocator.zig][tb-static]); `src/repl.zig` uses it. The
  server's `main` builds a general-purpose allocator over huge pages
  (`src/tigerbeetle/main.zig` at 6f8e6b5); how the replica refuses
  later allocation was not traced here.
- **Safety on in production.** `build.zig` prefers ReleaseSafe, and the
  release script asserts that each release binary reports
  "build.mode=builtin.OptimizeMode.ReleaseSafe"
  ([release.zig][tb-release]). Verified: TigerBeetle ships with safety
  checks on.
- **The VOPR.** "In the simulator, all non-deterministic parts of the
  system are stubbed out. This includes the clock, network, and disk
  operations"; a run is fixed by a seed, and "the seed and Git commit
  hash can be used to replay back the exact simulation and bug"
  ([VOPR][vopr]).
- **Jepsen on 0.16.11** (Kyle Kingsbury, 6 June 2025) found "seven
  client and server crashes, including a segfault on client close and
  several panics during server upgrades", "only two safety issues"
  (missing results for multi-predicate queries, wrong debug
  timestamps) and "exceptional resilience to disk corruption". Fixes
  landed from 0.16.12 to 0.16.43; requests that never time out stay
  open. The VOPR missed the bitflip panics because "it corrupted entire
  sectors, rather than single bits", and was revised ([Jepsen][jepsen]).
  The client segfault, an invalid pointer dereference on close, was a
  memory-safety bug in Zig code under TigerStyle (inference from
  Jepsen's description).
- **One core, many runtimes.** The Go, Java, .NET, Node, Python, Ruby
  and Rust clients wrap one Zig library behind `tb_client.h`; the Go
  client links a static `libtb_client` per target through cgo
  ([clients][tb-clients]), and the Java client is "implemented via a
  JNI binding to a client library written in Zig" ([Jepsen][jepsen]).

### How far TigerStyle fits ParleyInn

- **The hook path fits.** A hook is a short-lived process with one JSON
  object in and a 50 MiB RSS budget (NFR-09). It can take its budget
  as one arena at start and never free before exit, which makes use
  after free impossible in that process, and a `-fsingle-threaded`
  build makes races impossible; SQLite takes a fixed memsys5 heap from
  the same budget (inference from the SQLite docs above).
- **Projections and I10 fit.** The VOPR's stubbed clock, network and
  disk are the shape of ParleyInn's S12 arrival-order test (PEER-03) and
  the ENG-06 kill loop. Zig 0.16 already passes clock, randomness,
  files and network as an `Io` value, so a simulator `Io` swaps them
  without a lint (inference; `Io.failing` is a stock example).
- **Long-lived components fit poorly.** TigerBeetle has fixed-size
  records and bounded batches. ParleyInn parses variable JSON transcripts
  and hook payloads up to SEC-05's limits, and its lane view, run
  component and peer run for days over unbounded lanes. They need an
  arena per request with a hard cap, freed whole; use after free then
  shrinks to a pointer outliving its request, which only a design rule
  and a lint catch (inference).
- **SQLite is C** with its own allocator in every option.

### A proposed HC-23 on memory safety

Proposed wording, mechanism-neutral:

> **HC-23 (proposed; I2, I8, I9, SEC-05, T8).** Code that parses or
> transforms content from outside the trusted boundary (hook payloads,
> transcripts, tool output, imported segments, peer and phone
> messages), and code that decides trust (redaction, the envelope,
> `TrustedText`, seals, signatures, owner acts), is memory-safe: an
> out-of-bounds access, a use after free, a double free, a read of
> uninitialised memory or a data race in it is impossible by
> construction or stops the process before the access completes. Every
> exception (an unsafe block, an unchecked region, a foreign library
> such as SQLite) is listed per component with its reason and the fuzz
> target that covers it, and CI fails on an unlisted one.

| Language | How it proves HC-23                                                                                                                                                                                                                                                                                                           | Verdict                                                                    |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Rust     | `#![forbid(unsafe_code)]` in those crates ([rustc lints][rust-lints]); unsafe only in listed crates such as the SQLite binding; races are compile errors in safe Rust                                                                                                                                                         | meets it by construction                                                   |
| Go       | no `unsafe` import and no cgo in those packages, shown by the import closure; bounds checks and a garbage collector; but races on interfaces, maps, slices and strings "can in turn lead to arbitrary memory corruption" ([Go memory model][go-mem]), so the race detector and single-writer designs carry that part          | meets it, with the race detector as evidence                               |
| Zig      | safe mode stops out-of-bounds, overflow and null; a lint bans `@setRuntimeSafety(false)`; a `-fsingle-threaded` executable cannot race; use after free, double free and `undefined` reads stay unchecked, so the hook path meets it by allocate-at-start and free-at-exit, and long-lived components only by arena discipline | hook and MCP path workable; lane view, run and peer weak; overall **weak** |

## 5. Per-component reach evidence

### What the measurement showed

- A Juicy Main hello world on the default `Io` (`Io.Threaded`), built
  `-Osafe` and unstripped, is 4.0 MB with 946 symbols, among them
  `Io.Threaded.netConnectIpPosix`, `netListenIpPosix`,
  `openSocketPosix`, `processSpawnPosix` and `posixExecveat`: 19
  network or process functions in a program that writes "hello".
- The cause: `Io.Threaded.io()` returns a vtable naming every
  implementation ([Threaded.zig][threaded]), so taking it makes all of
  them reachable, and `std.debug` uses a global `Threaded` as its
  `debug_io` unless the root overrides it ([std.zig][std-zig]). A
  `main` that never touched `Io` still carried the same 19.
- With `std_options_debug_io = Io.failing` and no `Threaded.io()`, the
  binary had 92 symbols and none for network or process; creating and
  destroying a `Threaded` without calling `io()` also added none.
- A vtable copied from `Threaded`'s at compile time, with every
  network and process entry replaced by `Io.failing`'s, still linked
  21: the copy references the original. `Threaded`'s implementations
  are private in a 19,966-line file, so a core that wants file I/O
  without socket code writes its own `Io` over `std.posix.system`, or
  forks `Threaded` (inference).
- So "unreferenced code is not compiled" holds, yet since 0.16 the
  standard `Io` references network code from any program that uses it.

### What carries over from Zig's design

- **No code before `main`.** Namespace-level variables are
  "order-independent and lazily analyzed" and their initial values are
  "implicitly comptime" ([language reference][langref]); Zig has no
  constructors. Measured: no `.init_array`, `.preinit_array` or
  `.ctors` section in any binary built here, the SQLite one included.
  HC-3's start-up question shrinks to `std.start` and linked C
  (inference).
- **Declared module imports.** A module can import only the modules
  `build.zig` wires to it. smalt's layering gate relies on it: "the
  compiler covers that half — `@import("sokol")` in a module with no
  `sokol` import wired does not build" (smalt
  `tools/layering/layering.zig` at c5903ed). That boundary is as firm
  as a Rust crate's, but `std` is wired into every module.
- **`Io` as a capability.** A component handed an `Io` whose network
  and process entries fail cannot reach them through `std`'s APIs; only
  `std.posix.system`, `std.os`, `std.c`, `asm` and `extern` bypass it,
  and each is visible in the syntax tree zlint is built on (inference).
- **A single-threaded core executable** (section 4).

### smalt's layering gate against SEC-01

- `tools/layering` (421 lines) walks every `.zig` file under each home,
  counts `@import("sokol")` in code rather than prose, and holds floors
  per home so an empty or mistyped home cannot pass (smalt
  `tools/layering/main.zig` at c5903ed).
- **What transfers** is the shape: walk every file, no skip list,
  floors against vacuous greens, a drift case (ENG-27).
- **What does not** is the proof. `std.Io.net`, `std.process.spawn` and
  `std.posix.socket` all arrive through `@import("std")`, so an import
  walk sees nothing. SEC-01 also needs: a lint per component root that
  bans `Io.Threaded`, `std.process.spawn`, `run` and `replace`,
  `std.posix`, `std.os`, `std.c`, `asm`, `extern` and
  `@setRuntimeSafety(false)`; a core `Io` built without `Threaded`'s
  vtable; a symbol check of the shipped artifact against a kept symbol
  map, which works since names like `Io.Threaded.netConnectIpPosix` are
  visible (measured); and the drift case that adds a socket to the core
  and turns CI red.

### An evidence design for H

- Ship the core (hooks, MCP, CLI, TUI, kernel worker) as its own
  single-threaded executable, and symbol-check that artifact. The lane
  view and run component ship as separate executables or inside the
  app shell. This matches the desktop shape of the
  [five-platform note](five-platform-apps.md), which keeps the hook and
  MCP entry apart from the window.
- A multi-call binary loses both the single-threaded guarantee and
  symbol evidence: per-entry reachability through vtables needs a call
  graph over machine code, and no such tool exists (inference).
- Cost: an own `Io` for files and clocks on three operating systems,
  rules in the zlint fork, a symbol checker (all unmeasured).

## 6. Embedding the core in each shell

### How Ghostty structures libghostty

- The embedded runtime is "when Ghostty is embedded within a parent
  host application, rather than owning the application lifecycle
  itself. This is used for example for the macOS build of Ghostty so
  that we can use a native Swift+XCode-based application"
  (`src/apprt/embedded.zig` at 5dc28bb). The host passes C function
  pointers (wakeup, action, clipboard) in an `extern struct`, so the
  core calls back into the shell (same file).
- `include/ghostty.h`: "The only consumer of this API is the macOS app
  … not designed for external use"; external embedders use
  libghostty-vt instead (same commit).
- `GhosttyKit.xcframework` wraps the static library with `ghostty.h`
  and a `module.modulemap` so Swift imports it
  (`src/build/GhosttyXCFramework.zig`).
- The lesson for ParleyInn: one core library; each host owns its event loop
  and windows; the core reports through callbacks; the C API is private
  to first-party shells (inference).

### Each shell against a Zig core

| Shell                 | How it links a Zig core                                                                         | Precedent                                                                                     | Status                                        |
| --------------------- | ----------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | --------------------------------------------- |
| Swift (macOS, iOS)    | static library in an xcframework with a module map                                              | Ghostty's macOS app; libghostty-vt's iOS slices                                               | proven on macOS; iOS library slices only      |
| Kotlin (Android)      | a `.dynamic` library per ABI with `Java_…` exports                                              | TigerBeetle's Java client over JNI; [zig-android-sdk][zas]; [zig-jni][zigjni] (MIT, stale)    | proven on the JVM; Android unverified for us  |
| Rust or Tauri         | `build.rs` runs `zig build` and links the static library                                        | herdr with libghostty-vt ([terminal note](terminal-components.md)); TigerBeetle's Rust client | proven; two toolchains                        |
| Flutter `dart:ffi`    | on iOS, statically linked symbols resolve through `DynamicLibrary.process` ([Flutter][flutter]) | none seen with Zig                                                                            | C ABI, language-neutral (unverified with Zig) |
| Go                    | cgo against a static library                                                                    | TigerBeetle's Go client                                                                       | breaks CON-02 for a Go binary                 |
| C webview on desktops | [webview/webview][webview] (MIT): GTK and WebKitGTK, Cocoa, WebView2; no mobile                 | none seen with Zig                                                                            | desktop only                                  |
| Browser               | the loopback page from `std.http.Server`                                                        | options A to E                                                                                | unchanged                                     |

### What iOS and Android change for ParleyInn

- App Review guideline 2.5.2: "Apps should be self-contained in their
  bundles … nor may they download, install, or execute code which
  introduces or changes features or functionality of the app"
  ([App Review][apple-review]). On a phone there is no harness, hook or
  kernel worker child (CMP-05); the phone is a client, as the
  [five-platform note](five-platform-apps.md) and the
  [phone note](phone-reach-and-live-delivery.md) find (inference).
- The shared part is small: the seal code (SHA-256 and Ed25519) is a
  483 KB iOS library (measured), so seals, canonical JSON and segment
  sync can run in process on both phones (inference).
- `std.crypto` verifies P-256 and ML-DSA signatures, the Secure
  Enclave's key types, with no dependency (inference; ParleyInn's chain is
  Ed25519 today).

## 7. Maintainers and hiring

- In Stack Overflow's 2025 survey, 2.1% of all respondents and 1.9% of
  professional developers used Zig, against 16.4% and 17.4% for Go and
  14.8% and 14.5% for Rust ([survey][so-2025]).
- ENG-25 asks for two active maintainers
  ([§10](../../../docs/srs/10-engineering-quality.md)). The stakeholder
  runs smalt: Zig 0.16.0, 793 `.zig` files and about 266k lines
  (counted here), built by agents with the same frit and mdsmith
  tooling as ParleyInn (smalt `CLAUDE.md`, `mise.toml`, `.claude/` at
  c5903ed). One fluent maintainer exists; the second is open
  (inference).
- Agents lag a moving language: the I/O, allocator and build APIs
  changed in each of the last three releases (section 1). smalt's
  mitigation is a reviewer agent that encodes "Zig 0.16 idioms
  (unmanaged ArrayList, std.Io, std.process.Init)"
  (`.claude/agents/zig-reviewer.md`) and a session hook that pins
  `ZIG_PINNED_VERSION` with its SHA-256
  (`.claude/hooks/session-start.sh`), checked by `tools/hook_pin_sync`,
  all at c5903ed. This note's own 0.16-style test code broke on 0.17.0
  (measured).
- Upstream: a compiler bug an agent finds needs a person to report it
  ([upstream and agents](#upstream-and-agents)).

## Where Zig beats Rust and Go, and where it loses

Zig wins:

- **Terminal.** libghostty-vt and Ghostty's pty code are Zig: no FFI,
  no WASM translation, no cgo. Rust links the same library through
  `build.rs` and the Zig toolchain anyway; Go runs it as WASM.
- **One core in five shells.** A C-ABI library with no runtime or
  garbage collector, as Ghostty and TigerBeetle ship. Rust matches it
  with `staticlib` but its binding generator UniFFI is MPL-2.0; Go
  carries its runtime, and gomobile is experimental.
- **C and cross-compilation.** One host builds SQLite and any C library
  for every target, statically on Linux, reproducibly across
  directories (measured). Rust needs a C toolchain per target; Go
  avoids C by WASM translation.
- **Determinism.** Clock, randomness and files arrive as an `Io`
  value, so projection code handed none cannot read them, and a
  simulator `Io` replaces them; Go and Rust rely on lints.
- **Standard library.** Crypto with AEADs, P-256 and ML-DSA, JSON, an
  HTTP server with WebSocket, all without dependencies; like Go, unlike
  Rust.
- **Start-up.** No runtime: 0.44 ms from start to exit for hello and
  1.2 ms for an in-memory FTS5 query (measured).

Zig loses:

- **Memory safety.** No compiler guarantee against use after free,
  double free, uninitialised reads or races; Rust has one and Go's
  collector covers all but races.
- **Stability.** Pre-1.0, breaking every five to eight months;
  libraries lag a release or more.
- **Ecosystem.** No MCP, ACP, canonical JSON, git, Starlark or CRDT
  library; QUIC and the RE2 port are months old. Rust is strongest; Go
  has the transports.
- **Reach evidence.** The standard `Io` links socket code into every
  binary; Go has an import-closure test today.
- **Verification tools.** No upstream coverage, vulnerability scanner,
  security linter or mutation tester; a fuzzer still short of AFL.
- **Phone targets.** iOS is Tier 3 with no CI; Android needs the NDK.
- **People.** 2.1% usage, and an upstream that refuses agent-written
  reports.

## How this note measured

All builds ran on 4 October 2026 in a scratch directory on linux/amd64
(4 vCPU), never in ParleyInn's tree or a real home.

- Toolchains: `zig-x86_64-linux-0.17.0.tar.xz` and the 0.16.0 tarball
  from ziglang.org, each checked against the SHA-256 in
  [index.json][zig-index].
- Symbols: unstripped `-Osafe` builds, `nm` counting names matching
  `Threaded.net`, `socket`, `processSpawn` or `execve`; ELF section
  names read with a short Python script; Mach-O load commands likewise.
- Reproducibility: the same source built in two directories with
  separate caches, `sha256sum` compared.
- SQLite: the 3.53.4 amalgamation with `-DSQLITE_ENABLE_FTS5
  -DSQLITE_THREADSAFE=1 -DSQLITE_SECURE_DELETE
  -DSQLITE_OMIT_LOAD_EXTENSION`, built in 47 to 70 s per target cold.
- Timing: 200 runs each from a Python loop, mean wall time; this is not
  the ENG-15 reference hardware, and peak RSS was not separable here.
- Race and threading: a two-thread counter built with
  `-fsanitize-thread`, then with `-fsingle-threaded`.

## Slot table

| Slot                      | Zig component or own code                                     | License                | Maturity                                            | Rust equivalent                            | Go equivalent                             |
| ------------------------- | ------------------------------------------------------------- | ---------------------- | --------------------------------------------------- | ------------------------------------------ | ----------------------------------------- |
| Store                     | SQLite amalgamation compiled in, own `extern` binding         | public domain (ruling) | SQLite mature; binding own; zig-sqlite API unstable | rusqlite, bundled                          | ncruces/go-sqlite3                        |
| Redaction regex (HC-22)   | zoptia0regex, RE2 port proven against Go                      | Apache-2.0             | three months old, two authors                       | `regex`                                    | `regexp`, standard library                |
| Canonical JSON (RFC 8785) | own code on `std.json`                                        | n/a                    | own                                                 | serde_json_canonicalizer                   | gowebpki/jcs or own                       |
| Hashes, signatures, AEAD  | `std.crypto`                                                  | MIT (Zig)              | standard library of a pre-1.0 language              | sha2, ed25519-dalek, chacha20poly1305      | standard library                          |
| MCP server over stdio     | own on `std.json`                                             | n/a                    | own                                                 | rmcp with HTTP off                         | own (the SDKs import `net/http`)          |
| Harness protocols         | own: stream-json, ACP, Codex app-server                       | n/a                    | own                                                 | agent-client-protocol, Codex's own types   | coder/acp-go-sdk; own                     |
| Kernel sandbox            | zware (WASM) or own Starlark                                  | MIT                    | interpreter; SIMD in progress; metering unverified  | wasmtime with fuel; starlark-rust          | wazero; starlark-go                       |
| Loopback server, SSE, WS  | `std.http.Server` with `WebSocket`; http.zig                  | MIT                    | reworked in 0.15; http.zig unversioned              | hyper or axum                              | `net/http`                                |
| Pty host                  | Ghostty `pty.zig`, POSIX and ConPTY                           | MIT                    | ships in Ghostty                                    | portable-pty                               | creack/pty plus a ConPTY module           |
| Headless VT model         | libghostty-vt as a Zig module                                 | MIT                    | shipped core, unstable API                          | libghostty-vt by C ABI; alacritty_terminal | ghostty-vt.wasm by wasm2go or wazero; cgo |
| TUI                       | libvaxis                                                      | MIT                    | 0.6.0                                               | ratatui                                    | Bubble Tea                                |
| Git reading               | own, starting from Zig's `git.zig`                            | MIT                    | packs and fetch only                                | gix                                        | go-git                                    |
| Peer transport            | zquic, or own Noise on `std.crypto`                           | MIT                    | zquic six months old, two authors                   | iroh, quinn, snow                          | quic-go, flynn/noise                      |
| CRDT                      | none; automerge-c is a Rust build                             | MIT                    | none in Zig                                         | automerge, loro                            | none without cgo                          |
| Phone core                | the same library: iOS xcframework, Android `.so` with the NDK | MIT (Zig)              | iOS Tier 3 without CI                               | Tauri or UniFFI (MPL-2.0)                  | gomobile, experimental                    |
| Shell embedding           | C ABI and callbacks, as libghostty and tb_client              | n/a                    | proven by Ghostty and TigerBeetle                   | `staticlib` plus UniFFI or cbindgen        | `c-archive` with the Go runtime           |
| Packaging                 | `zig build`, `@embedFile`, static per target                  | MIT toolchain          | reproducible across directories (measured)          | cargo with `include_dir`                   | `go build` with `embed.FS`                |
| Fuzzing, races, coverage  | built-in fuzzer, `-fsanitize-thread`, smalt's tracer          | MIT; smalt's own       | fuzzer short of AFL; coverage on Linux x86_64 only  | cargo-fuzz, Miri, llvm-cov                 | `go test -fuzz`, `-race`, `-cover`        |

## Option H in the options template

**Thesis.** One Zig core library, `libparinn`, compiled for all five
platforms and linked in process by each platform's shell, plus the
same core as the CLI, hook and MCP executable on desktops: the
Ghostty and TigerBeetle shape, with libghostty-vt native.

- **Core.** Zig 0.17, pinned. SQLite with FTS5 compiled in, under a
  fixed memsys5 heap in the hook; `std.crypto`; own RFC 8785 code; an
  own stdio MCP server on `std.json`; zoptia0regex for redaction; an
  own core `Io` with no network or process entries; a
  `-fsingle-threaded` executable that allocates at start and frees at
  exit. The kernel is a WASM guest under zware, or Starlark nowhere
  until S5 answers (OQ-10).
- **Lane view.** On desktops, `std.http.Server` on loopback serves the
  page to browsers and to the stage-one phone. Inside an app shell, the
  same TypeScript page runs in the system webview and calls `libparinn`
  in process, with no port.
- **Run component.** Ghostty's `pty.zig` (POSIX and ConPTY) and
  libghostty-vt as a Zig module; the same library compiled to wasm32
  feeds the page's terminal pane.
- **TUI.** libvaxis over libghostty-vt.
- **Phone, stage two.** `libparinn` for aarch64-ios in an xcframework
  under a Swift shell, and for Android through the NDK under a Kotlin
  shell over JNI, sharing seals, canonical JSON and sync. The device
  key is a P-256 Secure Enclave key or a wrapped Ed25519 key (common
  ground item 10 of the [options](options.md)).
- **Peer.** ParleyInn's own sealed-range exchange over zquic or a Noise
  handshake built from `std.crypto`; no CRDT without a second
  toolchain.
- **One binary.** Per desktop target, one release holding the
  single-threaded `parinn` core executable and an app shell that links
  `libparinn`; a multi-call binary only at the cost of the evidence in
  section 5.
- **Evidence.** The core never takes `Io.Threaded` and overrides
  `std_options_debug_io`; a zlint-fork rule bans `std.posix`,
  `std.os`, `std.c`, process APIs, `asm` and `extern` in core modules;
  a symbol check of the shipped core executable shows no network or
  process function; no constructor sections; its drift case.
- **Direct dependencies.** SQLite, libghostty-vt, libvaxis,
  zoptia0regex and zware: five in Zig, plus the page's packages as in
  option B, plus the three Go test modules if the godog bindings stay
  as black-box drivers. Transitive: libghostty-vt's uucode, Wuffs,
  simdutf and Highway, libvaxis's zigimg. No vulnerability scanner
  covers them.
- **Precedent.** Ghostty (a Zig core under SwiftUI and GTK shells),
  TigerBeetle (one Zig client library in seven runtimes, ReleaseSafe in
  production, the VOPR), herdr and zmx on libghostty-vt, and smalt (an
  agent-built Zig 0.16 project of 266k lines). Against it: Bun left Zig
  for Rust in 2026 over memory-safety bugs.
- **Biggest risk.** Memory safety in long-lived components that parse
  hostile input, which was Bun's reason to leave, compounded by a
  pre-1.0 language whose every release breaks code and whose upstream
  refuses agent-written bug reports.

### Draft ratings for option H

On the fifteen criteria of the
[evaluation frame](constraints.md#comparison-criteria), with the
[comparison's](comparison.md) scale.

| #     | Criterion                         | H. Zig core library | Reason                                                                                                                                                                 |
| ----- | --------------------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | Hook path under the full binary   | strong              | no runtime; 0.44 ms hello and 1.2 ms FTS5 query from start to exit (measured, not on reference hardware); peak RSS unmeasured                                          |
| 2     | Boundary evidence in one binary   | weak                | the standard `Io` links socket and spawn code into every binary (measured); clean evidence needs an own `Io`, own lints and a separate core executable                 |
| 3     | Determinism and cryptography      | strong              | every primitive in `std.crypto`; clock and randomness arrive as an `Io` value; RFC 8785 is own code; cross-platform rebuild unmeasured                                 |
| 4     | Store engine                      | workable            | SQLite with FTS5 builds statically for every desktop target (measured); own binding; Android and MSVC need vendor SDKs; public-domain ruling                           |
| 5     | Dependency budget and licences    | workable            | about five Zig dependencies on top of a broad standard library; young libraries; no vulnerability scanner; SQLite's licence needs a ruling                             |
| 6     | One reproducible, signed artifact | workable            | identical across directories for three targets (measured); two-builder check, phone builds on Xcode and NDK, and SBOM tooling unverified                               |
| 7     | Run component: hosting and pty    | strong              | Ghostty's pty with ConPTY and libghostty-vt native, no FFI; API unstable; keystroke-to-echo unmeasured                                                                 |
| 8     | Lane view and the three panes     | workable            | `std.http.Server` with WebSocket on loopback, the same TS page as B; inside app shells the page runs in a webview fed in process                                       |
| 9     | Terminal parity                   | workable            | libvaxis 0.6.0 on libghostty-vt, built from the same core code; younger than Bubble Tea or ratatui                                                                     |
| 10    | One view model across surfaces    | strong              | one library computes it and every shell, the page and the TUI receive it as data through the C ABI                                                                     |
| 11    | Git and signatures in the core    | weak                | no Zig git library; Zig's own `git.zig` covers packs and fetch only; libgit2 is off the allow-list; SSH and OpenPGP checks are own code                                |
| 12    | Kernel sandbox and MCP            | weak                | no Starlark; zware is an interpreter with metering unverified, wasm3 in minimal maintenance; MCP is own code                                                           |
| 13    | P2 reach without a rewrite        | weak                | zquic is months old with two authors; no verified Noise library; no CRDT without a Rust toolchain; iroh only through MPL-2.0 UniFFI                                    |
| 14    | Verification toolchain            | workable            | weak without smalt; with it: built-in fuzzer, TSan (measured), SafeAllocator, smalt's coverage and lint gates; coverage on Linux x86_64 only; no vulnerability scanner |
| 15    | Cost of the change                | weak                | smalt shows the factory already runs Zig 0.16 with its gates; still rewords CON-01, ENG-01 to 09 and ENG-16, supersedes ADRs, yearly upgrades, 2.1% usage              |
| HC-23 | Memory safety (proposed)          | weak                | safe mode plus allocate-and-exit carries the hook path; long-lived components rest on arena discipline, not a guarantee                                                |

[zig-index]: https://ziglang.org/download/index.json
[rn15]: https://ziglang.org/download/0.15.1/release-notes.html
[rn16]: https://ziglang.org/download/0.16.0/release-notes.html
[rn17]: https://ziglang.org/download/0.17.0/release-notes.html
[langref]: https://ziglang.org/documentation/0.17.0/
[jb-1-0]: https://blog.jetbrains.com/blog/2026/06/05/why-zig-isn-t-1-0-yet/
[codeberg]: https://ziglang.org/news/migrating-from-github-to-codeberg/
[zig-coc]: https://ziglang.org/code-of-conduct/
[threaded]: https://codeberg.org/ziglang/zig/src/tag/0.17.0/lib/std/Io/Threaded.zig
[std-zig]: https://codeberg.org/ziglang/zig/src/tag/0.17.0/lib/std/std.zig
[std-json]: https://codeberg.org/ziglang/zig/src/tag/0.17.0/lib/std/json
[crypto]: https://codeberg.org/ziglang/zig/src/tag/0.17.0/lib/std/crypto.zig
[http-server]: https://codeberg.org/ziglang/zig/src/tag/0.17.0/lib/std/http/Server.zig
[zig-git]: https://codeberg.org/ziglang/zig/src/tag/0.17.0/lib/compiler/Maker/Fetch/git.zig
[tb-build]: https://github.com/tigerbeetle/tigerbeetle/blob/6f8e6b58d1811bb21cd6ea4fb93cb2e9d77abd81/build.zig
[tb-dl]: https://github.com/tigerbeetle/tigerbeetle/blob/6f8e6b58d1811bb21cd6ea4fb93cb2e9d77abd81/zig/download.sh
[tb-static]: https://github.com/tigerbeetle/tigerbeetle/blob/6f8e6b58d1811bb21cd6ea4fb93cb2e9d77abd81/src/static_allocator.zig
[tb-release]: https://github.com/tigerbeetle/tigerbeetle/blob/6f8e6b58d1811bb21cd6ea4fb93cb2e9d77abd81/src/scripts/release.zig
[tb-clients]: https://github.com/tigerbeetle/tigerbeetle/tree/6f8e6b58d1811bb21cd6ea4fb93cb2e9d77abd81/src/clients
[tigerstyle]: https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md
[vopr]: https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md
[jepsen]: https://jepsen.io/analyses/tigerbeetle-0.16.11
[bun-rust]: https://bun.com/blog/bun-in-rust
[bun-zig]: https://github.com/oven-sh/zig
[infoq-bun]: https://www.infoq.com/news/2026/09/bun-AI-rewrite-zig-rust-4-months/
[zsql]: https://github.com/vrischmann/zig-sqlite
[zigjni]: https://github.com/FalsePattern/zig-jni
[zas]: https://github.com/silbinarywolf/zig-android-sdk
[ghostty-win]: https://github.com/WolftacDigital/ghostty-windows
[sqlite-malloc]: https://www.sqlite.org/malloc.html
[zre]: https://github.com/zoptia/zoptia0regex
[httpz]: https://github.com/karlseguin/http.zig
[zig-mcp]: https://github.com/nzrsky/zig-mcp
[zquic]: https://github.com/zigstack/zquic
[automerge-c]: https://github.com/automerge/automerge/tree/main/rust/automerge-c
[libgit2]: https://github.com/libgit2/libgit2/blob/main/COPYING
[zware]: https://github.com/malcolmstill/zware
[wasm3]: https://github.com/wasm3/wasm3
[zlint]: https://github.com/DonIsaac/zlint
[rust-lints]: https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
[go-mem]: https://go.dev/ref/mem
[flutter]: https://docs.flutter.dev/platform-integration/ios/c-interop
[webview]: https://github.com/webview/webview
[apple-review]: https://developer.apple.com/app-store/review/guidelines/
[so-2025]: https://survey.stackoverflow.co/2025/technology/
