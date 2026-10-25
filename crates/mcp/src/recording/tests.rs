use super::*;
use serde_json::json;

#[test]
fn every_text_in_the_arguments_and_every_name_is_redacted() {
    let redact = |t: &str| {
        let rules: Vec<String> = t.matches("secret").map(|_| "rule".to_owned()).collect();
        (t.replace("secret", "[R]"), rules)
    };
    let arguments = json!({
        "query": "a secret",
        "any_of": ["secret", 3, {"secret": "x secret"}],
        "k": 5,
        "flag": true,
        "none": null,
    });

    let mut rules = Vec::new();

    let out = redacted(&arguments, &redact, &mut rules);

    assert_eq!(
        out,
        json!({
            "query": "a [R]",
            "any_of": ["[R]", 3, {"[R]": "x [R]"}],
            "k": 5,
            "flag": true,
            "none": null,
        })
    );
    assert_eq!(rules.len(), 4, "one rule for each redaction");
}
