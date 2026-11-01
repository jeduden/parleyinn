//! The drifts of the architecture checks: reach, crates, deadlines,
//! waits and confinement.

use super::*;

/// The drifts the core executable's reach evidence exists to catch
/// (SEC-01, ENG-16): a crate linked in, or the core's own code opening
/// a socket or starting a program.
pub(super) fn reach_cases() -> Vec<Case> {
    vec![
        case(
            "a crate linked into the core executable",
            "the_core_executable_links_only_crates_its_reach_allows",
            Injection::replace(
                "crates/parinn/Cargo.toml",
                "tracing.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\ntestkit.workspace = true\n",
                "tracing.workspace = true\ntestkit.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\n",
            ),
            cargo_test(
                "parinn",
                "reach",
                "the_core_executable_links_only_crates_its_reach_allows",
            ),
            "parinn links crates its reach does not allow",
        ),
        case(
            "the core executable opening a socket",
            "SEC-01",
            Injection::replace(
                "crates/parinn/src/lib.rs",
                "use std::io::{self, Write};",
                "use std::io::{self, Write};\n\npub fn dial() -> Option<std::net::TcpStream> {\n    None\n}",
            ),
            clippy("parinn"),
            "use of a disallowed type `std::net::TcpStream`",
        ),
    ]
}

/// The drifts ENG-02 exists to catch: a crate depending where its
/// manifest's declared directions forbid, and a manifest leaving out a
/// direction or a responsibility. The edits keep Cargo.lock as it is,
/// so the checks run under `--locked`.
pub(super) fn architecture_cases() -> Vec<Case> {
    const GATE: &str = "the_workspace_keeps_its_declared_directions";
    let gate = || cargo_test("engineering", "checkout", GATE);
    vec![
        case(
            "the core executable depending on a tooling crate",
            "ENG-02",
            Injection::replace(
                "crates/parinn/Cargo.toml",
                "tracing.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\ntestkit.workspace = true\n",
                "tracing.workspace = true\ntestkit.workspace = true\n\n[dev-dependencies]\nrusqlite.workspace = true\n",
            ),
            scenario("ENG-02"),
            "product crate parinn depends on tooling crate testkit, which a product crate never does",
        ),
        case(
            "a tooling crate depending where its declared directions forbid",
            GATE,
            Injection::replace(
                "tooling/adr/Cargo.toml",
                "may-depend-on = [\"srs\"]",
                "may-depend-on = []",
            ),
            gate(),
            "crate adr depends on crate srs, which its declared directions forbid",
        ),
        case(
            "a crate declaring no directions",
            GATE,
            Injection::replace("tooling/drift/Cargo.toml", "may-depend-on = []\n", ""),
            gate(),
            "crate drift declares no may-depend-on directions",
        ),
        case(
            "a library crate declaring no responsibility",
            "ENG-02",
            Injection::replace(
                "tooling/srs/Cargo.toml",
                "responsibility = \"Reads the SRS requirement tables, appendices and personas for the gates\"\n",
                "",
            ),
            scenario("ENG-02"),
            "library crate srs declares no responsibility",
        ),
    ]
}

/// The drifts the product crates' reach and determinism bans exist to
/// catch: a crate's clippy.toml dropping one (I4, I10), and the core
/// reading a clock or a random order (I10). [`deadline_cases`] holds
/// ENG-03's.
pub(super) fn product_crate_cases() -> Vec<Case> {
    const LIB: &str = "crates/parinn/src/lib.rs";
    const AT: &str = "use std::io::{self, Write};";
    vec![
        case(
            "a product crate's clippy.toml dropping a reach ban",
            "the_product_crates_keep_their_bans",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::net::UdpSocket\", reason = \"the core opens no socket (SEC-01)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_keep_their_bans",
            ),
            "crates/parinn/clippy.toml does not ban std::net::UdpSocket in disallowed-types",
        ),
        case(
            "a product crate's clippy.toml dropping a determinism ban",
            "I10",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::time::SystemTime::now\", reason = \"derived artifacts read no clock (I10)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_keep_their_bans",
            ),
            "crates/parinn/clippy.toml does not ban std::time::SystemTime::now in disallowed-methods",
        ),
        case(
            "the core reading the clock",
            "I10",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn now() -> std::time::Instant {\n    std::time::Instant::now()\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::time::Instant::now`",
        ),
        case(
            "the core keeping a hash map's random order",
            "I10",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub type Index = std::collections::HashMap<u8, u8>;",
            ),
            clippy("parinn"),
            "use of a disallowed type `std::collections::HashMap`",
        ),
    ]
}

/// The drifts ENG-03's deadline bans exist to catch: a product crate's
/// clippy.toml dropping one. [`core_wait_cases`] holds the core's code.
pub(super) fn deadline_cases() -> Vec<Case> {
    vec![
        case(
            "a product crate's clippy.toml dropping a deadline ban",
            "the_product_crates_ban_every_wait_with_no_deadline",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::sync::mpsc::Receiver::recv\", reason = \"it waits with no deadline; use recv_timeout (ENG-03)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_ban_every_wait_with_no_deadline",
            ),
            "crates/parinn/clippy.toml does not ban std::sync::mpsc::Receiver::recv in disallowed-methods",
        ),
        case(
            "a product crate's clippy.toml dropping the file lock's deadline ban",
            "the_product_crates_ban_every_wait_with_no_deadline",
            Injection::replace(
                "crates/parinn/clippy.toml",
                "  { path = \"std::fs::File::lock\", reason = \"it waits for another process with no deadline; poll try_lock up to a deadline (ENG-03)\" },\n",
                "",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "the_product_crates_ban_every_wait_with_no_deadline",
            ),
            "crates/parinn/clippy.toml does not ban std::fs::File::lock in disallowed-methods",
        ),
    ]
}

/// The drifts ENG-03 exists to catch in the core's code: a wait with no
/// deadline, and mutable global state.
pub(super) fn core_wait_cases() -> Vec<Case> {
    const LIB: &str = "crates/parinn/src/lib.rs";
    const AT: &str = "use std::io::{self, Write};";
    vec![
        case(
            "the core waiting on a channel with no deadline",
            "ENG-03",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn wait(rx: &std::sync::mpsc::Receiver<u8>) -> Option<u8> {\n    rx.recv().ok()\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::sync::mpsc::Receiver::recv`",
        ),
        case(
            "the core joining a thread with no deadline",
            "ENG-03",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn wait(h: std::thread::JoinHandle<()>) {\n    let _ = h.join();\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::thread::JoinHandle::join`",
        ),
        case(
            "the core reading standard input with no deadline",
            "ENG-03",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub fn input() -> std::io::Stdin {\n    std::io::stdin()\n}",
            ),
            clippy("parinn"),
            "use of a disallowed method `std::io::stdin`",
        ),
        case(
            "a product crate keeping mutable global state",
            "no_product_crate_keeps_mutable_global_state",
            Injection::replace(
                LIB,
                AT,
                "use std::io::{self, Write};\n\npub static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);",
            ),
            cargo_test(
                "engineering",
                "checkout",
                "no_product_crate_keeps_mutable_global_state",
            ),
            // The line number moves with the file; the finding is stable.
            ": mutable global state: pub static SEQ",
        ),
    ]
}

/// The drifts ENG-12's core row exists to catch: the core started
/// outside the end-to-end suite's confinement, and the core opening a
/// socket, which the confined suite fails naming the core and the test.
pub(super) fn confinement_cases() -> Vec<Case> {
    vec![
        case(
            "the core started outside the end-to-end suite's confinement",
            "the_core_s_tests_start_it_confined",
            Injection::replace(
                "crates/parinn/tests/e2e_parinn.rs",
                "use testkit::boundary::{Component, Confinement};",
                "use testkit::boundary::{Component, Confinement};\n\nfn _unconfined() -> std::process::Command {\n    std::process::Command::new(env!(\"CARGO_BIN_EXE_parinn\"))\n}",
            ),
            cargo_test(
                "engineering",
                "confinement",
                "the_core_s_tests_start_it_confined",
            ),
            "crates/parinn/tests/e2e_parinn.rs starts a component's executable outside the end-to-end suite's confinement (ENG-12)",
        ),
        case(
            "the core opening a socket in the confined suite",
            "ENG-12",
            Injection::replace(
                "crates/parinn/src/lib.rs",
                "        Some(\"version\") => version(args, ctx.build, stdout, stderr),",
                "        Some(\"version\") => {\n            let _ = std::net::UdpSocket::bind((\"127.0.0.1\", 0));\n            version(args, ctx.build, stdout, stderr)\n        }",
            ),
            cargo_test("parinn", "e2e_parinn", "version_prints_the_build_s_version"),
            "ENG-12: the core opened a socket outside its boundary, which denies all network; started by version_prints_the_build_s_version",
        ),
    ]
}
