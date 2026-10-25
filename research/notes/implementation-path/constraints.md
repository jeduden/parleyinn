# Implementation path: the evaluation frame

Scope: the fixed frame every implementation option for OQ-32 must
answer: language, runtime, storage engine, UI delivery and packaging.
It reads SRS 2.0-draft of 3 October 2026 and the stakeholder's
direction of 4 October 2026: pitch v12 decisions 8, 9 and 11, the
surfaces and onboarding UX, the entity model, the concepts and the
room protocol of plan 2610012322. The SRS is normative; where the pitch
asks for more than the SRS states (the three-pane room UI, the
running-app preview, marks and links), this note says so and lists it
under open points. The pitch's "room" is the SRS's "lane" (concepts
Q29); this note uses "lane" where it cites the SRS. It recommends no
option. Step 2 compares options against it.

Sources: [§2 context][ctx], [§4 architecture][arch], [§5 REC to
OPS][fr], [§5 LANE and VIEW][lane], [§5 OWN and PEER][own],
[§6 security][sec], [§7 NFR][nfr], [§8 data][data],
[§9 interfaces][ifc], [§9.7 vocabulary][voc], [§10 ENG][eng],
[§12 delivery][dlv], [§13 open questions][oq], the [pitch][pitch], the
[surfaces UX][ux], the [entity model][ent], the [concepts][cpt], the
[room protocol][rp] and the [ADRs][adr-dir].

## Slots

Every option fills the same six slots. A slot is a role with a
boundary, not a packaging unit (glossary "Core", SEC-01): several slots
may share one executable, but each process runs exactly one of them.

| Slot                               | Boundary         | Register rows   | Priority                      | Started by                                 |
| ---------------------------------- | ---------------- | --------------- | ----------------------------- | ------------------------------------------ |
| Core                               | B0               | 1–7             | P0; rows 3, 5, 6 P1; row 7 P2 | the harness (hooks, MCP); the person (CLI) |
| Lane-view or room server           | B1               | 8, 9, 11        | P1                            | the person: `parinn ui`                    |
| Run component                      | B1               | 10, 25, 26      | P1                            | the person: `parinn run -- <harness>`      |
| Clients                            | through B1 or B0 | 4, 5, 9, 11, 17 | P1; row 17 P2                 | the person's browser, terminal or phone    |
| Peer (B2), publish and bridge (B3) | B2, B3           | 12–23, 27       | P2                            | the person: `parinn peer on`, owner acts   |
| Packaging                          | all              | all             | P0                            | the release pipeline and the plugin        |

### Core (B0)

What the SRS requires of it:

- **Hooks** (row 1, [§9.1][ifc]): one JSON object on stdin, at most one
  on stdout. Budgets at p95 on 1M events, process start included:
  `UserPromptSubmit` ≤ 50 ms, `SessionStart` ≤ 150 ms, `PostToolUse`,
  `Stop`, `SubagentStop`, `Notification` ≤ 100 ms, `PreCompact` ≤ 2 s,
  `SessionEnd` ≤ 1 s, held `PermissionRequest` ≤ 50 ms beyond the hold.
  They hold with the lane view open and ten harnesses writing (NFR-01).
  Internal deadline and work marker (NFR-02); fail open except I2, I4,
  I8 (NFR-06); input limits (SEC-05); schema validation (SEC-16); path
  confinement (SEC-18); incremental ingestion (REC-13); a seal after
  every appending hook (REC-19); a worktree checkpoint at `Stop` and
  `SessionEnd` (REC-20). Peak RSS ≤ 50 MiB per hook and no resident
  process between sessions (NFR-09, ADR-01).
- **MCP server** (row 2): stdio, the tools of [§9.2][ifc] (RCL-01), the
  envelope of §9.3 with content as JSON strings (SEC-06), `search`
  ≤ 200 ms and `expand` ≤ 100 ms p95 at 10M events (NFR-03). Spike S7
  decides SDK or in-house server.
- **Kernel worker** (row 3): the only program the core starts. A
  hermetic Starlark interpreter by default (ADR-04, unproven until
  spike S5 answers OQ-10; a WASM sandbox is the language-neutral
  alternative, see criterion 12) with allow-listed
  globals (CMP-03), no filesystem, network, process or environment
  (CMP-04), step budget and OS limits of 10 s and 512 MiB (CMP-05),
  enveloped and tainted output (CMP-06).
- **CLI and TUI** (rows 4–6): every verb of [§9.5][ifc] with `--json`
  and exit codes 0–3; owner-act verbs refuse without a terminal
  (OWN-12). `parinn lanes` is a full-screen TUI that reads the store
  directly and opens no socket, with the vocabulary and keymap of
  [§9.7][voc] (VIEW-14). Status line (VIEW-15) and terminal signals
  (VIEW-17) never reach the model.
- **Store**: append-only, one log per writer, sealed segments, payloads
  apart; formats and engine not decided (CON-05). Addresses (writer,
  seq), gap-free (REC-06); atomic payload writes (REC-09); a full-text
  index (REC-11 names SQLite FTS5); versioned segments (REC-21); at most
  48 segments per writer per day (REC-25). ≥ 5,000 events/s on one core
  (NFR-04); 10M events, 100 GiB payloads, 50 concurrent writers
  (NFR-05); crash consistency (NFR-07, ENG-06); no corruption under
  concurrency (NFR-08). Migrations, backup, purge, rebuild and verify
  (ADM-05 to ADM-09); secure delete (SEC-14 names SQLite
  `secure_delete`); encryption at rest (SEC-09, OQ-04); quotas
  (ADM-15); `0700` and `0600` (SEC-02).
- **Canonical encoding and commitments**: RFC 8785 with SHA-256 for the
  hash and audit chains ([§8.3][data]); a per-writer chain over a
  header (REC-10); a hiding, binding commitment under a per-event key
  of ≥ 256 bits, erased on purge (REC-17, SEC-31); a byte-identical
  restore block (INJ-06).
- **Signing and keys**: seals over writer id, seq and head (REC-18, P1,
  P0 with PEER); a node-bound writer key (REC-24); keys in a platform
  key store reached with no socket and no program start, else a `0600`
  file (SEC-10); rotation and revocation (SEC-27); receipts checkable
  offline (VIEW-10); owner, device and writer chain (PRV-10, P2).
- **Git reading**: root commit, HEAD, refs, trees and patch-ids inside
  the core's boundary, with no program start and no network (ASM-20,
  S11, LANE-02, LANE-06, REC-20); offline verification of commit
  signatures (LANE-15, P2); never a git write (ADM-13).
- **Trust and redaction**: gitleaks-compatible redaction before storage
  (SEC-08); literal-term query compiler (SEC-04); `TrustedText` only in
  restore blocks (INJ-03, SEC-07); audit chain and counters (OPS-01,
  OPS-02); strict layered TOML with managed policy at a system path
  (ADM-04).

### Lane-view or room server (B1)

What the SRS requires of it:

- **Reach** (row 8): listens on loopback only, on a port chosen at
  launch; no outbound; off by default; managed policy can lock it
  (SEC-22).
- **Credential** (SEC-20): minted per launch, ≥ 128 bits, never in a
  request line, argv, an environment another UID can read, a log or a
  referrer; exchanged once for a session credential only its own
  origin, port included, can read or send. `Host` and `Origin` checked,
  no cross-origin request, every rejection audited (T15).
- **Rendering** (SEC-21, T16): record content as inert text; a
  Content-Security-Policy forbidding any resource from another origin;
  untrusted content marked (VIEW-06, VIEW-07).
- **No configuration writes** (SEC-23): it shows the diff and the CLI
  command (VIEW-18).
- **Optional client** (VIEW-03, ADR-01, NG5): it reads through the
  core's read path, read-only; every capability also exists in the CLI
  or MCP; no hook, ingestion or recall path depends on it.
- **Surfaces** (VIEW-01 to VIEW-21): Fleet, the lane with every harness
  live, Needs you, Catch up, search, replay, integrity seals, intent
  and verdicts, delegations, token use.
- **Latency** (VIEW-02, NFR-15): a transcript line on screen within
  2 s; an ingested event within 1 s; Catch up paints within 1 s;
  search shows first results within 300 ms; a hit opens in context
  within 150 ms; replay steps within 50 ms and rebuilds a worktree
  within 500 ms, at 10M events across 50 lanes. A miss is shown on
  screen.
- **Owner acts** (OWN-02, OWN-05, OWN-11, OWN-16): it writes `operator`
  events as an authenticated surface. It raises presence checks bound
  to its exact origin, port included, with attestation chaining to a
  trust root ParleyInn ships. It enforces credential scopes on the server,
  the phone's included. An answer on any surface clears every other.
- **Budgets and failures**: peak RSS ≤ 256 MiB and ≤ 5% of one core
  idle (NFR-09); every failure audited and counted, never only in a
  browser (OPS-06); status in `parinn status` (ADM-16).
- **Beyond the SRS** (pitch decisions 8, 9, 11; [entity model][ent]):
  rooms ranked on the left, one conversation with every agent in the
  middle, the live outcome on the right (running-app preview, diff,
  test runs, an agent's file edits followed live), and marks on a line,
  a sentence or an image region that yield a link. No VIEW requirement
  covers the outcome window, presentations, snapshots or marks yet.

### Run component (B1)

What the SRS requires of it:

- **Reach** (row 10, SEC-29): the only component that starts programs
  (the core's kernel worker excepted); hosts their terminals; reaches
  the lane view only over loopback or a local endpoint only the same
  user can reach, refusing a peer of another UID; no outbound; off by
  default; its own register row and threat review. Each program it
  starts has its own row: the harness (row 25, adding no reach) and the
  witness-run command (row 26).
- **Structured hosting**: steering, replies and endorsements reach an
  agent only through the harness's own input interface: the Agent SDK,
  ACP, the Codex app-server or the hosted terminal (OWN-03, ASM-19,
  S10). A steer names its turn (OWN-13); no surface shows a stop before
  the harness acknowledges it (OWN-14); a control an adapter lacks is
  shown as unavailable (OWN-15); `parinn install` can route every launch
  through the run component (OWN-15).
- **Starting sessions**: new delegate sessions (OWN-23) and retries from
  a checkpoint (OWN-28) start here, never in the core; the launcher
  records each session's sandbox state (OWN-22); a launch with no hook
  event shows as `unrecorded` (VIEW-04).
- **Terminal hosting** (OWN-19): pty hosting and takeover live only
  here, on the harness's machine; input at a no-echo prompt is never
  stored; no takeover across machines before PEER (X3).
- **Witness runs** (OWN-18, LANE-05): a fresh checkout of the exact
  commit, network and the user's home denied by default, or the view
  says the platform cannot deny them.
- **Commands** (OWN-18, T24): only commands the person typed or
  confirmed, with hidden characters shown.
- **Budgets** (NFR-09): ≤ 10 ms p95 added keystroke-to-echo; peak RSS
  ≤ 256 MiB and ≤ 5% idle CPU in total for ten concurrent instances.
- **Why it matters**: no terminal multiplexer is multiplayer; hosting
  the harness is what lets a live collaborator (U6) see and take over
  a session, so the stakeholder's multiplayer rests on this slot.

### Clients

What the SRS requires of each:

- **Browser lane view** (decision 9, rows 8 and 9): the full experience
  on a loopback page; the keymap of [§9.7.7][voc]; desktop
  notifications raised only by the open page, carrying class, alias
  and count (VIEW-17); presence checks with an authenticator on the
  same device (row 9, OWN-11).
- **TUI and CLI** (row 4): "works in the terminal too" means the B0 TUI,
  same words, marks, ids and Needs you order (VIEW-14). A reduced
  surface says what it left out and where to see it.
- **Phone, stage one** (row 11, P1): an owner-run tunnel to the loopback
  lane view; `parinn ui --device phone` mints a credential that can only
  read, allow once and deny (OWN-16), by a widening act.
- **Phone, stage two** (row 17, P2): a paired device writing signed
  allow and deny events in its own segments over B2 (OWN-17, PRV-10).
  A phone view served over B2 is rejected (X2); Web Push and vendor
  relays are rejected (X4).
- **Harness strip** (rows 5 and 6): status line, banners and cleared
  prompts for the human, never the model (VIEW-15).
- **Desktop or editor shells**: optional, with no register row today.
  Any shell is a client of the B1 lane view, never on a hook path
  (VIEW-03), and keeps SEC-20's credential rules.

### Peer (B2), publish and bridge (B3)

P2 (NG4): not built now, but "specified now so v1 does not preclude it"
([index][srs]). An option must show its language reaches these needs:

- **Peer** (PEER-01 to PEER-12, rows 12–17, 23, 27): its own component,
  started only by a tenant action, never by a core process (PEER-01).
  Encrypted, mutually authenticated transport keyed by enrolled keys;
  listeners on named addresses only, never a wildcard (SEC-24).
  Sandboxes dial out, with no relay host (X7, CON-06). Local discovery
  with a random per-boot id (row 13). Only sealed ranges cross, checked
  before import (SEC-25). Byte-identical convergence in any order
  (PEER-03); 5 s propagation (PEER-04); equivocation kept as evidence
  (PEER-10); blind peers storing ranges encrypted to members' keys
  (PEER-12); a conflict-free merge chosen by ADR (LANE-09, row 23).
- **Publish** (rows 18, 19): read-only signed bundles (SEC-26) and the
  git carrier on the owner's remote (PEER-08). Program start is the run
  component's alone (ENG-16), so the carrier speaks git's protocol in
  process.
- **Bridge** (rows 20–22, SEC-28): forge read, CI attestations,
  notifications carrying alias, class and count; outbound only, per
  destination, imports untrusted.

### Packaging

- One single, statically linked binary per target for the core
  (CON-02, NFR-10): linux/amd64, linux/arm64, darwin/arm64.
- Distribution as a Claude Code plugin with the static binary (ADM-01,
  ADR-06); a working setup in ≤ 5 minutes with zero configuration
  (NFR-12).
- Reproducible, signed, attested release (ENG-19, ENG-20); prototype
  code absent from release artifacts (ENG-29).
- The stakeholder wants one binary for every slot; the
  [packaging section](#packaging-and-one-binary) sets what that binary
  must still guarantee.

## Hard constraints

Musts every option satisfies, with how an option could prove each.
"Go", "Rust" and "TS" name the three families step 2 is likely to
weigh; TS means TypeScript compiled into one executable with Bun or
Deno. A proof marked "all" is language-neutral.

| #     | Ids                                          | Constraint                                                                                                                                                                           | How an option proves it                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ----- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| HC-1  | SEC-01, CON-04, I4                           | The core opens no socket and connects nowhere                                                                                                                                        | Go: per-component import closure (today's [imports_test.go][imports] and [depguard][lint]) plus a call graph from each entry. Rust: one crate per component, clippy `disallowed-types` and `disallowed-methods` on `std::net`, a symbol check of the build. TS: `fetch` and `WebSocket` are globals, so an import closure proves nothing; it needs lint bans on globals and a runtime permission such as Deno's `--deny-net`. All: an ENG-12 run under a network-denied sandbox |
| HC-2  | SEC-01, X8                                   | One component per process, fixed at start; a core process never starts a B1–B3 component                                                                                             | All: a dispatch-table test (one subcommand, one component, chosen before any component code runs); reach evidence shows no program start in the core except `parinn kernel-worker`; `parinn open` and the TUI's `o` print, never start                                                                                                                                                                                                                                          |
| HC-3  | SEC-19, ENG-16, ENG-27                       | Build-time reach evidence per component, start-up code included; CI fails when missing or wider than the register, or when an unlisted process exists                                | Go: `init` of every linked package runs in every process, so the evidence covers them. Rust: no code before `main` except `.init_array` constructors; ban constructor crates. TS: every module in the loaded graph evaluates its top level, so each component loads its own graph. All: a drift case that adds a socket to the core and turns CI red                                                                                                                            |
| HC-4  | SEC-29, ENG-16, T24                          | Program start only in the run component (the kernel worker excepted)                                                                                                                 | HC-1's machinery for process APIs: Go `os/exec`, `syscall.ForkExec`; Rust `std::process::Command`, `posix_spawn`; TS `child_process`, `Bun.spawn`, `Deno.Command`. All: a sandbox that denies `execve` outside the run component in ENG-12                                                                                                                                                                                                                                      |
| HC-5  | SEC-20, SEC-21, SEC-23, T15, T16             | Loopback only; per-launch credential; a session credential only its origin and port can read or send; `Host` and `Origin` checks; same-origin CSP; inert rendering; no config writes | All: black-box HTTP tests (rebinding `Host`, cross-origin request, replay from another loopback port, the credential absent from `/proc/<pid>/cmdline` and environ); a golden CSP header; fuzzing the renderer with HTML and script. Cookies give no port isolation (RFC 6265 §8.5), so a cookie-only design fails this                                                                                                                                                         |
| HC-6  | ADR-01, NFR-09, VIEW-03, I9                  | No resident core process; hooks, MCP and recall never depend on a B1–B3 component                                                                                                    | All: no ParleyInn process left after a session ends; a scenario that kills the lane view mid-session with no hook degraded. Rules out a hook that calls a daemon                                                                                                                                                                                                                                                                                                                |
| HC-7  | NFR-01, NFR-02, NFR-09                       | Hook budgets with process start included, ≤ 50 MiB RSS, lane view open, ten harnesses writing                                                                                        | All: the ENG-15 benchmark on the reference hardware (2 vCPU, 4 GiB) running the shipped binary at 1M events, cold and warm. Runtime start-up cost decides it                                                                                                                                                                                                                                                                                                                    |
| HC-8  | I10, ENG-03, ENG-08, ADM-08, INJ-06, PEER-03 | Deterministic projections: no clock, no randomness, rebuild byte-identical in any arrival order                                                                                      | Go: a custom analyser banning `time.Now`, `math/rand`, `crypto/rand` and unsorted map ranges in projection packages. Rust: clippy bans on `SystemTime::now` and `HashMap` iteration. TS: lint bans on `Date.now`, `Math.random`, `crypto.getRandomValues`. All: the S12 property test, two writers in either order, run on linux and darwin                                                                                                                                     |
| HC-9  | INJ-03, SEC-07, I2                           | Only `TrustedText` reaches a restore block, constructible only in the injection module                                                                                               | Go: an unexported constructor and a static check. Rust: a private field and a `pub(crate)` constructor; the compiler enforces it. TS: types vanish at runtime and a cast forges one; needs a lint ban on casts and a module-private runtime brand                                                                                                                                                                                                                               |
| HC-10 | §8.3, REC-10, REC-17, REC-18                 | RFC 8785 canonical JSON, SHA-256 chains, hiding commitments, signed seals                                                                                                            | All: RFC 8785 test vectors, number formatting included; vectors shared with every client that verifies (browser, phone); the ENG-08 round-trip property                                                                                                                                                                                                                                                                                                                         |
| HC-11 | CON-02, CON-03, NFR-10, ENG-01               | One static binary per target; no runtime dependency on Python, Node.js or a database server                                                                                          | All: CI checks `ldd` and `otool -L` (only the system library on darwin) and runs the binary in an empty container. TS: whether a runtime embedded by `bun build --compile` or `deno compile` is a "runtime dependency" needs a ruling                                                                                                                                                                                                                                           |
| HC-12 | ENG-19, ENG-20, T11                          | Bit-identical artifacts from two builders; Sigstore keyless signature, SLSA Build L3 provenance, SPDX SBOM, signed checksums                                                         | All: [release.yml][release]'s two-builder comparison for all three targets, with any embedded web bundle built inside it; an SBOM that lists every ecosystem the binary carries                                                                                                                                                                                                                                                                                                 |
| HC-13 | ENG-18, ENG-26                               | Every direct dependency has an accepted ADR; licence on the allow-list (Apache-2.0, MIT, BSD, ISC); ≤ 10 direct as the target                                                        | All: the ENG-18 scenario, extended from `go.mod` to every manifest (`Cargo.toml`, `package.json`). Test-only and frontend packages count; three test-only Go modules count today                                                                                                                                                                                                                                                                                                |
| HC-14 | SEC-15, CON-06, X4, X5, X7                   | No telemetry, crash reporting, update check, central or vendor service                                                                                                               | All: inspection plus ENG-12 runs that fail on any socket; a runtime or framework that phones home by default is out unless off by construction                                                                                                                                                                                                                                                                                                                                  |
| HC-15 | SEC-02, SEC-03, I8                           | `0700` and `0600`; refuse a home or store file another UID owns                                                                                                                      | All: unit and scenario tests over `stat` and the umask                                                                                                                                                                                                                                                                                                                                                                                                                          |
| HC-16 | SEC-10                                       | Keys in a platform key store reached with no socket and no program start, else a `0600` file; other credentials loaded only by the component that uses them                          | All: a per-OS test. The macOS Keychain is a C framework, so Go needs cgo or a `dlopen` shim against CON-02's wording; Linux's Secret Service speaks D-Bus over a socket, out of the core's reach; a TPM device file or the `0600` file stays in reach                                                                                                                                                                                                                           |
| HC-17 | SEC-22, ADM-04                               | Managed policy disables each B1–B3 component and the run component per host; a disabled start is refused and audited                                                                 | All: one scenario per component subcommand, with a policy file the tenant can write                                                                                                                                                                                                                                                                                                                                                                                             |
| HC-18 | ENG-12                                       | The end-to-end suite runs each component's entry under its boundary's sandbox                                                                                                        | All: a harness driving each subcommand under a network namespace or seccomp on Linux and Seatbelt on macOS; loopback only for B1                                                                                                                                                                                                                                                                                                                                                |
| HC-19 | ENG-29                                       | Prototype code provably absent from release artifacts and tagged releases                                                                                                            | Go: build tags; Rust: cargo features; TS: bundle entry points. All: a release step that probes the artifact for prototype symbols or strings                                                                                                                                                                                                                                                                                                                                    |
| HC-20 | CMP-03, CMP-04, CMP-05                       | A hermetic kernel: allow-listed globals, no filesystem, network, process or environment; a child process under a step budget and OS limits                                           | All: an allow-list test of every global; limit tests; the worker applies its own limits before it reads input                                                                                                                                                                                                                                                                                                                                                                   |
| HC-21 | OPS-01, OPS-06, I6                           | Every failed, dropped, rejected or timed-out operation audited and counted, in every component, never only in a browser                                                              | All: a counter scenario per component; browser-side failures reported to the lane-view server, which audits them                                                                                                                                                                                                                                                                                                                                                                |
| HC-22 | SEC-08, T8                                   | gitleaks-compatible redaction before anything is written                                                                                                                             | All: gitleaks' rules and their test cases run through the option's regex engine. gitleaks rules are Go RE2 expressions: Go's `regexp` and Rust's `regex` match RE2 in linear time; a backtracking engine, such as JavaScript's, differs in syntax and invites ReDoS                                                                                                                                                                                                             |

### Requirements worded for Go or SQLite

These hold as written only for Go and SQLite. A language or engine
change rewords them, which is a stakeholder-approved SRS change
(ENG-21).

| Id or record                  | Wording today                                                    | Neutral restatement                                                                                                                   |
| ----------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| CON-01, [index][srs] header   | "Implementation language is Go"                                  | The language OQ-32 selects                                                                                                            |
| CON-02, ENG-01                | `CGO_ENABLED=0`, Go `toolchain` directive, `-trimpath`, VCS info | No dynamic dependency but the OS's system library; toolchain pinned in the repository; build paths stripped; source revision embedded |
| ENG-02                        | `cmd/parinn` plus `internal/` packages                           | One entry module plus private modules of one responsibility each, import directions declared and enforced                             |
| ENG-03, §4.2                  | `context.Context` with a deadline; package-level state           | Every blocking call takes a cancellable deadline; no mutable module-level state                                                       |
| ENG-04                        | Errors wrapped with `%w`; panics stop at entry points            | Errors keep their cause chain; typed per counter class; crashes stop at the hook and MCP entry points                                 |
| ENG-05, OPS-05                | `log/slog`                                                       | Structured logging through redaction                                                                                                  |
| ENG-07                        | Native Go fuzzing                                                | Coverage-guided fuzzing with a committed corpus                                                                                       |
| ENG-09                        | The race detector                                                | A data-race detector, or the language's static guarantee, plus the 50-writer soak                                                     |
| ENG-16                        | `go vet`, `staticcheck`, `gosec`, `errcheck`, `govulncheck`      | Vet, a static analyser, a security linter, an unchecked-error check and a reachability-aware vulnerability scan                       |
| INJ-03                        | "an unexported constructor"                                      | A constructor no code outside the injection module can call, compiler-enforced where the language allows                              |
| REC-11, SEC-04, §8.2, ADR-05  | SQLite FTS5, FTS5 syntax                                         | A BM25 full-text index; the query compiler escapes the engine's syntax                                                                |
| SEC-14                        | SQLite `secure_delete`                                           | Secure deletion and a rewrite of the index's files                                                                                    |
| ADR-04, ADR-07, S2, S7, OQ-03 | Starlark in pure Go, pure-Go SQLite, the MCP Go SDK, a CGO build | The same questions asked of the chosen language                                                                                       |
| Appendix B, §4.1 diagram      | "A Go test", "parinn core (Go library)"                          | Non-normative; follow the code                                                                                                        |

Repository records that assume Go: CLAUDE.md (Go, gofmt, scenario
bindings in `cmd/parinn/bdd_<section>_test.go`), [development
guide][dev], [DEPENDENCIES.md][deps] ("every direct Go dependency"),
[.golangci.yml][lint], [imports_test.go][imports], [release.yml][release]
and the gates under `internal/`. The ADRs split:

- [ADR-2609292234][adr-test] (godog, cucumber messages, testify):
  Go-only; superseded by a language change.
- ADR-2609302341 (ncruces/go-sqlite3, proposed): Go-only.
  Its measurements of FTS5 at 10M events carry over to any SQLite
  binding; the driver choice does not.
- [ADR-2609301941][adr-review] (agent review): language-neutral; its
  gate, `cmd/review-gate`, is Go tooling that can stay Go.
- ADR-2610032155 (SRS 2.0 invariants): language-neutral, and
  already written for one executable or several: "some runtimes run
  every linked module's start-up code in every process".

## Comparison criteria

Step 2 scores every option on these fifteen, in this order. Each names
the requirements and personas ([§2.5][ctx]) it traces to, then how to
measure it.

1. **Hook path under the full binary.** Traces: NFR-01, NFR-02,
   NFR-09, ADR-01; U2, U9, U1. Measure: p95 wall time and peak RSS of
   `parinn hook` for `UserPromptSubmit`, `SessionStart` (compact) and
   `PostToolUse` at 1M events on the reference hardware, from the one
   shipped binary, cold and warm, with the lane view open and ten
   writers; plus start-to-exit of a no-op subcommand.
2. **Boundary evidence inside one binary.** Traces: SEC-01, SEC-19,
   SEC-29, CON-04, ENG-12, ENG-16, ENG-27; U8. Measure: a prototype
   binary holding a core, a loopback server and a pty host. Does CI
   derive per-component reach evidence, start-up code included, from
   the shipped artifact, and does an injected socket in the core turn
   it red? Count the tools and the custom code it takes.
3. **Determinism and cryptographic primitives.** Traces: I10, §8.3,
   REC-10, REC-17, REC-18, SEC-27, ADM-08, PEER-03; U3, U4, U8.
   Measure: which of SHA-256, Ed25519, HMAC, an AEAD and RFC 8785 come
   from the standard library and which from dependencies; whether a
   lint can ban clock and randomness in projection code; an S12-style
   rebuild byte-identical across linux/amd64 and darwin/arm64.
4. **Store engine and performance.** Traces: CON-05, REC-11, NFR-03,
   NFR-04, NFR-05, NFR-07, NFR-08, SEC-09, SEC-14, OQ-03, OQ-04; U1,
   U4, U9. Measure: S2's harness at 10M events (ranked search and
   expand p95, ingest rate, store overhead ≤ 1.5×, 50 concurrent
   writers, the ENG-06 kill loop, encryption at rest) in a static,
   cross-built binary.
5. **Dependency budget and licences.** Traces: ENG-18, ENG-26, ENG-20,
   ENG-16, T11; U1, U8. Measure: the direct dependencies needed to fill
   every slot, counted for P0, P1 and P2 apart, test and frontend
   packages included; their licences; the transitive count; a
   reachability-aware vulnerability scanner.
6. **One reproducible, signed artifact.** Traces: ENG-01, ENG-19,
   ENG-20, CON-02, NFR-10; U1, U8. Measure: the prototype binary, web
   bundle embedded, built on ubuntu and macos builders for all three
   targets and compared byte for byte; build time; SBOM coverage of the
   embedded assets.
7. **Run component: hosting and pty.** Traces: SEC-29, OWN-03, OWN-13,
   OWN-14, OWN-15, OWN-18, OWN-19, NFR-09, ASM-19; U2, U6. Measure: one
   run process hosting ten harnesses, five over a pty and five over a
   stdio JSON protocol: added keystroke-to-echo p95 (≤ 10 ms), total
   RSS (≤ 256 MiB), idle CPU (≤ 5%); no-echo detection; a same-UID
   endpoint with a peer-credential check on linux and darwin;
   witness-run confinement per OS.
8. **Lane-view server and the three-pane page.** Traces: SEC-20,
   SEC-21, VIEW-01, VIEW-02, VIEW-08, VIEW-09, NFR-15, NFR-09, OWN-05,
   OWN-11; U2, U4, U5, U6. Measure: a prototype page with the live
   harness stream, a diff and a test-run pane: event-to-screen p95
   (≤ 1 s), first search results (≤ 300 ms), a replay step (≤ 50 ms),
   server RSS and idle CPU, while NFR-01 holds; a strict CSP with no
   inline script or `eval`; HC-5's tests pass.
9. **Terminal parity.** Traces: VIEW-14, register row 4, §9.7.7, §9.5,
   OWN-12; U2, U1. Measure: `parinn lanes` with the page's statuses,
   marks and keymap, built from the same projection code; start time;
   no socket; a link copied through the terminal's OSC 52 sequence,
   since a clipboard program would be a program start in B0.
10. **One view model across surfaces.** Traces: VIEW-04, VIEW-05,
    VIEW-14, §9.7, I10; U2, U4. Measure: how many places implement
    status derivation, the Needs you order and the vocabulary across
    page, TUI, CLI `--json` and phone. The target is one, in the core,
    sent to clients as data.
11. **Git and signatures inside the core's boundary.** Traces: ASM-20,
    S11, LANE-02, LANE-06, REC-20, LANE-15, ADM-13; U5, U7. Measure:
    read root commit, refs and trees, and compute a worktree diff and
    patch-id in process within the `Stop` budget on a large repository;
    verify SSH and OpenPGP commit signatures offline; the dependency
    cost.
12. **Kernel and MCP.** Traces: ADR-04, CMP-03 to CMP-07, S5, S7,
    RCL-01, §9.2, NFR-03; U9. Measure: a hermetic, metered sandbox
    with an allow-list of every global, whatever the guest language:
    a Starlark interpreter, or a WASM runtime running Python, Starlark
    or JavaScript, where a module given no imports has no filesystem,
    network or process; how steps are budgeted (an interpreter's step
    count, a runtime's fuel, or a wall-clock limit plus the worker's OS
    limits); an MCP server over stdio with cancellation and structured
    content. The guest language is S5's question (OQ-10), not the host
    language's, so this criterion rates the host's sandbox and MCP
    server only.
13. **P2 reach without a rewrite.** Traces: PEER-01 to PEER-12, SEC-24,
    SEC-25, LANE-09, PEER-08, register row 17; U3, U6, U7. Measure: for
    each P2 need (mutually authenticated transport, dial-out, discovery,
    a CRDT, git's protocol in process, AEAD to member keys, a phone that
    writes its own segments), a library on the allow-list with no
    foreign-function bridge to another toolchain, or the cost of
    writing it.
14. **Verification toolchain.** Traces: ENG-06 to ENG-11, ENG-13,
    ENG-15, ENG-16, ENG-17, ENG-27, ENG-28; U8, U1. Measure: native
    fuzzing, race detection, per-package coverage, mutation testing, a
    benchmark regression gate, static analysers and a Gherkin binding;
    which gates under `internal/` and `cmd/review-gate` must be ported.
15. **Cost of the change.** Traces: CON-01, OQ-32, ENG-21, ENG-25,
    ENG-26, ENG-29; every persona. Measure: requirement rows to reword
    (the table above), ADRs to supersede, CI jobs and repository files
    to port; maintainers and reviewer agents fluent in the language
    (ENG-25 asks for two maintainers).

### Criteria added in revision 2

The stakeholder's direction after revision 1 added these; the
[comparison](comparison.md) rates them beside the fifteen.

- **0. The app on five platforms.** Traces: the pitch, decisions 8 and 9;
  CON-02, NFR-10, OWN-16, OWN-17, NG4; every persona. Measure: start
  ParleyInn on Linux, Windows, macOS, iOS and Android and see the person's
  rooms; whether a phone verifies, syncs and signs with the core's own
  code ([five platforms](five-platform-apps.md)).
- **16. Agent fluency.** Traces: ENG-21, ENG-25. Measure:
  first-attempt pass rate and iterations to green on the bake-off
  slice.
- **17. Compiler as reviewer.** Traces: ENG-11, ENG-16,
  HC-23. Measure: the classes of defect the compiler rejects that
  review or fuzzing would otherwise have to catch.
- **18. Iteration speed.** Traces: ENG-15, NFR-01.
  Measure: CPU and wall time per edit, build and test cycle at
  ParleyInn's size ([compile times](compile-times-bun.md)).
- **19. Toolchain stability.** Traces: ENG-01, ENG-20.
  Measure: breaking changes per release and the upgrade cost across
  five platform builds ([Zig](zig-option.md)).
- **20. Accessibility and text input.** Traces: no SRS row yet, a gap
  every option must close. Measure: a screen reader reads the room and
  its messages, and IME input and text selection work, on all five
  platforms.
- **HC-23, memory safety** (a hard constraint). Traces: I2, I8, I9,
  SEC-05, T8. Hostile-input and trust code cannot perform an
  out-of-bounds access, a use after free, a double free, an
  uninitialised read or a data race, by construction or by stopping
  before the access; every exception is listed with its fuzz target
  ([Zig note](zig-option.md)).

## Packaging and one binary

### What the SRS says

- **Core**: one statically linked binary for three targets (CON-02,
  NFR-10). The glossary calls the core "a role with a boundary, not a
  packaging unit".
- **Every other component**: "whether they ship in one executable or
  several is not decided (OQ-32), and that packaging MUST NOT change
  any boundary" (CON-02). SEC-01 repeats it: "Packaging (one executable
  or several) is not decided and MUST NOT change any boundary". So do
  the register's introduction ([§6.3][sec]), the note under
  [§9.5][ifc] ("SEC-01 holds either way") and PEER-01.
- **Per-process guarantee**: every process runs exactly one component,
  fixed when it starts, and a core process never starts a B1–B3
  component, in process or as a child (SEC-01, X8).
- **Per-component evidence**: build-time evidence of each component's
  reach, "dependencies and code that runs as the process starts
  included", whatever the packaging (SEC-01, SEC-19, ENG-16).
- **The trade, from the security review** (ADR-2610032155,
  change 3): "the shipped file contains network code, so the guarantee
  is a property of each process".
- **Distribution**: the plugin ships hooks, the MCP registration and
  the static binary (ADM-01); uninstall lists every artifact any
  component created (ADM-02).

One binary for every slot is therefore allowed. It moves the proof
from "this file has no network code" to "this process cannot reach
it".

### What a multi-call binary must still guarantee

1. **Dispatch first.** The subcommand picks exactly one component
   before any component code runs. No mode runs two, and no component
   switches later (SEC-01).
2. **Inert start-up code.** Start-up code of every linked module runs
   in every process: Go's `init`, Rust's constructors, a JS bundle's
   top-level evaluation. It opens no socket, starts no program and
   reads no credential, and the evidence shows it (ENG-16, SEC-10).
3. **Evidence from the shipped artifact.** Today's import-closure test
   checks the whole binary and fails the moment B1 code links. It
   becomes per-entry evidence over the artifact that ships, with an
   allow-list per component and its drift case (ENG-16, ENG-27). An
   evidence build that differs from the shipped build proves nothing
   about the shipped build.
4. **Sandboxed per subcommand.** ENG-12 runs each subcommand under its
   boundary's sandbox. An option may add confinement at process start
   (seccomp or Landlock on Linux, Seatbelt on macOS) as defence in
   depth; it does not replace the build-time evidence.
5. **The core starts no component.** The only program the core starts
   is its own kernel worker (row 3), which a multi-call binary runs as
   `parinn kernel-worker`. `parinn open` and the TUI print links; nothing
   in B0 starts `parinn ui` or `parinn run` (X8).
6. **Policy before code.** The managed-policy lock is checked at
   dispatch, before component code runs; a refused start is audited
   and counted (SEC-22, ADM-04).
7. **A fast hook path.** Embedded web assets and B1–B3 code cost the
   hook path nothing measurable: assets load lazily, no eager
   initialisation (NFR-01, NFR-09 ≤ 50 MiB per hook).
8. **Credentials stay with their component.** Shared configuration
   code never resolves another component's secret reference in a core
   process (SEC-10).
9. **One artifact, fully accounted.** One signature, one provenance and
   an SBOM covering every component and the embedded web bundle
   (ENG-19, ENG-20). A component still a prototype stays out of the
   release binary by build configuration (ENG-29).
10. **Version skew handled.** After an upgrade replaces the file, a
    long-running `parinn ui` or `parinn run` from the old version meets
    newer segments or schemas; it refuses them and says so (REC-21,
    ADM-05).
11. **Residual risk unchanged.** An unsandboxed agent able to run
    `parinn` can start `parinn ui` or `parinn run` (T21, R5); several
    executables on the PATH give it the same reach. Managed policy is
    the lock (SEC-22), and OWN-12 keeps a stopped component from
    lowering any confirmation.

### What one binary trades

- **For it**: one artifact to sign, verify and ship through the plugin;
  hooks, lane view and run always at one version; one dependency
  graph; no PATH lookup between components.
- **Against it**: the network code sits on every host, even where
  policy locks B1–B3 off, so a vulnerability in it ships everywhere and
  the vulnerability scan has to report reach per component. The
  evidence is per process, not per file. The binary is larger, which
  item 7 must show costs nothing.
- **Desktop and editor shells** cannot live inside the one binary as
  a native window without dynamic system libraries, against CON-02's
  static build. They are either the user's browser or a second,
  optional artifact.

## Open points

Positions every option takes in step 2:

1. **Evidence mechanism.** How the option derives per-component reach
   from the one shipped artifact: call graph, symbol analysis, runtime
   permissions, or per-component link units fused after evidence; and
   whether it adds self-confinement at process start.
2. **Who opens the browser.** `parinn ui` "opens the default browser" in
   [UX 7.5][ux], but starting `xdg-open` or `open` is a program start
   outside the run component (ENG-16), and the D-Bus portal is a
   socket. Print only (`--print` as the default), hand it to the run
   component, or add a register row.
3. **Session credential transport.** UX 7.5 proposes an `HttpOnly`,
   `SameSite=Strict` cookie, but cookies are shared across ports
   (RFC 6265 §8.5), so a dev server on another loopback port receives
   it, against SEC-20. The option says where the session credential
   lives (page memory, a header, the first WebSocket message) and how
   live updates authenticate, since `EventSource` sends no custom
   header.
4. **The running-app preview.** Pitch decision 11 ships it in the first
   release, yet no requirement covers it, SEC-21's CSP forbids any
   resource from another origin, and the entity model keeps a running
   app "out of the lane view's own origin". Snapshots taken by the run
   component (a headless browser is an external program and its own
   register row), a sandboxed frame with a SEC-21 change, or a separate
   tab.
5. **Marks and image-region links.** A link points at data, never at
   compute ([entity model][ent], rule 3), so marking a running app
   takes a snapshot first. Who takes it, where it is stored as an
   event, and how the TUI shows an image mark (VIEW-14's "say what it
   left out").
6. **Following edits live.** From the record at VIEW-02 and NFR-15's
   latencies, as VIEW-03 requires, or by watching the worktree, which
   VIEW-03 does not allow as written.
7. **The terminal stream path.** Browser to lane view to run component
   over a same-UID endpoint, never the run component as a second
   browser origin (SEC-20). Terminal state held server-side for late
   joiners and takeover, or bytes relayed; which browser terminal
   emulator, counted under ENG-18.
8. **The frontend toolchain.** Whether build-only tools (bundler,
   compiler) count toward ENG-18's ten; how the web bundle builds
   bit-identically on both builders; a framework that runs under a CSP
   with no inline script and no `eval`.
9. **Where the view model lives.** Statuses, the Needs you order and the
   vocabulary (VIEW-04, VIEW-05, §9.7) computed once in the core and
   sent to clients as data, or reimplemented per client.
10. **Embedded runtimes and CON-03.** Whether an executable that embeds
    a JS runtime, or hosts the Agent SDK as a library rather than
    driving the harness's own stdio protocol, counts as a runtime
    dependency on Node.js.
11. **The platform key store.** How SEC-10's store is reached on macOS
    without cgo or a foreign-function bridge, and which Linux store
    needs no socket.
12. **Presence checks.** WebAuthn takes a domain as its relying-party
    id, and an IP origin such as `127.0.0.1` is not one, so the lane
    view's origin is likely `localhost` (to verify in a spike).
    Attestation verification (CBOR, COSE, X.509 chains to a root ParleyInn
    ships, OWN-11) is in-house or a dependency.
13. **The phone at stage two.** Row 17 makes the phone a writer of
    signed segments under PRV-10: a native app, a web app with
    WebCrypto keys, or none; and how much core code (canonical
    encoding, seals) it shares.
14. **Desktop and editor shells.** None, the browser in app mode, or a
    separate optional artifact; how a shell receives the launch
    credential without argv (SEC-20) and whether presence checks work
    inside its webview.
15. **Scenario bindings and gates.** Port the godog bindings and the
    gates under `internal/`, or keep them in Go and drive the binary as
    a black box through the CLI and MCP, which VIEW-03 makes complete.
16. **The storage engine.** Keep SQLite with FTS5 (REC-11 and SEC-14
    assume it) or replace it; the segment format, with C2SP tlog-tiles
    among the candidates of [plan 2610022338][net].
17. **The git reader.** A library or in-house code for in-process git
    objects, diffs and patch-ids (ASM-20, S11), and for offline
    signature checks (LANE-15).
18. **The P2 libraries.** The candidates named so far (iroh, Hypercore,
    Willow; Automerge, Loro, Yjs in [plan 2610022338][net]) are Rust or
    JavaScript. Whether the language reaches them without a
    foreign-function bridge, or reimplements them.
19. **macOS signing.** Whether release binaries carry an Apple
    Developer ID signature, which changes bytes after the
    reproducibility check, beside the Sigstore signature ENG-20
    requires.

[ctx]: ../../../docs/srs/02-context.md
[arch]: ../../../docs/srs/04-reference-architecture.md
[fr]: ../../../docs/srs/05-functional-requirements.md
[lane]: ../../../docs/srs/05b-lane-requirements.md
[own]: ../../../docs/srs/05c-owner-and-peer-requirements.md
[sec]: ../../../docs/srs/06-security.md
[nfr]: ../../../docs/srs/07-non-functional-requirements.md
[data]: ../../../docs/srs/08-data-and-storage.md
[ifc]: ../../../docs/srs/09-interfaces.md
[voc]: ../../../docs/srs/09b-lane-vocabulary.md
[eng]: ../../../docs/srs/10-engineering-quality.md
[dlv]: ../../../docs/srs/12-delivery-plan.md
[oq]: ../../../docs/srs/13-open-questions-and-risks.md
[srs]: ../../../docs/srs/index.md
[pitch]: ../../../plan/2610012322_parinn-for-agent-fleets/pitch.md
[ux]: ../../../plan/2610012322_parinn-for-agent-fleets/ux/users-onboarding-surfaces.md
[ent]: ../../../plan/2610012322_parinn-for-agent-fleets/entity-model.md
[cpt]: ../../../plan/2610012322_parinn-for-agent-fleets/concepts.md
[rp]: ../../../plan/2610012322_parinn-for-agent-fleets/room-protocol.md
[net]: ../../../plan/2610022338_parinn-network-side/plan.md
[adr-dir]: ../../../docs/adr/
[adr-test]: ../../../docs/adr/ADR-2609292234-test-stack.md
[adr-review]: ../../../docs/adr/ADR-2609301941-agent-review.md
[dev]: ../../../docs/development.md
[deps]: ../../../DEPENDENCIES.md
[lint]: ../../../.golangci.yml
[imports]: ../../../cmd/parinn/imports_test.go
[release]: ../../../.github/workflows/release.yml
