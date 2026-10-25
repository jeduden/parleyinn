//! ENG-08's rebuild-digest property for the search index (I10, §8.3),
//! over randomly generated records: the index a store keeps as events
//! are appended has the canonical digest of one rebuilt from nothing,
//! and two stores that hold the same writer logs share it and answer a
//! search alike, whatever order the writers' appends arrived in. Every
//! case is seeded, so a failure names a seed that reproduces it;
//! `PARINN_PROPERTY_CASES` runs more.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::time::Duration;

use record::commitment::CommitmentKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, SeatId, WriterId};
use search::Request;
use serde_json::json;
use store::{NewEvent, Search, Store, StoredSeat};
use testkit::TempDir;

/// How many records the property is checked over by default.
const CASES: u64 = 32;

/// The words events are made of: few, so searches match many events.
const WORDS: [&str; 8] = [
    "deploy",
    "staging",
    "quokka-early",
    "wombat",
    "Café",
    "ünïcode",
    "OR",
    "NEAR(x)",
];

/// A small seeded generator (xorshift64*): no clock, no OS randomness, so
/// a seed replays its record exactly.
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

    /// A number below `n`.
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn room() -> RoomId {
    RoomId::derive(&[1; 32], &[2; 16])
}

/// One append: `n` events of random words, one in eight large enough
/// for the payload store, one in sixteen with no text.
fn batch(rng: &mut Rng, n: usize) -> Vec<NewEvent> {
    (0..n)
        .map(|_| {
            let mut text: Vec<&str> = (0..1 + rng.below(12))
                .map(|_| WORDS[rng.below(WORDS.len() as u64) as usize])
                .collect();
            if rng.below(8) == 0 {
                text.extend(std::iter::repeat_n("padding", 2000));
            }
            let mut key = [0; 32];
            key[..8].copy_from_slice(&rng.next().to_le_bytes());
            let content = if rng.below(16) == 0 {
                json!({"note": "no text"})
            } else {
                json!({"text": text.join(" ")})
            };
            NewEvent {
                kind: EventKind::UnparsedLine,
                provenance: Provenance::Unparsed,
                content,
                commitment_key: CommitmentKey::from_bytes(key),
                transcript_timestamp: None,
            }
        })
        .collect()
}

/// Each of `writers` writers' appends, in the order each makes them.
fn record(rng: &mut Rng, writers: u8) -> Vec<(u8, Vec<Vec<NewEvent>>)> {
    (1..=writers)
        .map(|w| {
            let appends = (0..1 + rng.below(5))
                .map(|_| {
                    let n = 1 + rng.below(25) as usize;
                    batch(rng, n)
                })
                .collect();
            (w, appends)
        })
        .collect()
}

/// A store holding `record`, the writers' appends interleaved as `rng`
/// chooses, each writer's own appends in their order.
fn arrive(dir: &TempDir, record: &[(u8, Vec<Vec<NewEvent>>)], rng: &mut Rng) -> Store {
    let mut store = Store::open(&dir.path().join("store.db")).unwrap();
    let mut next = vec![0; record.len()];
    let mut seated = vec![false; record.len()];
    while next.iter().zip(record).any(|(n, (_, a))| *n < a.len()) {
        let pick = rng.below(record.len() as u64) as usize;
        let (w, appends) = &record[pick];
        let Some(events) = appends.get(next[pick]) else {
            continue;
        };
        if !seated[pick] {
            store
                .add_seat(&StoredSeat {
                    writer: writer(*w),
                    seat: SeatId::derive(&room(), &[*w; 32]),
                    room: room(),
                    certificate: json!({}),
                })
                .unwrap();
            seated[pick] = true;
        }
        store.append(writer(*w), events).unwrap();
        next[pick] += 1;
    }

    store
}

/// What searching for each word finds, with scores.
fn answers(store: &Store) -> Vec<Vec<(String, u64)>> {
    WORDS
        .iter()
        .map(|word| {
            let query = search::compile(&Request {
                text: word,
                ..Request::default()
            })
            .unwrap();
            let search = Search {
                query: &query,
                room: room(),
                writer: None,
                provenance: &[],
                kinds: &[],
                since: None,
                until: None,
                k: 50,
                deadline: Duration::from_secs(60),
            };
            store
                .search(&search)
                .unwrap()
                .iter()
                .map(|h| (h.address.to_string(), h.score.to_bits()))
                .collect()
        })
        .collect()
}

fn cases() -> u64 {
    std::env::var("PARINN_PROPERTY_CASES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(CASES)
}

#[test]
fn the_index_kept_per_append_has_the_digest_of_a_rebuild_whatever_order_writers_arrived_in() {
    for seed in 0..cases() {
        let mut rng = Rng::new(seed);
        let record = record(&mut rng, 3);
        let (one, two) = (TempDir::new().unwrap(), TempDir::new().unwrap());
        let mut first = arrive(&one, &record, &mut Rng::new(seed + 1_000));
        let second = arrive(&two, &record, &mut Rng::new(seed + 2_000));

        let kept = first.search_index_digest().unwrap();
        assert_eq!(
            second.search_index_digest().unwrap(),
            kept,
            "seed {seed}: order"
        );
        assert_eq!(answers(&first), answers(&second), "seed {seed}: answers");
        first.rebuild_search_index().unwrap();
        assert_eq!(
            first.search_index_digest().unwrap(),
            kept,
            "seed {seed}: rebuild"
        );
        assert_eq!(answers(&first), answers(&second), "seed {seed}: rebuilt");
    }
}
