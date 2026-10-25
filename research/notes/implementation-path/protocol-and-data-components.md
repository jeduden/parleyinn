# Protocol and data components per candidate language

Scope: the libraries and protocols that Go, Rust, Zig, TypeScript (Bun,
Deno, Node) and Kotlin offer for ParleyInn's protocol and data slots. The
slots are harness hosting, the MCP server, SQLite, canonical JSON and
signatures, peer sync, and signing on a phone. Sources were read on
4 October 2026: package registries, repositories, documentation and
module source. Each component gets its slot, language, license,
maturity and how it fits one binary. The Codex app-server detail in
[OpenAI agent harness UIs](../openai-agent-ui/openai-agent-ui.md) and
the key chain in [identity keys](../identity-keys/identity-keys.md) are
not repeated.

Method notes:

- Versions and dates come from the registries, read on 4 October 2026:
  the Go module proxy, crates.io, npm, PyPI and Maven Central.
  "Latest" means the newest stable version listed there. A Go module
  without tags shows a pseudo-version, dated by its commit.
- The GitHub API refused this session, so stars, issue counts and
  commit cadence are not given. Raw files and `git ls-remote` worked.
- Where an import or a feature flag decides SEC-01 fit, the module
  source was read from the Go proxy or crates.io archive.
- Three claims were tested on linux/amd64: FTS5 and WebCrypto Ed25519
  in Bun 1.3.14 and Node 22.22.0, and a `bun build --compile` binary.
- Licenses are checked against ENG-18's allow-list: Apache-2.0, MIT,
  BSD, ISC ([§10](../../../docs/srs/10-engineering-quality.md)).
- "(unverified)" marks a claim no primary source confirmed;
  "(inference)" marks a conclusion drawn from cited facts.

## Key findings

1. Every harness ParleyInn must host speaks JSON over stdio: Claude Code's
   `-p` stream-json mode, ACP and the Codex app-server. The run
   component can therefore be written in any language. Anthropic's
   docs tell callers in other languages to "run the CLI as a
   subprocess" ([Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview)).
2. The official Claude Agent SDK exists only for TypeScript and
   Python, under Anthropic's Commercial Terms, which are not on
   ENG-18's allow-list. Go has MIT community ports. The Rust crate
   named `claude-agent-sdk` points at an Anthropic repository that is
   not public.
3. ACP's official SDKs are Rust, TypeScript, Kotlin (JVM only), Java
   and Python; Go has only community SDKs. Claude Code and Codex reach
   ACP only through Node adapters.
4. MCP rates the Go, Rust and TypeScript SDKs Tier 1. Both Go MCP
   SDKs import `net/http` in the package a server needs, and the
   official one also imports `os/exec`, which ParleyInn's import-closure
   test bans from the core. rmcp puts HTTP and child processes behind
   Cargo features.
5. SQLite with FTS5 is strong in Go, Rust and Bun. Node's
   `node:sqlite` is a release candidate; Zig's binding is pre-1.0.
6. Peer sync is richest in Rust (Iroh 1.x, p2panda, Willow, Automerge,
   Loro, yrs) and JavaScript (Hypercore, Autobase, js-libp2p, Automerge,
   Yjs). Go has strong transports but no maintained CRDT. Each NAT
   traversal stack defaults to a vendor or commons service, which
   CON-06 requires replacing with enrolled peers.
7. No iPhone holds an Ed25519 key in hardware: the Secure Enclave
   offers P-256, ML-KEM and ML-DSA only. Android's TEE must support
   Ed25519; StrongBox does not. On an iPhone, PRV-10's phone key is
   either a software Ed25519 key or a hardware P-256 key.

## 1. Harness hosting protocols

OWN-03 lists the inputs ParleyInn may use: "the Agent SDK, ACP, the Codex
app-server, or the terminal the run component hosts"
([§5c](../../../docs/srs/05c-owner-and-peer-requirements.md)). The run
component (B1) is the one that "starts programs and hosts their
terminals" ([§6 register, row 10](../../../docs/srs/06-security.md)).

### Claude Code: the Agent SDK and stream-json

- **Official languages.** The Agent SDK is "programmable in Python and
  TypeScript", "A library that runs the Claude Code binary". "To drive
  the same agent loop from a language other than Python or
  TypeScript, run the CLI as a subprocess" —
  [Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview).
- **TypeScript SDK.** `@anthropic-ai/claude-agent-sdk` 0.3.289,
  3 October 2026
  ([npm](https://www.npmjs.com/package/@anthropic-ai/claude-agent-sdk)).
  Its registry metadata lists eight per-platform optional packages
  (linux glibc and musl, darwin, win32) that carry the CLI, and peer
  dependencies on `@modelcontextprotocol/sdk` and `zod`.
- **Python SDK.** `claude-agent-sdk` 0.2.163, 30 September 2026
  ([PyPI](https://pypi.org/project/claude-agent-sdk/)); "The Claude
  Code CLI is automatically bundled with the package"
  ([README](https://github.com/anthropics/claude-agent-sdk-python)).
- **License.** "Use of the Claude Agent SDK is governed by Anthropic's
  Commercial Terms of Service"
  ([overview](https://code.claude.com/docs/en/agent-sdk/overview)).
  The TypeScript package says "SEE LICENSE IN README.md"; the Python
  repository also holds an MIT `LICENSE` file
  ([LICENSE](https://github.com/anthropics/claude-agent-sdk-python/blob/main/LICENSE)).
  Linking either into ParleyInn needs an ENG-18 decision (inference).
- **Login.** "Anthropic does not allow third party developers to offer
  claude.ai login or rate limits for their products, including agents
  built on the Claude Agent SDK"
  ([overview](https://code.claude.com/docs/en/agent-sdk/overview)).
  Whether this covers a run component that starts the owner's own
  signed-in `claude` is not stated (open).
- **stream-json, the language-neutral path.** `--input-format` takes
  "`text`, `stream-json`"; `--output-format stream-json` is
  "newline-delimited JSON". `--replay-user-messages` echoes input.
  `--permission-prompts` "With the default `host`, Claude Code sends
  them to the Agent SDK host or the `--permission-prompt-tool` tool"
  ([CLI reference](https://code.claude.com/docs/en/cli-reference)).
  The headless page documents `system/init` with a `capabilities`
  array for feature detection, `system/api_retry`, `permission_denied`
  and the final `result`
  ([headless](https://code.claude.com/docs/en/headless)).
- **Bare mode.** `--bare` skips hooks, plugins, MCP servers and
  CLAUDE.md; it "is the recommended mode for scripted and SDK calls,
  and will become the default for `-p` in a future release"
  ([headless](https://code.claude.com/docs/en/headless)). A ParleyInn-hosted
  session would then have to pass its hooks and MCP server explicitly
  through `--settings` and `--mcp-config` (inference).
- **Gap.** The control messages behind SDK permission callbacks and
  hook callbacks share the same stdin and stdout. The docs index has
  no wire specification for them (unverified), so community ports
  reimplement them from the SDK source.
- **Go ports.** `severity1/claude-agent-sdk-go` v0.9.0, 4 October 2026,
  MIT: "Unofficial", "100% Python SDK compatibility", and it needs
  Node.js and Claude Code installed
  ([README](https://github.com/severity1/claude-agent-sdk-go),
  [proxy](https://proxy.golang.org/github.com/severity1/claude-agent-sdk-go/@latest)).
  Others:
  - `tggo/claude-agent-go` v0.2.1, 16 July 2026
    ([proxy](https://proxy.golang.org/github.com/tggo/claude-agent-go/@latest)).
  - `panbanda/claude-agent-sdk-go`, pseudo-version of 15 July 2026
    ([pkg.go.dev](https://pkg.go.dev/github.com/panbanda/claude-agent-sdk-go)).
  - `next-bin/claude-agent-sdk-golang` v0.1.59-beta.2, 15 April 2026
    ([proxy](https://proxy.golang.org/github.com/next-bin/claude-agent-sdk-golang/@latest)).
- **Rust ports.** `claude-agent-sdk` 0.1.1 (30 September 2025, MIT,
  4,123 downloads) names `anthropics/claude-agent-sdk-rust` as its
  repository ([crates.io](https://crates.io/crates/claude-agent-sdk)).
  `git ls-remote` found no such public repository, while the
  TypeScript and Python ones answered, so the crate is not Anthropic's
  (inference). `cc-agent-sdk` 0.1.7 and `claude-agents-sdk` 0.1.7 have
  under 500 downloads each
  ([cc-agent-sdk](https://crates.io/crates/cc-agent-sdk),
  [claude-agents-sdk](https://crates.io/crates/claude-agents-sdk)).
- **Zig and Kotlin.** No port found; stream-json by hand.

### Agent Client Protocol (ACP)

- **Wire.** JSON-RPC 2.0; under stdio "The client launches the agent
  as a subprocess" and "Messages are delimited by newlines"
  ([Transports](https://agentclientprotocol.com/protocol/v2/transports)).
- **Versions.** "The current stable ACP protocol version is `1`"
  ([README](https://github.com/agentclientprotocol/agent-client-protocol)).
  "The v2 protocol surface as a whole is still labeled draft"; v2
  drops the client file-system, terminal and session-mode APIs
  ([v2 migration](https://agentclientprotocol.com/protocol/v2/migration)).
- **Official SDKs** ([README](https://github.com/agentclientprotocol/agent-client-protocol)):
  - Rust: `agent-client-protocol` 2.2.0, 18 September 2026,
    Apache-2.0 ([crates.io](https://crates.io/crates/agent-client-protocol)).
  - TypeScript: `@agentclientprotocol/sdk` 1.7.0, 2 October 2026,
    Apache-2.0
    ([npm](https://www.npmjs.com/package/@agentclientprotocol/sdk)).
    The old `@zed-industries/agent-client-protocol` is deprecated.
  - Kotlin: `com.agentclientprotocol:acp` 0.30.1, Maven Central,
    25 August 2026
    ([Central](https://central.sonatype.com/artifact/com.agentclientprotocol/acp));
    "It currently supports JVM, other targets are in progress"
    ([Kotlin](https://agentclientprotocol.com/libraries/kotlin)).
  - Java and Python (`agent-client-protocol` 0.12.1,
    [PyPI](https://pypi.org/project/agent-client-protocol/)).
- **Go.** Community only; seven libraries are listed
  ([community](https://agentclientprotocol.com/libraries/community)).
  `coder/acp-go-sdk` v0.13.5, 2 June 2026, Apache-2.0, ships examples
  that bridge to Claude Code and to Gemini CLI
  ([README](https://github.com/coder/acp-go-sdk)).
- **Zig.** None listed.
- **Harnesses** ([Agents](https://agentclientprotocol.com/get-started/agents)):
  - Native, among about forty: Gemini CLI, OpenCode, Cursor, GitHub
    Copilot (public preview), Goose, Junie, Kiro CLI, Qwen Code.
  - Gemini CLI starts with `gemini --acp`, "standard input/output
    (stdio) using the JSON-RPC 2.0 protocol"
    ([ACP mode](https://geminicli.com/docs/cli/acp-mode/)).
  - `opencode acp` "communicates with your editor over JSON-RPC via
    stdio" ([OpenCode ACP](https://opencode.ai/docs/acp/)).
  - Through adapters: Claude ("Zed's SDK adapter"), Codex ("ACP's
    adapter") and Pi.
- **Adapters need Node.** `@agentclientprotocol/claude-agent-acp`
  0.85.1 (2 October 2026, Apache-2.0) depends on the TypeScript Agent
  SDK and declares Node 22 or later
  ([npm](https://www.npmjs.com/package/@agentclientprotocol/claude-agent-acp)).
  `@agentclientprotocol/codex-acp` 2.1.1 (1 October 2026) depends on
  `@openai/codex`
  ([npm](https://www.npmjs.com/package/@agentclientprotocol/codex-acp)).
  Hosting Claude over ACP thus runs Node and Commercial Terms code,
  whatever language ParleyInn uses (inference).

### Codex app-server

- The protocol is in
  [the OpenAI note, §2](../openai-agent-ui/openai-agent-ui.md#2-the-codex-app-server-protocol):
  JSON-RPC without the `jsonrpc` field, JSONL over stdio by default,
  schemas emitted by `codex app-server generate-json-schema`, and an
  "experimental" label.
- `@openai/codex` and `@openai/codex-sdk` 0.160.0, 1 October 2026,
  Apache-2.0 ([npm](https://www.npmjs.com/package/@openai/codex-sdk)).
- The protocol types are Rust, in `codex-rs/app-server-protocol` of
  [openai/codex](https://github.com/openai/codex/tree/main/codex-rs/app-server-protocol).
  The crates.io `codex-app-server-protocol` 0.63.0 is published from a
  fork, `namastexlabs/codex`, not by OpenAI
  ([crates.io](https://crates.io/crates/codex-app-server-protocol)).
- Fit: any language with a JSON Schema code generator; Rust can take
  the upstream crate as a git dependency (inference).

### OpenCode server

- `opencode serve` "runs a headless HTTP server that exposes an
  OpenAPI endpoint", on 127.0.0.1:4096 by default, with an OpenAPI 3.1
  spec at `/doc`, server-sent events at `/global/event`, and basic
  auth through `OPENCODE_SERVER_PASSWORD`
  ([Server](https://opencode.ai/docs/server/)).
- `opencode-ai` and `@opencode-ai/sdk` 1.18.34, 30 September 2026, MIT
  ([npm](https://www.npmjs.com/package/@opencode-ai/sdk)). The Go SDK
  `sst/opencode-sdk-go` v0.19.2 (18 December 2025) "is generated with
  Stainless" ([README](https://github.com/sst/opencode-sdk-go)).
- Fit: the HTTP route needs a loopback HTTP client in the run
  component; `opencode acp` over stdio avoids it (inference).

### Which paths are language-neutral

| Path               | Wire                                  | Official typed clients                       | Needs at runtime                     |
| ------------------ | ------------------------------------- | -------------------------------------------- | ------------------------------------ |
| Claude `-p` stream | NDJSON over stdio                     | TypeScript, Python (Agent SDK)               | the `claude` binary                  |
| ACP                | JSON-RPC 2.0, NDJSON over stdio       | Rust, TypeScript, Kotlin (JVM), Java, Python | the agent; Node for Claude and Codex |
| Codex app-server   | JSON-RPC lite, JSONL over stdio       | TypeScript SDK; schema generators            | the `codex` binary                   |
| OpenCode `serve`   | HTTP and SSE on loopback, OpenAPI 3.1 | TypeScript, Go (generated)                   | the `opencode` binary                |
| OpenCode `acp`     | ACP over stdio                        | as ACP                                       | the `opencode` binary                |

All five are language-neutral on the wire. A Go, Zig or Kotlin run
component pays only for writing or generating the message types.
TypeScript has official typed clients for every path, and Rust has the
official ACP SDK and Codex's own types (inference from the table).

## 2. MCP server SDKs

ParleyInn's MCP server is a core (B0) component speaking stdio with the
harness ([§6 register, row 2](../../../docs/srs/06-security.md)). MCP's
stdio binding "is just newline-delimited JSON-RPC"
([transports, 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports)).

- **Tiers.** The current spec is 2026-07-28. TypeScript, Python, C#,
  Go, Rust and Ruby are Tier 1, Java Tier 2, and Swift, PHP and Kotlin
  Tier 3 ([SDKs](https://modelcontextprotocol.io/docs/2026-07-28/sdk)).
  Tier 1 means a "100% pass rate" on conformance tests and new
  features "Before new spec version release"
  ([tiers](https://modelcontextprotocol.io/community/sdk-tiers)).
- **Go, official.** `modelcontextprotocol/go-sdk` v1.8.0, 4 September
  2026 ([proxy](https://proxy.golang.org/github.com/modelcontextprotocol/go-sdk/@latest)).
  "v1.7.0+" supports spec 2026-07-28; "licensed under Apache 2.0 for
  new contributions, with existing code under MIT"
  ([README](https://github.com/modelcontextprotocol/go-sdk)).
  - In the v1.8.0 archive, package `mcp` imports `net` in five files,
    `net/http` thirteen times, and `os/exec` in `cmd.go`
    ([module zip](https://proxy.golang.org/github.com/modelcontextprotocol/go-sdk/@v/v1.8.0.zip)).
  - Its JSON-RPC layer, `internal/jsonrpc2`, imports no network or
    process package, but it is internal.
- **Go, community.** `mark3labs/mcp-go` v1.1.1, 23 September 2026,
  MIT; its `server` package imports `net/http` 27 times
  ([proxy](https://proxy.golang.org/github.com/mark3labs/mcp-go/@latest)).
- **Go fit.** Linking either means allow-listing `net/http`, which
  [imports_test.go](../../../cmd/parinn/imports_test.go) bans from the
  core. A stdio-only server written against the spec is the clean fit
  (inference).
- **Rust.** `rmcp` 3.5.0, 28 September 2026, Apache-2.0, 32 million
  downloads, Rust 1.88 or later
  ([crates.io](https://crates.io/crates/rmcp)). From its
  `Cargo.toml`:
  - Default features are `base64`, `macros` and `server`; stdio is
    `transport-io` on `tokio/io-std`.
  - HTTP (`transport-streamable-http-*`, `reqwest`, `hyper`) and
    `transport-child-process` (`tokio/process`) are opt-in.
  - A core crate can build with only `server` and `transport-io`, and
    `cargo tree` shows its reach. `std::net` and `std::process` stay
    in the standard library, so a lint such as Clippy's
    `disallowed-methods` is still needed (inference).
- **TypeScript.** `@modelcontextprotocol/sdk` 1.32.0, 2 October 2026,
  MIT, the reference SDK
  ([npm](https://www.npmjs.com/package/@modelcontextprotocol/sdk)).
- **Kotlin.** `io.modelcontextprotocol:kotlin-sdk` 0.15.0, Maven
  Central, 28 July 2026, Tier 3
  ([Central](https://central.sonatype.com/artifact/io.modelcontextprotocol/kotlin-sdk)).
- **Zig.** No SDK on the official list; community libraries only
  (maturity unverified).

## 3. SQLite

The store needs FTS5 with BM25, query cancellation, WAL writes from 50
processes and a static binary
(ADR-2609302341).

- **Go.**
  - `ncruces/go-sqlite3` v0.35.6, 23 September 2026, MIT, the chosen
    driver: cgo-free through wasm2go, with encrypting VFSes
    ([proxy](https://proxy.golang.org/github.com/ncruces/go-sqlite3/@latest)).
  - `modernc.org/sqlite` v1.60.1, 29 September 2026, BSD-3-Clause
    ([LICENSE](https://gitlab.com/cznic/sqlite/-/raw/master/LICENSE));
    the ADR records that it links `os/exec` and `net`.
  - `mattn/go-sqlite3` v1.14.52, 5 September 2026, MIT, needs cgo and
    breaks CON-02
    ([proxy](https://proxy.golang.org/github.com/mattn/go-sqlite3/@latest)).
- **Rust.**
  - `rusqlite` 0.40.2, 8 August 2026, MIT
    ([crates.io](https://crates.io/crates/rusqlite)). Its `bundled`
    feature compiles SQLite 3.53.2 with `-DSQLITE_ENABLE_FTS5`
    (read in `libsqlite3-sys` 0.38.2's `build.rs`). It needs a C
    compiler at build time and links SQLite statically.
  - `sqlx` 0.9.0, 21 May 2026, MIT or Apache-2.0, async over a runtime
    ([crates.io](https://crates.io/crates/sqlx)).
  - `turso` 0.8.1, 29 September 2026, MIT, from
    `tursodatabase/turso` ([crates.io](https://crates.io/crates/turso));
    FTS5 parity unverified.
- **Bun.** `bun:sqlite` is built in. On Linux and Windows "Bun
  statically links its own SQLite build"; on macOS "Bun uses the
  system-provided SQLite" unless `Database.setCustomSQLite(path)` is
  called ([Bun SQLite](https://bun.com/docs/runtime/sqlite)).
  - Tested: Bun 1.3.14 carries SQLite 3.53.0, and an FTS5 table with
    `bm25()` works, also inside a `bun build --compile` executable.
  - FTS5 in macOS's system SQLite is unverified.
- **Node.** `node:sqlite` is "Stability: 1.2 - Release candidate" in
  v26.10.0; release candidate since v25.7.0, unflagged since v23.4.0
  and v22.13.0; it has a session and changeset API
  ([Node docs](https://nodejs.org/api/sqlite.html)).
  - Tested: FTS5 works in Node 22.22.0, which carries SQLite 3.50.4.
  - `better-sqlite3` 13.0.3, 5 August 2026, MIT, is a native addon
    ([npm](https://www.npmjs.com/package/better-sqlite3)).
- **Deno.** Exposes the same `node:sqlite` `DatabaseSync` API
  ([Deno docs](https://docs.deno.com/api/node/sqlite/)).
- **Zig.** `vrischmann/zig-sqlite`, MIT: "the API is still subject to
  changes"; it tracks Zig master and 0.15.1
  ([README](https://github.com/vrischmann/zig-sqlite)), while Zig 0.17.0
  shipped on 1 October 2026
  ([downloads](https://ziglang.org/download/index.json)). It compiles
  the amalgamation and takes options such as FTS5.
- **Kotlin.**
  - `org.xerial:sqlite-jdbc` 3.53.4.0, 26 August 2026
    ([Central](https://central.sonatype.com/artifact/org.xerial/sqlite-jdbc));
    its JNI packaging and FTS5 are unverified.
  - `app.cash.sqldelight` 2.4.0, 18 September 2026
    ([Central](https://central.sonatype.com/artifact/app.cash.sqldelight/runtime)).
  - `androidx.sqlite:sqlite-bundled` 2.8.0-alpha01 on Google Maven
    ([metadata](https://dl.google.com/android/maven2/androidx/sqlite/sqlite-bundled/maven-metadata.xml)).

## 4. Canonical JSON, Ed25519 and SHA-256

The hash chain uses "the JSON Canonicalization Scheme (RFC 8785) over a
documented field set, with SHA-256"
([§8.3](../../../docs/srs/08-data-and-storage.md)). RFC 8785 is by
A. Rundgren, B. Jordan and S. Erdtman
([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)). The reference
repository lists implementations in Rust, JavaScript, Java, Go, .NET
and Python
([cyberphone/json-canonicalization](https://github.com/cyberphone/json-canonicalization)).

| Language | JCS                                                                                        | Ed25519                                                               | SHA-256                           |
| -------- | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------- | --------------------------------- |
| Go       | `gowebpki/jcs` v1.0.2, 21 Sep 2026, Apache-2.0; reference code, 13 Dec 2024 pseudo-version | `crypto/ed25519`, standard library                                    | `crypto/sha256`, standard library |
| Rust     | `serde_json_canonicalizer` 0.3.2, 3 Feb 2026, MIT; `serde_jcs` 0.2.0, MIT or Apache-2.0    | `ed25519-dalek` 3.0.0, 6 Jul 2026, BSD-3-Clause; `ring`; `aws-lc-rs`  | `sha2` 0.11.0, MIT or Apache-2.0  |
| TS       | `canonicalize` 5.1.0, 18 Sep 2026, Apache-2.0, by co-author Erdtman                        | WebCrypto (tested in Bun and Node); `@noble/ed25519` 3.2.0, MIT       | WebCrypto; `@noble/hashes` 2.4.0  |
| Zig      | none found                                                                                 | `std.crypto.sign.Ed25519`                                             | `std.crypto.hash.sha2.Sha256`     |
| Kotlin   | `java-json-canonicalization` 1.1, August 2018                                              | JDK 15 EdDSA (JEP 339); `cryptography-kotlin` 0.6.0 for multiplatform | JDK `MessageDigest`               |

Sources for the table:

- Go: [gowebpki/jcs](https://github.com/gowebpki/jcs),
  [crypto/ed25519](https://pkg.go.dev/crypto/ed25519).
- Rust: [serde_json_canonicalizer](https://crates.io/crates/serde_json_canonicalizer),
  [serde_jcs](https://crates.io/crates/serde_jcs),
  [ed25519-dalek](https://crates.io/crates/ed25519-dalek),
  [sha2](https://crates.io/crates/sha2).
- TypeScript: [canonicalize](https://www.npmjs.com/package/canonicalize),
  [@noble/ed25519](https://www.npmjs.com/package/@noble/ed25519).
- Zig: [lib/std/crypto.zig](https://codeberg.org/ziglang/zig/src/branch/master/lib/std/crypto.zig).
- Kotlin: [java-json-canonicalization](https://central.sonatype.com/artifact/io.github.erdtman/java-json-canonicalization),
  [JEP 339](https://openjdk.org/jeps/339),
  [cryptography-kotlin](https://central.sonatype.com/artifact/dev.whyoleg.cryptography/cryptography-core).

Notes:

- JCS serialises numbers the way ECMAScript does
  ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)), so JavaScript
  gets that part from the runtime; other languages port the reference
  number formatting. Over a documented field set, an own
  implementation is small in any language (inference).
- The identity-keys note proposes the owner's SSH key through
  `ssh-keygen -Y sign`. Go has `hiddeco/sshsig` v0.2.0 (12 April 2025,
  Apache-2.0,
  [proxy](https://proxy.golang.org/github.com/hiddeco/sshsig/@latest));
  Rust has `ssh-key` 0.6.7 (Apache-2.0 or MIT,
  [crates.io](https://crates.io/crates/ssh-key)). Reaching ssh-agent
  takes a Unix socket, so it belongs in a signer process outside the
  core (inference from SEC-01).

## 5. Peer sync and replication

B2 traffic must be "encrypted and mutually authenticated with enrolled
keys" and carry only sealed ranges (SEC-24,
[§6](../../../docs/srs/06-security.md)). No third-party service may be
needed "for enrollment, discovery or relay" (PEER-02), and a relay hop
goes "through an enrolled peer" (PEER-04,
[§5c](../../../docs/srs/05c-owner-and-peer-requirements.md)).

### Transports, NAT traversal and tunnels

- **Iroh (Rust).** `iroh` 1.3.0, 28 September 2026; 1.0.0 shipped on
  15 June 2026; MIT or Apache-2.0
  ([crates.io](https://crates.io/crates/iroh)).
  - "Iroh hardcodes a set of public relays provided by n0.computer"
    ([Relays](https://docs.iroh.computer/concepts/relays)).
  - DNS lookup is "part of the `presets::N0` defaults", pointed at
    "the n0-hosted server at `dns.iroh.link`"
    ([DNS](https://docs.iroh.computer/connecting/dns-address-lookup)).
  - Self-hosted relays and mDNS lookup are documented
    ([self-hosted](https://docs.iroh.computer/iroh-services/relays/self-hosted),
    [mDNS](https://docs.iroh.computer/connecting/local-address-lookup)).
    For CON-06, ParleyInn would drop the N0 preset and run the relay on
    an enrolled peer (inference).
  - The higher protocols are still 0.x: `iroh-gossip` 0.101.0,
    `iroh-blobs` 0.103.0, `iroh-docs` 0.101.0, all 15 June 2026
    ([iroh-gossip](https://crates.io/crates/iroh-gossip)).
- **Iroh from other languages.** iroh-ffi "defines Python, Swift,
  Kotlin and Node.js bindings" that "mirror the stabilized iroh 1.0
  surface"; gossip, blobs and docs are out of scope, and Go is
  "Community maintained" ([iroh-ffi](https://github.com/n0-computer/iroh-ffi)).
  - Packages at 1.1.0, July 2026:
    [`@number0/iroh`](https://www.npmjs.com/package/@number0/iroh),
    [PyPI `iroh`](https://pypi.org/project/iroh/),
    [`computer.iroh:iroh`](https://central.sonatype.com/artifact/computer.iroh/iroh).
  - It builds on `uniffi` 0.31.1
    ([Cargo.toml](https://github.com/n0-computer/iroh-ffi/blob/main/Cargo.toml)),
    whose runtime is MPL-2.0, off ENG-18's list.
- **libp2p.**
  - Go: `go-libp2p` v0.50.0, 21 September 2026, MIT
    ([proxy](https://proxy.golang.org/github.com/libp2p/go-libp2p/@latest)).
  - Rust: `libp2p` 0.57.0, 11 September 2026, MIT
    ([crates.io](https://crates.io/crates/libp2p)).
  - JavaScript: `libp2p` 3.3.11, 2 September 2026, Apache-2.0 or MIT
    ([npm](https://www.npmjs.com/package/libp2p)); Noise through
    `@chainsafe/libp2p-noise` 17.0.0
    ([npm](https://www.npmjs.com/package/@chainsafe/libp2p-noise)).
  - JVM: published on Cloudsmith, not Maven Central; on Android "we are
    not aware of anyone using it in production"
    ([jvm-libp2p](https://github.com/libp2p/jvm-libp2p)).
- **Holepunch (JavaScript).** `hypercore` 11.37.1, `hyperswarm`
  4.17.2 and `hyperdht` 6.34.0 (MIT) and `autobase` 7.28.2
  (Apache-2.0), all September 2026
  ([hypercore](https://www.npmjs.com/package/hypercore),
  [hyperswarm](https://www.npmjs.com/package/hyperswarm),
  [hyperdht](https://www.npmjs.com/package/hyperdht),
  [autobase](https://www.npmjs.com/package/autobase)).
  - The default DHT bootstrap is three `nodeN.hyperdht.org` hosts,
    "publicly served on behalf of the commons"; an isolated DHT starts
    from `bootstrap: []`
    ([hyperdht](https://github.com/holepunchto/hyperdht)).
  - It runs on Node and on Bare; Bun support is unverified.
- **Noise.** Go `flynn/noise` v1.1.0, 2 February 2024, BSD-3-Clause
  ([LICENSE](https://github.com/flynn/noise/blob/master/LICENSE)).
  Rust `snow` 0.10.0, 19 July 2025, Apache-2.0 or MIT
  ([crates.io](https://crates.io/crates/snow)). JavaScript
  `noise-handshake` 4.2.0, Apache-2.0
  ([npm](https://www.npmjs.com/package/noise-handshake)). None found
  for Zig or Kotlin.
- **QUIC.** Go `quic-go` v0.63.0, 22 September 2026, MIT
  ([proxy](https://proxy.golang.org/github.com/quic-go/quic-go/@latest));
  Rust `quinn` 0.11.12, 14 September 2026, MIT or Apache-2.0
  ([crates.io](https://crates.io/crates/quinn)).
- **WireGuard in user space.**
  - `wireguard-go`, MIT, pseudo-version of 22 May 2026
    ([proxy](https://proxy.golang.org/golang.zx2c4.com/wireguard/@latest)), has a
    `tun/netstack` package on gVisor's TCP/IP stack, so one process can
    speak WireGuard without an OS TUN device
    ([netstack](https://github.com/WireGuard/wireguard-go/blob/master/tun/netstack/tun.go)).
  - `boringtun` 0.7.1, 1 May 2026, BSD-3-Clause; its README warns it
    "is currently undergoing a restructuring"
    ([boringtun](https://github.com/cloudflare/boringtun)).
  - WireGuard brings keys and tunnels, not discovery or NAT traversal,
    which ParleyInn would add (inference).
- **Tailscale `tsnet` (Go).** "Package tsnet embeds a Tailscale node
  directly into a Go program"; "ControlURL optionally specifies the
  coordination server URL"
  ([tsnet](https://pkg.go.dev/tailscale.com/tsnet)). `tailscale.com`
  v1.104.0, 30 September 2026, BSD-3-Clause
  ([proxy](https://proxy.golang.org/tailscale.com/@latest),
  [LICENSE](https://github.com/tailscale/tailscale/blob/main/LICENSE)).
  - The default control server is Tailscale's service, which CON-06
    rules out.
  - Headscale v0.29.4 (23 September 2026, BSD-3-Clause) is a
    self-hosted control server
    ([proxy](https://proxy.golang.org/github.com/juanfont/headscale/@latest)),
    but it adds a coordination server the owner must keep up (inference).

### Log replication and CRDTs

ParleyInn replicates sealed ranges of per-writer, signed, hash-chained logs
(PEER-03), and a conflict-free method for live co-editing waits for an
ADR (NG9, [§1](../../../docs/srs/01-introduction.md)). Signed
append-only logs are the closer prior art; CRDTs matter later.

- **p2panda (Rust).** `p2panda-core`, `-net` and `-sync` 0.7.1,
  21 August 2026, MIT or Apache-2.0
  ([crates.io](https://crates.io/crates/p2panda-sync)).
  - It builds on iroh and uses "BLAKE3, Ed25519, CBOR, TLS, QUIC";
    "the APIs are not yet considered stable"; `p2panda-sync` is
    "Local-first sync for append-only logs"
    ([p2panda](https://github.com/p2panda/p2panda)).
  - Experimental FFI covers "Node.js, Python and Go support via
    UniFFI".
  - BLAKE3 is not ParleyInn's SHA-256, so formats do not carry over, only
    designs (inference).
- **Willow (Rust).** `willow25` 0.7.9 (27 August 2026) and
  `willow-data-model` 0.8.0, MIT or Apache-2.0
  ([crates.io](https://crates.io/crates/willow25)), on Codeberg; its
  tagline promises "destructive edits"
  ([willow_rs](https://codeberg.org/worm-blossom/willow_rs)). Pruning by
  overwrite conflicts with I1 (inference).
- **Earthstar (TypeScript).** "Earthstar v11 is now in beta", with "a
  new specification powered by Willow"
  ([Earthstar](https://earthstar-project.org/)). The npm `earthstar`
  10.2.2 (31 August 2023) is LGPL-3.0-only, off ENG-18's list
  ([npm](https://www.npmjs.com/package/earthstar)).
- **Hypercore and Autobase (JavaScript).** Signed append-only logs
  and multi-writer linearising, as listed above.
- **Automerge.**
  - Rust `automerge` 0.12.0 and JavaScript `@automerge/automerge`
    3.5.0, 16 September 2026, MIT
    ([crates.io](https://crates.io/crates/automerge),
    [npm](https://www.npmjs.com/package/@automerge/automerge)).
  - Java `org.automerge:automerge` 0.0.9, April 2026
    ([Central](https://central.sonatype.com/artifact/org.automerge/automerge)),
    "implemented by
    wrapping the Rust automerge implementation", with a Gradle setup
    for Android ([automerge-java](https://github.com/automerge/automerge-java)).
  - Go `automerge-go` "uses cgo"; its last pseudo-version is from
    30 October 2024 and no `LICENSE` file sits at the repository root
    ([automerge-go](https://github.com/automerge/automerge-go)). It
    fails CON-02.
- **Loro.** Rust `loro` 1.16.2 and JavaScript `loro-crdt` 1.16.4,
  September 2026, MIT ([crates.io](https://crates.io/crates/loro),
  [npm](https://www.npmjs.com/package/loro-crdt)).
  Swift, Python and React Native bindings are official; Go's
  `loro-go` is community-run
  ([loro-ffi](https://github.com/loro-dev/loro-ffi)).
- **Yjs and yrs.** `yjs` 13.6.33 and `yrs` 0.28.0, September 2026, MIT
  ([crates.io](https://crates.io/crates/yrs),
  [npm](https://www.npmjs.com/package/yjs)). Bindings: a C FFI, WASM,
  Python, Ruby, Swift and Kotlin; none for Go
  ([y-crdt](https://github.com/y-crdt/y-crdt)).
- **Go gap.** No maintained, cgo-free CRDT exists for Go among these. A
  Go core would reach one through a C FFI or WASM, as ncruces runs
  SQLite on wazero (inference; feasibility unverified).

## 6. Mobile reach for device-key signing

A device key must be certified with a scope (PRV-10,
[§5](../../../docs/srs/05-functional-requirements.md)), and "A paired
phone MUST be limited to allowing or denying held permission requests"
(OWN-17). The phone writes "signed allow or deny events in the phone's
own segments" ([§6 register, row 17](../../../docs/srs/06-security.md)).
SEC-10 wants a platform key store where one exists.

### Hardware key stores

- **iOS.** CryptoKit's `SecureEnclave` lists `P256`, `MLKEM768`,
  `MLKEM1024`, `MLDSA65` and `MLDSA87`, and no Curve25519 type
  ([SecureEnclave](https://developer.apple.com/documentation/cryptokit/secureenclave),
  read from the page's JSON). An Ed25519 key on an iPhone is a
  software key, which the Keychain can hold (unverified).
- **Android.** KeyMint TEE devices "must support curve 25519 for
  Purpose::SIGN (Ed25519, as specified in RFC 8032)", with "a message
  size limit of 16 KiB"; "STRONGBOX IKeyMintDevices do not support
  curve 25519"
  ([IKeyMintDevice.aidl](https://android.googlesource.com/platform/hardware/interfaces/+/refs/heads/main/security/keymint/aidl/android/hardware/security/keymint/IKeyMintDevice.aidl)).
  Which Android release first exposes it to apps is unverified.
- **Consequence.** An Ed25519-only chain cannot bind an iPhone's
  device key to hardware. PRV-10 either admits P-256 (or ML-DSA)
  device keys or accepts software keys on iOS. Signing a SHA-256
  digest fits Android's 16 KiB limit (inference).

### Shared cores on the phone

- **gomobile (Go).** "The Go Mobile project is experimental. Use this
  at your own risk."; "neither Google nor the Go team can provide
  end-user support" ([golang/mobile](https://github.com/golang/mobile)).
  Latest pseudo-version 8 September 2026, BSD-3-Clause
  ([proxy](https://proxy.golang.org/golang.org/x/mobile/@latest)).
- **UniFFI (Rust).** "used extensively by Mozilla in Firefox mobile and
  desktop browsers"; Kotlin, Swift, Python and Ruby; "ready for
  production use, but UniFFI is a long way from a 1.0 release"
  ([uniffi-rs](https://github.com/mozilla/uniffi-rs)). `uniffi`
  0.32.2, 23 September 2026, is MPL-2.0
  ([crates.io](https://crates.io/crates/uniffi)), off ENG-18's list.
  Third-party generators add Kotlin Multiplatform (Gobley), Go and
  React Native.
- **Kotlin Multiplatform.** Since Kotlin 1.9.20 (November 2023) it
  "has become Stable and is now 100% ready for use in production";
  Compose for iOS was then Alpha
  ([JetBrains](https://blog.jetbrains.com/kotlin/2023/11/kotlin-multiplatform-stable/)).
  `cryptography-kotlin` 0.6.0 (April 2026) offers multiplatform
  crypto. Android is native ground; iOS goes through Kotlin/Native.
- **TypeScript on phones.**
  - Capacitor 8.5.2 (11 September 2026, MIT,
    [npm](https://www.npmjs.com/package/@capacitor/core)) runs in the
    system WebView. WebCrypto Ed25519 arrived in Safari 17, Chrome 137
    and Firefox 129, and Android's WebView mirrors Chrome
    ([MDN data](https://github.com/mdn/browser-compat-data/blob/main/api/SubtleCrypto.json)).
  - React Native 0.87.1 (26 August 2026,
    [npm](https://www.npmjs.com/package/react-native)) has no built-in
    WebCrypto (unverified); `react-native-quick-crypto` 1.1.7 (MIT,
    [npm](https://www.npmjs.com/package/react-native-quick-crypto))
    fills the gap; its coverage table lists Ed25519
    ([coverage](https://github.com/margelo/react-native-quick-crypto/blob/main/.docs/implementation-coverage.md)).
  - Holepunch's Bare runs JavaScript in iOS and Android apps through
    bare-kit worklets ([bare-kit](https://github.com/holepunchto/bare-kit)).
- **Zig.** Cross-compiles C-ABI libraries; mobile toolchains and
  bindings are unverified.

## 7. Fitting into one binary

CON-02 wants the core as "a single statically linked binary
(`CGO_ENABLED=0`)"; packaging the other components is open (OQ-32),
and SEC-01 wants build-time evidence per component that nothing it can
execute reaches beyond its boundary
([§2](../../../docs/srs/02-context.md),
[§6](../../../docs/srs/06-security.md)).

- **Go.** A static binary is the default with cgo off. The import
  closure per entry point is the per-component evidence, already
  tested in [imports_test.go](../../../cmd/parinn/imports_test.go).
- **Rust.** A static binary is routine on musl targets; `rusqlite`
  with `bundled` adds a C toolchain per target. Cargo features and
  `cargo tree` per crate give evidence, plus lints for `std::net` and
  `std::process` (inference).
- **Bun.**
  - `bun build --compile` targets Linux x64 and arm64 (glibc and
    musl), Windows and macOS, and "You can embed `.node` files into
    executables" ([executables](https://bun.com/docs/bundler/executables)).
  - Tested: a script using `bun:sqlite` and WebCrypto compiled to a
    94.6 MB executable, dynamically linked to glibc.
  - `fetch`, `Bun.listen` and `Bun.spawn` are globals, so no import
    graph shows a component's reach (inference). Bun's docs index
    lists no permission model (unverified absence).
  - Anthropic acquired Bun on 3 December 2025; "Bun will remain open
    source and MIT-licensed", and it "directly drove the recent launch
    of Claude Code's native installer"
    ([Anthropic](https://www.anthropic.com/news/anthropic-acquires-bun-as-claude-code-reaches-usd1b-milestone)).
- **Node and Deno.** Node's permission model is "Stability: 2 -
  Stable" with `--allow-net` and `--allow-child-process`
  ([permissions](https://nodejs.org/api/permissions.html)); its single
  executable applications are "1.1 - Active development"
  ([SEA](https://nodejs.org/api/single-executable-applications.html)).
  Deno gates the network behind `--allow-net`
  ([Deno security](https://docs.deno.com/runtime/fundamentals/security/)).
  Both give a runtime sandbox, not build-time evidence (inference).
- **Zig.** Cross-compiles static binaries well, but the language is
  pre-1.0 at 0.17.0
  ([downloads](https://ziglang.org/download/index.json)).
- **Kotlin.** Runs on the JVM by default; one native executable needs
  GraalVM Native Image or Kotlin/Native, and whether the JNI SQLite
  driver and the ACP and MCP SDKs survive that is unverified.

## Matrix

Ratings: strong (maintained, fits the constraint as is), workable
(usable with own code or a caveat), weak (experimental, stale or a
poor fit), none (nothing found).

| Slot                        | Go                                                                 | Rust                                            | TS/Bun                                                            | Zig                                   | Kotlin                                                 |
| --------------------------- | ------------------------------------------------------------------ | ----------------------------------------------- | ----------------------------------------------------------------- | ------------------------------------- | ------------------------------------------------------ |
| Claude Code hosting         | workable: stream-json; `severity1/claude-agent-sdk-go` (community) | workable: stream-json; ports thin               | strong: official Agent SDK, but Commercial Terms (off ENG-18)     | weak: stream-json by hand             | weak: stream-json by hand                              |
| ACP client                  | workable: `coder/acp-go-sdk` (community)                           | strong: official `agent-client-protocol` 2.2    | strong: official `@agentclientprotocol/sdk` 1.7                   | weak: JSON-RPC by hand                | strong: official `acp` 0.30 (JVM only)                 |
| Codex app-server client     | workable: types from JSON Schema                                   | strong: Codex's own Rust types (git dependency) | strong: `@openai/codex-sdk`, `generate-ts`                        | weak: by hand                         | workable: types from JSON Schema                       |
| MCP server in the core (B0) | workable: Tier 1 SDK pulls `net/http`; own stdio server            | strong: `rmcp` 3.5 with HTTP features off       | strong: reference SDK 1.32; no static reach evidence              | weak: community only                  | workable: Tier 3 `kotlin-sdk` 0.15                     |
| SQLite with FTS5            | strong: `ncruces/go-sqlite3` (chosen)                              | strong: `rusqlite` bundled, FTS5 on             | strong: `bun:sqlite` (tested); `node:sqlite` RC                   | workable: `zig-sqlite`, unstable API  | workable: `sqlite-jdbc` (JNI), SQLDelight              |
| JCS (RFC 8785)              | workable: `gowebpki/jcs` or own                                    | strong: `serde_json_canonicalizer`              | strong: `canonicalize` (RFC co-author)                            | none: own code                        | workable: `java-json-canonicalization` (2018)          |
| Ed25519 and SHA-256         | strong: standard library                                           | strong: `ed25519-dalek` 3, `sha2`               | strong: WebCrypto (tested), `@noble/*`                            | strong: `std.crypto`                  | strong: JDK 15 EdDSA                                   |
| Peer transport (B2)         | strong: `quic-go`, `go-libp2p`, `flynn/noise`, `wireguard-go`      | strong: `iroh` 1.3, `quinn`, `snow`, `libp2p`   | workable: `js-libp2p`, Hyperswarm, `@number0/iroh`                | none                                  | workable: iroh-ffi Kotlin; `jvm-libp2p` off Central    |
| Log sync and CRDT           | weak: no cgo-free CRDT; own sealed-range sync                      | strong: p2panda, Automerge, Loro, yrs           | strong: Hypercore, Autobase, Automerge, Yjs, Loro                 | none                                  | workable: `automerge-java`, `ykt`                      |
| Phone signing core          | weak: gomobile experimental                                        | workable: UniFFI, but MPL-2.0                   | workable: Capacitor or React Native with Ed25519                  | weak: C ABI only                      | strong: Kotlin Multiplatform; Android Keystore Ed25519 |
| One binary and SEC-01 proof | strong: static, import-closure test exists                         | strong: static; features and lints              | workable: `--compile` 95 MB; runtime globals, no build-time proof | strong: static cross-compile; pre-1.0 | weak: JVM; native image unverified                     |

## What this means for the language choice

- **Harness hosting does not decide the language.** Every path is JSON
  over stdio, and the run component is its own process. TypeScript's
  official Agent SDK is the one advantage, and its Commercial Terms
  keep it outside ENG-18's allow-list in any language. ParleyInn could
  still drive Claude Code through stream-json alone.
- **The core slots favour Go and Rust.** SQLite with FTS5, JCS,
  Ed25519 and SHA-256, and a stdio MCP server are strong in both. Go
  gives the cleanest SEC-01 evidence, but it must hand-write its MCP
  stdio server. Rust can use `rmcp` with HTTP features off and needs
  lints for `std::net`.
- **Peer sync favours Rust, with Go workable.** Iroh 1.x, p2panda and
  Willow are Rust-first, and iroh-ffi skips Go. Go has the
  transports (QUIC, libp2p, Noise, WireGuard) for a custom sealed-range
  exchange, which PEER-03 needs in any language. The later CRDT
  choice (NG9) is Rust or JavaScript territory; a Go core would reach
  it by FFI or WASM.
- **Mobile does not decide the core language.** No language gives a
  hardware-held Ed25519 key on an iPhone. Kotlin is strongest on
  Android, UniFFI covers both platforms under MPL-2.0, and gomobile is
  experimental. The phone's scope is small (allow or deny events), so
  a native Swift and Kotlin signer that reimplements JCS, SHA-256 and
  sealing may beat sharing the core (inference).
- **Zig is out.** Only its standard-library crypto is strong; it has
  no MCP, ACP, JCS or peer libraries, and the language is pre-1.0.
- **Kotlin fits phones, not the core.** It has the official ACP SDK
  and the best Android story, but Tier 3 MCP and no static binary
  without native-image work.
- **TypeScript on Bun is the richest ecosystem** for harnesses, CRDTs
  and Hypercore. It cannot show per-component reach at build time, as
  SEC-01 requires, and its executable is ~95 MB.
- **Net.** Go stays viable for the core as CON-02 and ADR-07 assume.
  Rust is the strongest alternative, and the gap widens if B2 peer
  sync and CRDTs weigh more than the existing Go gates.
