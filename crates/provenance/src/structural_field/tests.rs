use super::*;

fn hash8(value: &str) -> String {
    record::digest::Digest::of(&[value.as_bytes()])
        .to_hex()
        .chars()
        .take(8)
        .collect()
}

#[test]
fn a_conforming_value_is_kept() {
    let s = sanitize("src/lib.rs", DEFAULT_LIMIT);
    assert_eq!(s.as_str(), "src/lib.rs");
    assert!(!s.is_replaced());
}

#[test]
fn every_allow_listed_character_is_kept() {
    let all = "AZaz09._/@:+-";
    assert_eq!(sanitize(all, DEFAULT_LIMIT).as_str(), all);
}

#[test]
fn an_empty_value_conforms() {
    let s = sanitize("", DEFAULT_LIMIT);
    assert_eq!(s.as_str(), "");
    assert!(!s.is_replaced());
}

#[test]
fn a_value_of_exactly_the_limit_is_kept() {
    let value = "a".repeat(DEFAULT_LIMIT);
    assert_eq!(sanitize(&value, DEFAULT_LIMIT).as_str(), value);
}

#[test]
fn a_value_one_past_the_limit_is_replaced_whole() {
    let value = "a".repeat(DEFAULT_LIMIT + 1);
    let s = sanitize(&value, DEFAULT_LIMIT);
    assert!(s.is_replaced());
    assert_eq!(s.as_str(), format!("<field:{}>", hash8(&value)));
}

#[test]
fn a_character_outside_the_set_replaces_the_whole_value() {
    let s = sanitize("a b", DEFAULT_LIMIT);
    assert!(s.is_replaced());
    assert_eq!(s.as_str(), format!("<field:{}>", hash8("a b")));
}

#[test]
fn restore_delimiters_never_survive() {
    for value in ["<x>", "a\nb", "a`b", "ünï", "a\u{0}b", "a;b", "a\\b"] {
        assert!(sanitize(value, DEFAULT_LIMIT).is_replaced(), "{value:?}");
    }
}

#[test]
fn the_limit_is_the_callers() {
    assert!(sanitize("abcd", 3).is_replaced());
    assert!(!sanitize("abc", 3).is_replaced());
}

#[test]
fn the_hash_is_of_the_original_bytes_and_lowercase() {
    let s = sanitize("Ünï", DEFAULT_LIMIT);
    let text = s.as_str();
    assert!(text.starts_with("<field:") && text.ends_with('>'));
    let hash = &text["<field:".len()..text.len() - 1];
    assert_eq!(hash.len(), 8);
    assert!(
        hash.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
}

#[test]
fn sanitizing_is_deterministic_and_a_replacement_is_not_reconformed() {
    let once = sanitize("a b", DEFAULT_LIMIT);
    assert_eq!(once, sanitize("a b", DEFAULT_LIMIT));
    // The replacement holds `<` and `>`, outside the set: sanitizing it
    // again replaces it again, so callers sanitize once, at the edge.
    assert!(sanitize(once.as_str(), DEFAULT_LIMIT).is_replaced());
}
