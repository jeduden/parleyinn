use super::account::HarnessAccountIdentifiers;
use super::{
    PatternRefused, Piece, Redacted, Redactor, extra_pattern, literal_end, merged, pieces,
};

const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

fn redactor(extra: &[&str]) -> Redactor {
    let extra: Vec<String> = extra.iter().map(|p| (*p).to_owned()).collect();
    let (r, refused) = Redactor::new(&extra).unwrap();
    assert!(refused.is_empty());

    r
}

#[test]
fn extra_pattern_names_the_principals_nth_pattern() {
    assert_eq!(extra_pattern(1), "extra_pattern_1");
}

#[test]
fn redact_text_replaces_each_secret_with_its_rules_marker() {
    let r = redactor(&["ACME-[0-9]{8}"]);
    let out = r.redact_text(&format!("a {AWS} b ACME-12345678 c"));
    assert_eq!(
        out,
        Redacted {
            text: "a [REDACTED:aws_access_key] b [REDACTED:extra_pattern_1] c".to_owned(),
            rules: vec!["aws_access_key".to_owned(), "extra_pattern_1".to_owned()],
        }
    );
    let clean = r.redact_text("nothing here");
    assert_eq!(clean.text, "nothing here");
    assert!(clean.rules.is_empty());
}

#[test]
fn overlapping_matches_merge_into_the_first_rules_marker() {
    let r = redactor(&["EXAMPLE tail"]);
    let out = r.redact_text(&format!("{AWS} tail!"));
    assert_eq!(out.text, "[REDACTED:aws_access_key]!");
    assert_eq!(out.rules, ["aws_access_key"]);
}

#[test]
fn merged_joins_overlaps_but_not_neighbours() {
    assert_eq!(
        merged(vec![(0, 4, 1), (2, 6, 0), (6, 8, 2), (7, 7, 3)]),
        [(0, 6, 1), (6, 8, 2)]
    );
    assert!(merged(Vec::new()).is_empty());
}

#[test]
fn a_refused_pattern_is_named_and_the_rest_still_apply() {
    let extra = vec!["(".to_owned(), "ACME".to_owned()];
    let (r, refused) = Redactor::new(&extra).unwrap();
    assert_eq!(refused.len(), 1);
    let PatternRefused { rule, .. } = &refused[0];
    assert_eq!(rule, "extra_pattern_1");
    assert_eq!(
        refused[0].to_string(),
        "redaction pattern extra_pattern_1 refused"
    );
    assert!(std::error::Error::source(&refused[0]).is_some());
    assert_eq!(r.redact_text("ACME").text, "[REDACTED:extra_pattern_2]");
}

#[test]
fn with_identifiers_redacts_the_harness_account_ahead_of_extra_patterns() {
    let mut ids = HarnessAccountIdentifiers::default();
    ids.learn(r#"{"attachment":{"type":"session_context","context":{"userEmail":"is user@example.com."}}}"#);
    let r = redactor(&["user@example"]).with_identifiers(&ids).unwrap();
    let out = r.redact_text("from user@example.com");
    assert_eq!(out.text, "from [REDACTED:account_email]");
    let none = redactor(&[])
        .with_identifiers(&HarnessAccountIdentifiers::default())
        .unwrap();
    assert_eq!(
        none.redact_text("user@example.com").text,
        "user@example.com"
    );
}

#[test]
fn redact_line_keeps_a_clean_line_byte_for_byte() {
    let r = redactor(&[]);
    let line = r#"{"b": 1,  "a": "x"}"#;
    assert_eq!(r.redact_line(line).text, line);
}

#[test]
fn redact_line_rewrites_a_json_line_with_an_escaped_secret() {
    let r = redactor(&[]);
    let line = r#"{"t":"AKIAIOSFODNN7EXAMPLE"}"#;
    let out = r.redact_line(line);
    assert_eq!(out.text, r#"{"t":"[REDACTED:aws_access_key]"}"#);
    assert_eq!(out.rules, ["aws_access_key"]);
}

#[test]
fn redact_line_redacts_raw_text_that_is_no_json() {
    let r = redactor(&[]);
    let out = r.redact_line(&format!("{{broken {AWS}"));
    assert_eq!(out.text, "{broken [REDACTED:aws_access_key]");
}

#[test]
fn a_marker_carries_nothing_of_the_secret() {
    let r = redactor(&[]);
    let out = r.redact_text(AWS);
    assert!(!out.text.contains("AKIA"));
    assert!(!out.text.contains("EXAMPLE"));
}

#[test]
fn redact_line_finds_a_secret_a_duplicate_key_hides_from_the_parser() {
    let r = redactor(&[]);
    let line = r#"{"a":"AKIAIOSFODNN7EXAMPLE","a":"x"}"#;
    let out = r.redact_line(line);
    assert_eq!(out.text, r#"{"a":"[REDACTED:aws_access_key]","a":"x"}"#);
    let escaped = r#"{"a":"AKIAIOSFODNN7EXAMPLE","a":"x"}"#;
    assert_eq!(r.redact_line(escaped).text, out.text);
}

#[test]
fn redact_line_decodes_each_string_of_a_line_that_is_no_json() {
    let r = redactor(&[]);
    let truncated = r#"{"a":"AKIAIOSFODNN7EXAMPLE","b":"cut"#;
    let out = r.redact_line(truncated);
    assert_eq!(out.text, r#"{"a":"[REDACTED:aws_access_key]","b":"cut"#);
    let bare = "plain AKIAIOSFODNN7EXAMPLE \"quoted\\\"AKIAIOSFODNN7EXAMPLE\"";
    let out = r.redact_line(bare);
    assert_eq!(out.rules.len(), 2);
    assert!(!out.text.contains("AKIA"), "{}", out.text);
    let lone = r#"{"a":"\ud800 AKIAIOSFODNN7EXAMPLE"}"#;
    assert!(!r.redact_line(lone).text.contains("AKIA"));
}

#[test]
fn redact_line_keeps_everything_but_the_redacted_strings_byte_for_byte() {
    let r = redactor(&[]);
    let line = r#"{"b": 1,  "a": "k AKIAIOSFODNN7EXAMPLE", "n": 2.50}"#;
    assert_eq!(
        r.redact_line(line).text,
        r#"{"b": 1,  "a": "k [REDACTED:aws_access_key]", "n": 2.50}"#
    );
}

#[test]
fn pieces_cut_a_line_at_its_string_literals() {
    assert_eq!(
        pieces(r#"{"a": "b\"c", x"open"#),
        [
            Piece::Between("{"),
            Piece::String(r#""a""#),
            Piece::Between(": "),
            Piece::String(r#""b\"c""#),
            Piece::Between(", x"),
            Piece::String(r#""open"#),
        ]
    );
    assert!(pieces("").is_empty());
    // An escape at the very end runs past it, and the literal ends there.
    assert_eq!(literal_end(b"ab\\", 0), 3);
}
