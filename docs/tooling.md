---
summary: >-
  Developer tool setup and cache behavior, with tool pins, check commands
  and Rust configuration imported from their source files by mdsmith.
---
# Tooling

The repository is one Cargo workspace. Set up its tools with
`MISE_ENV=dev scripts/setup-tools.sh`. The script installs mise, checked
against [scripts/mise.sha256](../scripts/mise.sha256). Then it runs
`mise install --locked`. [mise.lock](../mise.lock) records the resolved
releases and available checksums for the supported platforms.
`MISE_ENV=dev` adds the developer tools of
[mise.dev.toml](../mise.dev.toml), locked in
[mise.dev.lock](../mise.dev.lock). CI leaves it unset and never installs
them.

The tool pins and check commands below are imported from
[mise.toml](../mise.toml). List tasks with `mise tasks`, run one with
`mise run TASK`, or run a single command with `mise exec -- COMMAND`.
Task arguments follow `--`: `mise run core -- version` runs the core,
and `mise run scenarios -- --tags @REC-03` selects one requirement.
Use `mise run gates` for the specification gates and `mise run benchmark`
to sample the suite. Compare samples with `mise run benchmark-gate --
--base FILE --head FILE`.

<?include
file: ../mise.toml
wrap: toml
?>

```toml
# rust-toolchain.toml is the single Rust version and component pin (ENG-01).
# mise reads that file through its rustup backend.
[settings]
idiomatic_version_file_enable_tools = ["rust"]
cargo.binstall = false

[tools]
"github:jeduden/mdsmith" = { version = "0.55.1", bin = "mdsmith" }
"github:rustsec/rustsec" = { version = "0.22.2", version_prefix = "cargo-audit/v", bin = "cargo-audit" }
"github:taiki-e/cargo-llvm-cov" = "0.9.1"
"github:EmbarkStudios/cargo-deny" = "0.20.2"
"aqua:tamasfe/taplo" = "0.10.0"
"aqua:zizmorcore/zizmor" = "1.29.0"
"aqua:anchore/syft" = "1.52.0"
"aqua:sigstore/cosign" = "3.0.6"

[tasks.test]
run = ["cargo build --release --locked -p parinn -p calibrate", "cargo test --workspace --locked"]

[tasks.lint]
run = ["cargo fmt --all --check", "cargo clippy --workspace --all-targets --locked -- -D warnings", "taplo fmt --check", "taplo lint", "mdsmith check .", "zizmor .github"]

[tasks.supply-chain]
run = ["cargo deny check licenses bans sources", "cargo audit", "cargo deny --manifest-path fuzz/Cargo.toml check licenses bans sources", "cargo audit --file fuzz/Cargo.lock"]

[tasks.coverage]
run = "cargo run --locked -p coverage"

[tasks.drift]
run = "cargo test --locked -p drift --test suite -- --ignored"

[tasks.core]
run = "cargo run --locked -p parinn --"

[tasks.scenarios]
run = "cargo test --locked -p scenario --test bdd --"

[tasks.gates]
run = ["cargo test --locked -p srs --test gates", "cargo test --locked -p scenario --test gate"]

[tasks.benchmark]
run = ["cargo build --release --locked -p parinn", 'cargo run --release --locked -p benchmark -- run --core "${CARGO_TARGET_DIR:-target}/release/parinn"']

[tasks.benchmark-gate]
run = "cargo run --locked -p benchmark -- gate"
```

<?/include?>

Rust's version and components come from
[rust-toolchain.toml](../rust-toolchain.toml) (ENG-01), which mise reads:

<?include
file: ../rust-toolchain.toml
wrap: toml
?>

```toml
# ENG-01: the exact Rust release the workspace builds with. rustup
# reads this file, so every cargo command in the checkout runs on it.
# mise and CI read the version and components from this file.
[toolchain]
channel = "1.99.0"
profile = "minimal"
components = ["clippy", "rustfmt", "llvm-tools-preview"]
```

<?/include?>

<?include
file: ../mise.dev.toml
wrap: toml
?>

```toml
# Developer tools: loaded on top of mise.toml when MISE_ENV=dev, so CI
# never installs them. Locked in mise.dev.lock.
[tools]
"github:jeduden/frit" = { version = "0.17.0", bin = "frit" }
```

<?/include?>

Edit these source files, then run `mdsmith fix docs/tooling.md`
to refresh the imports. CI's `mdsmith check .` rejects stale copies.
After changing pins, run `mise lock` to refresh the platforms already
recorded in the lockfile. Taplo and cargo-audit publish no checksums and
no attestations. Verify and record their asset SHA-256 values in the
lockfile when bumping them. mise then checks them by that checksum alone.

The script is the one install path. CI's setup action, a Claude Code web
environment and a developer machine all run it. It downloads and verifies
tools. It never builds them. It refuses a tool whose backend is not
`github:` or `aqua:`. mise checks each download against the lockfile. It
also verifies each attestation the lockfile records. That needs GitHub API
access to the release's repository. Without it, the install stops rather
than skip the check.

A Claude Code web environment uses this setup script:

```sh
# pins: mise 2026.9.1 (bump with scripts/mise.sha256)
cd parleyinn && MISE_ENV=dev scripts/setup-tools.sh --profile
```

Outside CI the script prints the lines that put the tools on `PATH`.
`--profile` also appends them to `~/.bashrc`, which a web environment
needs. `scripts/setup-tools.sh --check` checks the mise pin and the tools'
backends without installing anything.

The environment caches what setup installs. The cache rebuilds only when
the setup script's text changes, or after about seven days. Bump the
comment with each mise pin so new sessions pick it up.

The shared setup action runs the script with each job's selected tools
and caches mise's installs and the exact Rustup toolchain. Release jobs disable
both caches, as do the review jobs that handle credentials. Linux
end-to-end tests need the host's `unshare`, `script`,
and `strace`; release builds use the toolchain and system dependency
pins in [build.yml](../.github/workflows/build.yml).
