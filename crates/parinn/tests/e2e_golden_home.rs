//! The golden home (ENG-23's first migration test): a small home the
//! build wrote at M1's close, committed under `testdata/golden-home/`,
//! that the current build must still verify and read back as the same
//! bytes. The writing and the reading code cannot share a fault here:
//! the home and the expected bytes were committed once, and only a
//! deliberate format change writes them again.
//!
//! The home is read in a copy, never in place: a git checkout keeps no
//! owner-only modes, and `parinn verify` and a recall write to the home.
//! To write the golden home again, after a change of format on purpose:
//!
//! ```text
//! cargo test -p parinn --test e2e_golden_home -- --ignored regenerate
//! ```

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a start the confinement refuses fails the test with its message"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use testkit::boundary::{Component, Confinement};

/// The version the golden home was written by.
const VERSION: &str = "0.0.0";

/// The transcript the golden home records: the proving slice's.
const TRANSCRIPT: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl";

/// The sequence numbers read back: the first line, a prompt, the last.
const SEQS: [u64; 3] = [1, 3, 57];

fn root() -> PathBuf {
    testkit::repo_root()
}

fn golden() -> PathBuf {
    root().join("testdata/golden-home").join(VERSION)
}

fn core() -> Confinement {
    Confinement::of(Component::Core).unwrap_or_else(|e| panic!("{e}"))
}

/// Runs the core in `world` (its `HOME`), confined.
#[track_caller]
fn parinn(world: &Path, args: &[&str]) -> std::process::Output {
    core()
        .run(Path::new(env!("CARGO_BIN_EXE_parinn")), args, world)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Copies `from` to `to`, then makes every directory owner-only and
/// every file readable and writable by its owner alone, as a home wants.
fn copy_owner_only(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    fs::set_permissions(to, fs::Permissions::from_mode(0o700)).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_owner_only(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
}

/// A world whose `.parinn` is a copy of `home`.
fn world_with(home: &Path) -> testkit::TempDir {
    let world = testkit::TempDir::new().unwrap();
    copy_owner_only(home, &world.path().join(".parinn"));
    world
}

/// What `parinn verify --json` prints in `world`, which it must exit 0 on.
fn verify_json(world: &Path) -> String {
    let out = parinn(world, &["verify", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// The reply `event_get` gives for `writer:seq` through `parinn mcp`.
fn event_reply(world: &Path, writer: &str, seq: u64) -> String {
    let requests = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "event_get",
            "arguments": {"address": format!("{writer}:{seq}"), "scope": "room"},
        }}),
    ];
    let input: String = requests.iter().map(|r| format!("{r}\n")).collect();
    let out = core()
        .run_with_input(
            Path::new(env!("CARGO_BIN_EXE_parinn")),
            &["mcp"],
            world,
            input.as_bytes(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .last()
        .unwrap()
        .to_owned()
}

fn manifest() -> Value {
    serde_json::from_slice(&fs::read(golden().join("golden.json")).unwrap()).unwrap()
}

fn expected(name: &str) -> String {
    fs::read_to_string(golden().join("expected").join(name)).unwrap()
}

#[test]
fn the_golden_home_verifies_with_the_current_build() {
    let world = world_with(&golden().join("home"));

    let printed = verify_json(world.path());

    assert_eq!(printed, expected("verify.json"));
}

#[test]
fn fixed_addresses_read_back_as_fixed_bytes() {
    let world = world_with(&golden().join("home"));
    let writer = manifest()["writer"].as_str().unwrap().to_owned();

    for seq in SEQS {
        assert_eq!(
            format!("{}\n", event_reply(world.path(), &writer, seq)),
            expected(&format!("event-{seq}.json")),
            "{writer}:{seq}"
        );
    }
}

#[test]
fn a_read_back_line_is_the_transcript_s_own_line() {
    // The transcript is the independent truth: a recall of line `seq`
    // carries that line's bytes, whatever wrote the home.
    let world = world_with(&golden().join("home"));
    let writer = manifest()["writer"].as_str().unwrap().to_owned();
    let lines: Vec<String> = fs::read_to_string(root().join(TRANSCRIPT))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();

    for seq in SEQS {
        let reply: Value = serde_json::from_str(&event_reply(world.path(), &writer, seq)).unwrap();
        let envelope: Value =
            serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        let item = &envelope["items"][0];
        assert_eq!(item["content"], lines[usize::try_from(seq).unwrap() - 1]);
        assert_eq!(item["trust"], "untrusted");
    }
}

#[test]
fn the_golden_home_holds_only_keys_made_for_it() {
    let keys = fs::read_dir(golden().join("home/keys")).unwrap().count();
    let note = fs::read_to_string(golden().join("NOTE.txt")).unwrap();

    assert_eq!(keys, 4, "the device, recovery and two seat keys");
    assert!(
        note.contains("not a secret"),
        "the home's note says its keys guard nothing"
    );
}

/// Writes the golden home and its expected bytes again. Not a test: an
/// explicit act, run with `--ignored`, after a change of format on
/// purpose.
#[test]
#[ignore = "writes testdata/golden-home; run it only for a format change on purpose"]
fn regenerate() {
    let world = testkit::TempDir::new().unwrap();
    let placed = testkit::place_transcript(world.path(), &root().join(TRANSCRIPT)).unwrap();
    let ingest = parinn(
        world.path(),
        &["ingest", "--path", placed.to_str().unwrap(), "--json"],
    );
    assert!(
        ingest.status.success(),
        "{}",
        String::from_utf8_lossy(&ingest.stderr)
    );
    let ingested: Value = serde_json::from_slice(&ingest.stdout).unwrap();

    let dir = golden();
    // The note beside the home stays; the home and what it yields go.
    let _ = fs::remove_dir_all(dir.join("home"));
    let _ = fs::remove_dir_all(dir.join("expected"));
    let home = dir.join("home");
    copy_owner_only(&world.path().join(".parinn"), &home);
    // Derived, and a log of the run that wrote it: neither is the record.
    let _ = fs::remove_dir_all(home.join("logs"));
    let _ = fs::remove_file(home.join("search_index"));

    let fresh = world_with(&home);
    fs::create_dir_all(dir.join("expected")).unwrap();
    fs::write(dir.join("expected/verify.json"), verify_json(fresh.path())).unwrap();
    let writer = ingested["writer"].as_str().unwrap();
    for seq in SEQS {
        let reading = world_with(&home);
        fs::write(
            dir.join(format!("expected/event-{seq}.json")),
            format!("{}\n", event_reply(reading.path(), writer, seq)),
        )
        .unwrap();
    }
    let manifest = json!({
        "version": VERSION,
        "room": ingested["room"],
        "writer": writer,
        "transcript": TRANSCRIPT,
    });
    fs::write(
        dir.join("golden.json"),
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();
}
