use super::{REDACTION_RULE_SET_VERSION, RULE_SET, RedactionRule, rule_set};

/// What `name`'s rule matches in `text`.
fn matched(name: &str, text: &str) -> Vec<String> {
    let rules = rule_set().unwrap();
    let rule = rules.iter().find(|r| r.name() == name).unwrap();

    rule.spans(text)
        .map(|(s, e)| text[s..e].to_owned())
        .collect()
}

#[test]
fn the_rule_set_is_version_1_and_every_rule_compiles() {
    assert_eq!(REDACTION_RULE_SET_VERSION, 1);
    let rules = rule_set().unwrap();
    let names: Vec<&str> = rules.iter().map(RedactionRule::name).collect();
    let listed: Vec<&str> = RULE_SET.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, listed);
}

#[test]
fn each_rule_matches_its_secret_format() {
    let cases = [
        (
            "aws_access_key",
            "key AKIAIOSFODNN7EXAMPLE end",
            "AKIAIOSFODNN7EXAMPLE",
        ),
        (
            "github_token",
            "ghp_0123456789abcdefghijABCDEFGHIJ012345 x",
            "ghp_0123456789abcdefghijABCDEFGHIJ012345",
        ),
        (
            "anthropic_api_key",
            "sk-ant-api03-abcdefghijklmnopqrstuvwxyz",
            "sk-ant-api03-abcdefghijklmnopqrstuvwxyz",
        ),
        (
            "openai_api_key",
            "sk-proj-abcdefghijklmnopqrstuvwxyz",
            "sk-proj-abcdefghijklmnopqrstuvwxyz",
        ),
        (
            "slack_token",
            "xoxb-1234567890-abcdef",
            "xoxb-1234567890-abcdef",
        ),
        (
            "google_api_key",
            "AIzaSyA-0123456789abcdefghijklmnopqrstu",
            "AIzaSyA-0123456789abcdefghijklmnopqrstu",
        ),
        (
            "stripe_key",
            "sk_live_0123456789abcdefABCD",
            "sk_live_0123456789abcdefABCD",
        ),
        (
            "jwt",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.abcdefghijk",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.abcdefghijk",
        ),
    ];
    for (rule, text, secret) in cases {
        assert_eq!(matched(rule, text), [secret], "{rule}");
    }
}

#[test]
fn a_private_key_block_is_matched_whole_or_to_the_end_when_cut_short() {
    let block =
        "-----BEGIN OPENSSH PRIVATE KEY-----\nb3Bl\nbnNz\n-----END OPENSSH PRIVATE KEY-----";
    assert_eq!(matched("private_key", &format!("a {block} b")), [block]);
    let cut = "-----BEGIN RSA PRIVATE KEY-----\nMIIE";
    assert_eq!(matched("private_key", &format!("a {cut}")), [cut]);
}

#[test]
fn a_rule_leaves_lookalikes_alone() {
    assert!(matched("aws_access_key", "AKIAshort").is_empty());
    assert!(matched("aws_access_key", "XAKIAIOSFODNN7EXAMPLE").is_empty());
    assert!(matched("jwt", "eyJ.eyJ.x").is_empty());
}

#[test]
fn a_literal_rule_matches_its_text_exactly() {
    let rule = RedactionRule::literal("account_email", "a.b+c@example.com").unwrap();
    assert_eq!(rule.name(), "account_email");
    let text = "to a.b+c@example.com and aXb+c@example.com";
    let spans: Vec<_> = rule.spans(text).collect();
    assert_eq!(spans, [(3, 20)]);
}

#[test]
fn a_rule_skips_empty_matches_and_refuses_a_bad_or_huge_pattern() {
    let rule = RedactionRule::new("extra_pattern_1", "x*").unwrap();
    assert_eq!(rule.spans("axxb").collect::<Vec<_>>(), [(1, 3)]);
    assert!(RedactionRule::new("extra_pattern_1", "(").is_err());
    assert!(RedactionRule::new("extra_pattern_1", r"\w{1000}{1000}").is_err());
}

#[test]
fn a_literal_rule_ignores_case() {
    let rule = RedactionRule::literal("account_email", "Person@Example.com").unwrap();
    assert_eq!(
        rule.spans("person@EXAMPLE.COM").collect::<Vec<_>>(),
        [(0, 18)]
    );
}
