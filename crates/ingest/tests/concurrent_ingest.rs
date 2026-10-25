//! REC-06 and REC-13 under concurrency: the harness session's MCP server
//! ingests a transcript live, on one thread, while `parinn ingest` reads
//! the same transcript source again and again on another, as the
//! harness appends to it. Whichever appends a line first records it;
//! the other's append behind it is refused in its transaction and reads
//! again. Every line is then recorded once, in its transcript order, and
//! each writer's `seq` stays gap-free.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use administration::home::Home;
use ingest::live::{RunSeats, ingest_live};
use ingest::{Inputs, ingest_with};
use key::random::OsEntropy;
use redaction::Redactor;
use serde_json::json;
use store::Store;
use testkit::TempDir;

/// How many lines the harness appends.
const LINES: u64 = 400;

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

fn line(n: u64) -> String {
    format!("{}\n", json!({"type": "mode", "mode": "normal", "n": n}))
}

/// Runs `read` until `done` is set, then once more.
fn until(done: &AtomicBool, mut read: impl FnMut()) {
    loop {
        let last = done.load(Ordering::SeqCst);
        read();
        if last {
            return;
        }
    }
}

#[test]
fn the_mcp_server_and_parinn_ingest_together_record_every_line_once() {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    let path = dir
        .path()
        .join("5e55a0de-0000-4000-8000-0000000000c1.jsonl");
    std::fs::write(&path, line(1)).unwrap();
    // The node's first start, before either reads.
    ingest_with(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut OsEntropy,
    )
    .unwrap();
    let done = Arc::new(AtomicBool::new(false));

    std::thread::scope(|scope| {
        scope.spawn(|| {
            let mut seats = RunSeats::default();
            until(&done, || {
                let read = ingest_live(
                    &home,
                    &path,
                    &redactor(),
                    &Inputs::default(),
                    &mut seats,
                    None,
                    &mut OsEntropy,
                );
                if let Err(e) = read {
                    assert!(e.position_moved(), "{e}");
                }
            });
        });
        scope.spawn(|| {
            until(&done, || {
                ingest_with(
                    &home,
                    &path,
                    &redactor(),
                    &Inputs::default(),
                    &mut OsEntropy,
                )
                .unwrap();
            });
        });
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        for n in 2..=LINES {
            file.write_all(line(n).as_bytes()).unwrap();
            // The harness appends a line at a time, as an agent runs.
            if n % 4 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }
        done.store(true, Ordering::SeqCst);
    });
    ingest_with(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut OsEntropy,
    )
    .unwrap();

    assert_every_line_once(&home, &path);
}

/// Fails unless the record holds each line of the transcript at `path`
/// once, in order, each writer's `seq` gap-free.
fn assert_every_line_once(home: &Home, path: &Path) {
    let store = Store::open(&home.store()).unwrap();
    let mut lines: BTreeMap<u64, usize> = BTreeMap::new();
    for generation in store.generations(&path.to_string_lossy()).unwrap() {
        for placed in store.placed(generation.id).unwrap() {
            *lines.entry(placed.place.line).or_default() += 1;
        }
    }
    let expected: BTreeMap<u64, usize> = (1..=LINES).map(|n| (n, 1)).collect();
    assert_eq!(lines, expected);
    for writer in store.writers().unwrap() {
        let seqs: Vec<u64> = store
            .log(writer)
            .unwrap()
            .iter()
            .map(|e| e.link.header.address.seq.get())
            .collect();
        let gap_free: Vec<u64> = (1..=seqs.len() as u64).collect();
        assert_eq!(seqs, gap_free, "writer {writer}");
    }
}
