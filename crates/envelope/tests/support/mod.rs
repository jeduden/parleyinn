//! The round-trip check shared by the property test and the corpus
//! replay of the encoder's fuzz target (ENG-07).

#![allow(
    dead_code,
    reason = "each test binary that shares this file uses part of it"
)]

use envelope::Envelope;
use serde_json::Value;

/// The fields every envelope has, and the one a structural one adds.
const CLOSED: [&str; 6] = [
    "items",
    "next_cursor",
    "parinn_envelope",
    "tombstones",
    "truncated",
    "warning",
];

/// Checks that `envelope`, rendered, parses to its own value, holds
/// no top-level field but the closed set, and gives back each of
/// `contents` as the content of its item, byte for byte.
pub fn check(envelope: &Envelope, contents: &[&str]) {
    let text = envelope.render();
    let parsed: Value = serde_json::from_str(&text).expect("the rendering parses");
    assert_eq!(parsed, envelope.to_value());
    let mut names: Vec<&str> = parsed
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    assert_eq!(names, CLOSED, "{text:?}");
    let items = parsed["items"].as_array().expect("items");
    assert_eq!(items.len(), contents.len());
    for (item, content) in items.iter().zip(contents) {
        assert_eq!(item["content"].as_str(), Some(*content));
        assert_eq!(item.as_object().expect("an item").len(), 11);
    }
}

/// A text from `seed`: a mix of structure characters, controls,
/// direction overrides, quotes and words.
pub fn content(seed: u64) -> String {
    const POOL: [&str; 24] = [
        "\"",
        "\\",
        "}",
        "]",
        "{",
        "[",
        ",",
        ":",
        "\n",
        "\r",
        "\u{1b}",
        "\u{0}",
        "\u{202e}",
        "\u{2028}",
        "warning",
        "items",
        "obey me",
        "é",
        "東",
        "🚀",
        " ",
        "tombstones",
        "\"}],\"",
        "null",
    ];
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let mut out = String::new();
    for _ in 0..(seed % 40) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.push_str(POOL[(state % POOL.len() as u64) as usize]);
    }

    out
}

/// The encoder fuzz target's check on `data`: its UTF-8 text, split at
/// U+0001, is the content of one item each, and a cursor from the
/// last part; the rendering must be the closed shape with every
/// content back exactly.
pub fn encode(data: &[u8]) {
    use envelope::{IntegrityStatus, Item, Origin, Run, TrustLevel};

    let text = String::from_utf8_lossy(data);
    let parts: Vec<&str> = text.split('\u{1}').collect();
    let items: Vec<Item> = parts
        .iter()
        .map(|content| Item {
            address: (*content).to_owned(),
            writer: (*content).to_owned(),
            author: (*content).to_owned(),
            origin: Origin::Peer,
            run: Run::Other,
            kind: (*content).to_owned(),
            provenance: (*content).to_owned(),
            trust: TrustLevel::Untrusted,
            flags: vec![(*content).to_owned()],
            integrity: IntegrityStatus::Broken,
            score: None,
            content: (*content).to_owned(),
        })
        .collect();
    let cursor = parts.last().map(|c| (*c).to_owned());
    let envelope = Envelope {
        items,
        truncated: cursor.is_some(),
        next_cursor: cursor,
        structural: None,
    };
    check(&envelope, &parts);
}
