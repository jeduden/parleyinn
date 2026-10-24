//! ENG-07, LMK-03: the structural field sanitizer on hostile text. The
//! input is sanitized at the default limit and at a limit of half its
//! length. The sanitizer must never panic, and what it returns must be
//! the value itself, made only of the allow-listed characters and
//! within the limit, or exactly `<field:HASH8>` (HASH8 eight lowercase
//! hex digits): never longer than the larger of the limit and 16 bytes.
#![no_main]

use libfuzzer_sys::fuzz_target;
use provenance::structural_field::{DEFAULT_LIMIT, sanitize};

/// Whether `text` is `<field:HASH8>`.
fn is_replacement(text: &str) -> bool {
    text.strip_prefix("<field:")
        .and_then(|rest| rest.strip_suffix('>'))
        .is_some_and(|hash| {
            hash.len() == 8
                && hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

/// Whether `text` holds only LMK-03's allow-listed characters.
fn is_conforming(text: &str) -> bool {
    text.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'/' | b'@' | b':' | b'+' | b'-')
    })
}

fn check(value: &str, limit: usize) {
    let out = sanitize(value, limit);
    let text = out.as_str();
    assert!(
        text.len() <= limit.max(16),
        "{value:?} at {limit} gave {text:?}"
    );
    if out.is_replaced() {
        assert!(is_replacement(text), "{value:?} gave {text:?}");
    } else {
        assert_eq!(text, value);
        assert!(is_conforming(text) && text.len() <= limit);
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(value) = std::str::from_utf8(data) else {
        return;
    };
    check(value, DEFAULT_LIMIT);
    check(value, value.len() / 2);
});
