//! Keys, seats and the personal room end to end (plan 2609292012,
//! phase 5), every start of the built core confined to its boundary
//! (ENG-12) in an isolated home:
//!
//! - SEC-10: the keys are kept in this OS's key store, `parinn status`
//!   names it and says an unsandboxed agent of the same OS user could
//!   read a key file, and no key reaches the store, a log, the audit log
//!   or any output;
//! - PRV-11: `parinn verify --json` reports each seat's kind and ingest
//!   flag from its seat certificate;
//! - REC-24 and SEC-27: a changed node identity mints a new device key
//!   and a new device seat in the same personal room before the next
//!   append, audited and counted, and nothing is appended under an old
//!   key.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a start the confinement refuses fails the test with its message"
)]

use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::Value;
use testkit::boundary::{Component, Confinement};

/// Runs the core with `args` in `home`, confined, its node identity read
/// from `home/machine-id`.
#[track_caller]
fn parinn(home: &Path, args: &[&str]) -> Output {
    let identity = home.join("machine-id").display().to_string();
    let out = Confinement::of(Component::Core)
        .unwrap_or_else(|e| panic!("{e}"))
        .run_in(
            Path::new(env!("CARGO_BIN_EXE_parinn")),
            args,
            home,
            &[("PARINN_TEST_NODE_IDENTITY", &identity)],
            None,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        out.status.success(),
        "parinn {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// `parinn <args> --json`'s output.
#[track_caller]
fn json(home: &Path, args: &[&str]) -> Value {
    let mut args = args.to_vec();
    args.push("--json");
    serde_json::from_slice(&parinn(home, &args).stdout).unwrap()
}

/// Ingests a transcript of one line, `text`.
#[track_caller]
fn ingest(home: &Path, text: &str) -> Value {
    let dir = home.join(".claude/projects/p");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("t.jsonl");
    fs::write(&path, format!("{text}\n")).unwrap();
    json(home, &["ingest", "--path", path.to_str().unwrap()])
}

/// Every file under `dir`, with its bytes.
fn files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(files(&path));
        } else {
            out.push((path.display().to_string(), fs::read(&path).unwrap()));
        }
    }
    out
}

#[test]
fn keys_stay_in_the_key_store_and_a_changed_node_identity_starts_a_new_device_seat() {
    let dir = testkit::TempDir::new().unwrap();
    let home = dir.path();
    fs::write(home.join("machine-id"), "first\n").unwrap();
    let first = ingest(home, "one");

    let status = json(home, &["status"]);
    assert_eq!(status["keys"]["store"], "file");
    assert_eq!(status["keys"]["readable_by_agents"], true);
    let text = String::from_utf8(parinn(home, &["status"]).stdout).unwrap();
    assert!(
        text.contains("an unsandboxed agent of the same OS user could read them"),
        "{text}"
    );
    // The development build reads the identity from the test seam's
    // file, and says so; a fork is found by a local scan for now.
    assert!(text.contains("node identity: checked (a file)\n"), "{text}");
    assert!(
        text.contains(
            "fork detection: local scan by parinn verify; peer detection with the peer network\n"
        ),
        "{text}"
    );
    assert_eq!(status["node_identity"]["checked"], true);
    assert_eq!(status["fork_detection"]["local"], "parinn verify");

    fs::write(home.join("machine-id"), "second\n").unwrap();
    let second = ingest(home, "two");

    assert_eq!(second["room"], first["room"]);
    let status = json(home, &["status"]);
    assert_eq!(status["counters"]["node_identity_changed"]["total"], 1);
    two_device_seats_in_one_room(&json(home, &["verify"]), &first["room"]);
    no_seed_outside_its_key_file(home);
}

/// Checks that `verified`, `parinn verify --json`'s report, names two
/// device seats in `room` and two ingest seats the two device keys
/// certified, each seat's kind read from its certificate.
fn two_device_seats_in_one_room(verified: &Value, room: &Value) {
    let seats = verified["seats"].as_array().unwrap();
    let devices: Vec<&Value> = seats
        .iter()
        .filter(|s| s["seat_kind"] == "device")
        .collect();
    assert_eq!(devices.len(), 2);
    assert!(devices.iter().all(|s| s["room"] == *room));
    let ingests: Vec<&Value> = seats.iter().filter(|s| s["ingest_seat"] == true).collect();
    assert_eq!(ingests.len(), 2);
    // The two ingest seats were certified by different device keys: the
    // second by the new one.
    assert_ne!(
        ingests[0]["certificate"]["certifying_key"],
        ingests[1]["certificate"]["certifying_key"]
    );
    for seat in seats {
        assert_eq!(seat["seat_kind"], seat["certificate"]["seat_kind"]);
        assert_eq!(seat["ingest_seat"], seat["certificate"]["ingest_seat"]);
    }
}

/// Checks that no key's seed is anywhere but its key file: not in the
/// store, a log, the audit log or any output.
fn no_seed_outside_its_key_file(home: &Path) {
    let keys = files(&home.join(".parinn/keys"));
    assert!(
        keys.len() >= 4,
        "{:?}",
        keys.iter().map(|k| &k.0).collect::<Vec<_>>()
    );
    let mut elsewhere = files(&home.join(".parinn"));
    elsewhere.retain(|(path, _)| !path.contains("/keys/"));
    for args in [&["status", "--json"][..], &["verify", "--json"]] {
        elsewhere.push(("output".to_owned(), parinn(home, args).stdout));
    }
    for (_, seed) in &keys {
        let seed = String::from_utf8(seed.clone()).unwrap();
        for (path, bytes) in &elsewhere {
            let found = bytes.windows(seed.len()).any(|w| w == seed.as_bytes());
            assert!(!found, "a key's seed is in {path}");
        }
    }
}
