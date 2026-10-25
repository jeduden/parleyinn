//! ENG-07, LMK-03: the `field_sanitizer` fuzz target's checks over its
//! committed seed corpus, so they run where cargo-fuzz and its nightly
//! do not.

use provenance::structural_field::{DEFAULT_LIMIT, sanitize};

fn conforming(text: &str) -> bool {
    text.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'/' | b'@' | b':' | b'+' | b'-')
    })
}

fn replacement(text: &str) -> bool {
    text.strip_prefix("<field:")
        .and_then(|rest| rest.strip_suffix('>'))
        .is_some_and(|h| {
            h.len() == 8
                && h.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

fn check(value: &str, limit: usize) {
    let out = sanitize(value, limit);
    let text = out.as_str();
    assert!(text.len() <= limit.max(16), "{value:?} at {limit}");
    if out.is_replaced() {
        assert!(replacement(text), "{value:?} gave {text:?}");
    } else {
        assert!(text == value && conforming(text) && text.len() <= limit);
    }
}

#[test]
fn the_seed_corpus_holds_the_fuzz_target_checks() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/field_sanitizer");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap();
        if let Ok(value) = std::str::from_utf8(&bytes) {
            check(value, DEFAULT_LIMIT);
            check(value, value.len() / 2);
        }
        seen += 1;
    }
    assert!(seen >= 5, "the seed corpus is committed");
}
