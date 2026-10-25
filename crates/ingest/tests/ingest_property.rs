//! ENG-08's two ingest properties, over randomly generated transcripts:
//!
//! - Ingest twice equals ingest once (REC-03): a transcript written in
//!   random pieces, some cut mid-line, and ingested after each and twice
//!   at its end, records the same events per run as one ingest of the
//!   whole.
//! - Where a fork observation marks a harness fork, every order of
//!   ingesting its transcripts within the hold window, on an injected
//!   clock, records each repeated line in the same writer with the same
//!   repeat references (REC-04).
//!
//! Every case is seeded, so a failure names a seed that reproduces it;
//! `PARINN_PROPERTY_CASES` runs more.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use administration::home::Home;
use ingest::{Inputs, ingest_with};
use key::random::OsEntropy;
use record::event::EventKind;
use record::id::{Address, WriterId};
use redaction::Redactor;
use serde_json::{Value, json};
use store::Store;
use testkit::TempDir;

/// How many cases each property is checked over by default.
const CASES: u64 = 16;

fn cases() -> u64 {
    std::env::var("PARINN_PROPERTY_CASES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(CASES)
}

/// A small seeded generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

fn home(dir: &TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

/// A random transcript of the harness session `session`: lines of a few
/// shapes, some of a subagent, some turn ends, some malformed.
fn transcript(rng: &mut Rng, session: &str) -> String {
    let n = 5 + rng.below(56);
    let mut text = String::new();
    for i in 0..n {
        let line = match rng.below(6) {
            0 => format!("not json {i}"),
            1 => json!({"type": "assistant", "uuid": format!("{session}-{i}"), "sessionId": session,
                        "message": {"stop_reason": "end_turn", "content": [{"type": "text", "text": i}]}})
            .to_string(),
            2 => json!({"type": "user", "uuid": format!("{session}-{i}"), "sessionId": session,
                        "agentId": format!("a{}", rng.below(2)), "n": i})
            .to_string(),
            _ => json!({"type": "x", "uuid": format!("{session}-{i}"), "sessionId": session, "n": i})
                .to_string(),
        };
        text.push_str(&line);
        text.push('\n');
    }
    text
}

/// What a home records of each run read from `path`: by agent id, the
/// kind, provenance and content of each event in its writer.
fn by_run(home: &Home, path: &Path) -> BTreeMap<Option<String>, Vec<(String, String, Value)>> {
    let store = Store::open(&home.store()).unwrap();
    let mut out = BTreeMap::new();
    for run in store.ingested_runs().unwrap() {
        if run.path != path.to_string_lossy() {
            continue;
        }
        let events: Vec<_> = store
            .log(run.writer)
            .unwrap()
            .into_iter()
            .map(|e| {
                (
                    e.link.header.kind.as_str().to_owned(),
                    e.link.header.provenance.name(),
                    e.content.unwrap(),
                )
            })
            .collect();
        out.insert(run.agent_id, events);
    }
    out
}

#[test]
fn ingest_twice_equals_ingest_once() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let text = transcript(&mut rng, "s");
        let (pieces, once) = (TempDir::new().unwrap(), TempDir::new().unwrap());
        let (home_pieces, home_once) = (home(&pieces), home(&once));
        let path_pieces = pieces.path().join("t.jsonl");
        let path_once = once.path().join("t.jsonl");
        let inputs = Inputs::default();
        let r = redactor();

        let mut cut = 0;
        while cut < text.len() {
            cut = (cut + 1 + rng.below(400) as usize).min(text.len());
            while !text.is_char_boundary(cut) {
                cut += 1;
            }
            std::fs::write(&path_pieces, &text[..cut]).unwrap();
            // A first piece holding no whole line is still read.
            let _ = ingest_with(&home_pieces, &path_pieces, &r, &inputs, &mut OsEntropy);
        }
        ingest_with(&home_pieces, &path_pieces, &r, &inputs, &mut OsEntropy).unwrap();
        std::fs::write(&path_once, &text).unwrap();
        ingest_with(&home_once, &path_once, &r, &inputs, &mut OsEntropy).unwrap();

        assert_eq!(
            by_run(&home_pieces, &path_pieces),
            by_run(&home_once, &path_once),
            "seed {seed}"
        );
    }
}

/// A parent transcript of `n` lines and a fork of it repeating its first
/// `k` lines, with a line of its own before and `m` after.
fn fork_pair(rng: &mut Rng) -> (String, String) {
    let n = 2 + rng.below(30);
    let k = 1 + rng.below(n);
    let m = rng.below(10);
    let line = |session: &str, uuid: &str, n: u64| {
        format!(
            "{}\n",
            json!({"type": "x", "uuid": uuid, "sessionId": session, "promptId": session, "n": n})
        )
    };
    let parent: String = (0..n).map(|i| line("p", &format!("p{i}"), i)).collect();
    let mut fork = String::from("{\"type\":\"mode\"}\n");
    fork.extend((0..k).map(|i| line("f", &format!("p{i}"), i)));
    fork.extend((0..m).map(|i| line("f", &format!("f{i}"), 100 + i)));
    (parent, fork)
}

/// One event as [`named`] gives it: its kind and line, and the run and
/// line a repeat reference names.
type NamedEvent = (String, u64, Option<(String, u64)>);

/// Each event a home records of the runs of `paths`, its writer named by
/// the stem of the transcript its run is read from: the kind and line,
/// and, for a repeat reference, the run and line it names.
fn named(home: &Home, paths: &[PathBuf]) -> BTreeMap<String, Vec<NamedEvent>> {
    let store = Store::open(&home.store()).unwrap();
    let mut runs: BTreeMap<WriterId, String> = BTreeMap::new();
    let mut lines: BTreeMap<Address, u64> = BTreeMap::new();
    for path in paths {
        let path = path.to_string_lossy();
        let stem = Path::new(path.as_ref())
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for generation in store.generations(&path).unwrap() {
            for placed in store.placed(generation.id).unwrap() {
                runs.insert(placed.address.writer, stem.clone());
                lines.insert(placed.address, placed.place.line);
            }
        }
    }
    let mut out: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for (writer, run) in &runs {
        for e in store.log(*writer).unwrap() {
            let address = e.link.header.address;
            let refers = (e.link.header.kind == EventKind::RepeatReference).then(|| {
                let (to, _) =
                    Address::parse(e.content.as_ref().unwrap()["repeats"].as_str().unwrap())
                        .unwrap();
                (runs[&to.writer].clone(), lines[&to])
            });
            out.entry(run.clone()).or_default().push((
                e.link.header.kind.as_str().to_owned(),
                lines[&address],
                refers,
            ));
        }
    }
    out
}

#[test]
fn every_order_of_ingesting_a_fork_records_its_repeated_lines_alike() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let (parent, fork) = fork_pair(&mut rng);
        let mut seen = Vec::new();
        for parent_first in [true, false] {
            let dir = TempDir::new().unwrap();
            let home = home(&dir);
            let p = dir.path().join("parent.jsonl");
            let f = dir.path().join("fork.jsonl");
            std::fs::write(&p, &parent).unwrap();
            std::fs::write(&f, &fork).unwrap();
            let order = if parent_first { [&p, &f] } else { [&f, &p] };
            let r = redactor();
            // Within the hold window: the clock moves, by less than 30 s.
            let mut now = 1_000_000;
            for path in order.iter().chain(order.iter()) {
                now += rng.below(5_000);
                let inputs = Inputs {
                    forks: ["fork".to_owned()].into(),
                    now_ms: now,
                    ..Inputs::default()
                };
                ingest_with(&home, path, &r, &inputs, &mut OsEntropy).unwrap();
            }
            seen.push(named(&home, &[p, f]));
        }

        assert_eq!(seen[0], seen[1], "seed {seed}");
        let fork_run = &seen[0]["fork"];
        assert!(fork_run.iter().all(|(kind, _, refers)| {
            kind != "repeat_reference" || refers.as_ref().is_some_and(|(run, _)| run == "parent")
        }));
        assert!(
            seen[0]["parent"]
                .iter()
                .all(|(kind, _, _)| kind != "repeat_reference"),
            "seed {seed}"
        );
    }
}
