---
summary: >-
  Every direct dependency, listed through the decision
  record that justifies it (ENG-18, ENG-26). The ENG-18 scenario fails
  the build when the manifests and the dependency ADRs disagree or a
  license is off the allow-list.
---
# Dependencies

SRS requirement ENG-18 asks for three things. Every direct dependency
is justified by an accepted decision record (ADR). Its license is
Apache-2.0, MIT, BSD or ISC. The direct dependencies linked into a
shipped executable stay at ten or fewer; one only tests, fuzz targets or
the repository tooling use needs its ADR and license all the same, but
does not count.

Each dependency decision lives in its own file under
[docs/adr](docs/adr/), per ENG-26. The file's Decision table gives
every module's purpose, license and maintenance status, and its
Alternatives section weighs what was passed over. The list below is
generated from those files by `mdsmith fix`, so it is never edited by
hand.

The `ENG-18` scenario in
[features/engineering.feature](features/engineering.feature) reads
`cargo metadata` and the ADRs. It fails the build in any
of these cases:

- a direct dependency, in `Cargo.toml` or the fuzz workspace's
  `fuzz/Cargo.toml`, is justified by no accepted ADR, or by more than
  one;
- an accepted ADR names a module no manifest requires any more,
  `fuzz/Cargo.toml`'s included;
- a row leaves a column empty, or its license is off the allow-list
  and named by no per-crate exception in [deny.toml](deny.toml);
- this file does not list a dependency ADR;
- more than ten direct dependencies link into a shipped executable,
  reached by normal dependencies from a binary under `crates/`.

Updates arrive as reviewed dependabot pull requests; see
[.github/dependabot.yml](.github/dependabot.yml).

<?catalog
glob: "docs/adr/*.md"
where: 'scope: "dependencies"'
sort: path
header: |

  | ADR | Status | Decision |
  | --- | ------ | -------- |
row: "| [{id}]({filename}) | {status} | {summary} |"
footer: |

?>

| ADR                                                                   | Status     | Decision                                                                                                                                                                                                                                                                                                                                                                  |
| --------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [ADR-2609292234](docs/adr/ADR-2609292234-test-stack.md)               | superseded | The executable requirement matrix runs through godog and reads Gherkin through cucumber's messages types; testify asserts in every test. All three are test-only.                                                                                                                                                                                                         |
| [ADR-2610101442](docs/adr/ADR-2610101442-rust-test-stack.md)          | superseded | The repository tooling in Rust runs the scenarios through cucumber-rs and reads JSON through serde and serde_json; testify asserts in the Go tests that remain. All four serve tests and tooling only; none links into a shipped executable.                                                                                                                              |
| [ADR-2610101913](docs/adr/ADR-2610101913-test-stack-after-go.md)      | superseded | The scenarios run through cucumber-rs, and the tooling and the tests read JSON through serde and serde_json. All three serve tests and tooling only; none links into the core executable, whose reach test refuses any crate.                                                                                                                                             |
| [ADR-2610110553](docs/adr/ADR-2610110553-sqlite-rusqlite.md)          | accepted   | The store reaches SQLite and FTS5 through rusqlite with SQLite bundled and default features off, built without extension loading. Spike S2, rerun for the Rust core, measured it at 10M events. Accepted with the M1 change that links the store.                                                                                                                         |
| [ADR-2610142218](docs/adr/ADR-2610142218-hash-sha2.md)                | accepted   | The record hashes through the sha2 crate's SHA-256: each writer's hash chain, ids derived from keys, fingerprints, and canonical digests. The commitment is HMAC-SHA-256 under the event's commitment key, written in the record crate over the same hash.                                                                                                                |
| [ADR-2610142219](docs/adr/ADR-2610142219-signatures-ed25519-dalek.md) | accepted   | Device and seat keys, and the recovery keys ParleyInn mints, are Ed25519 keys; every signature ParleyInn makes is Ed25519 through ed25519-dalek, verified strictly. Each signature signs a label, its kind and the canonical body, so one kind never verifies as another (PRV-12).                                                                                        |
| [ADR-2610142220](docs/adr/ADR-2610142220-random-getrandom.md)         | accepted   | Every random bit ParleyInn draws, for keys, commitment keys and room-id nonces, comes from the OS's random source through getrandom, behind a trait a test replaces. No user-space generator stands between.                                                                                                                                                              |
| [ADR-2610142221](docs/adr/ADR-2610142221-json-serde-json.md)          | accepted   | The scenarios run through cucumber-rs and the tooling decodes JSON through serde. serde_json reads and writes JSON in the tooling and, from M1, in the core executable: the MCP server, the store's content, the envelope and the canonical encoding.                                                                                                                     |
| [ADR-2610152132](docs/adr/ADR-2610152132-settings-toml.md)            | accepted   | Every settings layer, managed policy, the principal's configuration and a repository's .parinn.toml, is read through the toml crate's parser into a table the configuration crate validates strictly itself. No serde derive, no writer, and the parser's nesting limit kept on.                                                                                          |
| [ADR-2610161200](docs/adr/ADR-2610161200-redaction-regex.md)          | accepted   | The redaction rule set and the principal's redaction patterns (redaction.extra_patterns) match through the regex crate: finite automata, linear time on every input, each pattern compiled under a size limit. Default features off; std, unicode and perf on.                                                                                                            |
| [ADR-2610161201](docs/adr/ADR-2610161201-logs-tracing.md)             | accepted   | ParleyInn logs through the tracing crate's events and macros, default features off, std on. A subscriber of ParleyInn's own writes each event as one JSON line to the home's log file, every field redacted first; no tracing-subscriber, no log bridge, no proc-macro attributes.                                                                                        |
| [ADR-2610171200](docs/adr/ADR-2610171200-fuzzing-libfuzzer-sys.md)    | accepted   | The fuzz targets of ENG-07 run under cargo-fuzz on libfuzzer-sys, LLVM's libFuzzer compiled in, in a fuzz/ crate that is its own cargo workspace and builds on a pinned nightly. Nothing it links reaches a shipped executable. Its license adds NCSA, which the allow-list does not name; the stakeholder's ruling of 19 October 2026 admits it for this one crate only. |
| [ADR-2610171300](docs/adr/ADR-2610171300-key-stores.md)               | accepted   | The core keeps each key in an entry named per home: on macOS in the Keychain through security-framework, on Windows in a file DPAPI protects through windows-sys in the platform crate (once the dpapi shim), the one crate with unsafe code, which Miri checks; on Linux in a 0600 file, and parinn status says so. The stakeholder's ruling 2 of 14 October 2026.       |
<?/catalog?>

Twelve direct dependencies. Ten link into the shipped `parinn`
executable, at the target of ten: `sha2`, `ed25519-dalek`,
`getrandom`, `rusqlite`, `serde_json`, `toml`, `regex` and `tracing`,
with `security-framework` on macOS and `windows-sys` on Windows only.
`cucumber` and `serde` serve tests and repository tooling alone.
`libfuzzer-sys` serves the fuzz targets alone, in `fuzz/`, a cargo
workspace of its own that the check above reads too, so it counts
toward no limit. Its NCSA license is admitted for that one crate by an
exception in [deny.toml](deny.toml), on the stakeholder's ruling of 19
October 2026, not by the allow-list. The executable's reach test,
[crates/parinn/tests/reach.rs](crates/parinn/tests/reach.rs), fails the
build when a crate links into it that its list does not allow, each
listed crate weighed for its reach when it joined.

[deny.toml](deny.toml) holds every crate in the tree, direct or not,
to the same allow-list, with each exception named, and to crates.io as
the only source; CI runs cargo-deny and cargo-audit over it (ENG-16).
Updates arrive weekly from dependabot for Cargo and actions.

Dev tools are no dependency. CI installs cargo-deny, cargo-audit,
cargo-llvm-cov and taplo at exact versions.
