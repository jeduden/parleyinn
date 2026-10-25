//! The dependency half of the core executable's reach evidence (SEC-01,
//! ENG-16): no crate links into `parinn` that could open a socket or
//! start a program behind clippy's back. clippy's disallowed types,
//! in this crate's clippy.toml, hold its own code; this holds what it
//! links. A dependency joins `REACHABLE` in the same change that adds
//! it, under review, with its reach weighed.

#![allow(
    clippy::disallowed_types,
    reason = "the test runs cargo metadata; the shipped code starts no program"
)]
#![allow(
    clippy::disallowed_methods,
    reason = "the test waits for cargo metadata; the test run's own limit bounds it"
)]
#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeSet;
use std::process::Command;

use serde_json::Value;

/// The crates the core executable may link, by name, each weighed for
/// its reach when it joined (plan 2609292012, phase 1). `cargo
/// metadata` resolves every platform and unifies features across the
/// workspace, so the list holds a few crates a given build leaves out.
const REACHABLE: [&str; 78] = [
    // The core's own library crates, each with the same clippy bans.
    "administration",
    "allowed_root",
    "audit",
    "configuration",
    "envelope",
    "hook",
    "ingest",
    "key",
    "mcp",
    "platform",
    "provenance",
    "record",
    "redaction",
    "repository",
    "search",
    "store",
    "structured_log",
    "transcript",
    // SHA-256 (ADR-2610142218): pure computation over bytes, with its
    // digest traits and the CPU feature probe of its fast paths.
    "sha2",
    "digest",
    "block-buffer",
    "crypto-common",
    "hybrid-array",
    "typenum",
    "const-oid",
    "cpufeatures",
    "cfg-if",
    // Ed25519 (ADR-2610142219): curve arithmetic and signatures, pure
    // computation; zeroize wipes secret keys, subtle compares in
    // constant time. Its derive macro, like every proc macro below,
    // runs only at build time.
    "ed25519-dalek",
    "ed25519",
    "signature",
    "curve25519-dalek",
    "curve25519-dalek-derive",
    "fiat-crypto",
    "subtle",
    "zeroize",
    // The OS's random source (ADR-2610142220): the getrandom system
    // call, or its platform's equivalent, through libc's declarations;
    // r-efi only on UEFI targets.
    "getrandom",
    "libc",
    "r-efi",
    // The key stores (ADR-2610171300): the macOS Keychain through
    // Security.framework and Core Foundation, items in the user's
    // keychain and the types that carry them, never a socket or a
    // program start; on Windows, DPAPI's declarations and the link
    // stubs windows-sys calls them through, reached only from the
    // platform shim.
    "security-framework",
    "security-framework-sys",
    "core-foundation",
    "core-foundation-sys",
    "windows-sys",
    "windows-link",
    // The store (ADR-2610110553): SQLite compiled in with extension
    // loading compiled out, so it opens files and no socket, and starts
    // no program.
    "rusqlite",
    "libsqlite3-sys",
    "bitflags",
    "smallvec",
    "fallible-iterator",
    "fallible-streaming-iterator",
    // JSON (ADR-2610142221): parsing and writing text in memory.
    "serde_json",
    "serde_core",
    "itoa",
    "memchr",
    "zmij",
    // Redaction's patterns (ADR-2610161200): compiling patterns and
    // matching text in memory, in linear time.
    "regex",
    "regex-automata",
    "regex-syntax",
    "aho-corasick",
    // Structured logs (ADR-2610161201): events and their fields in
    // memory, and the lazily set cell tracing-core keeps its registry
    // of call sites in; the core's own subscriber writes them.
    "tracing",
    "tracing-core",
    "pin-project-lite",
    "once_cell",
    // TOML (ADR-2610152132): parsing a settings layer's text in memory;
    // no writer, the nesting limit kept. cargo metadata also lists its
    // optional writer and ordered map, which the core's features leave
    // out: data structures and text formatting in memory.
    "toml",
    "toml_parser",
    "toml_datetime",
    "serde_spanned",
    "winnow",
    "toml_writer",
    "indexmap",
    "hashbrown",
    "equivalent",
    // serde with its derive, which the workspace's feature unification
    // turns on for serde_json, and the proc-macro crates every derive
    // needs; they run only at build time.
    "serde",
    "serde_derive",
    "proc-macro2",
    "quote",
    "syn",
    "unicode-ident",
];

/// The packages `root` links: its normal dependencies and theirs, by
/// name. Dev and build dependencies never reach the executable.
fn linked(metadata: &Value, root: &str) -> Result<BTreeSet<String>, String> {
    let packages = metadata["packages"].as_array().ok_or("no packages")?;
    let name_of = |id: &str| {
        packages
            .iter()
            .find(|p| p["id"] == id)
            .and_then(|p| p["name"].as_str())
            .map(str::to_owned)
    };
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("no resolve")?;
    let root_id = packages
        .iter()
        .find(|p| p["name"] == root)
        .and_then(|p| p["id"].as_str())
        .ok_or(format!("no package {root}"))?;
    let (mut seen, mut todo) = (BTreeSet::new(), vec![root_id.to_owned()]);
    while let Some(id) = todo.pop() {
        let node = nodes.iter().find(|n| n["id"] == id.as_str());
        let deps = node
            .and_then(|n| n["deps"].as_array())
            .into_iter()
            .flatten();
        for dep in deps.filter(|d| is_normal(d)) {
            let pkg = dep["pkg"].as_str().unwrap_or_default().to_owned();
            if seen.insert(pkg.clone()) {
                todo.push(pkg);
            }
        }
    }

    Ok(seen.iter().filter_map(|id| name_of(id)).collect())
}

/// Whether a resolved dependency links into the executable: it has a
/// normal (kind null) edge.
fn is_normal(dep: &Value) -> bool {
    dep["dep_kinds"]
        .as_array()
        .is_some_and(|kinds| kinds.iter().any(|k| k["kind"].is_null()))
}

#[test]
fn the_core_executable_links_only_crates_its_reach_allows() {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let out = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let metadata: Value = serde_json::from_slice(&out.stdout).unwrap();

    let extra: Vec<String> = linked(&metadata, "parinn")
        .unwrap()
        .into_iter()
        .filter(|name| !REACHABLE.contains(&name.as_str()))
        .collect();

    assert!(
        extra.is_empty(),
        "parinn links crates its reach does not allow: {extra:?}"
    );
}

#[test]
fn linked_follows_normal_dependencies_only() {
    let metadata = serde_json::json!({
        "packages": [
            {"id": "a", "name": "parinn"},
            {"id": "b", "name": "lib"},
            {"id": "c", "name": "deep"},
            {"id": "d", "name": "devonly"},
        ],
        "resolve": {"nodes": [
            {"id": "a", "deps": [
                {"pkg": "b", "dep_kinds": [{"kind": null}]},
                {"pkg": "d", "dep_kinds": [{"kind": "dev"}]},
            ]},
            {"id": "b", "deps": [{"pkg": "c", "dep_kinds": [{"kind": null}]}]},
            {"id": "c", "deps": [{"pkg": "b", "dep_kinds": [{"kind": null}]}]},
        ]},
    });

    let got = linked(&metadata, "parinn").unwrap();

    assert_eq!(got, BTreeSet::from(["deep".to_owned(), "lib".to_owned()]));
    assert_eq!(
        linked(&metadata, "other"),
        Err("no package other".to_owned())
    );
    assert_eq!(
        linked(&serde_json::json!({}), "parinn"),
        Err("no packages".to_owned())
    );
}
