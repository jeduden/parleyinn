//! Redaction of the harness account identifiers (SEC-08) over every
//! transcript recorded from Claude Code under `testdata/claude-code`:
//! each is ingested into a fresh home, and no email address or
//! organization id the harness's own fields carry reaches the store,
//! neither an event's content nor the store's file.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use administration::home::Home;
use key::random::OsEntropy;
use redaction::Redactor;
use serde_json::Value;
use store::Store;
use testkit::TempDir;

/// Every recorded transcript, by path.
fn transcripts(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            transcripts(&path, found);
        } else if path.extension().is_some_and(|e| e == "jsonl")
            && path.components().any(|c| c.as_os_str() == "transcripts")
        {
            found.push(path);
        }
    }
}

/// The identifiers the harness's own fields carry in `text`, read
/// apart from the redactor: each `userEmail`'s address and each
/// `organizationUuid`.
fn identifiers(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let attachment = &value["attachment"];
        if let Some(id) = attachment["organizationUuid"].as_str() {
            out.insert(id.to_owned());
        }
        if let Some(email) = attachment["context"]["userEmail"].as_str() {
            out.extend(
                email
                    .split_whitespace()
                    .filter(|w| w.contains('@'))
                    .map(|w| w.trim_end_matches('.').to_owned()),
            );
        }
    }
    out
}

#[test]
fn no_harness_account_identifier_from_a_recorded_transcript_reaches_the_store() {
    let mut found = Vec::new();
    transcripts(
        &testkit::repo_root().join("testdata/claude-code"),
        &mut found,
    );
    assert!(found.len() > 10, "too few transcripts: {}", found.len());
    let redactor = Redactor::new(&[]).unwrap().0;
    let mut checked = 0;
    for path in found {
        let ids = identifiers(&fs::read_to_string(&path).unwrap());
        let dir = TempDir::new().unwrap();
        let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();

        let ingested = ingest::ingest_path(&home, &path, &redactor, &mut OsEntropy).unwrap();

        let store = Store::open(&home.store()).unwrap();
        let contents: Vec<String> = store
            .log(ingested.writer)
            .unwrap()
            .iter()
            .map(|e| e.content.clone().unwrap().to_string())
            .collect();
        drop(store);
        let file = fs::read(home.store()).unwrap();
        for id in &ids {
            assert!(
                !contents.iter().any(|c| c.contains(id.as_str())),
                "{} leaves {id} in an event",
                path.display()
            );
            assert!(
                !file.windows(id.len()).any(|w| w == id.as_bytes()),
                "{} leaves {id} in the store's file",
                path.display()
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "no transcript names a harness account");
}
